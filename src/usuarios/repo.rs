use sqlx::Row;

use chrono::{Datelike, NaiveDate, NaiveDateTime};

use crate::{
  infraestructura::{DBError, PoolConexion, dia_semana_letra},
  usuarios::{Dia, Horario},
};

/// Implementación del repositorio de los horarios de usuario.
pub struct HorarioRepo {
  pool: PoolConexion,
}

impl HorarioRepo {
  pub fn new(pool: PoolConexion) -> Self {
    HorarioRepo { pool }
  }
}

impl HorarioRepo {
  /// Obtiene el horario más cercano a una hora dada para un usuario.
  pub(in crate::usuarios) async fn horario_cercano(
    &self,
    usuario: u64,
    hora: NaiveDateTime,
  ) -> Result<Horario, DBError> {
    let fecha_creacion = sqlx::query_scalar::<_, Option<NaiveDate>>(
      r"SELECT MAX(fecha_creacion) 
    FROM usuario_horarios 
    WHERE usuario = ? 
    AND fecha_creacion < ?",
    )
    .bind(usuario)
    .bind(hora)
    .fetch_optional(self.pool.conexion())
    .await
    .map_err(DBError::consulta_from)?
    .ok_or_else(|| {
      DBError::registro_vacio(format!(
        "No se ha encontrado ningún horario configurado \
        para el usuario: {} y fecha: {}",
        usuario, hora
      ))
    })?;

    let dia = dia_semana_letra(hora.weekday()).to_string();

    let result = sqlx::query(
      r"SELECT h.id, h.dia, h.hora_inicio, h.hora_fin
        FROM horarios h
        JOIN usuario_horarios uh ON h.id = uh.horario
        WHERE uh.usuario = ?
        AND uh.fecha_creacion = ?
        AND h.dia = ?
        AND ? BETWEEN h.hora_inicio AND h.hora_fin;",
    )
    .bind(usuario)
    .bind(fecha_creacion)
    .bind(&dia)
    .bind(hora.time())
    .fetch_optional(self.pool.conexion())
    .await
    .map_err(DBError::consulta_from)?;

    if let Some(row) = result {
      Ok(Horario {
        id: row.get("id"),
        dia: Dia::desde_str(row.get::<String, _>("dia").as_str()).unwrap(),
        hora_inicio: row.get("hora_inicio"),
        hora_fin: row.get("hora_fin"),
      })
    } else {
      // Si no encuentra un horario entre las horas de inicio y fin,
      // devuelve el más cercano al inicio.
      let result = sqlx::query(
        r"SELECT h.id, h.dia, h.hora_inicio, h.hora_fin
            FROM horarios h
            JOIN usuario_horarios uh ON h.id = uh.horario
            WHERE uh.usuario = ?
            AND uh.fecha_creacion = ?
            AND h.dia = ?
            AND h.hora_inicio > ?
            LIMIT 1;",
      )
      .bind(usuario)
      .bind(fecha_creacion)
      .bind(&dia)
      .bind(hora.time())
      .fetch_optional(self.pool.conexion())
      .await
      .map_err(DBError::consulta_from)?;

      if let Some(row) = result {
        Ok(Horario {
          id: row.get("id"),
          dia: Dia::desde_str(row.get::<String, _>("dia").as_str()).unwrap(),
          hora_inicio: row.get("hora_inicio"),
          hora_fin: row.get("hora_fin"),
        })
      } else {
        Err(DBError::registro_vacio(format!(
          "No se ha encontrado ningún horario \
          para el usuario: {}, fecha: {} y día de la seamana: {}",
          usuario, hora, &dia
        )))
      }
    }
  }
}
