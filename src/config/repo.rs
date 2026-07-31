use sqlx::Row;

use crate::infra::{DBError, PoolConexion};

use super::dominio::{ConfigGlobal, Localizacion};

/// Implementación del repositorio de la configuración global.
pub struct ConfigRepo {
  pool: PoolConexion,
}

impl ConfigRepo {
  pub fn new(pool: PoolConexion) -> Self {
    ConfigRepo { pool }
  }
}

impl ConfigRepo {
  /// Obtiene la configuración global almacenada en base de datos.
  ///
  /// Consulta el único registro de la tabla `config` y si las columnas
  /// de localización no son nulas, construye
  /// una instancia de [`Localizacion`]. Si alguna columna es nula,
  /// devuelve la configuración sin localización.
  pub async fn data(&self) -> Result<ConfigGlobal, DBError> {
    const QUERY: &str =
      "SELECT lat, lng, accuracy, margen_recinto FROM config WHERE id = 1";

    let row = sqlx::query(QUERY)
      .fetch_optional(self.pool.conexion())
      .await
      .map_err(DBError::from_sqlx)?;

    if let Some(row) = row {
      let lat: Option<f64> = row.get("lat");
      let lng: Option<f64> = row.get("lng");
      let accuracy: Option<f64> = row.get("accuracy");
      let margen_recinto: Option<i32> = row.get("margen_recinto");

      let localizacion = match (lat, lng, accuracy) {
        (Some(lat), Some(lng), Some(accuracy)) => {
          Some(Localizacion { lat, lng, accuracy })
        }
        _ => None,
      };

      Ok(ConfigGlobal {
        localizacion,
        margen_recinto,
      })
    } else {
      Err(DBError::registro_vacio(
        "No se ha encontrado configuración".to_string(),
      ))
    }
  }

  /// Actualiza la configuración global en base de datos.
  ///
  /// Almacena o elimina la localización según si `config.localizacion`
  /// es `Some` o `None`. Si la localización está presente se guardan,
  /// si es `None` se establecen
  /// a NULL.
  pub async fn actualizar(&self, config: &ConfigGlobal) -> Result<(), DBError> {
    const QUERY: &str = "UPDATE config
       SET lat = ?, lng = ?, accuracy = ?, margen_recinto = ? WHERE id = 1";

    let (lat, lng, accuracy) = match &config.localizacion {
      Some(loc) => (Some(loc.lat), Some(loc.lng), Some(loc.accuracy)),
      None => (None, None, None),
    };

    let res = sqlx::query(QUERY)
      .bind(lat)
      .bind(lng)
      .bind(accuracy)
      .bind(config.margen_recinto)
      .execute(self.pool.conexion())
      .await
      .map_err(DBError::from_sqlx)?;

    if res.rows_affected() == 0 {
      Err(DBError::registro_vacio(
        "Actualizando configuración".to_string(),
      ))
    } else {
      Ok(())
    }
  }
}
