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
  IncConflicto = 9,
  IncReSolictar = 10,
}

#[repr(u8)]
#[derive(Copy, Clone, Debug)]
pub enum Entidad {
  Usuario = 1,
  Incidencia = 2,
}

#[derive(Builder, Debug)]
#[builder(pattern = "owned", build_fn(private, name = "final_build"))]
pub struct Traza {
  pub autor: Option<u32>,
  pub tipo: TipoTraza,
  pub entidad: Entidad,
  pub entidad_id: u32,
  pub fecha: NaiveDateTime,
  pub motivo: Option<String>,
}

impl TrazaBuilder {
  pub fn with_usuario(tipo: TipoTraza, id: u32) -> TrazaBuilder {
    TrazaBuilder::default()
      .autor(None)
      .tipo(tipo)
      .entidad(Entidad::Usuario)
      .entidad_id(id)
      .motivo(None)
  }

  pub fn with_inc(tipo: TipoTraza, id: u32) -> TrazaBuilder {
    TrazaBuilder::default()
      .autor(None)
      .tipo(tipo)
      .entidad(Entidad::Incidencia)
      .entidad_id(id)
      .motivo(None)
  }

  pub fn build(mut self, tz: &Tz) -> Traza {
    self.fecha = Some(Utc::now().with_timezone(tz).naive_local());
    self.final_build().expect("Error al formar traza")
  }
}
