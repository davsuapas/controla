//! Módulo para gestionar las trazas del sistema.
//!
//! Las trazas registran eventos mediante un mensaje y se
//! registran para una fecha y usuario.

mod dominio;
mod repo;

pub use dominio::*;
pub use repo::*;
