//! Gestiona los casos de uso para el marcaje horario por
//! parte de los empleados y la gestión de incidencias

/// Módulo que gestiona el acceso a datos para marcaje de los empleados
mod repo;

/// Módulo que define el dominio para el marcaje horario
mod dominio;
/// Módulo con los servicios para el marcaje horario
mod servicio;

pub use dominio::*;
pub use repo::*;
pub use servicio::*;
