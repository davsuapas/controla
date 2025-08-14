use chrono::{NaiveDate, NaiveTime};

use crate::{
  db_pool_getter,
  infraestructura::{DBError, PoolConexion, Transaccion},
  registro::{Registro, Traza},
};

/// Implementación del repositorio de registros.
pub struct RegistroRepo {
  pool: PoolConexion,
}

impl RegistroRepo {
  pub fn new(pool: PoolConexion) -> Self {
    RegistroRepo { pool }
  }
}

db_pool_getter!(RegistroRepo);

impl RegistroRepo {
  /// Agrega un nuevo registro a la base de datos.
  ///
  /// Valida que la hora de inicio y fin no estén ya asignadas al usuario.
  ///
  /// Devuelve el ID del registro creado.
  pub(in crate::registro) async fn agregar(
    &self,
    trans: &mut Transaccion<'_>,
    reg: &Registro,
  ) -> Result<u64, DBError> {
    let hora_asignada = self
      .hora_asignada(reg.usuario.id, reg.fecha, reg.hora_inicio)
      .await?;

    if hora_asignada {
      return Err(DBError::validacion_incorrecta(format!(
        "La hora de inicio: {} ya está asignada al usuario: {} en la fecha: {}",
        reg.hora_inicio, reg.usuario.nombre, reg.fecha
      )));
    }

    if let Some(hora_fin) = reg.hora_fin {
      let hora_asignada = self
        .hora_asignada(reg.usuario.id, reg.fecha, hora_fin)
        .await?;

      if hora_asignada {
        return Err(DBError::validacion_incorrecta(format!(
          "La hora de finalización: {} \
          ya está asignada al usuario: {} en la fecha: {}",
          hora_fin, reg.usuario.nombre, reg.fecha
        )));
      }
    }

    let result = sqlx::query(
      r"INSERT INTO registros
      (usuario, fecha, hora_inicio, hora_fin, horas_a_trabajar)
      VALUES (?, ?, ?, ?, ?)",
    )
    .bind(reg.usuario.id)
    .bind(reg.fecha)
    .bind(reg.hora_inicio)
    .bind(reg.hora_fin)
    .bind(reg.horas_a_trabajar)
    .execute(&mut **trans.deref_mut())
    .await
    .map_err(DBError::consulta_from)?;

    Ok(result.last_insert_id())
  }

  /// Verifica si una hora está asignada a un usuario en un registro.
  async fn hora_asignada(
    &self,
    usuario: u64,
    fecha: NaiveDate,
    hora: NaiveTime,
  ) -> Result<bool, DBError> {
    Ok(
      sqlx::query(
        r"SELECT id
        FROM registros
        WHERE usuario = ? 
        AND fecha = ?
        AND ? BETWEEN hora_inicio AND hora_fin;",
      )
      .bind(usuario)
      .bind(fecha)
      .bind(hora)
      .fetch_optional(self.pool.conexion())
      .await
      .map_err(DBError::consulta_from)?
      .is_some(),
    )
  }
}

/// Implementación del repositorio de trazas.
pub struct TrazaRepo {}

impl TrazaRepo {
  pub fn new() -> Self {
    TrazaRepo {}
  }
}

impl TrazaRepo {
  /// Agrega una nueva traza a la base de datos.
  pub(in crate::registro) async fn agregar(
    &self,
    trans: &mut Transaccion<'_>,
    traza: &Traza,
  ) -> Result<u64, DBError> {
    let result = sqlx::query(
      r"INSERT INTO trazas
      (registro, usuario, fecha, tipo)
      VALUES (?, ?, ?, ?)",
    )
    .bind(traza.reg_id)
    .bind(traza.user_id)
    .bind(traza.fecha)
    .bind(traza.tipo.as_u8())
    .execute(&mut **trans.deref_mut())
    .await
    .map_err(DBError::consulta_from)?;

    Ok(result.last_insert_id())
  }
}
