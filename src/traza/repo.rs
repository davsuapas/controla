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
      (tipo, fecha, usuario, horario, registro, motivo)
      VALUES (?, ?, ?, ?, ?, ?)",
    )
    .bind(traza.tipo as u8)
    .bind(traza.fecha)
    .bind(traza.usuario_id)
    .bind(traza.horario_id)
    .bind(traza.registro_id)
    .bind(&traza.motivo)
    .execute(&mut **trans.deref_mut())
    .await
    .map_err(DBError::consulta_from)?;

    Ok(result.last_insert_id())
  }
}
