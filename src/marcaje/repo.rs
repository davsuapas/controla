use chrono::{NaiveDate, NaiveTime};

use sqlx::Row;

use crate::{
  infra::{
    DBError, DominioWithCacheUsuario, PoolConexion, ShortDateTimeFormat,
    Transaccion,
  },
  marcaje::{DescriptorMarcaje, Marcaje},
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
    tr: Option<&mut Transaccion<'_>>,
    reg: &Marcaje,
    horario: u32,
  ) -> Result<u32, DBError> {
    const QUERY: &str = "INSERT INTO marcajes
      (usuario, fecha, horario, hora_inicio, hora_fin, usuario_registrador)
      VALUES (?, ?, ?, ?, ?, ?)";

    let query = sqlx::query(QUERY)
      .bind(reg.usuario)
      .bind(reg.fecha)
      .bind(horario)
      .bind(reg.hora_inicio)
      .bind(reg.hora_fin)
      .bind(reg.usuario_reg);

    let result = if let Some(tr) = tr {
      query
        .execute(&mut **tr.deref_mut())
        .await
        .map_err(DBError::from_sqlx)?
    } else {
      query
        .execute(self.pool.conexion())
        .await
        .map_err(DBError::from_sqlx)?
    };

    Ok(result.last_insert_id() as u32)
  }

  /// Actualiza modificado_por
  ///
  /// Devuelve True si se actualizo
  pub(in crate::marcaje) async fn actualizar_modificado_por(
    &self,
    trans: &mut Transaccion<'_>,
    id: u32,
    modificar_por: u32,
  ) -> Result<bool, DBError> {
    const QUERY: &str = "UPDATE marcajes SET modificado_por = ? WHERE id = ?";

    let result = sqlx::query(QUERY)
      .bind(modificar_por)
      .bind(id)
      .execute(&mut **trans.deref_mut())
      .await
      .map_err(DBError::from_sqlx)?;

    Ok(result.rows_affected() > 0)
  }

  /// Marca un marcaje como eliminado
  ///
  /// Devuelve True si se actualizo
  pub(in crate::marcaje) async fn marcar_marcaje_eliminado(
    &self,
    trans: &mut Transaccion<'_>,
    id: u32,
  ) -> Result<bool, DBError> {
    const QUERY: &str = "UPDATE marcajes SET eliminado = TRUE WHERE id = ?";

    let result = sqlx::query(QUERY)
      .bind(id)
      .execute(&mut **trans.deref_mut())
      .await
      .map_err(DBError::from_sqlx)?;

    Ok(result.rows_affected() > 0)
  }

  pub(in crate::marcaje) async fn actualizar_hora_fin(
    &self,
    id: u32,
    hora_fin: NaiveTime,
  ) -> Result<bool, DBError> {
    const QUERY: &str = "UPDATE marcajes SET hora_fin = ? WHERE id = ?";

    let result = sqlx::query(QUERY)
      .bind(hora_fin)
      .bind(id)
      .execute(self.pool.conexion())
      .await
      .map_err(DBError::from_sqlx)?;

    Ok(result.rows_affected() > 0)
  }

  /// Verifica si la hora de fin para cualquier marcaje
  /// horario de un determinado usuario y fecha está vacía.
  ///
  /// Se puede excluir un marcaje pasado como parámetro
  /// Si no quiere excluir ningún marcaje use 0
  /// La exclusión puede ser muy útil cuando se quiere
  /// realizar una modificación de este marcaje
  pub(in crate::marcaje) async fn hora_fin_vacia(
    &self,
    usuario: u32,
    fecha: NaiveDate,
    excluir_marcaje_id: u32,
  ) -> Result<bool, DBError> {
    const QUERY: &str = "SELECT id
      FROM marcajes
      WHERE usuario = ? AND fecha = ?
      AND id <> ? AND modificado_por IS NULL AND eliminado IS NULL
      AND hora_fin IS NULL
      LIMIT 1;";

    Ok(
      sqlx::query_scalar::<_, bool>(QUERY)
        .bind(usuario)
        .bind(fecha)
        .bind(excluir_marcaje_id)
        .fetch_optional(self.pool.conexion())
        .await
        .map_err(DBError::from_sqlx)?
        .is_some(),
    )
  }

  /// Verifica si existen horas posteriores a la hora dada
  /// de un marcaje horario para un usuario y fecha.
  ///
  /// Se puede excluir un marcaje pasado como parámetro
  /// Si no quiere excluir ningún marcaje use 0
  /// La exclusión puede ser muy útil cuando se quiere
  /// realizar una modificación de este marcaje
  pub(in crate::marcaje) async fn hora_asignada_posterior(
    &self,
    usuario: u32,
    fecha: NaiveDate,
    hora: NaiveTime,
    excluir_marcaje_id: u32,
  ) -> Result<bool, DBError> {
    const QUERY: &str = "SELECT id
        FROM marcajes
        WHERE usuario = ? AND fecha = ?
        AND id <> ? AND modificado_por IS NULL AND eliminado IS NULL
        AND hora_inicio >= ?
        LIMIT 1;";

    Ok(
      sqlx::query(QUERY)
        .bind(usuario)
        .bind(fecha)
        .bind(excluir_marcaje_id)
        .bind(hora)
        .fetch_optional(self.pool.conexion())
        .await
        .map_err(DBError::from_sqlx)?
        .is_some(),
    )
  }

  /// Verifica si una hora se encuentra en el rango de horas
  /// de un marcaje horario para un usuario y fecha.
  ///
  /// Se puede excluir un marcaje pasado como parámetro
  /// Si no quiere excluir ningún marcaje use 0
  /// La exclusión puede ser muy útil cuando se quiere
  /// realizar una modificación de este marcaje
  pub(in crate::marcaje) async fn hora_asignada(
    &self,
    usuario: u32,
    fecha: NaiveDate,
    hora: NaiveTime,
    excluir_marcaje_id: u32,
  ) -> Result<bool, DBError> {
    const QUERY: &str = "SELECT id
        FROM marcajes
        WHERE usuario = ? AND fecha = ?
        AND id <> ? AND modificado_por IS NULL AND eliminado IS NULL
        AND ? BETWEEN hora_inicio AND hora_fin
        LIMIT 1;";

    Ok(
      sqlx::query(QUERY)
        .bind(usuario)
        .bind(fecha)
        .bind(excluir_marcaje_id)
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
  ///
  /// Se puede excluir un marcaje pasado como parámetro
  /// Si no quiere excluir ningún marcaje use 0
  /// La exclusión puede ser muy útil cuando se quiere
  /// realizar una modificación de este marcaje
  pub(in crate::marcaje) async fn horas_solapadas(
    &self,
    usuario: u32,
    fecha: NaiveDate,
    hora_ini: NaiveTime,
    hora_fin: NaiveTime,
    excluir_marcaje_id: u32,
  ) -> Result<bool, DBError> {
    const QUERY: &str = "SELECT id
        FROM marcajes
        WHERE usuario = ? AND fecha = ?
        AND id <> ? AND modificado_por IS NULL AND eliminado IS NULL
        AND ( hora_inicio < ? AND hora_fin > ?
        OR ? BETWEEN hora_inicio AND hora_fin )
        LIMIT 1;";

    Ok(
      sqlx::query(QUERY)
        .bind(usuario)
        .bind(fecha)
        .bind(excluir_marcaje_id)
        .bind(hora_fin)
        .bind(hora_ini)
        .bind(hora_fin)
        .fetch_optional(self.pool.conexion())
        .await
        .map_err(DBError::from_sqlx)?
        .is_some(),
    )
  }

  /// Obtiene el descriptor marcaje cuya hora fin es nula
  pub(in crate::marcaje) async fn marcaje_sin_hora_fin(
    &self,
    usuario: u32,
    fecha: NaiveDate,
  ) -> Result<Option<DescriptorMarcaje>, DBError> {
    const QUERY: &str = "SELECT id, hora_inicio, hora_fin
      FROM marcajes
      WHERE usuario = ? AND fecha = ?
      AND hora_fin IS NULL
      AND modificado_por IS NULL AND eliminado IS NULL
      LIMIT 1;";

    let row = sqlx::query(QUERY)
      .bind(usuario)
      .bind(fecha)
      .fetch_optional(self.pool.conexion())
      .await
      .map_err(DBError::from_sqlx)?;

    if let Some(row) = row {
      Ok(Some(DescriptorMarcaje {
        id: row.get("id"),
        hora_inicio: row.get("hora_inicio"),
        hora_fin: row.get("hora_fin"),
      }))
    } else {
      Ok(None)
    }
  }

  /// Obtiene los últimos marcajes horarios de un usuario.
  pub(in crate::marcaje) async fn ultimos_marcajes(
    &self,
    usuario: u32,
    top: Option<&str>,
  ) -> Result<DominioWithCacheUsuario<Marcaje>, DBError> {
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

  /// Obtiene los marcaje dado el usuario entre dos fechas.
  ///
  /// Si la fechas son Nones se filtra solos por usuario.
  /// Si el usuario_reg es igual a 0, significa que es supervisor
  /// y puede ver todos los marcajes de caulquier usuario.
  /// Si el usuario es diferente el usuario_reg, significa
  /// que es usuario registrador y por tanto puede ver solo
  /// los marcajes que registro el.
  /// Si son iguales el usuario es empleado y solo puede ver
  /// sus marcajes.
  ///
  /// Los marcajes deben no tener asignada una incidencia.
  pub(in crate::marcaje) async fn marcajes_entre_fechas_reg(
    &self,
    usuario: u32,
    fecha_inicio: Option<NaiveDate>,
    fecha_fin: Option<NaiveDate>,
    usuario_reg: Option<u32>,
  ) -> Result<DominioWithCacheUsuario<Marcaje>, DBError> {
    const ORDER_BY: &str = "r.fecha DESC, r.hora_inicio DESC";

    let filter = match (usuario_reg, fecha_inicio, fecha_fin) {
      (Some(usr), Some(_), Some(_)) if usr == usuario || usr == 0 => {
        "r.usuario = ? AND r.fecha BETWEEN ? AND ?"
      }
      (Some(_), Some(_), Some(_)) => {
        r"r.usuario = ? AND r.fecha BETWEEN ? AND ?
         AND r.usuario_registrador = ?"
      }
      (Some(usr), None, None) if usr == usuario || usr == 0 => "r.usuario = ?",
      (Some(_), None, None) => "r.usuario = ? AND r.usuario_registrador = ?",
      (None, Some(_), Some(_)) => "r.usuario = ? AND r.fecha BETWEEN ? AND ?",
      _ => "r.usuario = ?",
    };

    use sqlx::Arguments;
    let mut args = sqlx::mysql::MySqlArguments::default();
    args
      .add(usuario)
      .map_err(|_| DBError::Parametros("Usuario"))?;

    if let (Some(fi), Some(ff)) = (fecha_inicio, fecha_fin) {
      args
        .add(fi.formato_sql())
        .map_err(|_| DBError::Parametros("Fecha inicio"))?;
      args
        .add(ff.formato_sql())
        .map_err(|_| DBError::Parametros("Fecha fin"))?;
    }

    if let Some(ur) = usuario_reg {
      if ur != usuario && ur != 0 {
        args
          .add(ur)
          .map_err(|_| DBError::Parametros("Usuario registrador"))?;
      }
    }

    self
      .marcajes(None, Some(filter), Some(args), Some(ORDER_BY))
      .await
  }

  /// Obtiene los marcaje dado el usuario y la fecha
  ///
  /// Los marcajes deben no tener asignada una incidencia
  /// Dependiendo del usuario registrador pasado como parámetro
  /// se filtran los marcajes de diferente forma.
  pub(in crate::marcaje) async fn marcajes_inc_por_fecha_reg(
    &self,
    usuario: u32,
    fecha: NaiveDate,
    usuario_reg: Option<u32>,
  ) -> Result<DominioWithCacheUsuario<Marcaje>, DBError> {
    const ORDER_BY: &str = "r.hora_inicio DESC";

    let filter = match usuario_reg {
      Some(usr) if usr == usuario => {
        // Si el usuario registrador es igual al usuario,
        // el registrador actua como empleado y solo se
        // filtra por sus solicitudes y no por las que registró
        "r.usuario = ? AND r.fecha = ? 
          AND NOT EXISTS (SELECT id FROM incidencias AS i 
          WHERE i.marcaje = r.id)"
      }
      Some(usr) if usr != 0 => {
        "r.usuario = ? AND r.fecha = ? AND r.usuario_registrador = ? 
          AND NOT EXISTS (SELECT id FROM incidencias AS i 
          WHERE i.marcaje = r.id)"
      }
      Some(_) => {
        // Cuando el usuario es igual a cero significa que
        // un supervidor puede ver todos los marcajes que
        // se registrarón por alguien que no era el usuario
        // del marcaje
        "r.usuario = ? AND r.fecha = ? AND r.usuario_registrador IS NOT NULL 
          AND NOT EXISTS (SELECT id FROM incidencias AS i 
          WHERE i.marcaje = r.id)"
      }
      _ => {
        "r.usuario = ? AND r.fecha = ? 
          AND NOT EXISTS (SELECT id FROM incidencias AS i
          WHERE i.marcaje = r.id)"
      }
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
      if ur != usuario && ur != 0 {
        args
          .add(ur)
          .map_err(|_| DBError::Parametros("Usuario registrador"))?;
      }
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
  ) -> Result<DominioWithCacheUsuario<Marcaje>, DBError> {
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
  ) -> Result<DominioWithCacheUsuario<Marcaje>, DBError> {
    const SELECT: &str = "SELECT r.id, r.fecha,
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

    let capacidad = rows.len();
    let mut resultado = DominioWithCacheUsuario::<Marcaje>::new(capacidad);

    for row in rows {
      resultado.push_usuario(DescriptorUsuario {
        id: row.get("u_id"),
        nombre: row.get("u_nombre"),
        primer_apellido: row.get("u_primer_apellido"),
        segundo_apellido: row.get("u_segundo_apellido"),
      });

      if let Ok(ur_id) = row.try_get::<u32, _>("ur_id") {
        resultado.push_usuario(DescriptorUsuario {
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

      resultado.push_entidad(marcaje);
    }

    Ok(resultado)
  }
}
