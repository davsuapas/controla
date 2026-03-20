use chrono::{NaiveDate, Utc};

use crate::{
  config::ConfigTrabajo,
  horario::{
    Calendario, CalendarioFecha, ConfigHorario, DescriptorHorario, HorarioRepo,
  },
  infra::{DBError, ServicioError, ShortDateTimeFormat},
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
  /// Devuelve el horario del usuario más cercano.
  ///
  /// Si no devuelve ningún horario se tracea el error y
  /// se devuelve una array vacío
  pub async fn horario_usuario_cercano(
    &self,
    usuario: u32,
    fecha: NaiveDate,
  ) -> Result<Option<DescriptorHorario>, ServicioError> {
    tracing::debug!(
      usuario = usuario,
      hora = %fecha,
      "Consultando el horario más cercano del usuario");

    let res = self.repo.horario_cercano(usuario, fecha).await;
    match res {
      Ok(horario) => Ok(Some(horario)),
      Err(err) => match err {
        DBError::RegistroVacio(e) => {
          tracing::warn!(
            error = %e,
           "Consulta horario cercano");
          Ok(None)
        }
        _ => {
          tracing::error!(
            usuario = usuario,
            hora = ?fecha,
            error = %err,
          "Consulta horario cercano");
          Err(ServicioError::from(err))
        }
      },
    }
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

    let id_config = match self.repo.agregar_config_usuario(config_horario).await
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

    if let Err(err) = self.repo.modificar_config_usuario(config_horario).await {
      tracing::error!(
        config_horario = ?config_horario,
        error = %err,
        "Modificando configuración de horario en bd"
      );
      return Err(ServicioError::from(err));
    }

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

    if let Err(err) = self.repo.eliminar_config_usuario(id_config).await {
      tracing::error!(
        id_config = id_config,
        error = %err,
        "Eliminando configuración de horario en bd"
      );
      return Err(ServicioError::from(err));
    }

    tracing::debug!(
      id_config = id_config,
      "Se ha completado satisfactoriamente la eliminación de la configuración de horario"
    );

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
