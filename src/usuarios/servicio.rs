use chrono::NaiveDateTime;

use crate::{
  infra::ServicioError,
  usuarios::{Horario, HorarioRepo},
};

///Servicio para manejar operaciones relacionadas con usuarios.
pub struct UsuarioServicio {
  horario_repo: HorarioRepo,
}

impl UsuarioServicio {
  pub fn new(horario_repo: HorarioRepo) -> Self {
    UsuarioServicio { horario_repo }
  }
}

impl UsuarioServicio {
  /// Devuelve el horario más cercano al usuario.
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
