use chrono::NaiveDateTime;

use crate::{
  config::ConfigTrabajo,
  infra::ServicioError,
  usuarios::{Horario, HorarioRepo},
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
  /// Devuelve el horario del usuario.
  ///
  /// Si no se proporciona una hora, devuelve el horario del día actual.
  /// Si se proporciona una hora, devuelve el horario más cercano a esa hora.
  #[inline]
  pub async fn horario_usuario(
    &self,
    usuario: u64,
    hora: Option<NaiveDateTime>,
  ) -> Result<Vec<Horario>, ServicioError> {
    match hora {
      None => self
        .horario_repo
        .horarios_hoy_usuario(&self.cnfg.zona_horaria, usuario)
        .await
        .map_err(ServicioError::from),
      Some(hora) => self
        .horario_cercano(usuario, hora)
        .await
        .map(|horario| vec![horario]),
    }
  }
  /// Devuelve el horario más cercano al usuario.
  #[inline]
  pub async fn horario_cercano(
    &self,
    usuario: u64,
    hora: NaiveDateTime,
  ) -> Result<Horario, ServicioError> {
    self
      .horario_repo
      .horario_cercano(usuario, hora)
      .await
      .map_err(ServicioError::from)
  }
}
