use chrono::{NaiveDate, Utc};

use crate::{
  config::ConfigTrabajo,
  inc::{
    EstadoIncidencia, Incidencia, IncidenciaMarcaje, IncidenciaProceso,
    IncidenciaRepo, TipoIncidencia,
  },
  infra::{DominiosWithCacheUsuario, ServicioError, Transaccion},
  marcaje::{Marcaje, MarcajeServicio},
  traza::{TipoTraza, TrazaBuilder, TrazaServicio},
};

/// Servicio que gestiona las incidencias del usuario
pub struct IncidenciaServicio {
  cnfg: ConfigTrabajo,
  repo: IncidenciaRepo,
  srv_traza: TrazaServicio,
  srv_marcaje: MarcajeServicio,
}

impl IncidenciaServicio {
  pub fn new(
    cnfg: ConfigTrabajo,
    repo: IncidenciaRepo,
    srv_traza: TrazaServicio,
    srv_marcaje: MarcajeServicio,
  ) -> Self {
    IncidenciaServicio {
      cnfg,
      repo,
      srv_traza,
      srv_marcaje,
    }
  }
}

impl IncidenciaServicio {
  /// Añade una incidencia
  ///
  /// Si la incidencia ya existe devuelve un error
  /// gestionado por los propios constraint de la base
  /// de datos
  pub async fn agregar(&self, inc: &Incidencia) -> Result<u32, ServicioError> {
    tracing::info!(
      incidencia = ?inc,
      "Se ha iniciado el servicio para crear una incidencia de marcaje");

    let id = match self.repo.agregar(inc).await {
      Ok(reg_id) => reg_id,
      Err(err) => {
        tracing::error!(
          incidencia = ?inc,
          error = %err,
          "Creando incidencia de marcaje"
        );
        return Err(ServicioError::from(err));
      }
    };

    tracing::debug!(
      incidencia = id,
      "Se ha completado satisfactoriamente la creación de la incidencia"
    );

    Ok(id)
  }

  /// Recorre un lista de incidencias y la procesa según su estado y tipo
  ///
  /// Los estados poueden ser resolver o rechazar.
  ///
  /// Si el estado es rechazar, se cambia el estado a rechazada
  /// y se especifica un motivo de rechazo.
  ///
  /// Si el estado es resolver, se intenta resolver la incidencia
  /// dependiendo del tipo. Si el tipo es nuevo marcaje, se crea
  /// un nuevo marcaje. Si el tipo es eliminación de marcaje,
  /// se elimina el marcaje indicado. Si el tipo es corrección
  /// de salida, se modifica la hora de salida del marcaje. Se cambia
  /// el estado a resuelta.
  /// Al procesar el marcaje, puede que surjan errores de validación
  /// en ese caso, se cambia el estado a conflicto para que el usuario.
  ///
  /// Si ocurre algún error durante el proceso, se cambia el estado
  /// a error resolver o error rechazar según corresponda, y se
  /// especifica el error ocurrido.
  ///
  /// Si el estado es rechazar o resolver, se comprueba que
  /// la base de datos siga teniendo el estado solicitud
  /// si no fuera así, significa que ya se ha procesado y
  /// se ignora.
  ///
  /// Puede que existan errores que no se puedan tratar
  /// Estos errores se tracean y se notifica a el usuario
  pub async fn procesar_incidencias(
    &self,
    usuario_gestor: u32,
    incidencias: &[IncidenciaProceso],
  ) -> Result<Vec<u32>, ServicioError> {
    let fecha_actual = Utc::now()
      .with_timezone(&self.cnfg.zona_horaria)
      .naive_local();

    let mut panic_inc = Vec::with_capacity(incidencias.len());

    let conexion = self.repo.conexion();

    for incp in incidencias {
      tracing::info!(
        incidencia = ?incp,
        "Procesando incidencia de marcaje"
      );

      let mut tr = match conexion.empezar_transaccion().await {
        Ok(transaccion) => transaccion,
        Err(err) => {
          tracing::error!(
          incidencia = ?incp,
          error = %err,
          "Iniciando transacción para procesar incidencia de marcaje");

          panic_inc.push(incp.id);
          continue;
        }
      };

      match incp.estado {
        EstadoIncidencia::Resolver => {
          // Lo primero es cambiar el estado para bloquear el registro
          let res = self
            .repo
            .cambiar_estado_resuelto(
              &mut tr,
              incp.id,
              usuario_gestor,
              fecha_actual,
            )
            .await;
          match res {
            Ok(estado_cambiado) => {
              if estado_cambiado {
                // Obtenemos la info mínima necesaria para procesar
                // la incidencia
                match self.repo.incidencia_para_marcaje(incp.id).await {
                  Ok(inc) => {
                    let mut error_message: Option<&'static str> = None;

                    match inc.tipo {
                      TipoIncidencia::NuevoMarcaje => {
                        if let Err(err) = self
                          .crear_marcaje(&mut tr, usuario_gestor, incp, &inc)
                          .await
                        {
                          error_message = Some(err);
                        }
                      }
                      TipoIncidencia::CorrecionSalida => {
                        if let Err(err) = self
                          .corregir_marcaje(&mut tr, usuario_gestor, incp, &inc)
                          .await
                        {
                          error_message = Some(err);
                        }
                      }
                      TipoIncidencia::EliminacionMarcaje => {
                        if let Err(err) =
                          self.eliminar_marcaje(&mut tr, incp, &inc).await
                        {
                          error_message = Some(err);
                        }
                      }
                    }

                    if error_message.is_some() {
                      // Cambiamos el estado a error resolver
                      if let Err(err) = self
                        .repo
                        .cambiar_estado_incidente(
                          &mut tr,
                          incp.id,
                          EstadoIncidencia::ErrorResolver,
                          error_message.unwrap(),
                          fecha_actual,
                        )
                        .await
                      {
                        tracing::error!(
                          id = incp.id,
                          incidencia = ?inc,
                          error = %err,
                          r"Cambiando el estado a error resolver
                          tras error eliminando marcaje"
                        );

                        panic_inc.push(incp.id);
                        continue;
                      }
                    } else {
                      tracing::info!(
                        id = incp.id,
                        incidencia = ?incp,
                        "La incidencia de marcaje ha sido resuelta correctamente"
                      );
                    }
                  }
                  Err(err) => {
                    tracing::error!(
                      incidencia = ?incp,
                      error = %err,
                      r"Obteniendo la información mínima necesaria
                      para procesar la incidencia"
                    );

                    panic_inc.push(incp.id);
                    continue;
                  }
                };
              } else {
                tracing::warn!(
                  incidencia = ?incp,
                  r"No se ha podido resolver la incidencia de marcaje,
                  posiblemente ya estaba procesada"
                );
              }
            }
            Err(err) => {
              tracing::error!(
              incidencia = ?incp,
              error = %err,
              "Error cambiando a estado resuelto");

              panic_inc.push(incp.id);
              continue;
            }
          }
        }
        EstadoIncidencia::Rechazar => {
          let res = self
            .repo
            .cambiar_estado_rechazado(
              &mut tr,
              incp.id,
              incp.motivo_rechazo.as_deref(),
              usuario_gestor,
              fecha_actual,
            )
            .await;
          match res {
            Ok(estado_cambiado) => {
              if estado_cambiado {
                tracing::info!(
                  incidencia = ?incp,
                  "La incidencia de marcaje ha sido rechazada correctamente"
                );
              } else {
                tracing::warn!(
                  incidencia = ?incp,
                  r"No se ha podido rechazar la incidencia de marcaje,
                  posiblemente ya estaba procesada"
                );
              }
            }
            Err(err) => {
              tracing::error!(
                incidencia = ?incp,
                error = %err,
                "Error cambiando a estado rechazado");

              panic_inc.push(incp.id);
              continue;
            }
          }
        }
        _ => {
          tracing::warn!(
            incidencia = ?incp,
            "Estado de incidencia no válido para procesar"
          );
        }
      }

      if let Err(err) = tr.commit().await {
        tracing::error!(
          incidencia = ?incp,
          error = %err,
          "Commit transacción cuando procesa una incidencia");

        panic_inc.push(incp.id);
      }
    }

    Ok(panic_inc)
  }

  /// Crea un nuevo marcaje asociado a la incidencia
  ///
  /// Si existe un error, se devuelve la descripción del mismo
  async fn crear_marcaje(
    &self,
    tr: &mut Transaccion<'_>,
    usuario_gestor: u32,
    incp: &IncidenciaProceso,
    inc: &IncidenciaMarcaje,
  ) -> Result<(), &'static str> {
    let marcaje = Marcaje {
      id: 0,
      usuario: inc.usuario,
      usuario_reg: if inc.usuario == inc.usuario_creador {
        None
      } else {
        Some(inc.usuario_creador)
      },
      horario: None,
      fecha: inc.fecha,
      hora_inicio: inc.hora_inicio.unwrap(),
      hora_fin: inc.hora_fin,
    };

    match self
      .srv_marcaje
      .agregar_with_trans(Some(tr), &marcaje, 0)
      .await
    {
      Ok(marcaje_id) => {
        tracing::info!(
          marcaje = marcaje_id,
          id = incp.id,
          incidencia = ?inc,
          "Marcaje creado correctamente al resolver la incidencia"
        );

        Ok(())
      }
      Err(err) => {
        tracing::error!(
          id = incp.id,
          incidencia = ?inc,
          error = %err,
          "Creando marcaje nuevo asociado a la incidencia"
        );

        self
          .manejar_conflicto(
            tr,
            incp.id,
            usuario_gestor,
            err,
            r"No se ha podido crear el marcaje nuevo asociado a la incidencia. 
          Consulte con el administrador del sistema.",
          )
          .await
      }
    }
  }

  /// Corregir la hora de salida
  ///
  /// Crea un marcaje nuevo con la hora de entrada existente
  /// en el marcaje asociado y la hora de salida solicitada
  ///
  /// Si existe un error, se devuelve la descripción del mismo
  async fn corregir_marcaje(
    &self,
    tr: &mut Transaccion<'_>,
    usuario_gestor: u32,
    incp: &IncidenciaProceso,
    inc: &IncidenciaMarcaje,
  ) -> Result<(), &'static str> {
    let marcaje_asociado = inc.marcaje.as_ref().unwrap();

    let marcaje = Marcaje {
      id: 0,
      usuario: inc.usuario,
      usuario_reg: if inc.usuario == inc.usuario_creador {
        None
      } else {
        Some(inc.usuario_creador)
      },
      horario: None,
      fecha: inc.fecha,
      hora_inicio: marcaje_asociado.hora_inicio.unwrap(),
      hora_fin: inc.hora_fin,
    };

    match self
      .srv_marcaje
      .agregar_with_trans(Some(tr), &marcaje, marcaje_asociado.id)
      .await
    {
      Ok(marcaje_id) => {
        match self
          .srv_marcaje
          .actualizar_modificado_por(tr, marcaje_asociado.id, marcaje_id)
          .await
        {
          Ok(modificado) => {
            if modificado {
              tracing::info!(
                marcaje = marcaje_id,
                id = incp.id,
                incidencia = ?inc,
                r"Salida del marcaje corregido correctamente
                al resolver la incidencia"
              );

              Ok(())
            } else {
              Err(
                r"El marcaje que se quiere corregir la salida ya no existe. 
                Consulte con el administrador",
              )
            }
          }
          Err(_) => Err(
            r"No se ha podido deshabilitar el marcaje que 
             se quiere corregir la salida. Consulte con el administrador",
          ),
        }
      }
      Err(err) => {
        tracing::error!(
          id = incp.id,
          incidencia = ?inc,
          error = %err,
          "Corrigiendo marcaje de salida asociado a la incidencia"
        );

        self
          .manejar_conflicto(
            tr,
            incp.id,
            usuario_gestor,
            err,
            r"No se ha podido corregir el marcaje de salida
            asociado a la incidencia. 
            Consulte con el administrador del sistema.",
          )
          .await
      }
    }
  }

  /// Elimina el marcaje asociado a la incidencia
  ///
  /// Si existe un error, se devuelve la descripción del mismo
  async fn eliminar_marcaje(
    &self,
    tr: &mut Transaccion<'_>,
    incp: &IncidenciaProceso,
    inc: &IncidenciaMarcaje,
  ) -> Result<(), &'static str> {
    match self
      .srv_marcaje
      .marcar_marcaje_eliminado(tr, inc.marcaje.as_ref().unwrap().id)
      .await
    {
      Ok(eliminado) => {
        if eliminado {
          tracing::info!(
            id = incp.id,
            incidencia = ?inc,
            "Marcaje eliminado correctamente al resolver la incidencia"
          );
        } else {
          tracing::warn!(
            id = incp.id,
            incidencia = ?inc,
            r"No existe el marcaje para eliminar. Se deshecha"
          );
        }

        Ok(())
      }
      Err(err) => {
        tracing::error!(
          incidencia = ?incp,
          error = %err,
          "Eliminando marcaje asociado a la incidencia"
        );

        Err(
          r"No se ha podido eliminar el marcaje asociado a la incidencia. 
          Consulte con el administrador del sistema.",
        )
      }
    }
  }

  /// Cambia el estado a conflicto si se produce un error gestionado
  ///
  /// Los errores gestionados son:
  /// - ServicioError::Usuario
  /// - ServicioError::Validacion
  /// - ServicioError::DB(
  ///      DBError::RegistroVacio(e) |
  ///      DBError::ConstraintViolation(e))
  ///
  /// Si el error no es gestionado, se devuelve un error genérico
  /// recibido como parámetro
  async fn manejar_conflicto(
    &self,
    tr: &mut Transaccion<'_>,
    incidencia_id: u32,
    usuario_gestor: u32,
    err: ServicioError,
    default_err: &'static str,
  ) -> Result<(), &'static str> {
    let fecha_actual = Utc::now()
      .with_timezone(&self.cnfg.zona_horaria)
      .naive_local();

    let mensaje = err.mensaje();

    if mensaje.is_empty() {
      return Err(default_err);
    } else {
      self
        .repo
        .cambiar_estado_incidente(
          tr,
          incidencia_id,
          EstadoIncidencia::Conflicto,
          &mensaje,
          fecha_actual,
        )
        .await
        .map_err(|e| {
          tracing::error!(
            id = incidencia_id,
            error = %e,
            r"Cambiando el estado a conflicto tras procesar incidencia"
          );
          default_err
        })?;

      let traza =
        TrazaBuilder::with_inc(TipoTraza::IncConflicto, incidencia_id)
          .autor(Some(usuario_gestor))
          .motivo(Some(format!(
            "Conflicto: '{}' en la fecha: {}",
            &mensaje, fecha_actual
          )))
          .build(&self.cnfg.zona_horaria);

      if let Err(err) = self.srv_traza.agregar(tr, &traza).await {
        tracing::error!(
                      incidencia = incidencia_id,
                      error = %err,
                      "Error generando traza cambiando a estado conflicto");

        return Err(default_err);
      }
    }

    Ok(())
  }

  /// Lista las incidencias que cumplen los filtros indicados.
  #[inline]
  pub async fn incidencias(
    &self,
    fecha_inicio: Option<NaiveDate>,
    fecha_fin: Option<NaiveDate>,
    estados: &[EstadoIncidencia],
    usuario: Option<u32>,
  ) -> Result<DominiosWithCacheUsuario<Incidencia>, ServicioError> {
    self
      .repo
      .incidencias(fecha_inicio, fecha_fin, estados, usuario)
      .await
      .map_err(|err| {
        tracing::error!(
          fecha_inicio = ?fecha_inicio, fecha_fin = ?fecha_fin,
          estados = ?estados, usuarios = usuario,
          error = %err,
          "Obteniendo incidencias");
        ServicioError::from(err)
      })
  }
}
