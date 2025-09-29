use chrono::{NaiveDate, NaiveTime};

use sqlx::Row;

use crate::{
  infra::{DBError, PoolConexion, ShortDateTimeFormat},
  marcaje::Marcaje,
  usuarios::{DescriptorUsuario, Horario},
};

/// Implementación del repositorio de marcajes.
pub struct MarcajeRepo {
  pool: PoolConexion,
}

impl MarcajeRepo {
  pub fn new(pool: PoolConexion) -> Self {
    MarcajeRepo { pool }
  }
}

impl MarcajeRepo {
  /// Agrega un nuevo marcaje a la base de datos.
  ///
  /// Devuelve el ID del marcaje creado.
  pub(in crate::marcaje) async fn agregar(
    &self,
    reg: &Marcaje,
    horario: u32,
  ) -> Result<u32, DBError> {
    const QUERY: &str = r"INSERT INTO marcajes
      (usuario, fecha, horario, hora_inicio, hora_fin, usuario_registrador)
      VALUES (?, ?, ?, ?, ?, ?)";

    let result = sqlx::query(QUERY)
      .bind(reg.usuario)
      .bind(reg.fecha)
      .bind(horario)
      .bind(reg.hora_inicio)
      .bind(reg.hora_fin)
      .bind(reg.usuario_reg.as_ref().map(|u| u.id))
      .execute(self.pool.conexion())
      .await
      .map_err(DBError::consulta_from)?;

    Ok(result.last_insert_id() as u32)
  }

  /// Verifica si la hora de fin para cualquier marcaje
  /// horario para un usuario y fecha está vacía.
  pub(in crate::marcaje) async fn hora_fin_vacia(
    &self,
    usuario: u32,
    fecha: NaiveDate,
  ) -> Result<bool, DBError> {
    const QUERY: &str = r"SELECT id
      FROM marcajes
      WHERE usuario = ? 
      AND fecha = ?
      AND hora_fin IS NULL
      LIMIT 1;";

    Ok(
      sqlx::query_scalar::<_, bool>(QUERY)
        .bind(usuario)
        .bind(fecha)
        .fetch_optional(self.pool.conexion())
        .await
        .map_err(DBError::consulta_from)?
        .is_some(),
    )
  }

  /// Verifica si una hora se encuentra en el rango de horas
  /// de un marcaje horario para un usuario y fecha.
  pub(in crate::marcaje) async fn hora_asignada(
    &self,
    usuario: u32,
    fecha: NaiveDate,
    hora: NaiveTime,
  ) -> Result<bool, DBError> {
    const QUERY: &str = r"SELECT id
        FROM marcajes
        WHERE usuario = ? 
        AND fecha = ?
        AND ? BETWEEN hora_inicio AND hora_fin
        LIMIT 1;";

    Ok(
      sqlx::query(QUERY)
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
  /// con otro rango de horas ya asignado a un usuario en un marcaje.
  /// También verifica que la hora no se encuentre entre los rangos de horas.
  pub(in crate::marcaje) async fn horas_solapadas(
    &self,
    usuario: u32,
    fecha: NaiveDate,
    hora_ini: NaiveTime,
    hora_fin: NaiveTime,
  ) -> Result<bool, DBError> {
    const QUERY: &str = r"SELECT id
        FROM marcajes
        WHERE usuario = ? AND fecha = ?
        AND hora_inicio < ? AND hora_fin > ?
        OR ? BETWEEN hora_inicio AND hora_fin
        LIMIT 1;";

    Ok(
      sqlx::query(QUERY)
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

  /// Obtiene los últimos marcajes horarios de un usuario.
  pub(in crate::marcaje) async fn ultimos_marcajes(
    &self,
    usuario: u32,
    top: Option<&str>,
  ) -> Result<Vec<Marcaje>, DBError> {
    const ORDER_BY: &str = "r.fecha DESC, r.hora_inicio DESC";
    let filter = format!("r.usuario = {}", usuario);

    self.marcajes(top, Some(&filter), Some(ORDER_BY)).await
  }

  /// Obtiene los marcaje dado el usuario y la fecha
  pub(in crate::marcaje) async fn marcajes_por_fecha(
    &self,
    usuario: u32,
    fecha: NaiveDate,
  ) -> Result<Vec<Marcaje>, DBError> {
    const ORDER_BY: &str = "r.hora_inicio DESC";
    let filter = format!(
      "r.usuario = {} AND r.fecha = '{}'",
      usuario,
      fecha.formato_sql()
    );

    self.marcajes(None, Some(&filter), Some(ORDER_BY)).await
  }

  async fn marcajes(
    &self,
    top: Option<&str>,
    filter: Option<&str>,
    order: Option<&str>,
  ) -> Result<Vec<Marcaje>, DBError> {
    const SELECT: &str = r"SELECT r.id, r.fecha, r.hora_inicio, r.hora_fin,
        r.usuario, r.usuario_registrador, r.horario,
        u.id AS u_id,
        ur.nombre AS ur_nombre, ur.primer_apellido AS ur_primer_apellido,
        ur.segundo_apellido AS ur_segundo_apellido,
        h.dia, h.hora_inicio AS h_hora_inicio, h.hora_fin AS h_hora_fin
        FROM marcajes r
        JOIN horarios h ON r.horario = h.id
        JOIN usuarios u ON r.usuario = u.id
        LEFT JOIN usuarios ur ON r.usuario_registrador = ur.id";

    let mut query = String::with_capacity(
      SELECT.len()
        + filter.map_or(0, |f| f.len() + 10)
        + order.map_or(0, |f| f.len() + 10)
        + top.map_or(0, |t| t.len() + 10),
    );

    query.push_str(SELECT);

    if let Some(f) = filter {
      query.push_str(" WHERE ");
      query.push_str(f);
    }

    if let Some(o) = order {
      query.push_str(" ORDER BY ");
      query.push_str(o);
    }

    if let Some(t) = top {
      query.push_str(" LIMIT ");
      query.push_str(t);
    }

    let rows = sqlx::query(query.as_str())
      .fetch_all(self.pool.conexion())
      .await
      .map_err(DBError::consulta_from)?;

    Ok(
      rows
        .iter()
        .map(|row| Marcaje {
          usuario: row.get("u_id"),
          usuario_reg: row.try_get::<u32, _>("usuario_registrador").ok().map(
            |id| DescriptorUsuario {
              id,
              nombre: row.get("ur_nombre"),
              primer_apellido: row.get("ur_primer_apellido"),
              segundo_apellido: row.get("ur_segundo_apellido"),
            },
          ),
          horario: Some(Horario {
            id: row.get("horario"),
            dia: row.get::<String, _>("dia").as_str().into(),
            hora_inicio: row.get("h_hora_inicio"),
            hora_fin: row.get("h_hora_fin"),
          }),
          fecha: row.get("fecha"),
          hora_inicio: row.get("hora_inicio"),
          hora_fin: row.get("hora_fin"),
        })
        .collect(),
    )
  }
}
