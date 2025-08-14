use thiserror::Error;

use crate::infraestructura::DBError;

#[derive(Debug, Error)]
pub enum ServicioError {
  #[error("Error de acceso a la base de datos: {0}")]
  DB(#[from] DBError),
}

/// Dada una fecha y hora, devuelve el día de la semana como una letra.
/// Lunes es 'L', Martes es 'M', Miércoles es 'X' y así sucesivamente.
pub fn dia_semana_letra(dia_semana: chrono::Weekday) -> char {
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
