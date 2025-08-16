use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use chrono_tz::Tz;

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

pub struct Traza {
  pub reg_id: u64,
  pub user_id: u64,
  pub fecha: NaiveDateTime,
  pub tipo: TipoTraza,
}

impl Traza {
  pub fn with_timezone(
    tz: Tz,
    reg_id: u64,
    user_id: u64,
    tipo: TipoTraza,
  ) -> Self {
    Traza {
      reg_id,
      user_id,
      fecha: chrono::Utc::now().with_timezone(&tz).naive_local(),
      tipo,
    }
  }
}

#[repr(u8)]
#[derive(Clone, Copy)]
pub enum TipoTraza {
  Registrado = 1,
}

impl TipoTraza {
  /// Hay ciertos processos que requieren una conversión explicita
  #[inline]
  pub fn as_u8(self) -> u8 {
    self as u8
  }
}
