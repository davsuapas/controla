//! Gestiona la api de acceso a la aplicación

/// Módulo que define las rutas de acceso
mod api;
/// Entidades de intercambio con la aplicación
mod dto;

use std::{sync::Arc, time::Duration};

pub use api::*;

use crate::{
  config::ConfigTrabajo,
  infra::{PoolConexion, middleware},
  marcaje::{MarcajeRepo, MarcajeServicio},
  traza::{TrazaRepo, TrazaServicio},
  usuarios::{UsuarioRepo, UsuarioServicio},
};

/// Estructura principal de la aplicación que contiene los servicios.
pub struct AppState {
  pub manejador_sesion: Arc<middleware::ManejadorSesion>,
  pub reg_servicio: MarcajeServicio,
  pub usuario_servicio: UsuarioServicio,
}

impl AppState {
  /// Inicia la aplicación y devuelve una instancia de `App`.
  pub fn iniciar(cnfg: &ConfigTrabajo, pool: PoolConexion) -> Self {
    AppState {
      manejador_sesion: Arc::new(middleware::ManejadorSesion::new(
        cnfg.secreto.clone(),
        Duration::from_secs(cnfg.caducidad_sesion),
        cnfg.produccion,
      )),
      usuario_servicio: UsuarioServicio::new(
        cnfg.clone(),
        UsuarioRepo::new(pool.clone()),
        TrazaServicio::new(TrazaRepo::new()),
      ),
      reg_servicio: MarcajeServicio::new(
        cnfg.clone(),
        MarcajeRepo::new(pool.clone()),
        UsuarioServicio::new(
          cnfg.clone(),
          UsuarioRepo::new(pool.clone()),
          TrazaServicio::new(TrazaRepo::new()),
        ),
      ),
    }
  }
}
