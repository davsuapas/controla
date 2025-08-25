use chrono_tz::Tz;
use sqlx::{Row, mysql::MySqlRow};

use chrono::{Datelike, NaiveDate, NaiveDateTime, Utc};

use crate::{
  infra::{DBError, PoolConexion, ShortDateFormat, ShortDateTimeFormat},
  usuarios::{DescriptorUsuario, Dia, Horario, Rol},
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
  /// Obtiene los usuarios que tienen un rol específico.
  pub(in crate::usuarios) async fn usuarios_por_rol(
    &self,
    rol: Rol,
  ) -> Result<Vec<DescriptorUsuario>, DBError> {
    const QUERY: &str = r"SELECT u.id, u.nombre,
          u.primer_apellido, u.segundo_apellido 
          FROM usuarios u
          JOIN roles_usuario ru ON u.id = ru.usuario
          WHERE ru.rol = ?;";

    let rows = sqlx::query(QUERY)
      .bind(rol as u32)
      .fetch_all(self.pool.conexion())
      .await
      .map_err(DBError::consulta_from)?;

    Ok(
      rows
        .into_iter()
        .map(|row| DescriptorUsuario {
          id: row.get("id"),
          nombre: row.get("nombre"),
          primer_apellido: row.get("primer_apellido"),
          segundo_apellido: row.get("segundo_apellido"),
        })
        .collect(),
    )
  }

  /// Obtiene el horario más cercano a una hora dada para un usuario.
  ///
  /// Busca un horario que esté entre las horas de inicio y fin
  /// del día de la semana y que no esté ya asignado a un registro horario.
  /// Si no encuentra un horario entre las horas de inicio y fin,
  /// devuelve el más cercano al inicio y que no esté ya asignado
  /// a un registro horario.
  pub(in crate::usuarios) async fn horario_cercano(
    &self,
    usuario: u32,
    hora: NaiveDateTime,
  ) -> Result<Horario, DBError> {
    let fecha_creacion = self.fecha_creacion_horario(usuario, hora).await?;

    let dia = crate::infra::letra_dia_semana(hora.weekday()).to_string();

    // Busca un horario que esté entre las horas de inicio y fin
    // del día de la semana y que no esté ya asignado a un registro horario.
    const QUERY: &str = r"SELECT h.id, h.dia, h.hora_inicio, h.hora_fin
        FROM horarios h
         JOIN usuario_horarios uh ON h.id = uh.horario
        WHERE uh.usuario = ?
         AND uh.fecha_creacion = ?
         AND h.dia = ?
         AND ? BETWEEN h.hora_inicio AND h.hora_fin
         AND NOT EXISTS 
         ( SELECT r.id
            FROM registros r
            WHERE r.usuario = uh.usuario
             AND r.fecha = ?
             AND r.horario = h.id);";

    let result = sqlx::query(QUERY)
      .bind(usuario)
      .bind(fecha_creacion)
      .bind(&dia)
      .bind(hora.time())
      .bind(hora.date())
      .fetch_optional(self.pool.conexion())
      .await
      .map_err(DBError::consulta_from)?;

    if let Some(row) = result {
      Ok(HorarioRepo::horario_from_row(&row))
    } else {
      // Si no encuentra un horario entre las horas de inicio y fin,
      // devuelve el más cercano al inicio
      // y que no esté ya asignado a un registro horario.
      const QUERY: &str = r"SELECT h.id, h.dia, h.hora_inicio, h.hora_fin
            FROM horarios h
            JOIN usuario_horarios uh ON h.id = uh.horario
            WHERE uh.usuario = ?
             AND uh.fecha_creacion = ?
             AND h.dia = ?
             AND h.hora_inicio > ?
             AND NOT EXISTS 
             ( SELECT r.id
                 FROM registros r
                 WHERE r.usuario = uh.usuario
                  AND r.fecha = ?
                  AND r.horario = h.id)
             LIMIT 1;";

      let result = sqlx::query(QUERY)
        .bind(usuario)
        .bind(fecha_creacion)
        .bind(&dia)
        .bind(hora.time())
        .bind(hora.date())
        .fetch_optional(self.pool.conexion())
        .await
        .map_err(DBError::consulta_from)?;

      if let Some(row) = result {
        Ok(HorarioRepo::horario_from_row(&row))
      } else {
        Err(DBError::registro_vacio(format!(
          "No se ha encontrado ningún horario registrado en la fecha: {}, \
            para el usuario en la fecha: {} y día de la seamana: {}. \
            Verifique que los horarios no estén ya asignados a un registro.",
          fecha_creacion.formato_corto(),
          hora,
          &dia
        )))
      }
    }
  }

  /// Obtiene el horario asignado al usuario para el día actual.
  pub(in crate::usuarios) async fn horarios_hoy_usuario(
    &self,
    tz: &Tz,
    usuario: u32,
  ) -> Result<Vec<Horario>, DBError> {
    let hora = Utc::now().with_timezone(tz).naive_local();
    let fecha_creacion = self.fecha_creacion_horario(usuario, hora).await?;
    let dia = crate::infra::letra_dia_semana(hora.weekday()).to_string();

    const QUERY: &str = r"SELECT h.id, h.dia, h.hora_inicio, h.hora_fin
        FROM horarios h
         JOIN usuario_horarios uh ON h.id = uh.horario
        WHERE uh.usuario = ?
         AND uh.fecha_creacion = ?
         AND h.dia = ?
         AND NOT EXISTS 
         ( SELECT r.id
            FROM registros r
            WHERE r.usuario = uh.usuario
             AND r.fecha = ?
             AND r.horario = h.id);";

    let rows = sqlx::query(QUERY)
      .bind(usuario)
      .bind(fecha_creacion)
      .bind(&dia)
      .bind(hora.date())
      .fetch_all(self.pool.conexion())
      .await
      .map_err(DBError::consulta_from)?;

    Ok(
      rows
        .into_iter()
        .map(|row| HorarioRepo::horario_from_row(&row))
        .collect(),
    )
  }

  /// Obtiene la fecha de creación del horario más reciente
  async fn fecha_creacion_horario(
    &self,
    usuario: u32,
    hora: NaiveDateTime,
  ) -> Result<NaiveDate, DBError> {
    const QUERY: &str = r"SELECT MAX(fecha_creacion) 
    FROM usuario_horarios 
    WHERE usuario = ? 
    AND fecha_creacion < ?";

    let fecha_creacion = sqlx::query_scalar::<_, Option<NaiveDate>>(QUERY)
      .bind(usuario)
      .bind(hora)
      .fetch_one(self.pool.conexion())
      .await
      .map_err(DBError::consulta_from)?
      .ok_or_else(|| {
        DBError::registro_vacio(format!(
          "No se ha encontrado ningún horario configurado \
        para el usuario en la fecha: {}",
          hora.formato_corto()
        ))
      })?;

    Ok(fecha_creacion)
  }

  fn horario_from_row(row: &MySqlRow) -> Horario {
    Horario {
      id: row.get("id"),
      dia: Dia::from(row.get::<String, _>("dia").chars().next().unwrap()),
      hora_inicio: row.get("hora_inicio"),
      hora_fin: row.get("hora_fin"),
    }
  }
}
