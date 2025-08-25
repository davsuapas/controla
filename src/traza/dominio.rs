use chrono::{NaiveDateTime, Utc};
use chrono_tz::Tz;
use derive_builder::Builder;

#[repr(u8)]
#[derive(Copy, Clone)]
pub enum TipoTraza {
  RegistroEliminado = 1,
}

#[derive(Builder)]
#[builder(pattern = "owned", build_fn(private, name = "final_build"))]
pub struct Traza {
  pub tipo: TipoTraza,
  pub usuario_id: u32,
  pub fecha: NaiveDateTime,
  pub motivo: Option<String>,
  pub horario_id: Option<u32>,
  pub registro_id: Option<u32>,
}

impl TrazaBuilder {
  pub fn build(mut self, tz: &Tz) -> Result<Traza, TrazaBuilderError> {
    self.fecha = Some(Utc::now().with_timezone(tz).naive_local());
    self.final_build()
  }
}
