//! Gestiona toda la funcionalidad asociada a los usuarios
//! incluyendo la autenticación, autorización, gestión de horarios
//! y calendarios asociados a los empleados.
//!
//! Los perfiles de usuarios podrán ser:
//! - Administrador
//! - Empleado
//! - Gestor de incidencias
//! - Registrador
//! - Inspector
//! - Director
//! - Supervisor
//!
//! La definición de cada rol se encuentra en: [`dominio::Rol`]

/// Módulo que gestiona el acceso a datos para los usuarios
mod repo;

/// Módulo que define el dominio de usuarios, roles y horarios
/// de los usuarios
mod dominio;
/// Módulo que expone los servicios del usuario
mod servicio;

pub use dominio::*;
pub use repo::*;
pub use servicio::*;
