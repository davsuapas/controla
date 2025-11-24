use chrono::{NaiveDate, NaiveDateTime};

use sqlx::Row;

use crate::{
  inc::{
    EstadoIncidencia, IncidenciaMarcaje, IncidenciaSolictud, IncidenciaTraza,
    dominio::Incidencia,
  },
  infra::{DBError, DominioWithCacheUsuario, PoolConexion, Transaccion},
  marcaje::DescriptorMarcaje,
  usuarios::DescriptorUsuario,
};

/// Implementación del repositorio de incidencias.
pub struct IncidenciaRepo {
  pool: PoolConexion,
}

impl IncidenciaRepo {
  pub fn new(pool: PoolConexion) -> Self {
    IncidenciaRepo { pool }
  }

  pub(in crate::inc) fn conexion(&self) -> &PoolConexion {
    &self.pool
  }
}

impl IncidenciaRepo {
  /// Añade una incidencia
  pub(in crate::inc) async fn agregar(
    &self,
    reg: &Incidencia,
  ) -> Result<u32, DBError> {
    const QUERY: &str = "INSERT INTO incidencias
      (tipo, fecha_solicitud, hora_inicio, hora_fin, marcaje, estado,
       error, usuario_creador, usuario_gestor, fecha, motivo_solicitud,
       motivo_rechazo, fecha_resolucion, fecha_estado, usuario)
      VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)";

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
      .bind(&reg.motivo_rechazo)
      .bind(reg.fecha_resolucion)
      .bind(reg.fecha_estado)
      .bind(reg.usuario)
      .execute(self.pool.conexion())
      .await
      .map_err(DBError::from_sqlx)?;

    Ok(result.last_insert_id() as u32)
  }

  /// Cambia el estado a solicitud desde rechazado
  ///
  /// La entidad incidencia lleva el estado del que proviene
  ///
  /// Si no proviene de un estado conocido no se actualiza y
  /// devuelve false
  pub(in crate::inc) async fn cambiar_estado_solictud(
    &self,
    trans: &mut Transaccion<'_>,
    inc: &IncidenciaSolictud,
  ) -> Result<bool, DBError> {
    const QUERY: &str = "UPDATE incidencias
      SET estado = ?,
       motivo_solicitud = ?, fecha_solicitud = ?,
       hora_inicio = ?, hora_fin = ?, usuario_creador = ?,
       motivo_rechazo = null, fecha_estado = null,
       error = null, usuario_gestor = null
      WHERE id = ? and estado = ?";

    let result = sqlx::query(QUERY)
      .bind(EstadoIncidencia::Solicitud as u8)
      .bind(&inc.motivo_solicitud)
      .bind(inc.fecha_solicitud)
      .bind(inc.hora_inicio)
      .bind(inc.hora_fin)
      .bind(inc.usuario_creador)
      .bind(inc.id)
      .bind(inc.estado as u8)
      .execute(&mut **trans.deref_mut())
      .await
      .map_err(DBError::from_sqlx)?;

    Ok(result.rows_affected() > 0)
  }

  /// Cambia el estado a resuelto
  ///
  /// Si no proviene de un estado conocido no se actualiza y
  /// devuelve false
  pub(in crate::inc) async fn cambiar_estado_resuelto(
    &self,
    trans: &mut Transaccion<'_>,
    id: u32,
    usuario_gestor: u32,
    fecha_resolucion: NaiveDateTime,
  ) -> Result<bool, DBError> {
    const QUERY: &str = "UPDATE incidencias
      SET estado = ?, error = null, usuario_gestor = ?,
       fecha_resolucion = ?, fecha_estado = null
      WHERE id = ? and estado IN (?, ?)";

    let result = sqlx::query(QUERY)
      .bind(EstadoIncidencia::Resuelta as u8)
      .bind(usuario_gestor)
      .bind(fecha_resolucion)
      .bind(id)
      .bind(EstadoIncidencia::Solicitud as u8)
      .bind(EstadoIncidencia::ErrorResolver as u8)
      .execute(&mut **trans.deref_mut())
      .await
      .map_err(DBError::from_sqlx)?;

    Ok(result.rows_affected() > 0)
  }

  /// Cambia el estado a rechazado
  ///
  /// Si no proviene de un estado conocido no se actualiza y
  /// devuelve false
  pub(in crate::inc) async fn cambiar_estado_rechazado(
    &self,
    trans: &mut Transaccion<'_>,
    id: u32,
    motivo: Option<&str>,
    usuario_gestor: u32,
    fecha_estado: NaiveDateTime,
  ) -> Result<bool, DBError> {
    const QUERY: &str = "UPDATE incidencias
      SET estado = ?, usuario_gestor = ?,
        fecha_estado = ?, motivo_rechazo = ?
      WHERE id = ? and estado = ?";

    let result = sqlx::query(QUERY)
      .bind(EstadoIncidencia::Rechazada as u8)
      .bind(usuario_gestor)
      .bind(fecha_estado)
      .bind(motivo)
      .bind(id)
      .bind(EstadoIncidencia::Solicitud as u8)
      .execute(&mut **trans.deref_mut())
      .await
      .map_err(DBError::from_sqlx)?;

    Ok(result.rows_affected() > 0)
  }

  /// Cambia el estado a conflicto o error
  pub(in crate::inc) async fn cambiar_estado_incidente(
    &self,
    trans: &mut Transaccion<'_>,
    id: u32,
    estado: EstadoIncidencia,
    error: &str,
    fecha_estado: NaiveDateTime,
  ) -> Result<(), DBError> {
    // La fecha resolución y el usuario gestor
    // se blanquea porque antes de cambiar el estado
    // se cambio a estado resolución para bloquear
    // el registro
    const QUERY: &str = "UPDATE incidencias
      SET estado = ?, error = ?, fecha_estado = ?,
      usuario_gestor = null, fecha_resolucion = null
      WHERE id = ?";

    sqlx::query(QUERY)
      .bind(estado as u8)
      .bind(error)
      .bind(fecha_estado)
      .bind(id)
      .execute(&mut **trans.deref_mut())
      .await
      .map_err(DBError::from_sqlx)?;

    Ok(())
  }

  /// Devuelve una incidencia para traza
  pub(in crate::inc) async fn incidencia_para_traza(
    &self,
    inc_id: u32,
  ) -> Result<IncidenciaTraza, DBError> {
    const QUERY: &str = "SELECT
      motivo_solicitud, fecha_solicitud,
      hora_inicio, hora_fin,
      motivo_rechazo, fecha_estado,
      usuario_creador, usuario_gestor, error
      FROM incidencias
      WHERE id = ?";

    let row = sqlx::query(QUERY)
      .bind(inc_id)
      .fetch_optional(self.pool.conexion())
      .await
      .map_err(DBError::from_sqlx)?;

    if let Some(row) = row {
      Ok(IncidenciaTraza {
        motivo_solicitud: row.get("motivo_solicitud"),
        fecha_solicitud: row.get("fecha_solicitud"),
        hora_inicio: row.get("hora_inicio"),
        hora_fin: row.get("hora_fin"),
        motivo_rechazo: row.get("motivo_rechazo"),
        fecha_estado: row.get("fecha_estado"),
        usuario_creador: row.get("usuario_creador"),
        usuario_gestor: row.try_get::<u32, _>("usuario_gestor").ok(),
        error: row.get("error"),
      })
    } else {
      Err(DBError::registro_vacio(format!(
        "No se ha encontrado la incidencia: {}",
        &inc_id
      )))
    }
  }

  /// Devuelve una incidencia con la info mínima necesaria para el marcaje.
  pub(in crate::inc) async fn incidencia_para_marcaje(
    &self,
    inc_id: u32,
  ) -> Result<IncidenciaMarcaje, DBError> {
    const QUERY: &str = "SELECT
      i.tipo, i.usuario, i.fecha, i.hora_inicio,
      i.hora_fin, i.marcaje, i.estado, i.usuario_creador,
      m.hora_inicio AS m_hora_inicio, m.hora_fin AS m_hora_fin      
      FROM incidencias i
      LEFT JOIN marcajes m ON i.marcaje = m.id
      WHERE i.id = ?";

    let row = sqlx::query(QUERY)
      .bind(inc_id)
      .fetch_optional(self.pool.conexion())
      .await
      .map_err(DBError::from_sqlx)?;

    if let Some(row) = row {
      Ok(IncidenciaMarcaje {
        tipo: row.get::<u8, _>("tipo").into(),
        usuario: row.get("usuario"),
        fecha: row.get("fecha"),
        hora_inicio: row.get("hora_inicio"),
        hora_fin: row.get("hora_fin"),
        marcaje: row.try_get::<u32, _>("marcaje").ok().map(|m_id| {
          DescriptorMarcaje {
            id: m_id,
            hora_inicio: row.get("m_hora_inicio"),
            hora_fin: row.get("m_hora_fin"),
          }
        }),
        usuario_creador: row.get("usuario_creador"),
      })
    } else {
      Err(DBError::registro_vacio(format!(
        "No se ha encontrado la incidencia: {}",
        &inc_id
      )))
    }
  }

  /// Lista las incidencias que cumplen los filtros indicados.
  ///
  /// Si se indica ID solo se devuelve esa incidencia
  ///
  /// Si como parámetro se especifica que es supervisor
  /// se obtiene las incidencias que se hicieron por los
  /// registradores o las suyas propias.
  pub(in crate::inc) async fn incidencias(
    &self,
    id: Option<u32>,
    fecha_inicio: Option<NaiveDate>,
    fecha_fin: Option<NaiveDate>,
    estados: &[EstadoIncidencia],
    supervisor: bool,
    usuario: Option<u32>,
  ) -> Result<DominioWithCacheUsuario<Incidencia>, DBError> {
    let mut qb = sqlx::QueryBuilder::<sqlx::MySql>::new(
      r"SELECT
      i.id, i.tipo, i.fecha_solicitud,
      i.fecha, i.hora_inicio, i.hora_fin, 
      i.marcaje, i.estado, i.error,
      i.motivo_solicitud, i.motivo_rechazo,
      i.fecha_resolucion, i.fecha_estado,
      u.id AS u_id, u.nombre AS u_nombre,
      u.primer_apellido AS u_primer_apellido,
      u.segundo_apellido AS u_segundo_apellido,
      uc.id AS uc_id, uc.nombre AS uc_nombre,
      uc.primer_apellido AS uc_primer_apellido,
      uc.segundo_apellido AS uc_segundo_apellido,
      ug.id AS ug_id, ug.nombre AS ug_nombre,
      ug.primer_apellido AS ug_primer_apellido,
      ug.segundo_apellido AS ug_segundo_apellido,
      m.hora_inicio AS m_hora_inicio, m.hora_fin AS m_hora_fin
      FROM incidencias i
      JOIN usuarios u ON i.usuario = u.id      
      JOIN usuarios uc ON i.usuario_creador = uc.id
      LEFT JOIN usuarios ug ON i.usuario_gestor = ug.id
      LEFT JOIN marcajes m ON i.marcaje = m.id
      WHERE ",
    );

    if let Some(id_incidencia) = id {
      // Si hay ID, buscar solo por ID
      qb.push("i.id = ");
      qb.push_bind(id_incidencia);
    } else {
      // Si no hay ID, aplicar los filtros normales
      qb.push("estado IN (");
      {
        let mut separated = qb.separated(", ");
        for e in estados {
          separated.push_bind(*e as u8);
        }
      }
      qb.push(")");

      if let (Some(fi), Some(ff)) = (fecha_inicio, fecha_fin) {
        qb.push(" AND ");
        qb.push("i.fecha_solicitud BETWEEN ");
        qb.push_bind(fi.and_hms_opt(0, 0, 0).unwrap()); // Inicio del día
        qb.push(" AND ");
        qb.push_bind(ff.and_hms_opt(23, 59, 59).unwrap()); // Fin del día
      }

      if let Some(u) = usuario {
        qb.push(" AND ");
        if supervisor {
          qb.push("(i.usuario_creador <> i.usuario or i.usuario_creador = ")
            .push_bind(u)
            .push(")");
        } else {
          qb.push("i.usuario_creador = ").push_bind(u);
        }
      }

      qb.push(" ORDER BY i.fecha_solicitud ASC, i.estado ASC, i.fecha ASC");
    }

    let rows = qb
      .build()
      .fetch_all(self.pool.conexion())
      .await
      .map_err(DBError::from_sqlx)?;

    let capacidad = rows.len();
    let mut resultado = DominioWithCacheUsuario::<Incidencia>::new(capacidad);

    for row in rows {
      resultado.push_usuario(DescriptorUsuario {
        id: row.get("u_id"),
        nombre: row.get("u_nombre"),
        primer_apellido: row.get("u_primer_apellido"),
        segundo_apellido: row.get("u_segundo_apellido"),
      });

      resultado.push_usuario(DescriptorUsuario {
        id: row.get("uc_id"),
        nombre: row.get("uc_nombre"),
        primer_apellido: row.get("uc_primer_apellido"),
        segundo_apellido: row.get("uc_segundo_apellido"),
      });

      if let Ok(ug_id) = row.try_get::<u32, _>("ug_id") {
        resultado.push_usuario(DescriptorUsuario {
          id: ug_id,
          nombre: row.get("ug_nombre"),
          primer_apellido: row.get("ug_primer_apellido"),
          segundo_apellido: row.get("ug_segundo_apellido"),
        });
      }

      let incidencia = Incidencia {
        id: row.get("id"),
        tipo: row.get::<u8, _>("tipo").into(),
        fecha_solicitud: row.get("fecha_solicitud"),
        fecha_resolucion: row.try_get("fecha_resolucion").ok(),
        fecha: row.get("fecha"),
        usuario: row.get("u_id"),
        hora_inicio: row.get("hora_inicio"),
        hora_fin: row.get("hora_fin"),
        marcaje: row.try_get::<u32, _>("marcaje").ok().map(|m_id| {
          DescriptorMarcaje {
            id: m_id,
            hora_inicio: row.get("m_hora_inicio"),
            hora_fin: row.get("m_hora_fin"),
          }
        }),
        estado: row.get::<u8, _>("estado").into(),
        fecha_estado: row.try_get("fecha_estado").ok(),
        error: row.get("error"),
        usuario_creador: row.get("uc_id"),
        usuario_gestor: row.try_get::<u32, _>("ug_id").ok(),
        motivo_solicitud: row.get("motivo_solicitud"),
        motivo_rechazo: row.get("motivo_rechazo"),
      };

      resultado.push_entidad(incidencia);
    }

    Ok(resultado)
  }
}
