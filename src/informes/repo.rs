use chrono::NaiveDate;
use sqlx::{Arguments, Row};

use crate::{
  horario::{CalendarioFecha, repo::config_horario_from_row},
  informes::{DiasInhabiles, HorariosUsuario, HorasEfectivasMarcajes},
  infra::{DBError, PoolConexion},
};

/// Repositorio encargado de la persistencia y recuperación de datos para
/// la generación de informes.
pub struct InformeRepo {
  pool: PoolConexion,
}

impl InformeRepo {
  pub fn new(pool: PoolConexion) -> Self {
    InformeRepo { pool }
  }

  fn fin_de_mes(anio: i32, mes: u32) -> Result<NaiveDate, DBError> {
    let (nuevo_anio, nuevo_mes) = if mes == 12 {
      (anio + 1, 1)
    } else {
      (anio, mes + 1)
    };

    NaiveDate::from_ymd_opt(nuevo_anio, nuevo_mes, 1)
      .ok_or(DBError::Parametros("Fecha fin inválida"))?
      .pred_opt()
      .ok_or(DBError::Parametros("Fecha fin inválida"))
  }
}

impl InformeRepo {
  /// Recupera las horas efectivas trabajadas por un usuario en un mes y año.
  ///
  /// Agrupa los marcajes válidos (con hora de fin, no eliminados ni modificados)
  /// por día y suma la duración total en horas.
  pub(in crate::informes) async fn marcajes_mes(
    &self,
    usuario: u32,
    mes: u32,
    anio: i32,
  ) -> Result<HorasEfectivasMarcajes, DBError> {
    let fecha_inicio = NaiveDate::from_ymd_opt(anio, mes, 1)
      .ok_or(DBError::Parametros("Fecha inicio inválida"))?;
    let fecha_fin = Self::fin_de_mes(anio, mes)?;

    const QUERY: &str = "SELECT EXTRACT(DAY FROM fecha) as dia,
      CAST(
       SUM(
        TIMESTAMPDIFF(
          SECOND, hora_inicio, hora_fin)) AS DOUBLE) / 3600.0 as horas
      FROM marcajes
      WHERE usuario = ? AND fecha BETWEEN ? AND ?
      AND hora_fin IS NOT NULL 
      AND modificado_por IS NULL AND eliminado IS NULL
      GROUP BY dia";

    let rows = sqlx::query(QUERY)
      .bind(usuario)
      .bind(fecha_inicio)
      .bind(fecha_fin)
      .fetch_all(self.pool.conexion())
      .await
      .map_err(DBError::from_sqlx)?;

    let dias = rows
      .into_iter()
      .map(|row| (row.get::<i64, _>("dia") as u32, row.get("horas")))
      .collect();

    Ok(HorasEfectivasMarcajes::new(dias))
  }

  /// Obtiene los periodos inhábiles que afectan a un usuario durante un mes y año.
  ///
  /// Realiza una búsqueda de rangos solapados para todas los calendarios
  /// de un usuario:
  /// recupera cualquier evento de calendario (vacaciones, festivos, bajas)
  /// que comience antes de que termine el mes y termine
  /// después de que empiece.
  pub(in crate::informes) async fn dias_inhabiles_mes(
    &self,
    usuario: u32,
    mes: u32,
    anio: i32,
  ) -> Result<DiasInhabiles, DBError> {
    let fecha_inicio = NaiveDate::from_ymd_opt(anio, mes, 1)
      .ok_or(DBError::Parametros("Fecha inicio inválida"))?;

    let fecha_fin = Self::fin_de_mes(anio, mes)?;

    const QUERY: &str = "SELECT cf.id, cf.calendario, cf.fecha_inicio, 
      cf.fecha_fin, cf.tipo
      FROM calendario_fechas cf
      JOIN calendarios_usuario cu ON cu.calendario = cf.calendario
      WHERE cu.usuario = ?
      AND cf.fecha_inicio <= ? AND cf.fecha_fin >= ?";

    let rows = sqlx::query(QUERY)
      .bind(usuario)
      .bind(fecha_fin)
      .bind(fecha_inicio)
      .fetch_all(self.pool.conexion())
      .await
      .map_err(DBError::from_sqlx)?;

    let fechas = rows
      .into_iter()
      .map(|row| CalendarioFecha {
        id: row.get("id"),
        calendario: row.get("calendario"),
        fecha_inicio: row.get("fecha_inicio"),
        fecha_fin: row.get("fecha_fin"),
        tipo: (row.get::<u16, _>("tipo") as u8).into(),
      })
      .collect();

    Ok(DiasInhabiles::new(fechas))
  }

  /// Recupera la configuración de horarios de un usuario necesaria
  /// para calcular su jornada teórica durante un mes.
  ///
  /// Para determinar correctamente el horario de cada día, obtiene:
  ///
  /// - La configuración vigente inmediatamente anterior al inicio del
  ///   mes (snapshot inicial).
  ///
  /// - Todas las nuevas configuraciones o cambios que se hayan creado
  ///   durante el transcurso del mes.
  pub(in crate::informes) async fn horarios_usuario_mes(
    &self,
    usuario: u32,
    mes: u32,
    anio: i32,
  ) -> Result<HorariosUsuario, DBError> {
    let fecha_inicio = NaiveDate::from_ymd_opt(anio, mes, 1)
      .ok_or(DBError::Parametros("Fecha inicio inválida"))?;

    let fecha_fin = Self::fin_de_mes(anio, mes)?;

    const QUERY_PREV: &str = "SELECT MAX(fecha_creacion) FROM usuario_horarios 
      WHERE usuario = ? AND fecha_creacion < ?";

    let fecha_prev: Option<NaiveDate> = sqlx::query_scalar(QUERY_PREV)
      .bind(usuario)
      .bind(fecha_inicio)
      .fetch_one(self.pool.conexion())
      .await
      .map_err(DBError::from_sqlx)?;

    let mut query = String::from(
      "SELECT h.id, uh.id AS uh_id,
        uh.usuario, uh.fecha_creacion,
        uh.caducidad_fecha_ini, uh.caducidad_fecha_fin,
        h.dia, h.hora_inicio, h.hora_fin
      FROM usuario_horarios uh
      JOIN horarios h ON uh.horario = h.id
      WHERE uh.usuario = ? AND (",
    );

    let mut args = sqlx::mysql::MySqlArguments::default();
    args
      .add(usuario)
      .map_err(|_| DBError::Parametros("Usuario"))?;

    if let Some(prev) = fecha_prev {
      query.push_str("uh.fecha_creacion = ? OR ");
      args
        .add(prev)
        .map_err(|_| DBError::Parametros("Fecha prev"))?;
    }

    query.push_str("uh.fecha_creacion BETWEEN ? AND ?)");
    args
      .add(fecha_inicio)
      .map_err(|_| DBError::Parametros("Fecha inicio"))?;
    args
      .add(fecha_fin)
      .map_err(|_| DBError::Parametros("Fecha fin"))?;

    let rows = sqlx::query_with(&query, args)
      .fetch_all(self.pool.conexion())
      .await
      .map_err(DBError::from_sqlx)?;

    let horarios = rows.iter().map(config_horario_from_row).collect();

    Ok(HorariosUsuario::new(horarios))
  }
}
