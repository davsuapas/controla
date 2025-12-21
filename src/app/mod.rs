//! Gestiona la api de acceso a la aplicación

/// Módulo que define las rutas de acceso
mod api;
/// Entidades de intercambio con la aplicación
mod dto;

use std::{sync::Arc, time::Duration};

pub use api::*;

use crate::{
  config::{Config, ConfigTrabajo},
  inc::{IncidenciaRepo, IncidenciaServicio},
  infra::{PoolConexion, middleware},
  marcaje::{MarcajeRepo, MarcajeServicio},
  traza::{TrazaRepo, TrazaServicio},
  usuarios::{UsuarioRepo, UsuarioServicio},
};

/// Estructura principal de la aplicación que contiene los servicios.
pub struct AppState {
  pub manejador_sesion: Arc<middleware::ManejadorSesion>,
  pub marcaje_servicio: MarcajeServicio,
  pub usuario_servicio: UsuarioServicio,
  pub inc_servicio: IncidenciaServicio,
}

impl AppState {
  /// Inicia la aplicación y devuelve una instancia de `App`.
  pub fn iniciar(cnfg: &ConfigTrabajo, pool: PoolConexion) -> Self {
    // Aunque se realizan varias clonaciones de los servicios,
    // estos son ligeros y no suponen un gran coste.
    // Además, solo se hacen al iniciar la aplicación.
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
      marcaje_servicio: MarcajeServicio::new(
        cnfg.clone(),
        MarcajeRepo::new(pool.clone()),
        UsuarioServicio::new(
          cnfg.clone(),
          UsuarioRepo::new(pool.clone()),
          TrazaServicio::new(TrazaRepo::new()),
        ),
      ),
      inc_servicio: IncidenciaServicio::new(
        cnfg.clone(),
        IncidenciaRepo::new(pool.clone()),
        TrazaServicio::new(TrazaRepo::new()),
        MarcajeServicio::new(
          cnfg.clone(),
          MarcajeRepo::new(pool.clone()),
          UsuarioServicio::new(
            cnfg.clone(),
            UsuarioRepo::new(pool.clone()),
            TrazaServicio::new(TrazaRepo::new()),
          ),
        ),
      ),
    }
  }
}

/// Lanza los procesos de inicio de la aplicación
///
/// Inenta crear el usuario administrador inicial
/// si se encuentra configurado para ello.
pub async fn lanzar_procesos_inicio(config: &Config, app: &AppState) {
  if config.boot_admin.crear {
    drop(
      app
        .usuario_servicio
        .crear_admin(&config.boot_admin)
        .await
        .inspect_err(
          |err| tracing::error!(error = %err, "Creando usuario administrador"),
        ),
    );
  }
}
