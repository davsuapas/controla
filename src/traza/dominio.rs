use chrono::{NaiveDateTime, Utc};
use chrono_tz::Tz;
use derive_builder::Builder;

#[repr(u8)]
#[derive(Copy, Clone, Debug)]
pub enum TipoTraza {
  CreacionUsuario = 1,
  ActualizacionUsuario = 2,
  UsrDniModificado = 3,
  UsrNombreModificado = 4,
  UsrRolesModificados = 5,
  UsrActivoModificado = 6,
  PasswordModificada = 7,
  PrimerInicio = 8,
}

#[derive(Builder, Debug)]
#[builder(pattern = "owned", build_fn(private, name = "final_build"))]
pub struct Traza {
  pub autor: Option<u32>,
  pub tipo: TipoTraza,
  pub usuario_id: u32,
  pub fecha: NaiveDateTime,
  pub motivo: Option<String>,
  pub horario_id: Option<u32>,
  pub registro_id: Option<u32>,
}

impl TrazaBuilder {
  pub fn with_usuario(tipo: TipoTraza, usuario_id: u32) -> TrazaBuilder {
    TrazaBuilder::default()
      .autor(None)
      .tipo(tipo)
      .usuario_id(usuario_id)
      .registro_id(None)
      .horario_id(None)
      .motivo(None)
  }

  pub fn build(mut self, tz: &Tz) -> Traza {
    self.fecha = Some(Utc::now().with_timezone(tz).naive_local());
    self.final_build().expect("Error al formar traza")
  }
}
