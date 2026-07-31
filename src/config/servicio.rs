use crate::infra::ServicioError;

use super::dominio::ConfigGlobal;
use super::repo::ConfigRepo;

/// Servicio para manejar operaciones relacionadas con la configuración global.
pub struct ConfigServicio {
  repo: ConfigRepo,
}

impl ConfigServicio {
  pub fn new(repo: ConfigRepo) -> Self {
    ConfigServicio { repo }
  }
}

impl ConfigServicio {
  /// Devuelve la configuración global de la aplicación.
  pub async fn data(&self) -> Result<ConfigGlobal, ServicioError> {
    tracing::debug!("Obteniendo configuración general");

    self.repo.data().await.map_err(|err| {
      tracing::error!(error = %err, "Obteniendo configuración general");
      ServicioError::from(err)
    })
  }

  /// Actualiza la configuración global de la aplicación.
  pub async fn actualizar(&self, config: &ConfigGlobal) -> Result<(), ServicioError> {
    tracing::debug!("Actualizando configuración general");

    self.repo.actualizar(config).await.map_err(|err| {
      tracing::error!(error = %err, "Actualizando configuración general");
      ServicioError::from(err)
    })
  }
}
