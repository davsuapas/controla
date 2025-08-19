//! Gestiona los casos de uso para registro horario por
//! parte de los empleados y la gestión de incidencias

/// Módulo que gestiona el acceso a datos para registro de los empleados
mod repo;

/// Módulo que define el dominio para el registro horario
mod dominio;
/// Módulo con los servicios para el registro horario
mod servicio;

pub use dominio::*;
pub use repo::*;
pub use servicio::*;
