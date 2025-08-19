use chrono::{NaiveDate, NaiveDateTime};
use thiserror::Error;

use crate::infra::DBError;

#[derive(Debug, Error)]
pub enum ServicioError {
  #[error("Error de acceso a la base de datos: {0}")]
  DB(#[from] DBError),
  #[error("{0}")]
  Usuario(String),
}

impl ServicioError {
  /// Si hay mensajes de error para el usuario devuelve la
  /// cadena con el mensaje si no devuelve una cadena vacía
  pub fn mensaje_usuario(&self) -> String {
    match self {
      ServicioError::Usuario(msg) => msg.clone(),
      ServicioError::DB(db_error) => match db_error {
        DBError::RegistroVacio(e) => e.to_string(),
        _ => "".to_string(),
      },
    }
  }
}

/// Dada una fecha y hora, devuelve el día de la semana como una letra.
/// Lunes es 'L', Martes es 'M', Miércoles es 'X' y así sucesivamente.
pub fn letra_dia_semana(dia_semana: chrono::Weekday) -> char {
  match dia_semana {
    chrono::Weekday::Mon => 'L',
    chrono::Weekday::Tue => 'M',
    chrono::Weekday::Wed => 'X',
    chrono::Weekday::Thu => 'J',
    chrono::Weekday::Fri => 'V',
    chrono::Weekday::Sat => 'S',
    chrono::Weekday::Sun => 'D',
  }
}

pub trait ShortDateFormat {
  /// Devuelve la fecha en formato corto "dd/mm/yyyy".
  fn formato_corto(&self) -> String;
}

pub trait ShortDateTimeFormat {
  /// Devuelve la fecha y hora en formato corto "dd/mm/yyyy HH:MM".
  fn formato_corto(&self) -> String;
}

impl ShortDateFormat for NaiveDate {
  fn formato_corto(&self) -> String {
    self.format("%d/%m/%Y").to_string()
  }
}

impl ShortDateTimeFormat for NaiveDateTime {
  fn formato_corto(&self) -> String {
    self.format("%d/%m/%Y %H:%M").to_string()
  }
}
