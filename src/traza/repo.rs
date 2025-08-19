use crate::{
  infra::{DBError, Transaccion},
  traza::Traza,
};

/// Implementación del repositorio de trazas.
pub struct TrazaRepo {}

impl TrazaRepo {
  pub fn new() -> Self {
    TrazaRepo {}
  }
}

impl TrazaRepo {
  /// Agrega una nueva traza a la base de datos.
  pub async fn agregar(
    &self,
    trans: &mut Transaccion<'_>,
    traza: &Traza,
  ) -> Result<u64, DBError> {
    let result = sqlx::query(
      r"INSERT INTO trazas
      (usuario, fecha, mensaje)
      VALUES (?, ?, ?)",
    )
    .bind(traza.usuario_id)
    .bind(traza.fecha)
    .bind(traza.mensaje.as_str())
    .execute(&mut **trans.deref_mut())
    .await
    .map_err(DBError::consulta_from)?;

    Ok(result.last_insert_id())
  }
}
