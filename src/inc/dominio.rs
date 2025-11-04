use chrono::{NaiveDate, NaiveDateTime, NaiveTime};

use crate::marcaje::DescriptorMarcaje;

// Si se modifica esta enumeración, hay que modificar también
// la enumeración equivalente en web/src/modelos/incidencias.ts
#[repr(u8)]
#[derive(Copy, Clone, Debug)]
pub enum EstadoIncidencia {
  Solicitud = 1,
  /// Cuando se registra el nuevo marcaje puede
  /// haber errores de validación. En este caso se queda como
  /// conflicto para la posterior correción del usuario
  Conflicto = 2,
  /// Son posibles errores del sistema
  ErrorResolver = 3,
  Rechazada = 4,
  Resuelta = 5,
  /// Todas estos estados posteriores son acciones
  Resolver = 6,
  Rechazar = 7,
}

impl From<u8> for EstadoIncidencia {
  fn from(value: u8) -> Self {
    match value {
      1 => EstadoIncidencia::Solicitud,
      2 => EstadoIncidencia::Conflicto,
      3 => EstadoIncidencia::ErrorResolver,
      4 => EstadoIncidencia::Rechazada,
      5 => EstadoIncidencia::Resuelta,
      6 => EstadoIncidencia::Resolver,
      7 => EstadoIncidencia::Rechazar,
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
  pub id: u32,
  pub tipo: TipoIncidencia,
  pub fecha_solicitud: NaiveDateTime,
  pub fecha_resolucion: Option<NaiveDateTime>,
  pub usuario: u32,
  pub fecha: NaiveDate,
  pub hora_inicio: Option<NaiveTime>,
  pub hora_fin: Option<NaiveTime>,
  pub marcaje: Option<DescriptorMarcaje>,
  pub estado: EstadoIncidencia,
  // La fecha solo para estados intermedios, no para solicitud ni resolución
  pub fecha_estado: Option<NaiveDateTime>,
  pub error: Option<String>,
  pub usuario_creador: u32,
  pub usuario_gestor: Option<u32>,
  pub motivo_solicitud: Option<String>,
  pub motivo_rechazo: Option<String>,
}

// Define la entidad mínima necesaria para realizar una incidencia de marcaje.
#[derive(Debug)]
pub struct IncidenciaMarcaje {
  pub tipo: TipoIncidencia,
  pub usuario: u32,
  pub fecha: NaiveDate,
  pub hora_inicio: Option<NaiveTime>,
  pub hora_fin: Option<NaiveTime>,
  pub marcaje: Option<DescriptorMarcaje>,
  pub usuario_creador: u32,
}

// Define la entidad con la info para procesar las incidencias.
#[derive(Debug)]
pub struct IncidenciaProceso {
  pub id: u32,
  pub estado: EstadoIncidencia,
  pub motivo_rechazo: Option<String>,
}
