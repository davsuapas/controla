use chrono::NaiveDateTime;

use crate::{
  config::ConfigTrabajo,
  infra::ServicioError,
  usuarios::{DescriptorUsuario, Horario, HorarioRepo, Rol},
};

///Servicio para manejar operaciones relacionadas con usuarios.
pub struct UsuarioServicio {
  cnfg: ConfigTrabajo,
  horario_repo: HorarioRepo,
}

impl UsuarioServicio {
  pub fn new(cnfg: ConfigTrabajo, horario_repo: HorarioRepo) -> Self {
    UsuarioServicio { cnfg, horario_repo }
  }
}

impl UsuarioServicio {
  /// Devuelve los usuarios que tienen un rol específico.
  #[inline]
  pub async fn usuarios_por_rol(
    &self,
    rol: Rol,
  ) -> Result<Vec<DescriptorUsuario>, ServicioError> {
    self
      .horario_repo
      .usuarios_por_rol(rol)
      .await
      .map_err(|err| {
        tracing::error!(
          rol = ?rol,
          error = %err,
         "Consultando usuarios por rol");
        ServicioError::from(err)
      })
  }

  /// Devuelve el horario del usuario.
  ///
  /// Si no se proporciona una hora, devuelve el horario del día actual.
  /// Si se proporciona una hora, devuelve el horario más cercano a esa hora.
  pub async fn horario_usuario(
    &self,
    usuario: u32,
    hora: Option<NaiveDateTime>,
  ) -> Result<Vec<Horario>, ServicioError> {
    match hora {
      None => self
        .horario_repo
        .horarios_hoy_usuario(&self.cnfg.zona_horaria, usuario)
        .await
        .map_err(|err| {
          tracing::error!(
          usuario = usuario,
          error = %err,
         "Consultando horario del usuario para el día actual");
          ServicioError::from(err)
        }),
      Some(hora) => self
        .horario_cercano(usuario, hora)
        .await
        .map(|horario| vec![horario])
        .inspect_err(|err| {
          tracing::error!(
          usuario = usuario,
          hora = ?hora,
          error = %err,
         "Consultando horario del usuario para una fecha y hora concreta");
        }),
    }
  }
  /// Devuelve el horario más cercano al usuario.
  #[inline]
  pub async fn horario_cercano(
    &self,
    usuario: u32,
    hora: NaiveDateTime,
  ) -> Result<Horario, ServicioError> {
    self
      .horario_repo
      .horario_cercano(usuario, hora)
      .await
      .map_err(ServicioError::from)
  }
}
