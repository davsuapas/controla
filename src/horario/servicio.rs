use chrono::{NaiveDate, NaiveDateTime, Utc};

use crate::{
  config::ConfigTrabajo,
  horario::{Calendario, CalendarioFecha, ConfigHorario, Horario, HorarioRepo},
  infra::{DBError, ServicioError, ShortDateTimeFormat, Transaccion},
};

/// Servicio para manejar operaciones relacionadas con horarios.
pub struct HorarioServicio {
  cnfg: ConfigTrabajo,
  repo: HorarioRepo,
}

impl HorarioServicio {
  pub fn new(cnfg: ConfigTrabajo, repo: HorarioRepo) -> Self {
    HorarioServicio { cnfg, repo }
  }
}

impl HorarioServicio {
  /// Devuelve los horarios del usuario sin asingnar.
  ///
  /// Si no se proporciona una hora, devuelve el horario del día actual.
  /// simpre que no este asignado.
  /// Si se proporciona una hora, devuelve el horario más cercano a esa hora.
  pub async fn horarios_usuario_sin_asignar(
    &self,
    usuario: u32,
    fecha: NaiveDate,
  ) -> Result<Vec<Horario>, ServicioError> {
    self
      .repo
      .horarios_usuario_sin_asignar(usuario, fecha)
      .await
      .map_err(|err| {
        tracing::error!(
        usuario = usuario,
        hora = ?fecha,
        error = %err,
       "Obteniendo horario del usuario sin asignar");
        ServicioError::from(err)
      })
  }

  /// Devuelve el horario del usuario más cercano.
  ///
  /// Si no devuelve ningún horario se tracea el error y
  /// se devuelve una array vacío
  pub async fn horario_usuario_cercano(
    &self,
    usuario: u32,
    hora: NaiveDateTime,
  ) -> Result<Vec<Horario>, ServicioError> {
    tracing::debug!(
      usuario = usuario,
      hora = %hora,
      "Consultando el horario más cercano del usuario");

    let res = self
      .repo
      .horario_cercano(usuario, hora, 0)
      .await
      .map(|(_, horario)| vec![horario]);
    match res {
      Ok(horarios) => Ok(horarios),
      Err(err) => match err {
        DBError::RegistroVacio(e) => {
          tracing::warn!(
            error = %e,
           "Consulta horario cercano");
          Ok(vec![])
        }
        _ => {
          tracing::error!(
            usuario = usuario,
            hora = ?hora,
            error = %err,
          "Consulta horario cercano");
          Err(ServicioError::from(err))
        }
      },
    }
  }

  /// Devuelve el horario más cercano al usuario.
  ///
  /// Se puede excluir un marcaje pasado como parámetro
  /// Si no quiere excluir ningún marcaje use 0
  /// La exclusión puede ser muy útil cuando se quiere
  /// realizar una modificación de este marcaje
  ///
  /// Devuelve el identificador de usuario, horario y el horario.
  pub async fn horario_cercano(
    &self,
    usuario: u32,
    hora: NaiveDateTime,
    excluir_marcaje_id: u32,
  ) -> Result<(u32, Horario), ServicioError> {
    self
      .repo
      .horario_cercano(usuario, hora, excluir_marcaje_id)
      .await
      .map_err(ServicioError::from)
  }

  /// Devuelve la configuración de horarios de un usuario.
  ///
  /// Recupera la lista de horarios configurados para el usuario especificado.
  pub async fn config_horario(
    &self,
    usuario: u32,
  ) -> Result<Vec<ConfigHorario>, ServicioError> {
    tracing::debug!(
      usuario = usuario,
      "Obteniendo configuración de horario del usuario"
    );

    let fecha_actual = Utc::now()
      .with_timezone(&self.cnfg.zona_horaria)
      .naive_local()
      .date();

    self
      .repo
      .config_horario(usuario, fecha_actual)
      .await
      .map_err(|err| {
        tracing::error!(
          usuario = usuario,
          error = %err,
          "Obteniendo configuración de horario del usuario"
        );
        ServicioError::from(err)
      })
  }

  /// Obtiene el horario dado el id.
  pub async fn config_horario_por_id(
    &self,
    id: u32,
  ) -> Result<ConfigHorario, ServicioError> {
    tracing::debug!(id = id, "Obteniendo horario por id");

    self.repo.config_horario_por_id(id).await.map_err(|err| {
      tracing::error!(
        id = id,
        error = %err,
        "Obteniendo horario por id"
      );
      ServicioError::from(err)
    })
  }

  /// Duplica la configuración de un horario
  ///
  /// Devuelve la configuración del horario duplicada
  pub async fn duplicar_config_horario(
    &self,
    usuario: u32,
    nueva_fecha_creacion: NaiveDate,
  ) -> Result<Vec<ConfigHorario>, ServicioError> {
    tracing::info!(
      usuario = usuario,
      nueva_fecha_creacion = %nueva_fecha_creacion,
      "Se duplica la configuración del horario");

    self
      .repo
      .duplicar_config_horario(usuario, nueva_fecha_creacion)
      .await
      .map_err(|err| {
        tracing::error!(
          usuario = usuario,
          nueva_fecha_creacion = %nueva_fecha_creacion,
          error = %err,
          "Duplicando grupo horario."
        );
        ServicioError::from(err)
      })?;

    self.config_horario(usuario).await
  }

  /// Añade una nueva configuración de horario.
  ///
  /// Verifica que no exista solapamiento con otros horarios.
  /// Si existe solapamiento se envía un error al usuario.
  /// Si no existe solapamiento se llama a [`Self::actualizar_horario`]
  /// para actualizar el maestro del horario y devolve el ide de horario.
  /// Por último se crea la configuración del horario.
  pub async fn agregar_config_horario(
    &self,
    config_horario: &ConfigHorario,
  ) -> Result<u32, ServicioError> {
    tracing::info!(
      usuario = config_horario.usuario,
      fecha = %config_horario.fecha_creacion,
      "Se ha iniciado el servicio para agregar una configuración de horario");

    if self
      .repo
      .config_horario_solape(config_horario)
      .await
      .map_err(|err| {
        tracing::error!(
          usuario = config_horario.usuario,
          error = %err,
          "Verificando solapamiento de horario"
        );
        ServicioError::from(err)
      })?
    {
      return Err(ServicioError::Validacion(format!(
        "El horario se solapa con otro existente para el usuario: {} en la fecha: {}",
        config_horario.usuario,
        config_horario.fecha_creacion.formato_corto()
      )));
    }

    let mut tr =
      self
        .repo
        .conexion()
        .empezar_transaccion()
        .await
        .map_err(|err| {
          tracing::error!(
           usuario = config_horario.usuario, error = %err,
           "Iniciando transacción para agregar configuración de horario");
          ServicioError::from(err)
        })?;

    let id_horario = self
      .obtener_o_crear_horario(&mut tr, &config_horario.horario)
      .await?;

    let id_config = match self
      .repo
      .agregar_config_usuario(&mut tr, config_horario, id_horario)
      .await
    {
      Ok(id) => id,
      Err(err) => {
        tracing::error!(
          usuario = config_horario.usuario,
          error = %err,
          "Agregando configuración de horario en bd"
        );

        return Err(ServicioError::from(err));
      }
    };

    tr.commit().await.map_err(|err| {
      tracing::error!(
         usuario = config_horario.usuario, error = %err,
        "Commit transacción para agregar configuración de horario");
      ServicioError::from(err)
    })?;

    tracing::debug!(
      id_config = id_config,
      "Se ha completado satisfactoriamente la agregación \
      de la configuración de horario"
    );

    Ok(id_config)
  }

  /// Modifica una configuración de horario.
  ///
  /// Verifica que no esté referenciada en un marcaje.
  /// Verifica que no exista solapamiento con otros horarios.
  /// Si no existe solapamiento se llama a [`Self::actualizar_horario`]
  /// para actualizar el maestro del horario y por último se
  /// modifica la configuración del horario.
  pub async fn modificar_config_horario(
    &self,
    config_horario: &ConfigHorario,
  ) -> Result<(), ServicioError> {
    tracing::info!(
      config_horario = ?config_horario,
      "Se ha iniciado el servicio para modificar una configuración de horario");

    if self
      .repo
      .esta_horario_en_marcaje(config_horario.id)
      .await
      .map_err(|err| {
        tracing::error!(
          config_horario_id = config_horario.id,
          error = %err,
          "Verificando si la configuración de horario está en un marcaje"
        );
        ServicioError::from(err)
      })?
    {
      return Err(ServicioError::Validacion(
        "No se puede modificar la configuración del horario porque \
        ya está referenciada en un marcaje."
          .to_string(),
      ));
    }

    if self
      .repo
      .config_horario_solape(config_horario)
      .await
      .map_err(|err| {
        tracing::error!(
          usuario = config_horario.usuario,
          error = %err,
          "Verificando solapamiento de horario"
        );
        ServicioError::from(err)
      })?
    {
      return Err(ServicioError::Validacion(format!(
        "El horario se solapa con otro existente para el usuario: {} en la fecha: {}",
        config_horario.usuario,
        config_horario.fecha_creacion.formato_corto()
      )));
    }

    let mut tr =
      self
        .repo
        .conexion()
        .empezar_transaccion()
        .await
        .map_err(|err| {
          tracing::error!(
           usuario = config_horario.usuario, error = %err,
           "Iniciando transacción para modificar configuración de horario");
          ServicioError::from(err)
        })?;

    let id_horario_antiguo = config_horario.horario.id;
    let id_nuevo_horario = self
      .obtener_o_crear_horario(&mut tr, &config_horario.horario)
      .await?;

    if let Err(err) = self
      .repo
      .modificar_config_usuario(&mut tr, config_horario, id_nuevo_horario)
      .await
    {
      tracing::error!(
        config_horario = ?config_horario,
        error = %err,
        "Modificando configuración de horario en bd"
      );
      return Err(ServicioError::from(err));
    }

    if id_horario_antiguo != id_nuevo_horario {
      self
        .eliminar_horario(&mut tr, id_horario_antiguo, config_horario.id)
        .await?;
    }

    tr.commit().await.map_err(|err| {
      tracing::error!(
         usuario = config_horario.usuario, error = %err,
        "Commit transacción para modificar configuración de horario");
      ServicioError::from(err)
    })?;

    tracing::debug!(
      id_config = config_horario.id,
      "Se ha completado satisfactoriamente la modificación \
      de la configuración de horario"
    );

    Ok(())
  }

  /// Elimina una configuración de horario.
  ///
  /// Verifica que no esté referenciada en un marcaje.
  /// Elimina la configuración y si el horario asociado no se usa
  /// en ninguna otra configuración, también se elimina.
  pub async fn eliminar_config_horario(
    &self,
    id_config: u32,
  ) -> Result<(), ServicioError> {
    tracing::info!(
      id_config = id_config,
      "Se ha iniciado el servicio para eliminar una configuración de horario"
    );

    if self
      .repo
      .esta_horario_en_marcaje(id_config)
      .await
      .map_err(|err| {
        tracing::error!(
          id_config = id_config,
          error = %err,
          "Verificando si la configuración de horario está en un marcaje"
        );
        ServicioError::from(err)
      })?
    {
      return Err(ServicioError::Validacion(
        "No se puede eliminar la configuración del horario porque ya está referenciada en un marcaje."
          .to_string(),
      ));
    }

    let config = self.config_horario_por_id(id_config).await?;

    let mut tr =
      self
        .repo
        .conexion()
        .empezar_transaccion()
        .await
        .map_err(|err| {
          tracing::error!(
           id_config = id_config, error = %err,
           "Iniciando transacción para eliminar configuración de horario");
          ServicioError::from(err)
        })?;

    if let Err(err) =
      self.repo.eliminar_config_usuario(&mut tr, id_config).await
    {
      tracing::error!(
        id_config = id_config,
        error = %err,
        "Eliminando configuración de horario en bd"
      );
      return Err(ServicioError::from(err));
    }

    self
      .eliminar_horario(&mut tr, config.horario.id, id_config)
      .await?;

    tr.commit().await.map_err(|err| {
      tracing::error!(
         id_config = id_config, error = %err,
        "Commit transacción para eliminar configuración de horario");
      ServicioError::from(err)
    })?;

    tracing::debug!(
      id_config = id_config,
      "Se ha completado satisfactoriamente la eliminación de la configuración de horario"
    );

    Ok(())
  }

  /// Obtiene el id de un horario existente o crea uno nuevo.
  ///
  /// Si existe ya un horario (mismo día y fechas) devuelve el id.
  /// Si no existe lo crea y devuelve el nuevo id.
  async fn obtener_o_crear_horario(
    &self,
    trans: &mut Transaccion<'_>,
    horario: &Horario,
  ) -> Result<u32, ServicioError> {
    tracing::info!(
      horario = ?horario,
      "Se ha iniciado el servicio para obtener o crear un horario");

    if let Some(id) =
      self
        .repo
        .horario_por_dia_horas(horario)
        .await
        .map_err(|err| {
          tracing::error!(
            horario = ?horario,
            error = %err,
            "Buscando horario por día y horas"
          );
          ServicioError::from(err)
        })?
    {
      Ok(id)
    } else {
      tracing::debug!("No existe el horario maestro. Se crea uno nuevo");
      match self.repo.crear_horario(trans, horario).await {
        Ok(id) => Ok(id),
        Err(err) => {
          tracing::error!(
            horario = ?horario,
            error = %err,
            "Creando nuevo horario maestro"
          );
          Err(ServicioError::from(err))
        }
      }
    }
  }

  /// Elimina el horario si no es usado por ninguna otra configuración.
  async fn eliminar_horario(
    &self,
    trans: &mut Transaccion<'_>,
    id_horario: u32,
    id_config_excluida: u32,
  ) -> Result<(), ServicioError> {
    if !self
      .repo
      .es_horario_usado_excepto(id_horario, id_config_excluida)
      .await
      .map_err(|err| {
        tracing::error!(
          id_horario = id_horario,
          id_config_excluida = id_config_excluida,
          error = %err,
          "Verificando si el horario está en uso"
        );
        ServicioError::from(err)
      })?
    {
      tracing::debug!(
        id_horario = id_horario,
        "El horario ya no se referencia. Se elimina"
      );
      self
        .repo
        .eliminar_horario(trans, id_horario)
        .await
        .map_err(|err| {
          tracing::error!(
            id_horario = id_horario,
            error = %err,
            "Eliminando horario no referenciado"
          );
          ServicioError::from(err)
        })?;
    }
    Ok(())
  }

  /// Devuelve la lista de todos los calendarios laborales definidos.
  pub async fn calendarios(&self) -> Result<Vec<Calendario>, ServicioError> {
    tracing::debug!("Obteniendo lista de calendarios");

    self.repo.calendarios().await.map_err(|err| {
      tracing::error!(error = %err, "Obteniendo lista de calendarios");
      ServicioError::from(err)
    })
  }

  /// Devuelve un calendario laboral por su identificador.
  pub async fn calendario(&self, id: u32) -> Result<Calendario, ServicioError> {
    tracing::debug!(id = id, "Obteniendo calendario por id");

    self.repo.calendario(id).await.map_err(|err| {
      tracing::error!(
        id = id, error = %err, "Obteniendo calendario por id");
      ServicioError::from(err)
    })
  }

  /// Crea un nuevo calendario laboral.
  pub async fn crear_calendario(
    &self,
    calendario: &Calendario,
  ) -> Result<u32, ServicioError> {
    tracing::info!(
      calendario = ?calendario,
      "Iniciando creación de calendario"
    );

    let id = self
      .repo
      .crear_calendario(calendario)
      .await
      .map_err(|err| {
        tracing::error!(
          calendario = ?calendario, error = %err,
          "Creando calendario");
        ServicioError::from(err)
      })?;

    tracing::debug!(id = id, "Calendario creado con éxito");

    Ok(id)
  }

  /// Actualiza los datos de un calendario laboral existente.
  pub async fn actualizar_calendario(
    &self,
    calendario: &Calendario,
  ) -> Result<(), ServicioError> {
    tracing::info!(
      calendario = ?calendario,
      "Iniciando actualización de calendario"
    );

    self
      .repo
      .actualizar_calendario(calendario)
      .await
      .map_err(|err| {
        tracing::error!(
          calendario = ?calendario, error = %err,
          "Actualizando calendario");
        ServicioError::from(err)
      })?;

    tracing::debug!(id = calendario.id, "Calendario actualizado con éxito");

    Ok(())
  }

  /// Elimina un calendario laboral por su identificador.
  pub async fn eliminar_calendario(
    &self,
    id: u32,
  ) -> Result<(), ServicioError> {
    tracing::info!(id = id, "Iniciando eliminación de calendario");

    self.repo.eliminar_calendario(id).await.map_err(|err| {
      tracing::error!(
          id = id, error = %err, "Eliminando calendario");
      ServicioError::from(err)
    })?;

    tracing::debug!(id = id, "Calendario eliminado con éxito");

    Ok(())
  }

  /// Devuelve las fechas señaladas de un calendario.
  ///
  /// Permite filtrar por un rango de fechas (inicio y fin).
  pub async fn calendario_fechas(
    &self,
    calendario_id: u32,
    fecha_inicio: Option<NaiveDate>,
    fecha_fin: Option<NaiveDate>,
  ) -> Result<Vec<CalendarioFecha>, ServicioError> {
    tracing::debug!(
      calendario_id = calendario_id,
      fecha_inicio = ?fecha_inicio,
      fecha_fin = ?fecha_fin,
      "Obteniendo fechas de calendario"
    );

    self
      .repo
      .calendario_fechas(calendario_id, fecha_inicio, fecha_fin)
      .await
      .map_err(|err| {
        tracing::error!(
          calendario_id = calendario_id,
          fecha_inicio = ?fecha_inicio,
          fecha_fin = ?fecha_fin,
          error = %err,
          "Obteniendo fechas de calendario"
        );
        ServicioError::from(err)
      })
  }

  /// Devuelve una fecha señalada por su identificador.
  pub async fn calendario_fecha(
    &self,
    id: u32,
  ) -> Result<CalendarioFecha, ServicioError> {
    tracing::debug!(id = id, "Obteniendo fecha de calendario por id");

    self.repo.calendario_fecha(id).await.map_err(|err| {
      tracing::error!(
        id = id, error = %err, "Obteniendo fecha de calendario por id");
      ServicioError::from(err)
    })
  }

  /// Crea una nueva fecha señalada en un calendario.
  pub async fn crear_calendario_fecha(
    &self,
    fecha: &CalendarioFecha,
  ) -> Result<u32, ServicioError> {
    tracing::info!(
      fecha = ?fecha,
      "Iniciando creación de fecha de calendario"
    );

    self
      .validar_conflicto_marcajes_calendario_fechas(
        fecha.calendario,
        fecha.fecha_inicio,
        fecha.fecha_fin,
      )
      .await?;

    self
      .repo
      .crear_calendario_fecha(fecha)
      .await
      .map_err(|err| {
        tracing::error!(
          fecha = ?fecha, error = %err,
          "Creando fecha de calendario");
        ServicioError::from(err)
      })?;

    tracing::debug!(id = fecha.id, "Fecha de calendario creada con éxito");

    Ok(fecha.id)
  }

  /// Actualiza una fecha señalada existente.
  pub async fn actualizar_calendario_fecha(
    &self,
    fecha: &CalendarioFecha,
  ) -> Result<(), ServicioError> {
    tracing::info!(
      fecha = ?fecha,
      "Iniciando actualización de fecha de calendario"
    );

    self
      .validar_conflicto_marcajes_calendario_fechas(
        fecha.calendario,
        fecha.fecha_inicio,
        fecha.fecha_fin,
      )
      .await?;

    self
      .repo
      .actualizar_calendario_fecha(fecha)
      .await
      .map_err(|err| {
        tracing::error!(
          fecha = ?fecha, error = %err,
          "Actualizando fecha de calendario");
        ServicioError::from(err)
      })?;

    tracing::debug!(id = fecha.id, "Fecha de calendario actualizada con éxito");

    Ok(())
  }

  /// Elimina una fecha señalada por su identificador.
  pub async fn eliminar_calendario_fecha(
    &self,
    id: u32,
  ) -> Result<(), ServicioError> {
    tracing::info!(id = id, "Iniciando eliminación de fecha de calendario");

    self
      .repo
      .eliminar_calendario_fecha(id)
      .await
      .map_err(|err| {
        tracing::error!(
          id = id, error = %err, "Eliminando fecha de calendario");
        ServicioError::from(err)
      })?;

    tracing::debug!(id = id, "Fecha de calendario eliminada con éxito");

    Ok(())
  }

  /// Validada que no existan marcajes entre las fechas de un calendario
  ///
  /// Los calendarios son fechas inhábiles.
  /// Valida que no exista ningún marcaje entre la fecha_inicio y fecha_fin
  /// que se quiere añadir para todos los usuarios que tiene ese calendario.
  ///
  /// Si existe, provoca un error con un texto significativo para el usuario.
  pub async fn validar_conflicto_marcajes_calendario_fechas(
    &self,
    calendario_id: u32,
    fecha_inicio: NaiveDate,
    fecha_fin: NaiveDate,
  ) -> Result<(), ServicioError> {
    let conflictos = self
      .repo
      .marcajes_conflictivos_en_calendario_fecha(
        calendario_id,
        fecha_inicio,
        fecha_fin,
      )
      .await?;

    if !conflictos.is_empty() {
      let mut mensaje_error =
       "Existen marcajes en las fechas especificadas para los siguientes usuarios:\n".to_string();

      for (nombre_usuario, fechas) in conflictos {
        let fechas_str: Vec<String> =
          fechas.into_iter().map(|f| f.formato_corto()).collect();

        mensaje_error.push_str(&format!(
          "Usuario: {}: Fechas de marcajes: {}.\n",
          nombre_usuario,
          fechas_str.join(", ")
        ));
      }
      return Err(ServicioError::Validacion(
        mensaje_error.trim_end().to_string(),
      ));
    }

    Ok(())
  }

  /// Verifica fecha no entre en conflicto con un calendario asignado al usuario.
  ///
  /// Devuelve la entidad CalendarioFecha del conflicto si existe.
  pub async fn conflicto_calendario_en_marcaje(
    &self,
    usuario: u32,
    fecha: NaiveDate,
  ) -> Result<Option<CalendarioFecha>, ServicioError> {
    self
      .repo
      .conflicto_calendario_en_marcaje(usuario, fecha)
      .await
      .map_err(|err| {
        tracing::error!(
          fecha = ?fecha, error = %err,
          "Conflictos del marcaje y fechas de calendario");
        ServicioError::from(err)
      })
  }
}
