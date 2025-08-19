use chrono::{NaiveDate, NaiveDateTime, NaiveTime};

use crate::usuarios::Usuario;

#[derive(Debug)]
pub struct Registro {
  pub usuario: Usuario,
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
