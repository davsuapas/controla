use chrono::{NaiveDate, NaiveTime};

use sqlx::Row;

use crate::{
  infra::{DBError, PoolConexion},
  registro::Registro,
  usuarios::{DescriptorUsuario, Horario, UsuarioNombre},
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

impl RegistroRepo {
  /// Agrega un nuevo registro a la base de datos.
  ///
  /// Devuelve el ID del registro creado.
  pub(in crate::registro) async fn agregar(
    &self,
    reg: &Registro,
    horario: u32,
  ) -> Result<u64, DBError> {
    const QUERY: &str = r"INSERT INTO registros
      (usuario, fecha, horario, hora_inicio, hora_fin, usuario_registrador)
      VALUES (?, ?, ?, ?, ?, ?)";

    let result = sqlx::query(QUERY)
      .bind(reg.usuario.id)
      .bind(reg.fecha)
      .bind(horario)
      .bind(reg.hora_inicio)
      .bind(reg.hora_fin)
      .bind(reg.usuario_reg.as_ref().map(|u| u.id))
      .execute(self.pool.conexion())
      .await
      .map_err(DBError::consulta_from)?;

    Ok(result.last_insert_id())
  }

  /// Verifica si la hora de fin para cualquier registro
  /// horario para un usuario y fecha está vacía.
  pub(in crate::registro) async fn hora_fin_vacia(
    &self,
    usuario: u32,
    fecha: NaiveDate,
  ) -> Result<bool, DBError> {
    const QUERY: &str = r"SELECT id
      FROM registros
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
  /// de un registro horario para un usuario y fecha.
  pub(in crate::registro) async fn hora_asignada(
    &self,
    usuario: u32,
    fecha: NaiveDate,
    hora: NaiveTime,
  ) -> Result<bool, DBError> {
    const QUERY: &str = r"SELECT id
        FROM registros
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
  /// con otro rango de horas ya asignado a un usuario en un registro.
  /// También verifica que la hora no se encuentre entre los rangos de horas.
  pub(in crate::registro) async fn horas_solapadas(
    &self,
    usuario: u32,
    fecha: NaiveDate,
    hora_ini: NaiveTime,
    hora_fin: NaiveTime,
  ) -> Result<bool, DBError> {
    const QUERY: &str = r"SELECT id
        FROM registros
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

  /// Obtiene la hora final de un registro previo
  /// para un usuario en una fecha y hora de inicio.
  pub(in crate::registro) async fn hora_fin_previa(
    &self,
    usuario: u32,
    fecha: NaiveDate,
    hora: NaiveTime,
  ) -> Result<Option<NaiveTime>, DBError> {
    const QUERY: &str = r"SELECT MAX(hora_fin) 
        FROM registros
        WHERE usuario = ? 
        AND fecha = ?
        AND hora_fin < ?";

    let result: Option<NaiveTime> = sqlx::query_scalar(QUERY)
      .bind(usuario)
      .bind(fecha)
      .bind(hora)
      .fetch_one(self.pool.conexion())
      .await
      .map_err(DBError::consulta_from)?;

    Ok(result)
  }

  /// Obtiene los últimos registros horarios de un usuario.
  pub(in crate::registro) async fn ultimos_registros(
    &self,
    usuario: u32,
    top: Option<&str>,
  ) -> Result<Vec<Registro>, DBError> {
    let filter = format!("r.usuario = {}", usuario);

    self.registros(top, Some(&filter)).await
  }

  async fn registros(
    &self,
    top: Option<&str>,
    filter: Option<&str>,
  ) -> Result<Vec<Registro>, DBError> {
    const SELECT: &str = r"SELECT r.id, r.fecha, r.hora_inicio, r.hora_fin,
        r.usuario, r.usuario_registrador, r.horario,
        u.nombre AS u_nombre,
        ur.nombre AS ur_nombre, ur.primer_apellido AS ur_primer_apellido,
        ur.segundo_apellido AS ur_segundo_apellido,
        h.dia, h.hora_inicio AS h_hora_inicio, h.hora_fin AS h_hora_fin
        FROM registros r
        JOIN horarios h ON r.horario = h.id
        JOIN usuarios u ON r.usuario = u.id
        LEFT JOIN usuarios ur ON r.usuario_registrador = ur.id";

    const ORDER_BY: &str = " ORDER BY r.fecha, r.hora_inicio";

    let mut query = String::with_capacity(
      SELECT.len()
        + filter.map_or(0, |f| f.len() + 20)
        + ORDER_BY.len()
        + top.map_or(0, |t| t.len() + 10),
    );

    query.push_str(SELECT);

    if let Some(f) = filter {
      query.push_str(" WHERE ");
      query.push_str(f);
    }

    query.push_str(ORDER_BY);

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
        .map(|row| Registro {
          usuario: UsuarioNombre {
            id: row.get("usuario"),
            nombre: row.get("u_nombre"),
          },
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
            dia: row.get::<String, _>("dia").chars().next().unwrap().into(),
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
