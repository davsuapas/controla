use thiserror::Error;

#[derive(Debug, Error)]
pub enum DBError {
  #[error("Error cuando se trabaja con transacciones: {0}")]
  Transaccion(anyhow::Error),

  #[error("Error en la consulta: {0}")]
  Consulta(anyhow::Error),

  #[error("No hay filas afectadas: {0}")]
  RegistroVacio(anyhow::Error),

  #[error("No cumple la validación: {0}")]
  ValidacionIncorrecta(anyhow::Error),
}

impl DBError {
  pub fn consulta_from<E: Into<anyhow::Error>>(err: E) -> Self {
    DBError::Consulta(err.into())
  }

  pub fn trans_from<E: Into<anyhow::Error>>(err: E) -> Self {
    DBError::Transaccion(err.into())
  }

  pub fn registro_vacio(msg: String) -> Self {
    DBError::RegistroVacio(anyhow::anyhow!(msg))
  }

  pub fn validacion_incorrecta(msg: String) -> Self {
    DBError::ValidacionIncorrecta(anyhow::anyhow!(msg))
  }
}

/// Estructura que representa una conexión a la base de datos.
///
/// Controla las conexiones y las transacciones para que
/// no se use directamente el pool de conexiones.
/// Si alguna vez es necesario cambiar la implementación
/// de la base de datos, solo se debe cambiar aquí.
#[derive(Clone)]
pub struct PoolConexion {
  pool: sqlx::Pool<sqlx::MySql>,
}

impl PoolConexion {
  pub fn new(pool: sqlx::Pool<sqlx::MySql>) -> Self {
    PoolConexion { pool }
  }
}

impl PoolConexion {
  // Obtiene la conexión interna.
  pub fn conexion(&self) -> &sqlx::Pool<sqlx::MySql> {
    &self.pool
  }
  /// Empieza una nueva transacción.
  pub async fn empezar_transaccion(&self) -> Result<Transaccion<'_>, DBError> {
    let transaction = self.pool.begin().await.map_err(DBError::trans_from)?;
    Ok(Transaccion { transaction })
  }
}

/// Gestiona las tranasciones de la base de datos.
pub struct Transaccion<'a> {
  transaction: sqlx::Transaction<'a, sqlx::MySql>,
}

impl<'a> Transaccion<'a> {
  // Obtiene la transacción interna
  pub fn deref_mut(&mut self) -> &mut sqlx::Transaction<'a, sqlx::MySql> {
    &mut self.transaction
  }

  /// Realiza un commit de la transacción.
  pub async fn commit(self) -> Result<(), DBError> {
    self.transaction.commit().await.map_err(DBError::trans_from)
  }

  /// Deshace el commit de la transacción.
  pub async fn rollback(self) -> Result<(), DBError> {
    self
      .transaction
      .rollback()
      .await
      .map_err(DBError::trans_from)
  }
}
