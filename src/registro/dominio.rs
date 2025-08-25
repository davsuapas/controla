use chrono::{NaiveDate, NaiveDateTime, NaiveTime};

use crate::usuarios::{DescriptorUsuario, Horario, UsuarioNombre};

#[derive(Debug)]
pub struct Registro {
  pub usuario: UsuarioNombre,
  pub usuario_reg: Option<DescriptorUsuario>,
  pub horario: Option<Horario>,
  pub fecha: NaiveDate,
  pub hora_inicio: NaiveTime,
  pub hora_fin: Option<NaiveTime>,
}

impl Registro {
  #[inline]
  pub fn hora_inicio_completa(&self) -> NaiveDateTime {
    NaiveDateTime::new(self.fecha, self.hora_inicio)
  }
}
