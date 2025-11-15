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
  ) -> Result<u32, DBError> {
    let result = sqlx::query(
      "INSERT INTO trazas
      (autor, tipo, fecha, entidad, entidad_id, motivo)
      VALUES (?, ?, ?, ?, ?, ?)",
    )
    .bind(traza.autor)
    .bind(traza.tipo as u8)
    .bind(traza.fecha)
    .bind(traza.entidad as u8)
    .bind(traza.entidad_id)
    .bind(&traza.motivo)
    .execute(&mut **trans.deref_mut())
    .await
    .map_err(DBError::from_sqlx)?;

    Ok(result.last_insert_id() as u32)
  }
}
