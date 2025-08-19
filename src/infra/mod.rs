//! Gestiona la ifraestructura de la aplicación.
//! Servicios comunes

/// Módulo que contiene la lógica común a la capa de aplicación
mod app;
/// Módulo que contiene la loǵica de acceso a datos general
mod db;
/// Módulo que contiene la loǵica de servicios general
mod servicio;

/// Módulo que contiene macros útiles para la infraestructura
pub mod macros;

//pub use app::*;
pub use db::*;
pub use servicio::*;
