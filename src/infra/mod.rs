//! Gestiona la ifraestructura de la aplicación.
//! Servicios comunes

/// Módulo que contiene la loǵica de acceso a datos general
mod db;
/// Módulo para manejar tipos genéricos del dominio
mod dominio;
/// Módulo que contiene la loǵica de servicios general
mod servicio;

/// Módulo que contiene la loǵica de middlewares (Autenticación web, etc)
pub mod middleware;

//pub use app::*;
pub use db::*;
pub use dominio::*;
pub use servicio::*;
