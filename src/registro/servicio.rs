use crate::{
  config::ConfigTrabajo,
  infraestructura::ServicioError,
  registro::{Registro, RegistroRepo, TipoTraza, Traza, TrazaRepo},
  usuarios::{Usuario, UsuarioServicio},
};

/// Servicio que gestiona los registros del usuario
pub struct RegistroServicio {
  config: ConfigTrabajo,
  repo: RegistroRepo,
  traza_repo: TrazaRepo,
  usuario_servico: UsuarioServicio,
}

impl RegistroServicio {
  pub fn new(
    config: ConfigTrabajo,
    repo: RegistroRepo,
    traza_repo: TrazaRepo,
    usuario_servico: UsuarioServicio,
  ) -> Self {
    RegistroServicio {
      config,
      repo,
      traza_repo,
      usuario_servico,
    }
  }
}

impl RegistroServicio {
  /// Añade un nuevo registro para el usuario.
  ///
  /// Para calcular las horas a trabajar utiliza el horario más
  /// cercano a la hora de inicio del registro.
  /// Si el usuario no tiene un horario configurado, se devuelve un error.
  /// Si la hora de inicio o fin ya están asignadas al usuario,
  /// se devuelve un error de validación.
  /// Si el usuario que añade el registro es diferente al usuario del registro,
  /// se añade una traza de registro.
  ///
  /// Devuelve el ID del registro creado.
  pub async fn agregar(
    &self,
    usuario_log: &Usuario,
    reg: Registro,
  ) -> Result<u64, ServicioError> {
    tracing::info!(
      registro = ?reg,
      "Se ha iniciado el servicio para crear un registro horario de usuario");

    let horario = self
      .usuario_servico
      .horario_cercano(reg.usuario.id, reg.hora_inicio_completa())
      .await
      .inspect_err(|err| {
        tracing::error!(
          registro = ?reg,
          error = %err,
         "Buscando el horario más cercano cuando se añade un registro");
      })?;

    let horas_a_trabajar = horario.horas_a_trabajar().num_hours() as u8;
    let mut reg_completo = reg;
    reg_completo.horas_a_trabajar = horas_a_trabajar;

    tracing::debug!(
      horario = ?horario,
      horas_a_trabajar = horas_a_trabajar,
      "Horario más cercano al registro del usuario");

    let mut trans = self.repo.conexion().empezar_transaccion().await?;

    let reg_id = match self.repo.agregar(&mut trans, &reg_completo).await {
      Ok(reg_id) => reg_id,
      Err(err) => {
        trans.rollback().await?;
        tracing::error!(
          registro = ?reg_completo,
          error = %err,
          "Creando registro"
        );
        return Err(ServicioError::from(err));
      }
    };

    if *usuario_log != reg_completo.usuario {
      // Si es el mismo usuario ahoramos espacio en la base de datos
      let traza = Traza::with_timezone(
        self.config.zona_horaria,
        reg_id,
        usuario_log.id,
        TipoTraza::Registrado,
      );

      match self.traza_repo.agregar(&mut trans, &traza).await {
        Ok(reg) => reg,
        Err(err) => {
          trans.rollback().await?;
          tracing::error!(
            regsitro = ?reg_completo,
            usuario_logeado = usuario_log.nombre,
            error = %err,
            "Añadiendo traza de registro horario creado"
          );
          return Err(ServicioError::from(err));
        }
      };
    }

    trans.commit().await?;

    tracing::debug!(
      id_registro = reg_id,
      "Se ha completado satisfactoriamente el registro horario"
    );

    Ok(reg_id)
  }
}
