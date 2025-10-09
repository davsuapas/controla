use chrono::{NaiveDate, NaiveTime};

use smallvec::SmallVec;
use sqlx::Row;

use crate::{
  infra::{
    DBError, DominiosWithCacheUsuario, PoolConexion, ShortDateTimeFormat,
  },
  marcaje::Marcaje,
  mysql_params,
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
      .bind(reg.usuario_reg)
      .execute(self.pool.conexion())
      .await
      .map_err(DBError::from_sqlx)?;

    Ok(result.last_insert_id() as u32)
  }

  /// Verifica si la hora de fin para cualquier marcaje
  /// horario, para un determinado usuario y fecha, está vacía.
  pub(in crate::marcaje) async fn hora_fin_vacia(
    &self,
    usuario: u32,
    fecha: NaiveDate,
  ) -> Result<bool, DBError> {
    const QUERY: &str = r"SELECT id
      FROM marcajes
      WHERE usuario = ? AND fecha = ?
      AND hora_fin IS NULL
      AND modificado_por IS NULL AND eliminado IS NULL
      LIMIT 1;";

    Ok(
      sqlx::query_scalar::<_, bool>(QUERY)
        .bind(usuario)
        .bind(fecha)
        .fetch_optional(self.pool.conexion())
        .await
        .map_err(DBError::from_sqlx)?
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
        WHERE usuario = ? AND fecha = ?
        AND ? BETWEEN hora_inicio AND hora_fin
        AND modificado_por IS NULL AND eliminado IS NULL
        LIMIT 1;";

    Ok(
      sqlx::query(QUERY)
        .bind(usuario)
        .bind(fecha)
        .bind(hora)
        .fetch_optional(self.pool.conexion())
        .await
        .map_err(DBError::from_sqlx)?
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
        AND modificado_por IS NULL AND eliminado IS NULL
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
        .map_err(DBError::from_sqlx)?
        .is_some(),
    )
  }

  /// Obtiene los últimos marcajes horarios de un usuario.
  pub(in crate::marcaje) async fn ultimos_marcajes(
    &self,
    usuario: u32,
    top: Option<&str>,
  ) -> Result<DominiosWithCacheUsuario<Marcaje>, DBError> {
    const ORDER_BY: &str = "r.fecha DESC, r.hora_inicio DESC";

    self
      .marcajes(
        top,
        Some("r.usuario = ?"),
        Some(mysql_params![usuario => "Usuario"]),
        Some(ORDER_BY),
      )
      .await
  }

  /// Obtiene los marcaje dado el usuario y la fecha
  /// para el registrador como 1parámetro
  /// que no tengan asigandas una incidencia
  pub(in crate::marcaje) async fn marcajes_inc_por_fecha_reg(
    &self,
    usuario: u32,
    fecha: NaiveDate,
    usuario_reg: Option<u32>,
  ) -> Result<DominiosWithCacheUsuario<Marcaje>, DBError> {
    const ORDER_BY: &str = "r.hora_inicio DESC";

    let filter = if usuario_reg.is_some() {
      "r.usuario = ? AND r.fecha = ? AND r.usuario_registrador = ? 
         AND NOT EXISTS (SELECT id FROM incidencias AS i 
         WHERE i.marcaje = r.id)"
    } else {
      "r.usuario = ? AND r.fecha = ? 
         AND NOT EXISTS (SELECT id FROM incidencias AS i
         WHERE i.marcaje = r.id)"
    };

    use sqlx::Arguments;
    let mut args = sqlx::mysql::MySqlArguments::default();
    args
      .add(usuario)
      .map_err(|_| DBError::Parametros("Usuario"))?;
    args
      .add(fecha.formato_sql())
      .map_err(|_| DBError::Parametros("Fecha"))?;

    if let Some(ur) = usuario_reg {
      args
        .add(ur)
        .map_err(|_| DBError::Parametros("Usuario registrador"))?;
    }

    self
      .marcajes(None, Some(filter), Some(args), Some(ORDER_BY))
      .await
  }

  /// Obtiene los marcaje dado el usuario y la fecha
  pub(in crate::marcaje) async fn marcajes_por_fecha(
    &self,
    usuario: u32,
    fecha: NaiveDate,
  ) -> Result<DominiosWithCacheUsuario<Marcaje>, DBError> {
    const ORDER_BY: &str = "r.hora_inicio DESC";

    self
      .marcajes(
        None,
        Some("r.usuario = ? AND r.fecha = ?"),
        Some(
          mysql_params![usuario => "usuario", fecha.formato_sql() => "fecha"],
        ),
        Some(ORDER_BY),
      )
      .await
  }

  async fn marcajes(
    &self,
    top: Option<&str>,
    filter: Option<&str>,
    filter_params: Option<sqlx::mysql::MySqlArguments>,
    order: Option<&str>,
  ) -> Result<DominiosWithCacheUsuario<Marcaje>, DBError> {
    const SELECT: &str = r"SELECT r.id, r.fecha,
        r.hora_inicio, r.hora_fin, r.horario,
        u.id AS u_id, u.nombre AS u_nombre,
        u.primer_apellido AS u_primer_apellido,
        u.segundo_apellido AS u_segundo_apellido,
        ur.id AS ur_id, ur.nombre AS ur_nombre,
        ur.primer_apellido AS ur_primer_apellido,
        ur.segundo_apellido AS ur_segundo_apellido,
        h.dia, h.hora_inicio AS h_hora_inicio, h.hora_fin AS h_hora_fin
        FROM marcajes r
        JOIN horarios h ON r.horario = h.id
        JOIN usuarios u ON r.usuario = u.id
        LEFT JOIN usuarios ur ON r.usuario_registrador = ur.id";

    let mut query = String::with_capacity(
      SELECT.len()
        + filter.map_or(0, |f| f.len() + 60)
        + order.map_or(0, |f| f.len() + 10)
        + top.map_or(0, |t| t.len() + 10),
    );

    query.push_str(SELECT);

    if let Some(f) = filter {
      query.push_str(" WHERE ");
      query.push_str(f);
      query.push_str(" AND modificado_por IS NULL AND eliminado IS NULL");
    }

    if let Some(o) = order {
      query.push_str(" ORDER BY ");
      query.push_str(o);
    }

    if let Some(t) = top {
      query.push_str(" LIMIT ");
      query.push_str(t);
    }

    let rows = if let Some(args) = filter_params {
      sqlx::query_with(&query, args)
        .fetch_all(self.pool.conexion())
        .await
        .map_err(DBError::from_sqlx)?
    } else {
      sqlx::query(&query)
        .fetch_all(self.pool.conexion())
        .await
        .map_err(DBError::from_sqlx)?
    };

    // El resto del código permanece igual...
    let capacidad = rows.len();
    let mut resultado = DominiosWithCacheUsuario::<Marcaje>::new(capacidad);
    let mut usuarios_buffer: SmallVec<[DescriptorUsuario; 2]> =
      SmallVec::with_capacity(2);

    for row in rows {
      usuarios_buffer.push(DescriptorUsuario {
        id: row.get("u_id"),
        nombre: row.get("u_nombre"),
        primer_apellido: row.get("u_primer_apellido"),
        segundo_apellido: row.get("u_segundo_apellido"),
      });

      if let Ok(ur_id) = row.try_get::<u32, _>("ur_id") {
        usuarios_buffer.push(DescriptorUsuario {
          id: ur_id,
          nombre: row.get("ur_nombre"),
          primer_apellido: row.get("ur_primer_apellido"),
          segundo_apellido: row.get("ur_segundo_apellido"),
        });
      }

      let marcaje = Marcaje {
        id: row.get("id"),
        usuario: row.get("u_id"),
        usuario_reg: row.try_get::<u32, _>("ur_id").ok(),
        horario: Some(Horario {
          id: row.get("horario"),
          dia: row.get::<String, _>("dia").as_str().into(),
          hora_inicio: row.get("h_hora_inicio"),
          hora_fin: row.get("h_hora_fin"),
        }),
        fecha: row.get("fecha"),
        hora_inicio: row.get("hora_inicio"),
        hora_fin: row.get("hora_fin"),
      };

      resultado.push_with_usuarios(marcaje, usuarios_buffer.drain(..));
    }

    Ok(resultado)
  }
}
