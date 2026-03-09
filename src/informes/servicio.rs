use chrono::{Datelike, NaiveDate, Utc};
use smallvec::SmallVec;

use crate::config::ConfigTrabajo;
use crate::informes::{CumplimientoHorario, InformeCumplimiento, InformeRepo};
use crate::infra::{DBError, ServicioError};

pub struct InformeServicio {
  cnfg: ConfigTrabajo,
  repo: InformeRepo,
}

impl InformeServicio {
  pub fn new(cnfg: ConfigTrabajo, repo: InformeRepo) -> Self {
    InformeServicio { cnfg, repo }
  }
}

impl InformeServicio {
  /// Genera un informe detallado de cumplimiento horario para un usuario
  /// y mes específicos.
  ///
  /// Este informe ofrece un desglose diario que compara la jornada laboral
  /// teórica del empleado con las horas reales registradas a través
  /// de sus marcajes. El objetivo es obtener un balance preciso de las
  /// horas trabajadas y detectar posibles inconsistencias.
  ///
  /// No se emite en el informe los días donde el usuario no tiene asignado
  /// horario y tampoco tiene en cuenta los día superiores a la fecha actual.
  ///
  /// Para cada día del mes, el informe calcula:
  /// - **Horas a trabajar**: La jornada teórica que el usuario debía
  ///   cumplir según su horario asignado.
  /// - **Horas efectivas**: La suma total de horas trabajadas,
  ///   calculada a partir de los marcajes de entrada y salida.
  /// - **Saldo diario**: La diferencia entre las horas efectivas
  ///   y las horas teóricas.
  /// - **Notas**: Anotaciones para aclarar situaciones especiales, como:
  ///   - Días inhábiles (vacaciones, festivos, bajas).
  ///   - Inconsistencias, como marcajes realizados en un día inhábil
  ///     o en un día sin un horario laboral definido.
  ///
  /// Finalmente, el informe consolida un **saldo total mensual**,
  /// que representa el cómputo global de horas extra o deficitarias
  /// del empleado durante el mes.
  pub async fn cumplimiento_horario(
    &self,
    usuario: u32,
    mes: u32,
    anio: i32,
  ) -> Result<InformeCumplimiento, ServicioError> {
    tracing::info!(
      usuario = usuario,
      mes = mes,
      anio = anio,
      "Generando el informe de cumplimiento de horario"
    );

    let dias_inhabiles = self
      .repo
      .dias_inhabiles_mes(usuario, mes, anio)
      .await
      .map_err(|err| {
        tracing::error!(
          usuario = usuario,
          mes = mes,
          anio = anio,
          error = %err,
          "Obteniendo días inhábiles para informe cumplimiento"
        );
        ServicioError::from(err)
      })?;

    let horas_efectivas_marcajes = self
      .repo
      .marcajes_mes(usuario, mes, anio)
      .await
      .map_err(|err| {
        tracing::error!(
          usuario = usuario,
          mes = mes,
          anio = anio,
          error = %err,
          "Obteniendo horas efectivas para informe cumplimiento"
        );
        ServicioError::from(err)
      })?;

    let horarios_usuario = self
      .repo
      .horarios_usuario_mes(usuario, mes, anio)
      .await
      .map_err(|err| {
        tracing::error!(
          usuario = usuario,
          mes = mes,
          anio = anio,
          error = %err,
          "Obteniendo horarios usuario para informe cumplimiento"
        );
        ServicioError::from(err)
      })?;

    let fecha_inicio = NaiveDate::from_ymd_opt(anio, mes, 1)
      .ok_or(DBError::Parametros("Fecha inicio inválida"))?;

    let (nuevo_anio, nuevo_mes) = if mes == 12 {
      (anio + 1, 1)
    } else {
      (anio, mes + 1)
    };

    let fecha_fin = NaiveDate::from_ymd_opt(nuevo_anio, nuevo_mes, 1)
      .ok_or(DBError::Parametros("Fecha fin inválida"))?
      .pred_opt()
      .ok_or(DBError::Parametros("Fecha fin inválida"))?;

    let mut lineas = SmallVec::<[CumplimientoHorario; 31]>::new();
    let mut total_saldo = 0.0;
    let mut curr = fecha_inicio;

    let fecha_actual = Utc::now()
      .with_timezone(&self.cnfg.zona_horaria)
      .naive_local()
      .date();

    while curr <= fecha_fin {
      if curr > fecha_actual {
        break;
      }

      let horas_a_trabajar = horarios_usuario.horas_a_trabajar(curr);

      if horas_a_trabajar == 0.0 {
        curr = curr.succ_opt().unwrap();
        continue;
      }

      let horas_efectivas = horas_efectivas_marcajes
        .horas_efectivas(curr.day())
        .unwrap_or(0.0);

      let linea = if let Some(inhabil) = dias_inhabiles.buscar(curr) {
        if horas_efectivas > 0.0 {
          let saldo = horas_efectivas - horas_a_trabajar;
          total_saldo += saldo;
          CumplimientoHorario {
            fecha: curr,
            horas_trabajo_efectivo: horas_efectivas,
            horas_trabajadas: horas_efectivas,
            horas_a_trabajar,
            saldo,
            nota: "No puede haber días inhábiles con marcajes".to_string(),
          }
        } else {
          CumplimientoHorario::with_fecha_y_nota(
            curr,
            format!("Día inhábil. Motivo: {:?}", inhabil.tipo),
          )
        }
      } else {
        let saldo = horas_efectivas - horas_a_trabajar;
        total_saldo += saldo;
        CumplimientoHorario {
          fecha: curr,
          horas_trabajo_efectivo: horas_efectivas,
          horas_trabajadas: horas_efectivas,
          horas_a_trabajar,
          saldo,
          nota: String::new(),
        }
      };

      lineas.push(linea);
      curr = curr.succ_opt().unwrap();
    }

    Ok(InformeCumplimiento {
      lineas,
      total_saldo,
    })
  }
}
