use thiserror::Error;

#[derive(Debug, Error)]
pub enum DBError {
  #[error("Error cuando se trabaja con transacciones: {0}")]
  Transaccion(anyhow::Error),
  #[error("Error en la consulta: {0}")]
  Consulta(anyhow::Error),
  #[error("Registro/s inexistente/s: {0}")]
  RegistroVacio(String),
  #[error("Error encriptando o desencriptando un campo: {0}")]
  Criptografia(anyhow::Error),
  #[error("Parámetros de la consulta: {0}")]
  Parametros(&'static str),
  #[error("{0}")]
  ConstraintViolation(String),
}

impl DBError {
  pub fn trans_from<E: Into<anyhow::Error>>(err: E) -> Self {
    DBError::Transaccion(err.into())
  }

  pub fn registro_vacio(msg: String) -> Self {
    DBError::RegistroVacio(msg)
  }

  pub fn cripto_from<E: Into<anyhow::Error>>(err: E) -> Self {
    DBError::Criptografia(err.into())
  }

  /// Convierte errores de SQLx detectando violaciones de constraints
  pub fn from_sqlx(err: sqlx::Error) -> Self {
    match &err {
      sqlx::Error::Database(dberr) => {
        if dberr.is_foreign_key_violation() {
          let msg = if let Some(constraint) = dberr.constraint() {
            format!(
              "No se puede eliminar o actualizar: 
              existen registros relacionados ({})",
              constraint
            )
          } else {
            "No se puede eliminar o actualizar: existen registros relacionados"
              .to_string()
          };
          DBError::ConstraintViolation(msg)
        } else if dberr.is_check_violation() {
          let msg = if let Some(constraint) = dberr.constraint() {
            format!("Validación de datos fallida: {}", constraint)
          } else {
            "Los datos no cumplen con las validaciones".to_string()
          };
          DBError::ConstraintViolation(msg)
        } else if dberr.is_unique_violation() {
          let msg = if let Some(constraint) = dberr.constraint() {
            format!("El registro ya existe: {}", constraint)
          } else {
            "El registro ya existe".to_string()
          };
          DBError::ConstraintViolation(msg)
        } else {
          DBError::Consulta(anyhow::anyhow!(err))
        }
      }
      _ => DBError::Consulta(anyhow::anyhow!(err)),
    }
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
