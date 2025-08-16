use thiserror::Error;

use crate::infra::DBError;

#[derive(Debug, Error)]
pub enum ServicioError {
  #[error("Error de acceso a la base de datos: {0}")]
  DB(#[from] DBError),
  #[error("{0}")]
  Usuario(String),
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
