//! Gestiona la api de acceso a la aplicación

/// Módulo que define las rutas de acceso
mod api;
/// Entidades de intercambio con la aplicación
mod dto;

pub use api::*;

use crate::{
  config::ConfigTrabajo,
  infra::PoolConexion,
  registro::{RegistroRepo, RegistroServicio},
  traza::{TrazaRepo, TrazaServicio},
  usuarios::{UsuarioRepo, UsuarioServicio},
};

/// Estructura principal de la aplicación que contiene los servicios.
pub struct AppState {
  pub reg_servicio: RegistroServicio,
  pub usuario_servicio: UsuarioServicio,
}

impl AppState {
  /// Inicia la aplicación y devuelve una instancia de `App`.
  pub fn iniciar(cnfg: &ConfigTrabajo, pool: PoolConexion) -> Self {
    AppState {
      usuario_servicio: UsuarioServicio::new(
        cnfg.clone(),
        UsuarioRepo::new(pool.clone()),
        TrazaServicio::new(TrazaRepo::new()),
      ),
      reg_servicio: RegistroServicio::new(
        cnfg.clone(),
        RegistroRepo::new(pool.clone()),
        UsuarioServicio::new(
          cnfg.clone(),
          UsuarioRepo::new(pool.clone()),
          TrazaServicio::new(TrazaRepo::new()),
        ),
      ),
    }
  }
}
