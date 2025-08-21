use crate::{
  infra::{ServicioError, ShortDateFormat},
  registro::{Registro, RegistroRepo},
  usuarios::{Usuario, UsuarioServicio},
};

/// Servicio que gestiona los registros del usuario
pub struct RegistroServicio {
  repo: RegistroRepo,
  usuario_servico: UsuarioServicio,
}

impl RegistroServicio {
  pub fn new(repo: RegistroRepo, usuario_servico: UsuarioServicio) -> Self {
    RegistroServicio {
      repo,
      usuario_servico,
    }
  }
}

impl RegistroServicio {
  /// Añade un nuevo registro horario para el usuario.
  ///
  /// Para calcular las horas a trabajar utiliza el horario más
  /// cercano a la hora de inicio del registro que todavía
  /// no haya sido asignado.
  ///
  /// Validaciones:
  /// * Si existen registros con alguna hora de fin sin registrar,
  /// * se devuelve un error.
  /// * Si el usuario no tiene un horario configurado, se devuelve un error.
  /// * Si la hora de inicio o fin ya están asignadas al usuario,
  ///   se devuelve un error.
  /// * El nuevo registro no se puede solapar con ningún otro registro.
  /// * La hora de inicio no sea anterior a la hora de fin
  ///   de un registro previo con un horario anterior al horario cercano
  ///   obtenido.
  ///
  /// Si el usuario que añade el registro es diferente al usuario del registro,
  /// se añade una traza de registro.
  ///
  /// Devuelve el ID del registro creado.
  pub async fn agregar(
    &self,
    usuario_log: &Usuario,
    reg: &Registro,
  ) -> Result<u64, ServicioError> {
    tracing::info!(
      registro = ?reg,
      "Se ha iniciado el servicio para crear un registro horario de usuario");

    self.validar_agregacion(reg).await?;

    let horario_cercano = self
      .usuario_servico
      .horario_cercano(reg.usuario.id, reg.hora_inicio_completa())
      .await
      .inspect_err(|err| {
        tracing::error!(
          registro = ?reg,
          error = %err,
         "Buscando el horario más cercano cuando se añade un registro");
      })?;

    // Validamos que la hora de inicio no sea anterior a la hora de fin
    // de un registro previo con un horario anterior al horario cercano
    // obtenido.
    if let Some(hora_fin_previa) = self
      .repo
      .hora_fin_previa(reg.usuario.id, reg.fecha, horario_cercano.hora_inicio)
      .await
      .map_err(ServicioError::from)?
    {
      if reg.hora_inicio < hora_fin_previa {
        return Err(ServicioError::Usuario(format!(
          "No se puede añadir un registro cuya hora de inicio: {} \
           es menor a un registro ya añadido cuya hora de fin fue: {} \
           para el usuario: {} y fecha: {}",
          reg.hora_inicio,
          hora_fin_previa,
          reg.usuario.nombre,
          reg.fecha.formato_corto()
        )));
      }
    }

    let horas_a_trabajar = horario_cercano.horas_a_trabajar().num_hours() as u8;

    tracing::debug!(
      horario = ?horario_cercano,
      horas_a_trabajar = horas_a_trabajar,
      "Horario más cercano al registro horario del usuario");

    let reg_id = match self
      .repo
      .agregar(reg, usuario_log.id, horario_cercano.id)
      .await
    {
      Ok(reg_id) => reg_id,
      Err(err) => {
        tracing::error!(
          registro = ?reg,
          error = %err,
          "Creando registro horario"
        );
        return Err(ServicioError::from(err));
      }
    };

    tracing::debug!(
      id_registro = reg_id,
      "Se ha completado satisfactoriamente el registro horario"
    );

    Ok(reg_id)
  }

  async fn validar_agregacion(
    &self,
    reg: &Registro,
  ) -> Result<(), ServicioError> {
    if self
      .repo
      .hora_fin_vacia(reg.usuario.id, reg.fecha)
      .await
      .map_err(ServicioError::from)?
    {
      return Err(ServicioError::Usuario(format!(
        "No puede se puede añadir un registro horario \
        con alguna hora de fin sin registrar \
        para el usuario: {} en la fecha: {}. \
        Por favor, registre antes la hora de fin.",
        reg.usuario.nombre,
        reg.fecha.formato_corto()
      )));
    }

    if self
      .repo
      .hora_asignada(reg.usuario.id, reg.fecha, reg.hora_inicio)
      .await
      .map_err(ServicioError::from)?
    {
      return Err(ServicioError::Usuario(format!(
        "La hora de inicio: {} se encuentra entre un rango de horas \
        ya registrado para el usuario: {} en la fecha: {}",
        reg.hora_inicio,
        reg.usuario.nombre,
        reg.fecha.formato_corto()
      )));
    }

    if let Some(hora_fin) = reg.hora_fin {
      let hora_asignada = self
        .repo
        .horas_solapadas(reg.usuario.id, reg.fecha, reg.hora_inicio, hora_fin)
        .await
        .map_err(ServicioError::from)?;

      if hora_asignada {
        return Err(ServicioError::Usuario(format!(
          "Ya existe un rango horario que se solapa con el \
          registro del usuario: {} en la fecha: {} desde: {} hasta: {}",
          reg.usuario.nombre,
          reg.fecha.formato_corto(),
          reg.hora_inicio,
          hora_fin
        )));
      }
    }

    Ok(())
  }
}
