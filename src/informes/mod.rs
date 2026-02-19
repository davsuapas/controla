//! Generador de informes
//!
//! Informes:
//!
//! - Informe de cumplimiento horario: Tiene como objetivo principal generar
//!   un balance mensual detallado que compara la jornada laboral teórica de
//!   un empleado contra su jornada real registrada.
mod repo;

/// Módulo que define el dominio para las entidades de los informes
mod dominio;
/// Módulo con los servicios para los informes
mod servicio;

pub use dominio::*;
pub use repo::*;
pub use servicio::*;
