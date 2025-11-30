use chrono::NaiveDate;

use crate::{
  config::ConfigTrabajo,
  infra::{
    DominioWithCacheUsuario, ServicioError, ShortDateTimeFormat, TimeConvert,
    Transaccion,
  },
  marcaje::{Marcaje, MarcajeRepo},
  usuarios::UsuarioServicio,
};

/// Servicio que gestiona los marcajes del usuario
pub struct MarcajeServicio {
  cnfg: ConfigTrabajo,
  repo: MarcajeRepo,
  usuario_servico: UsuarioServicio,
}

impl MarcajeServicio {
  pub fn new(
    cnfg: ConfigTrabajo,
    repo: MarcajeRepo,
    usuario_servico: UsuarioServicio,
  ) -> Self {
    MarcajeServicio {
      cnfg,
      repo,
      usuario_servico,
    }
  }
}

impl MarcajeServicio {
  #[inline]
  /// Añade un nuevo marcaje horario para el usuario.
  ///
  /// Para más detalles vea: [`agregar_with_trans`].
  pub async fn agregar(&self, reg: &Marcaje) -> Result<u32, ServicioError> {
    self.agregar_with_trans(None, reg, 0).await
  }
  /// Añade un nuevo marcaje horario para el usuario.
  ///
  /// Para calcular las horas a trabajar utiliza el horario más
  /// cercano a la hora de inicio del marcaje que todavía
  /// no haya sido asignado.
  ///
  /// Validaciones:
  /// * Si existen marcajes con alguna hora de fin sin registrar,
  /// * se devuelve un error.
  /// * Si el usuario no tiene un horario configurado, se devuelve un error.
  /// * Si la hora de inicio o fin ya están asignadas al usuario,
  ///   se devuelve un error.
  /// * El nuevo marcaje no se puede solapar con ningún otro marcaje.
  /// * La hora de inicio no puede ser anterior a la hora de fin
  ///   de un marcaje previo con un horario anterior al horario cercano
  ///   obtenido.
  ///
  /// Se puede excluir un marcaje pasado como parámetro
  /// Si no quiere excluir ningún marcaje use 0
  /// La exclusión puede ser muy útil cuando se quiere
  /// realizar una modificación de este marcaje
  ///
  /// Devuelve el ID del marcaje creado.
  pub async fn agregar_with_trans(
    &self,
    tr: Option<&mut Transaccion<'_>>,
    reg: &Marcaje,
    excluir_marcaje_id: u32,
  ) -> Result<u32, ServicioError> {
    tracing::info!(
      marcaje = ?reg,
      excluir_marcaje_id = excluir_marcaje_id,
      "Se ha iniciado el servicio para crear un marcaje horario de usuario");

    self.validar_agregacion(reg, excluir_marcaje_id).await?;

    let horario_cercano = self
      .usuario_servico
      .horario_cercano(
        reg.usuario,
        reg.hora_inicio_completa(),
        excluir_marcaje_id,
      )
      .await
      .inspect_err(|err| {
        tracing::error!(
          marcaje = ?reg,
          error = %err,
         "Buscando el horario más cercano cuando se añade un marcaje");
      })?;

    let horas_a_trabajar = horario_cercano.horas_a_trabajar();

    tracing::debug!(
      horario = ?horario_cercano,
      horas_a_trabajar = format!("{:.2}", horas_a_trabajar),
      "Horario más cercano a el marcajes horario del usuario");

    let id = match self.repo.agregar(tr, reg, horario_cercano.id).await {
      Ok(reg_id) => reg_id,
      Err(err) => {
        tracing::error!(
          marcaje = ?reg,
          error = %err,
          "Creando marcaje horario"
        );
        return Err(ServicioError::from(err));
      }
    };

    tracing::debug!(
      id_marcaje = id,
      "Se ha completado satisfactoriamente el marcaje horario"
    );

    Ok(id)
  }

  /// Actualiza el campo modificado_por del marcaje
  ///
  /// Devuelve True si se actualizo
  pub async fn actualizar_modificado_por(
    &self,
    trans: &mut Transaccion<'_>,
    id: u32,
    modificar_por: u32,
  ) -> Result<bool, ServicioError> {
    self
      .repo
      .actualizar_modificado_por(trans, id, modificar_por)
      .await
      .map_err(|err| {
        tracing::error!(
          id_marcaje = id,
          modificar_por = modificar_por,
          error = %err,
          "Actualizando modificado_por del marcaje"
        );
        ServicioError::from(err)
      })
  }

  /// Marca un marcaje como eliminado
  ///
  /// Devuelve True si se actualizo
  pub async fn marcar_marcaje_eliminado(
    &self,
    trans: &mut Transaccion<'_>,
    id: u32,
  ) -> Result<bool, ServicioError> {
    self
      .repo
      .marcar_marcaje_eliminado(trans, id)
      .await
      .map_err(|err| {
        tracing::error!(
          id_marcaje = id,
          error = %err,
          "Marcando el marcaje como eliminado"
        );
        ServicioError::from(err)
      })
  }

  /// Registrar una marcaje como finalizado.
  ///
  /// Verifica que la hora de inicio del marcaje,
  /// cuya hora fin sea nula, sea anterior a la hora fin
  /// que se quiere registrar.
  /// Si el registro no existe devuelve un error.
  pub async fn finalizar_marcaje(
    &self,
    usuario: u32,
    fecha_hora_fin: chrono::NaiveDateTime,
  ) -> Result<(), ServicioError> {
    let hora_fin = fecha_hora_fin.time().to_short_time();

    tracing::info!(
      usuario = usuario,
      fecha_hora_fin = %fecha_hora_fin,
      "Iniciando el servicio para finalizar el marcaje del usuario"
    );

    match self
      .repo
      .marcaje_sin_hora_fin(usuario, fecha_hora_fin.date())
      .await
    {
      Ok(Some(marcaje)) => {
        if marcaje.hora_inicio.unwrap() >= hora_fin {
          return Err(ServicioError::Usuario(format!(
            "La hora de fin: {} debe ser posterior a la hora de inicio: {} \
            del marcaje sin finalizar para el usuario: {} en la fecha: {}",
            hora_fin,
            marcaje.hora_inicio.unwrap(),
            usuario,
            fecha_hora_fin.date().formato_corto()
          )));
        }

        if !self
          .repo
          .actualizar_hora_fin(marcaje.id, hora_fin)
          .await
          .map_err(|err| {
            tracing::error!(
              id_marcaje = marcaje.id,
              hora_fin = %fecha_hora_fin,
              error = %err,
              "Registrando la hora fin del marcaje para finalizarlo"
            );
            ServicioError::from(err)
          })?
        {
          return Err(ServicioError::Usuario(format!(
            "No existe ningún marcaje iniciado para el usuario: {} \
            en la fecha: {}. No se puede registrar la hora de salida.",
            usuario,
            fecha_hora_fin.date().formato_corto()
          )));
        }

        tracing::debug!(
          id_marcaje = marcaje.id,
          hora_fin = %hora_fin,
          "Se ha finalizado el marcaje auto del usuario correctamente"
        );

        Ok(())
      }
      Ok(None) => Err(ServicioError::Usuario(format!(
        "No existe ningún marcaje iniciado para el usuario: {} \
        en la fecha: {}. No se puede registrar la hora de salida.",
        usuario,
        fecha_hora_fin.date().formato_corto()
      ))),
      Err(err) => {
        tracing::error!(
          usuario = usuario,
          fecha = %fecha_hora_fin.date(),
          error = %err,
          "Buscando el marcaje sin finalizar para registrar la hora fin"
        );
        Err(ServicioError::from(err))
      }
    }
  }

  /// Obtiene los marcajes entre fechas para un usuario.
  ///
  /// Dependiendo del valor de usuario_reg se añaden más filtros
  /// Ver [`MarcajeRepo::marcajes_inc_por_fecha_reg`]
  /// para más información
  pub async fn marcajes_entre_fechas_reg(
    &self,
    usuario: u32,
    fecha_inicio: Option<NaiveDate>,
    fecha_fin: Option<NaiveDate>,
    usuario_reg: Option<u32>,
  ) -> Result<DominioWithCacheUsuario<Marcaje>, ServicioError> {
    tracing::debug!(
      usuario = usuario,
      fecha_inicio = ?fecha_inicio,
      fecha_fin = ?fecha_fin,
      usuario_reg = ?usuario_reg,
      "Obtiene los marcajes entre fechas para un usuario"
    );

    self
      .repo
      .marcajes_entre_fechas_reg(usuario, fecha_inicio, fecha_fin, usuario_reg)
      .await
      .map_err(|err| {
        tracing::error!(
          usuario = usuario,
          fecha_inicio = ?fecha_inicio,
          fecha_fin = ?fecha_fin,
          usuario_reg = ?usuario_reg,
          error = %err,
          "Obtiene los marcajes entre fechas para un usuario"
        );
        ServicioError::from(err)
      })
  }

  /// Obtiene los marcaje si no se han asignado a una incidencia.
  ///
  /// Dependiendo del valor de usuario_reg se añaden más filtros
  /// Ver [`crate::marcaje::MarcajeRepo::marcajes_inc_por_fecha_reg`]
  /// para más información
  pub async fn marcajes_inc_por_fecha_reg(
    &self,
    usuario: u32,
    fecha: NaiveDate,
    usuario_reg: Option<u32>,
  ) -> Result<DominioWithCacheUsuario<Marcaje>, ServicioError> {
    tracing::debug!(
      usuario = usuario,
      fecha = %fecha,
      usuario_reg = ?usuario_reg,
      "Obtiene los marcajes por fecha no asignados a incidencias"
    );

    self
      .repo
      .marcajes_inc_por_fecha_reg(usuario, fecha, usuario_reg)
      .await
      .map_err(|err| {
        tracing::error!(
          usuario = usuario,
          fecha = ?fecha,
          usuario_reg = ?usuario_reg,
          error = %err,
          "Obteniendo los marcajes por fecha sin incidencias"
        );
        ServicioError::from(err)
      })
  }

  // Determina si la hora fin esta vacía para un
  // un usuario y fecha de marcaje
  pub async fn hora_fin_vacia(
    &self,
    usuario: u32,
    fecha: NaiveDate,
  ) -> Result<bool, ServicioError> {
    return self
      .repo
      .hora_fin_vacia(usuario, fecha, 0)
      .await
      .map_err(ServicioError::from);
  }

  /// Obtiene el marcaje dado un usuario y la fecha
  pub async fn marcaje_por_fecha(
    &self,
    usuario: u32,
    fecha: NaiveDate,
  ) -> Result<DominioWithCacheUsuario<Marcaje>, ServicioError> {
    self
      .repo
      .marcajes_por_fecha(usuario, fecha)
      .await
      .map_err(|err| {
        tracing::error!(
          usuario = usuario,
          fecha = ?fecha,
          error = %err,
          "Obteniendo el marcaje por usuario y fecha"
        );
        ServicioError::from(err)
      })
  }

  /// Obtiene los últimos marcajes horarios de un usuario.
  pub async fn ultimos_marcajes(
    &self,
    usuario: u32,
  ) -> Result<DominioWithCacheUsuario<Marcaje>, ServicioError> {
    self
      .repo
      .ultimos_marcajes(
        usuario,
        Some(&self.cnfg.limites.ultimos_marcajes.to_string()),
      )
      .await
      .map_err(|err| {
        tracing::error!(
          usuario = usuario,
          error = %err,
          "Obteniendo los últimos marcaje horarios del usuario"
        );
        ServicioError::from(err)
      })
  }

  /// Valida añadir un nuevo marcaje
  ///
  /// Se puede excluir un marcaje pasado como parámetro
  /// Si no quiere excluir ningún marcaje use 0
  /// La exclusión puede ser muy útil cuando se quiere
  /// realizar una modificación de este marcaje
  async fn validar_agregacion(
    &self,
    reg: &Marcaje,
    excluir_marcaje_id: u32,
  ) -> Result<(), ServicioError> {
    if self
      .repo
      .hora_fin_vacia(reg.usuario, reg.fecha, excluir_marcaje_id)
      .await
      .map_err(ServicioError::from)?
    {
      return Err(ServicioError::Usuario(format!(
        "No puede se puede añadir un marcaje horario \
        con alguna hora de fin sin registrar \
        para el usuario: {} en la fecha: {}. \
        Por favor, registre antes la hora de fin.",
        &reg.usuario,
        &reg.fecha.formato_corto()
      )));
    }

    // Si la hora fin es nula el marcaje es automático
    if reg.hora_fin.is_none()
      && self
        .repo
        .hora_asignada_posterior(
          reg.usuario,
          reg.fecha,
          reg.hora_inicio,
          excluir_marcaje_id,
        )
        .await
        .map_err(ServicioError::from)?
    {
      return Err(ServicioError::Usuario(format!(
        "Existen un marcaje registrado posterior a la hora: {} \
        para el usuario: {} en la fecha: {}. No cree marcajes manuales \
        si registra marcajes automáticos.",
        reg.hora_inicio,
        &reg.usuario,
        &reg.fecha.formato_corto()
      )));
    }

    if self
      .repo
      .hora_asignada(
        reg.usuario,
        reg.fecha,
        reg.hora_inicio,
        excluir_marcaje_id,
      )
      .await
      .map_err(ServicioError::from)?
    {
      return Err(ServicioError::Usuario(format!(
        "La hora de inicio: {} se encuentra entre un rango de horas \
        ya registrado para el usuario: {} en la fecha: {}",
        reg.hora_inicio,
        &reg.usuario,
        &reg.fecha.formato_corto()
      )));
    }

    if let Some(hora_fin) = reg.hora_fin {
      let hora_asignada = self
        .repo
        .horas_solapadas(
          reg.usuario,
          reg.fecha,
          reg.hora_inicio,
          hora_fin,
          excluir_marcaje_id,
        )
        .await
        .map_err(ServicioError::from)?;

      if hora_asignada {
        return Err(ServicioError::Usuario(format!(
          "Ya existe un rango horario que se solapa con el \
          marcaje del usuario: {} en la fecha: {} desde: {} hasta: {}",
          &reg.usuario,
          &reg.fecha.formato_corto(),
          reg.hora_inicio,
          hora_fin
        )));
      }
    }

    Ok(())
  }
}
