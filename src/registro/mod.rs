/// Módulo que gestiona el registro de los empleados
pub mod repo;

pub mod dominio;
pub mod servicio;

pub use dominio::*;
pub use repo::*;
pub use servicio::*;
