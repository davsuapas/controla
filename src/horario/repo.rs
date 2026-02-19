use std::{collections::HashMap, ops::Add};

use chrono::{Datelike, NaiveDate, NaiveDateTime};
use sqlx::{Row, mysql::MySqlRow};

use crate::{
  horario::{
    Calendario, CalendarioFecha, ConfigHorario, Dia, Horario,
    TipoCalendarioFecha,
  },
  infra::{
    DBError, DateOptional, NONE_DATE, PoolConexion, ShortDateTimeFormat,
    Transaccion,
  },
};

/// Implementación del repositorio de los horarios.
pub struct HorarioRepo {
  pool: PoolConexion,
}

impl HorarioRepo {
  pub fn new(pool: PoolConexion) -> Self {
    HorarioRepo { pool }
  }

  pub(in crate::horario) fn conexion(&self) -> &PoolConexion {
    &self.pool
  }
}

impl HorarioRepo {
  /// Obtiene el horario más cercano a una hora dada para un usuario.
  pub(in crate::horario) async fn horario_cercano(
    &self,
    usuario: u32,
    hora: NaiveDateTime,
    excluir_marcaje_id: u32,
  ) -> Result<(u32, Horario), DBError> {
    let fecha = hora.date();
    let dia = crate::infra::letra_dia_semana(hora.weekday());
    let hora_buscar = hora.time();

    let fecha_creacion = self.fecha_creacion_horario(usuario, fecha).await?;

    tracing::debug!(
      usuario = usuario,
      fecha = %fecha,
      hora = %hora_buscar,
      fecha_creacion = %fecha_creacion,
      dia = %dia,
      excluir_marcaje_id = excluir_marcaje_id,
      "Buscando el horario más cercano del usuario"
    );

    const QUERY: &str = "SELECT uh.id AS uh_id,
         h.id, h.dia, h.hora_inicio, h.hora_fin
        FROM usuario_horarios uh
         JOIN horarios h ON h.id = uh.horario
        WHERE uh.usuario = ?
         AND uh.fecha_creacion = ?
         AND (uh.caducidad_fecha_fin IS NULL OR ? >= uh.caducidad_fecha_ini)
         AND (uh.caducidad_fecha_fin IS NULL OR ? <= uh.caducidad_fecha_fin)         
         AND h.dia = ?
         AND ? BETWEEN h.hora_inicio AND h.hora_fin
         AND NOT EXISTS 
         (SELECT r.id
            FROM marcajes r
            WHERE r.usuario = uh.usuario AND r.fecha = ?
             AND r.id <> ? AND modificado_por IS NULL AND eliminado IS NULL
             AND r.usuario_horario = uh.id)
         AND ? > COALESCE(
         (SELECT MAX(r2.hora_fin)
            FROM marcajes r2
            JOIN usuario_horarios uh2 ON uh2.id = r2.usuario_horario
            JOIN horarios h2 ON h2.id = uh2.horario
            WHERE r2.usuario = uh.usuario AND r2.fecha = ?
              AND r2.id <> ? AND modificado_por IS NULL AND eliminado IS NULL
              AND h2.hora_inicio < h.hora_inicio),
        '00:00:00')";

    let result = sqlx::query(QUERY)
      .bind(usuario)
      .bind(fecha_creacion)
      .bind(fecha)
      .bind(fecha)
      .bind(dia)
      .bind(hora_buscar)
      .bind(fecha)
      .bind(excluir_marcaje_id)
      .bind(hora_buscar)
      .bind(fecha)
      .bind(excluir_marcaje_id)
      .fetch_optional(self.pool.conexion())
      .await
      .map_err(DBError::from_sqlx)?;

    if let Some(row) = result {
      Ok((row.get("uh_id"), horario_from_row(&row)))
    } else {
      const QUERY: &str = "SELECT uh.id AS uh_id,
              h.id, h.dia, h.hora_inicio, h.hora_fin
            FROM horarios h
            JOIN usuario_horarios uh ON h.id = uh.horario
            WHERE uh.usuario = ?
             AND uh.fecha_creacion = ?
             AND (uh.caducidad_fecha_fin IS NULL OR ? >= uh.caducidad_fecha_ini)
             AND (uh.caducidad_fecha_fin IS NULL OR ? <= uh.caducidad_fecha_fin)         
             AND h.dia = ?
             AND h.hora_inicio > ?
             AND NOT EXISTS 
             ( SELECT r.id
                FROM marcajes r
                WHERE r.usuario = uh.usuario AND r.fecha = ?
                 AND r.id <> ? AND modificado_por IS NULL AND eliminado IS NULL
                 AND r.usuario_horario = uh.id)
            AND ? > COALESCE(
            (SELECT MAX(r2.hora_fin)
              FROM marcajes r2
                JOIN usuario_horarios uh2 ON uh2.id = r2.usuario_horario
                JOIN horarios h2 ON h2.id = uh2.horario
              WHERE r2.usuario = uh.usuario
               AND r2.fecha = ?
               AND r2.id <> ? AND modificado_por IS NULL AND eliminado IS NULL
               AND h2.hora_inicio < h.hora_inicio),
            '00:00:00')
            LIMIT 1;";

      let result = sqlx::query(QUERY)
        .bind(usuario)
        .bind(fecha_creacion)
        .bind(fecha)
        .bind(fecha)
        .bind(dia)
        .bind(hora_buscar)
        .bind(fecha)
        .bind(excluir_marcaje_id)
        .bind(hora_buscar)
        .bind(fecha)
        .bind(excluir_marcaje_id)
        .fetch_optional(self.pool.conexion())
        .await
        .map_err(DBError::from_sqlx)?;

      if let Some(row) = result {
        Ok((row.get("uh_id"), horario_from_row(&row)))
      } else {
        Err(DBError::registro_vacio(format!(
          "No se ha encontrado ningún horario próximo a la fecha: {}, \
           que no este ya asignado. \
           Verifique sus horarios creados en la fecha: {}",
          hora,
          fecha_creacion.formato_corto()
        )))
      }
    }
  }

  /// Obtiene los horarios sin asignar para un usuario en una fecha dada.
  pub(in crate::horario) async fn horarios_usuario_sin_asignar(
    &self,
    usuario: u32,
    fecha: NaiveDate,
  ) -> Result<Vec<Horario>, DBError> {
    let fecha_creacion = self.fecha_creacion_horario(usuario, fecha).await?;
    let dia = crate::infra::letra_dia_semana(fecha.weekday());

    const QUERY: &str = "SELECT h.id, h.dia, h.hora_inicio, h.hora_fin
        FROM usuario_horarios uh
         JOIN horarios h ON h.id = uh.horario
        WHERE uh.usuario = ?
         AND uh.fecha_creacion = ?
         AND (uh.caducidad_fecha_fin IS NULL OR ? >= uh.caducidad_fecha_ini)
         AND (uh.caducidad_fecha_fin IS NULL OR ? <= uh.caducidad_fecha_fin)         
         AND h.dia = ?
         AND NOT EXISTS 
         ( SELECT r.id
            FROM marcajes r
            WHERE r.usuario = uh.usuario AND r.fecha = ?
             AND r.usuario_horario = uh.id
             AND modificado_por IS NULL AND eliminado IS NULL)
        ORDER BY h.hora_inicio;";

    let rows = sqlx::query(QUERY)
      .bind(usuario)
      .bind(fecha_creacion)
      .bind(fecha)
      .bind(fecha)
      .bind(dia)
      .bind(fecha)
      .fetch_all(self.pool.conexion())
      .await
      .map_err(DBError::from_sqlx)?;

    Ok(rows.into_iter().map(|row| horario_from_row(&row)).collect())
  }

  /// Crea una nueva configuración de horario para un usuario.
  pub(in crate::horario) async fn agregar_config_usuario(
    &self,
    trans: &mut Transaccion<'_>,
    config: &ConfigHorario,
    id_horario: u32,
  ) -> Result<u32, DBError> {
    const QUERY: &str = "INSERT INTO usuario_horarios
      (usuario, horario, fecha_creacion, 
      caducidad_fecha_ini, caducidad_fecha_fin)
      VALUES (?, ?, ?, ?, ?);";

    let cad_fecha_ini = config.caducidad_fecha_ini.convert_to_date();

    let res = sqlx::query(QUERY)
      .bind(config.usuario)
      .bind(id_horario)
      .bind(config.fecha_creacion)
      .bind(cad_fecha_ini)
      .bind(config.caducidad_fecha_fin)
      .execute(&mut **trans.deref_mut())
      .await
      .map_err(DBError::from_sqlx)?;

    Ok(res.last_insert_id() as u32)
  }

  /// Modifica una configuración de horario para un usuario.
  pub(in crate::horario) async fn modificar_config_usuario(
    &self,
    trans: &mut Transaccion<'_>,
    config: &ConfigHorario,
    id_horario: u32,
  ) -> Result<(), DBError> {
    const QUERY: &str = "UPDATE usuario_horarios SET
        horario = ?, caducidad_fecha_ini = ?, caducidad_fecha_fin = ?
        WHERE id = ?;";

    let cad_fecha_ini = config.caducidad_fecha_ini.convert_to_date();

    let res = sqlx::query(QUERY)
      .bind(id_horario)
      .bind(cad_fecha_ini)
      .bind(config.caducidad_fecha_fin)
      .bind(config.id)
      .execute(&mut **trans.deref_mut())
      .await
      .map_err(DBError::from_sqlx)?;

    if res.rows_affected() == 0 {
      Err(DBError::registro_vacio(
        "Modificando configuración de horario".to_string(),
      ))
    } else {
      Ok(())
    }
  }

  /// Elimina una configuración de horario para un usuario.
  pub(in crate::horario) async fn eliminar_config_usuario(
    &self,
    trans: &mut Transaccion<'_>,
    id: u32,
  ) -> Result<(), DBError> {
    const QUERY: &str = "DELETE FROM usuario_horarios WHERE id = ?;";

    let res = sqlx::query(QUERY)
      .bind(id)
      .execute(&mut **trans.deref_mut())
      .await
      .map_err(DBError::from_sqlx)?;

    if res.rows_affected() == 0 {
      Err(DBError::registro_vacio(
        "Eliminando configuración de horario".to_string(),
      ))
    } else {
      Ok(())
    }
  }

  /// Duplica la configuración de un horario.
  pub(in crate::horario) async fn duplicar_config_horario(
    &self,
    usuario: u32,
    nueva_fecha_creacion: NaiveDate,
  ) -> Result<(), DBError> {
    let fecha_creacion = self
      .fecha_creacion_horario(
        usuario,
        nueva_fecha_creacion.add(chrono::Duration::days(1)),
      )
      .await?;

    const QUERY: &str = "INSERT INTO usuario_horarios
      (usuario, horario, fecha_creacion,
       caducidad_fecha_ini, caducidad_fecha_fin)
      SELECT usuario, horario, ?, caducidad_fecha_ini, caducidad_fecha_fin
      FROM usuario_horarios
      WHERE usuario = ? AND fecha_creacion = ?
      AND caducidad_fecha_fin IS NULL;";

    sqlx::query(QUERY)
      .bind(nueva_fecha_creacion)
      .bind(usuario)
      .bind(fecha_creacion)
      .execute(self.pool.conexion())
      .await
      .map_err(DBError::from_sqlx)?;

    Ok(())
  }

  /// Obtiene un horario configurado dado el id.
  pub(in crate::horario) async fn config_horario_por_id(
    &self,
    id: u32,
  ) -> Result<ConfigHorario, DBError> {
    const QUERY: &str = "SELECT h.id, uh.id AS uh_id,
        uh.usuario, uh.fecha_creacion,
        uh.caducidad_fecha_ini, uh.caducidad_fecha_fin,
        h.dia, h.hora_inicio, h.hora_fin
      FROM usuario_horarios uh
      JOIN horarios h ON uh.horario = h.id
      WHERE uh.id = ?";

    let row = sqlx::query(QUERY)
      .bind(id)
      .fetch_optional(self.pool.conexion())
      .await
      .map_err(DBError::from_sqlx)?;

    if let Some(row) = row {
      Ok(config_horario_from_row(&row))
    } else {
      Err(DBError::registro_vacio(format!(
        "No se ha encontrado ningún horario configurado con id: {}",
        id
      )))
    }
  }

  /// Obtiene una lista de horarios configurados para un usuario
  pub(in crate::horario) async fn config_horario(
    &self,
    usuario: u32,
    fecha_actual: NaiveDate,
  ) -> Result<Vec<ConfigHorario>, DBError> {
    let fecha_creacion = match self
      .fecha_creacion_horario(
        usuario,
        fecha_actual.add(chrono::Duration::days(1)),
      )
      .await
    {
      Ok(fecha) => fecha,
      Err(DBError::RegistroVacio(_)) => return Ok(vec![]),
      Err(e) => return Err(e),
    };

    const QUERY: &str = "SELECT h.id, uh.id AS uh_id,
        uh.usuario, uh.fecha_creacion,
        uh.caducidad_fecha_ini, uh.caducidad_fecha_fin,
        h.dia, h.hora_inicio, h.hora_fin
      FROM usuario_horarios uh
      JOIN horarios h ON uh.horario = h.id
      WHERE uh.usuario = ? AND uh.fecha_creacion = ?
      ORDER BY h.dia, h.hora_inicio;";

    let rows = sqlx::query(QUERY)
      .bind(usuario)
      .bind(fecha_creacion)
      .fetch_all(self.pool.conexion())
      .await
      .map_err(DBError::from_sqlx)?;

    Ok(rows.iter().map(config_horario_from_row).collect())
  }

  /// Verifica que una configuración no se solape con otras para el mismo día.
  pub(in crate::horario) async fn config_horario_solape(
    &self,
    config_horario: &ConfigHorario,
  ) -> Result<bool, DBError> {
    const QUERY: &str = "SELECT CAST(COUNT(*) AS UNSIGNED) 
      FROM usuario_horarios uh
      JOIN horarios h ON uh.horario = h.id
      WHERE uh.usuario = ?
      AND uh.fecha_creacion = ?
      AND uh.id <> ?
      AND h.dia = ?
      AND h.hora_inicio < ?
      AND h.hora_fin > ?
      AND (
        uh.caducidad_fecha_fin IS NULL 
        OR ? IS NULL 
        OR (uh.caducidad_fecha_ini <= ? AND uh.caducidad_fecha_fin >= ?)
      );";

    let cad_fecha_ini = config_horario.caducidad_fecha_ini.convert_to_date();
    let cad_fecha_fin = config_horario.caducidad_fecha_fin;

    let count: u32 = sqlx::query_scalar(QUERY)
      .bind(config_horario.usuario)
      .bind(config_horario.fecha_creacion)
      .bind(config_horario.id)
      .bind(config_horario.horario.dia.letra())
      .bind(config_horario.horario.hora_fin)
      .bind(config_horario.horario.hora_inicio)
      .bind(cad_fecha_fin)
      .bind(cad_fecha_fin)
      .bind(cad_fecha_ini)
      .fetch_one(self.pool.conexion())
      .await
      .map_err(DBError::from_sqlx)?;

    Ok(count > 0)
  }

  /// Busca un horario por su día y horas.
  pub(in crate::horario) async fn horario_por_dia_horas(
    &self,
    horario: &Horario,
  ) -> Result<Option<u32>, DBError> {
    const QUERY: &str = "SELECT id FROM horarios 
      WHERE dia = ? AND hora_inicio = ? AND hora_fin = ?";

    let row = sqlx::query_scalar(QUERY)
      .bind(horario.dia.letra())
      .bind(horario.hora_inicio)
      .bind(horario.hora_fin)
      .fetch_optional(self.pool.conexion())
      .await
      .map_err(DBError::from_sqlx)?;

    Ok(row)
  }

  /// Verifica si un horario está en uso por otra configuración.
  pub(in crate::horario) async fn es_horario_usado_excepto(
    &self,
    id_horario: u32,
    id_config_excluida: u32,
  ) -> Result<bool, DBError> {
    const QUERY: &str = "SELECT CAST(COUNT(*) AS UNSIGNED) 
      FROM usuario_horarios 
      WHERE horario = ? AND id <> ?;";

    let count: u32 = sqlx::query_scalar(QUERY)
      .bind(id_horario)
      .bind(id_config_excluida)
      .fetch_one(self.pool.conexion())
      .await
      .map_err(DBError::from_sqlx)?;

    Ok(count > 0)
  }

  /// Busca si el horario de un usuario se encuentra referenciado en el marcaje
  pub(in crate::horario) async fn esta_horario_en_marcaje(
    &self,
    usuario_horario: u32,
  ) -> Result<bool, DBError> {
    const QUERY: &str = "SELECT CAST(COUNT(*) AS UNSIGNED) 
      FROM marcajes 
      WHERE usuario_horario = ?;";

    let count: u32 = sqlx::query_scalar(QUERY)
      .bind(usuario_horario)
      .fetch_one(self.pool.conexion())
      .await
      .map_err(DBError::from_sqlx)?;

    Ok(count > 0)
  }

  /// Crea un nuevo horario.
  pub(in crate::horario) async fn crear_horario(
    &self,
    trans: &mut Transaccion<'_>,
    horario: &Horario,
  ) -> Result<u32, DBError> {
    const QUERY: &str = "INSERT INTO horarios (dia, hora_inicio, hora_fin)
       VALUES (?, ?, ?);";

    let res = sqlx::query(QUERY)
      .bind(horario.dia.letra())
      .bind(horario.hora_inicio)
      .bind(horario.hora_fin)
      .execute(&mut **trans.deref_mut())
      .await
      .map_err(DBError::from_sqlx)?;

    Ok(res.last_insert_id() as u32)
  }

  /// Elimina un horario.
  pub(in crate::horario) async fn eliminar_horario(
    &self,
    trans: &mut Transaccion<'_>,
    id_horario: u32,
  ) -> Result<(), DBError> {
    const QUERY: &str = "DELETE FROM horarios WHERE id = ?;";

    sqlx::query(QUERY)
      .bind(id_horario)
      .execute(&mut **trans.deref_mut())
      .await
      .map_err(DBError::from_sqlx)?;

    Ok(())
  }

  /// Obtiene la fecha de creación más reciente del horario.
  async fn fecha_creacion_horario(
    &self,
    usuario: u32,
    fecha: NaiveDate,
  ) -> Result<NaiveDate, DBError> {
    const QUERY: &str = "SELECT MAX(fecha_creacion) 
    FROM usuario_horarios 
    WHERE usuario = ? 
    AND fecha_creacion < ?";

    let fecha_creacion = sqlx::query_scalar::<_, Option<NaiveDate>>(QUERY)
      .bind(usuario)
      .bind(fecha)
      .fetch_one(self.pool.conexion())
      .await
      .map_err(DBError::from_sqlx)?
      .ok_or_else(|| {
        DBError::registro_vacio(format!(
          "No se ha encontrado ningún horario configurado \
        para el usuario en la fecha: {}",
          fecha.formato_corto()
        ))
      })?;

    Ok(fecha_creacion)
  }
}

pub(crate) fn config_horario_from_row(row: &MySqlRow) -> ConfigHorario {
  ConfigHorario {
    id: row.get("uh_id"),
    usuario: row.get("usuario"),
    horario: horario_from_row(row),
    fecha_creacion: row.get("fecha_creacion"),
    caducidad_fecha_ini: {
      // 01/01/1900 es equivalente a nulo, pero no se utiliza
      // nulo porque se encuentra en un índice
      let fecha: NaiveDate = row.get("caducidad_fecha_ini");
      if fecha == NONE_DATE {
        None
      } else {
        Some(fecha)
      }
    },
    caducidad_fecha_fin: row.get("caducidad_fecha_fin"),
  }
}

fn horario_from_row(row: &MySqlRow) -> Horario {
  Horario {
    id: row.get("id"),
    dia: Dia::from(row.get::<String, _>("dia").as_str()),
    hora_inicio: row.get("hora_inicio"),
    hora_fin: row.get("hora_fin"),
  }
}

impl HorarioRepo {
  /// Devuelve todos los calendarios ordenados por nombre.
  pub(in crate::horario) async fn calendarios(
    &self,
  ) -> Result<Vec<Calendario>, DBError> {
    const QUERY: &str =
      "SELECT id, nombre, descripcion FROM calendarios ORDER BY nombre";

    let rows = sqlx::query(QUERY)
      .fetch_all(self.pool.conexion())
      .await
      .map_err(DBError::from_sqlx)?;

    Ok(rows.iter().map(calendario_from_row).collect())
  }

  /// Devuelve un calendario por su id.
  pub(in crate::horario) async fn calendario(
    &self,
    id: u32,
  ) -> Result<Calendario, DBError> {
    const QUERY: &str =
      "SELECT id, nombre, descripcion FROM calendarios WHERE id = ?";

    let row = sqlx::query(QUERY)
      .bind(id)
      .fetch_optional(self.pool.conexion())
      .await
      .map_err(DBError::from_sqlx)?;

    row.map(|r| calendario_from_row(&r)).ok_or_else(|| {
      DBError::registro_vacio(format!("Calendario no encontrado: {}", id))
    })
  }

  /// Crea un nuevo calendario.
  pub(in crate::horario) async fn crear_calendario(
    &self,
    calendario: &Calendario,
  ) -> Result<u32, DBError> {
    const QUERY: &str =
      "INSERT INTO calendarios (nombre, descripcion) VALUES (?, ?)";

    let res = sqlx::query(QUERY)
      .bind(&calendario.nombre)
      .bind(&calendario.descripcion)
      .execute(self.pool.conexion())
      .await
      .map_err(DBError::from_sqlx)?;

    Ok(res.last_insert_id() as u32)
  }

  /// Actualiza un calendario existente.
  pub(in crate::horario) async fn actualizar_calendario(
    &self,
    calendario: &Calendario,
  ) -> Result<(), DBError> {
    const QUERY: &str =
      "UPDATE calendarios SET nombre = ?, descripcion = ? WHERE id = ?";

    let res = sqlx::query(QUERY)
      .bind(&calendario.nombre)
      .bind(&calendario.descripcion)
      .bind(calendario.id)
      .execute(self.pool.conexion())
      .await
      .map_err(DBError::from_sqlx)?;

    if res.rows_affected() == 0 {
      Err(DBError::registro_vacio(
        "Actualizando calendario".to_string(),
      ))
    } else {
      Ok(())
    }
  }

  /// Elimina un calendario.
  pub(in crate::horario) async fn eliminar_calendario(
    &self,
    id: u32,
  ) -> Result<(), DBError> {
    const QUERY: &str = "DELETE FROM calendarios WHERE id = ?";

    let res = sqlx::query(QUERY)
      .bind(id)
      .execute(self.pool.conexion())
      .await
      .map_err(DBError::from_sqlx)?;

    if res.rows_affected() == 0 {
      Err(DBError::registro_vacio("Eliminando calendario".to_string()))
    } else {
      Ok(())
    }
  }

  /// Devuelve las fechas de un calendario filtradas por rango.
  pub(in crate::horario) async fn calendario_fechas(
    &self,
    calendario_id: u32,
    fecha_inicio: Option<NaiveDate>,
    fecha_fin: Option<NaiveDate>,
  ) -> Result<Vec<CalendarioFecha>, DBError> {
    let mut qb = sqlx::QueryBuilder::<sqlx::MySql>::new(
      "SELECT id, calendario, fecha_inicio, fecha_fin, tipo
       FROM calendario_fechas WHERE calendario = ",
    );
    qb.push_bind(calendario_id);

    if let Some(inicio) = fecha_inicio {
      qb.push(" AND fecha_fin >= ");
      qb.push_bind(inicio);
    }

    if let Some(fin) = fecha_fin {
      qb.push(" AND fecha_inicio <= ");
      qb.push_bind(fin);
    }

    qb.push(" ORDER BY fecha_inicio DESC");

    let rows = qb
      .build()
      .fetch_all(self.pool.conexion())
      .await
      .map_err(DBError::from_sqlx)?;

    Ok(rows.iter().map(calendario_fecha_from_row).collect())
  }

  /// Devuelve una fecha de calendario por su id.
  pub(in crate::horario) async fn calendario_fecha(
    &self,
    id: u32,
  ) -> Result<CalendarioFecha, DBError> {
    const QUERY: &str = "SELECT id, calendario, fecha_inicio, fecha_fin, tipo
     FROM calendario_fechas WHERE id = ?";

    let row = sqlx::query(QUERY)
      .bind(id)
      .fetch_optional(self.pool.conexion())
      .await
      .map_err(DBError::from_sqlx)?;

    row.map(|r| calendario_fecha_from_row(&r)).ok_or_else(|| {
      DBError::registro_vacio(format!(
        "Fecha de calendario no encontrada: {}",
        id
      ))
    })
  }

  /// Crea una nueva fecha en el calendario.
  pub(in crate::horario) async fn crear_calendario_fecha(
    &self,
    fecha: &CalendarioFecha,
  ) -> Result<u32, DBError> {
    const QUERY: &str = "INSERT INTO calendario_fechas 
    (calendario, fecha_inicio, fecha_fin, tipo) VALUES (?, ?, ?, ?)";

    let res = sqlx::query(QUERY)
      .bind(fecha.calendario)
      .bind(fecha.fecha_inicio)
      .bind(fecha.fecha_fin)
      .bind(fecha.tipo as u8)
      .execute(self.pool.conexion())
      .await
      .map_err(DBError::from_sqlx)?;

    Ok(res.last_insert_id() as u32)
  }

  /// Actualiza una fecha del calendario.
  pub(in crate::horario) async fn actualizar_calendario_fecha(
    &self,
    fecha: &CalendarioFecha,
  ) -> Result<(), DBError> {
    const QUERY: &str = "UPDATE calendario_fechas SET calendario = ?,
     fecha_inicio = ?, fecha_fin = ?, tipo = ? WHERE id = ?";

    let res = sqlx::query(QUERY)
      .bind(fecha.calendario)
      .bind(fecha.fecha_inicio)
      .bind(fecha.fecha_fin)
      .bind(fecha.tipo as u8)
      .bind(fecha.id)
      .execute(self.pool.conexion())
      .await
      .map_err(DBError::from_sqlx)?;

    if res.rows_affected() == 0 {
      Err(DBError::registro_vacio(
        "Actualizando fecha de calendario".to_string(),
      ))
    } else {
      Ok(())
    }
  }

  /// Elimina una fecha del calendario.
  pub(in crate::horario) async fn eliminar_calendario_fecha(
    &self,
    id: u32,
  ) -> Result<(), DBError> {
    const QUERY: &str = "DELETE FROM calendario_fechas WHERE id = ?";

    let res = sqlx::query(QUERY)
      .bind(id)
      .execute(self.pool.conexion())
      .await
      .map_err(DBError::from_sqlx)?;

    if res.rows_affected() == 0 {
      Err(DBError::registro_vacio(
        "Eliminando fecha de calendario".to_string(),
      ))
    } else {
      Ok(())
    }
  }

  /// Obtiene los marcajes conflictivos de los usuarios de un calendario
  ///
  /// Devuelve un mapa con el nombre completo del usuario
  /// y las fechas de sus marcajes conflictivos.
  pub(in crate::horario) async fn marcajes_conflictivos_en_calendario_fecha(
    &self,
    calendario: u32,
    fecha_inicio: NaiveDate,
    fecha_fin: NaiveDate,
  ) -> Result<HashMap<String, Vec<NaiveDate>>, DBError> {
    const QUERY: &str = "SELECT
            CONCAT(u.nombre, ' ', u.primer_apellido) as nombre_completo,
            m.fecha
        FROM marcajes m
        JOIN calendarios_usuario cu ON m.usuario = cu.usuario
        JOIN usuarios u ON m.usuario = u.id
        WHERE cu.calendario = ?
          AND m.fecha BETWEEN ? AND ?
          AND modificado_por IS NULL AND eliminado IS NULL
        ORDER BY nombre_completo, m.fecha";

    let rows = sqlx::query(QUERY)
      .bind(calendario)
      .bind(fecha_inicio)
      .bind(fecha_fin)
      .fetch_all(self.pool.conexion())
      .await
      .map_err(DBError::from_sqlx)?;

    let mut conflictos: HashMap<String, Vec<NaiveDate>> = HashMap::new();
    for row in rows {
      let nombre_completo: String = row.get("nombre_completo");
      let fecha: NaiveDate = row.get("fecha");
      conflictos.entry(nombre_completo).or_default().push(fecha);
    }

    Ok(conflictos)
  }

  /// Verifica si la fecha de un marcaje entra en conflicto con un calendario asignado al usuario.
  ///
  /// Devuelve la entidad CalendarioFecha del conflicto si existe.
  pub(in crate::horario) async fn conflicto_calendario_en_marcaje(
    &self,
    usuario: u32,
    fecha: NaiveDate,
  ) -> Result<Option<CalendarioFecha>, DBError> {
    const QUERY: &str =
      "SELECT cf.id, cf.calendario, cf.fecha_inicio, cf.fecha_fin, cf.tipo
        FROM calendario_fechas cf
        JOIN calendarios_usuario cu ON cf.calendario = cu.calendario
        WHERE cu.usuario = ?
          AND ? BETWEEN cf.fecha_inicio AND cf.fecha_fin
        LIMIT 1";

    let row = sqlx::query(QUERY)
      .bind(usuario)
      .bind(fecha)
      .fetch_optional(self.pool.conexion())
      .await
      .map_err(DBError::from_sqlx)?;

    Ok(row.map(|r| calendario_fecha_from_row(&r)))
  }
}

fn calendario_from_row(row: &MySqlRow) -> Calendario {
  Calendario {
    id: row.get("id"),
    nombre: row.get("nombre"),
    descripcion: row.get("descripcion"),
  }
}

fn calendario_fecha_from_row(row: &MySqlRow) -> CalendarioFecha {
  CalendarioFecha {
    id: row.get("id"),
    calendario: row.get("calendario"),
    fecha_inicio: row.get("fecha_inicio"),
    fecha_fin: row.get("fecha_fin"),
    tipo: TipoCalendarioFecha::from(row.get::<u8, _>("tipo")),
  }
}
