use crate::{
  inc::dominio::Incidencia,
  infra::{DBError, PoolConexion},
};

/// Implementación del repositorio de incidencias.
pub struct IncidenciaRepo {
  pool: PoolConexion,
}

impl IncidenciaRepo {
  pub fn new(pool: PoolConexion) -> Self {
    IncidenciaRepo { pool }
  }
}

impl IncidenciaRepo {
  pub(in crate::inc) async fn agregar(
    &self,
    reg: &Incidencia,
  ) -> Result<u32, DBError> {
    const QUERY: &str = r"INSERT INTO incidencias
      (tipo, fecha_solicitud, hora_inicio, hora_fin, marcaje, estado,
       error, usuario_creador, usuario_gestor, fecha, motivo_solicitud)
      VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)";

    let result = sqlx::query(QUERY)
      .bind(reg.tipo as u8)
      .bind(reg.fecha_solicitud)
      .bind(reg.hora_inicio)
      .bind(reg.hora_fin)
      .bind(reg.marcaje.as_ref().map(|m| m.id))
      .bind(reg.estado as u8)
      .bind(&reg.error)
      .bind(reg.usuario_creador)
      .bind(reg.usuario_gestor)
      .bind(reg.fecha)
      .bind(&reg.motivo_solicitud)
      .execute(self.pool.conexion())
      .await
      .map_err(DBError::from_sqlx)?;

    Ok(result.last_insert_id() as u32)
  }
}
