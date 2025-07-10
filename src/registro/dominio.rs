use chrono::{NaiveDate, NaiveTime};

use crate::usuarios::dominio::Usuario;

pub struct Registro {
  pub usuario: Usuario,
  pub fecha: NaiveDate,
  pub hora_inicio: NaiveTime,
  pub hora_fin: NaiveTime,
}
