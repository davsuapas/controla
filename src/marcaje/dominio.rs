use chrono::{NaiveDate, NaiveDateTime, NaiveTime};

use crate::usuarios::Horario;

#[derive(Debug)]
pub struct DescriptorMarcaje {
  pub id: u32,
}

#[derive(Debug)]
pub struct Marcaje {
  pub id: u32,
  pub usuario: u32,
  pub usuario_reg: Option<u32>,
  pub horario: Option<Horario>,
  pub fecha: NaiveDate,
  pub hora_inicio: NaiveTime,
  pub hora_fin: Option<NaiveTime>,
}

impl Marcaje {
  #[inline]
  pub fn hora_inicio_completa(&self) -> NaiveDateTime {
    NaiveDateTime::new(self.fecha, self.hora_inicio)
  }

  #[inline]
  pub fn horas_trabajadas(&self) -> Option<f64> {
    self.hora_fin.map(|fin| {
      let diferencia = fin - self.hora_inicio;
      diferencia.num_milliseconds() as f64 / 3_600_000.0
    })
  }
}
