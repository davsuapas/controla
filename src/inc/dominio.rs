use chrono::{NaiveDate, NaiveDateTime, NaiveTime};

use crate::marcaje::DescriptorMarcaje;

// Si se modiifca esta enumeración, hay que modificar también
// la enumeración equivalente en web/src/modelos/incidencias.ts
#[repr(u8)]
#[derive(Copy, Clone, Debug)]
pub enum EstadoIncidencia {
  Solicitud = 1,
  /// Cuando se registra el nuevo marcaje o la modificación puede
  /// haber errores de validación. En este caso se queda como
  /// inconsistente para la posterior correción del usuario
  #[allow(dead_code)]
  Inconsistente = 2,
  /// Son posibles errores del sistema
  #[allow(dead_code)]
  ErrorInterno = 3,
  #[allow(dead_code)]
  Rechazada = 4,
  #[allow(dead_code)]
  Resuelta = 5,
}

impl From<u8> for EstadoIncidencia {
  fn from(value: u8) -> Self {
    match value {
      1 => EstadoIncidencia::Solicitud,
      2 => EstadoIncidencia::Inconsistente,
      3 => EstadoIncidencia::ErrorInterno,
      4 => EstadoIncidencia::Rechazada,
      5 => EstadoIncidencia::Resuelta,
      _ => panic!("Valor de estado de incidencia no válido"),
    }
  }
}

// Si se modiifca esta enumeración, hay que modificar también
// la enumeración equivalente en web/src/modelos/incidencias.ts
#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum TipoIncidencia {
  NuevoMarcaje = 1,
  EliminacionMarcaje = 2,
  CorrecionSalida = 3,
}

impl From<u8> for TipoIncidencia {
  fn from(value: u8) -> Self {
    match value {
      1 => TipoIncidencia::NuevoMarcaje,
      2 => TipoIncidencia::EliminacionMarcaje,
      3 => TipoIncidencia::CorrecionSalida,
      _ => panic!("Valor de Tipo de incidencia no válido"),
    }
  }
}

#[derive(Debug)]
pub struct Incidencia {
  #[allow(dead_code)]
  pub id: u32,
  pub tipo: TipoIncidencia,
  pub fecha_solicitud: NaiveDateTime,
  pub fecha: NaiveDate,
  pub hora_inicio: Option<NaiveTime>,
  pub hora_fin: Option<NaiveTime>,
  pub marcaje: Option<DescriptorMarcaje>,
  pub estado: EstadoIncidencia,
  pub error: Option<String>,
  pub usuario_creador: u32,
  pub usuario_gestor: Option<u32>,
  pub motivo_solicitud: Option<String>,
}
