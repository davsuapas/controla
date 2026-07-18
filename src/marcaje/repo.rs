use chrono::{NaiveDate, NaiveTime};

use sqlx::{QueryBuilder, Row};

use crate::{
  horario::DescriptorHorario,
  infra::{
    DBError, DominioWithCacheUsuario, PoolConexion, ShortDateTimeFormat,
    Transaccion,
  },
  marcaje::{DescriptorMarcaje, Marcaje},
  usuarios::DescriptorUsuario,
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
    tracing::debug!(
      usuario = usuario,
      fecha = ?fecha,
      excluir_marcaje_id = excluir_marcaje_id,
      "Verificando si existe un marcaje sin hora fin para el usuario y fecha"
    );

    const QUERY: &str = "SELECT id
      FROM marcajes
      WHERE usuario = ? AND fecha = ?
      AND id <> ? AND modificado_por IS NULL AND eliminado IS NULL
      AND hora_fin IS NULL
      LIMIT 1;";

    Ok(
      sqlx::query_scalar::<_, u32>(QUERY)
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
    limit: Option<&str>,
  ) -> Result<DominioWithCacheUsuario<Marcaje>, DBError> {
    const ORDER_BY: &str = "r.fecha DESC, r.hora_inicio DESC";

    self
      .marcajes(limit, Some(ORDER_BY), |qb| {
        qb.push("r.usuario = ");
        qb.push_bind(usuario);
        Ok(())
      })
      .await
  }

  /// Obtiene los marcaje dado el usuario entre dos fechas.
  ///
  /// Si la fechas son Nones se filtra solos por usuario.
  /// Si el usuario_reg es igual a 0, significa que es supervisor
  /// y puede ver todos los marcajes de cualquier usuario.
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
    limit: u8,
  ) -> Result<DominioWithCacheUsuario<Marcaje>, DBError> {
    const ORDER_BY: &str = "r.fecha DESC, r.hora_inicio DESC";

    let top = if fecha_inicio.is_none() && fecha_fin.is_none() {
      Some(limit.to_string())
    } else {
      None
    };

    self
      .marcajes(top.as_deref(), Some(ORDER_BY), |qb| {
        qb.push("r.usuario = ");
        qb.push_bind(usuario);

        if let (Some(fi), Some(ff)) = (fecha_inicio, fecha_fin) {
          qb.push(" AND r.fecha BETWEEN ");
          qb.push_bind(fi.formato_sql());
          qb.push(" AND ");
          qb.push_bind(ff.formato_sql());
        }

        if let Some(ur) = usuario_reg
          && ur != usuario
          && ur != 0
        {
          qb.push(" AND r.usuario_registrador = ");
          qb.push_bind(ur);
        }

        Ok(())
      })
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

    self
      .marcajes(None, Some(ORDER_BY), |qb| {
        qb.push("r.usuario = ");
        qb.push_bind(usuario);
        qb.push(" AND r.fecha = ");
        qb.push_bind(fecha.formato_sql());

        match usuario_reg {
          Some(ur) if ur == usuario => {
            // No extra usuario_registrador condition needed
          }
          Some(ur) if ur != 0 => {
            qb.push(" AND r.usuario_registrador = ");
            qb.push_bind(ur);
          }
          Some(_) => {
            // Supervisor: show marcajes registered by someone else
            qb.push(" AND r.usuario_registrador IS NOT NULL");
          }
          _ => {}
        }

        qb.push(
          " AND NOT EXISTS
           (SELECT id FROM incidencias AS i 
           WHERE i.marcaje = r.id AND estado <> 8)",
        );

        Ok(())
      })
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
      .marcajes(None, Some(ORDER_BY), |qb| {
        qb.push("r.usuario = ");
        qb.push_bind(usuario);
        qb.push(" AND r.fecha = ");
        qb.push_bind(fecha.formato_sql());
        Ok(())
      })
      .await
  }

  async fn marcajes<B>(
    &self,
    limit: Option<&str>,
    order: Option<&str>,
    build_where: B,
  ) -> Result<DominioWithCacheUsuario<Marcaje>, DBError>
  where
    B: FnOnce(&mut QueryBuilder<sqlx::MySql>) -> Result<(), DBError>,
  {
    const SELECT: &str = "SELECT r.id, r.fecha,
        r.hora_inicio, r.hora_fin,
        u.id AS u_id, u.nombre AS u_nombre,
        u.primer_apellido AS u_primer_apellido,
        u.segundo_apellido AS u_segundo_apellido,
        ur.id AS ur_id, ur.nombre AS ur_nombre,
        ur.primer_apellido AS ur_primer_apellido,
        ur.segundo_apellido AS ur_segundo_apellido,
        h.id AS h_id, h.dia, h.horas
        FROM marcajes r
        JOIN horarios h ON h.id = r.horario
        JOIN usuarios u ON u.id = r.usuario
        LEFT JOIN usuarios ur ON ur.id = r.usuario_registrador";

    let mut qb = QueryBuilder::new(SELECT);

    qb.push(" WHERE ");
    build_where(&mut qb)?;
    qb.push(" AND modificado_por IS NULL AND eliminado IS NULL");

    if let Some(o) = order {
      qb.push(" ORDER BY ");
      qb.push(o);
    }

    if let Some(t) = limit {
      qb.push(" LIMIT ");
      qb.push(t);
    }

    let rows = qb
      .build()
      .fetch_all(self.pool.conexion())
      .await
      .map_err(DBError::from_sqlx)?;

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
        horario: Some(DescriptorHorario {
          id: row.get("h_id"),
          dia: row.get::<String, _>("dia").as_str().into(),
          horas: row.get("horas"),
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
