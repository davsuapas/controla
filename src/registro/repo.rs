use chrono::{NaiveDate, NaiveTime};

use crate::{
  db_pool_getter,
  infra::{DBError, PoolConexion, Transaccion},
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
  /// Devuelve el ID del registro creado.
  pub(in crate::registro) async fn agregar(
    &self,
    trans: &mut Transaccion<'_>,
    reg: &Registro,
    horario: u64,
  ) -> Result<u64, DBError> {
    let result = sqlx::query(
      r"INSERT INTO registros
      (usuario, fecha, horario, hora_inicio, hora_fin)
      VALUES (?, ?, ?, ?, ?)",
    )
    .bind(reg.usuario.id)
    .bind(reg.fecha)
    .bind(horario)
    .bind(reg.hora_inicio)
    .bind(reg.hora_fin)
    .execute(&mut **trans.deref_mut())
    .await
    .map_err(DBError::consulta_from)?;

    Ok(result.last_insert_id())
  }

  /// Verifica si la hora de fin para cualquier registro
  /// horario para un usuario y fecha está vacía.
  pub(in crate::registro) async fn hora_fin_vacia(
    &self,
    usuario: u64,
    fecha: NaiveDate,
  ) -> Result<bool, DBError> {
    Ok(
      sqlx::query_scalar::<_, bool>(
        r"SELECT id
      FROM registros
      WHERE usuario = ? 
      AND fecha = ?
      AND hora_fin IS NULL
      LIMIT 1;",
      )
      .bind(usuario)
      .bind(fecha)
      .fetch_optional(self.pool.conexion())
      .await
      .map_err(DBError::consulta_from)?
      .is_some(),
    )
  }

  /// Verifica si una hora se encuentra en el rango de horas
  /// de un registro horario para un usuario y fecha.
  pub(in crate::registro) async fn hora_asignada(
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
        AND ? BETWEEN hora_inicio AND hora_fin
        LIMIT 1;",
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

  /// Verifica si un rango de horas como parámetro se solapa
  /// con otro rango de horas ya asignado a un usuario en un registro.
  /// También verifica que la hora no se encuentre entre los rangos de horas.
  pub(in crate::registro) async fn horas_solapadas(
    &self,
    usuario: u64,
    fecha: NaiveDate,
    hora_ini: NaiveTime,
    hora_fin: NaiveTime,
  ) -> Result<bool, DBError> {
    Ok(
      sqlx::query(
        r"SELECT id
        FROM registros
        WHERE usuario = ? AND fecha = ?
        AND hora_inicio < ? AND hora_fin > ?
        OR ? BETWEEN hora_inicio AND hora_fin
        LIMIT 1;",
      )
      .bind(usuario)
      .bind(fecha)
      .bind(hora_fin)
      .bind(hora_ini)
      .bind(hora_fin)
      .fetch_optional(self.pool.conexion())
      .await
      .map_err(DBError::consulta_from)?
      .is_some(),
    )
  }

  /// Obtiene la hora final de un registro previo
  /// para un usuario en una fecha y hora de inicio.
  pub(in crate::registro) async fn hora_fin_previa(
    &self,
    usuario: u64,
    fecha: NaiveDate,
    hora: NaiveTime,
  ) -> Result<Option<NaiveTime>, DBError> {
    let result: Option<NaiveTime> = sqlx::query_scalar(
      r"SELECT MAX(hora_fin) 
        FROM registros
        WHERE usuario = ? 
        AND fecha = ?
        AND hora_fin < ?",
    )
    .bind(usuario)
    .bind(fecha)
    .bind(hora)
    .fetch_one(self.pool.conexion())
    .await
    .map_err(DBError::consulta_from)?;

    Ok(result)
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
