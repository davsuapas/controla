//! Gestiona la ifraestrcutura de la aplicación.

/// Módulo que contiene la loǵica de acceso a datos general
mod db;
/// Módulo que contiene la loǵica de servicios general
mod servicio;

/// Módulo que contiene macros útiles para la infraestructura
pub mod macros;

pub use db::*;
pub use servicio::*;
