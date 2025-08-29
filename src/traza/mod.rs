//! Módulo para gestionar la auditoría del sistema.
//!
//! Las trazas registran los eventos provocados por los usuarios.
//! Cada traza contiene información sobre el tipo de evento,
//! la fecha en que ocurrió, el usuario que lo provocó, y
//! opcionalmente, el horario y registro afectados, así como
//! un motivo descriptivo del evento.

/// Módulo que gestiona el repositorio de las trazas.
mod repo;

/// Módulo que gestiona el dominio de las trazas.
mod dominio;
/// Módulo que gestiona el servicio de las trazas.
mod servicio;

pub use dominio::*;
pub use repo::*;
pub use servicio::*;
