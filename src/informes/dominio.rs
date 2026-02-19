use std::collections::HashMap;

use crate::horario::{CalendarioFecha, ConfigHorario};
use chrono::{Datelike, NaiveDate};
use smallvec::SmallVec;

/// Representa una línea del informe de cumplimiento horario.
#[derive(Debug)]
pub struct CumplimientoHorario {
  pub fecha: NaiveDate,
  pub horas_trabajo_efectivo: f64,
  pub horas_trabajadas: f64,
  pub horas_a_trabajar: f64,
  pub saldo: f64,
  pub nota: String,
}

impl CumplimientoHorario {
  pub fn with_fecha_y_nota(fecha: NaiveDate, nota: String) -> Self {
    CumplimientoHorario {
      fecha,
      horas_trabajo_efectivo: 0.0,
      horas_trabajadas: 0.0,
      horas_a_trabajar: 0.0,
      saldo: 0.0,
      nota,
    }
  }
}

/// Entidad que representa el informe de cumplimiento horario de un usuario
/// para un mes concreto.
#[derive(Debug)]
pub struct InformeCumplimiento {
  pub lineas: SmallVec<[CumplimientoHorario; 31]>,
  pub total_saldo: f64,
}

/// Entidad que almacena el resumen de horas efectivas trabajadas por día.
///
/// Representa la suma total de horas que un usuario ha fichado realmente
/// (diferencia entre entrada y salida) agrupadas por el día del mes.
#[derive(Debug)]
pub struct HorasEfectivasMarcajes {
  dias: HashMap<u32, f64>,
}

impl HorasEfectivasMarcajes {
  pub fn new(dias: HashMap<u32, f64>) -> Self {
    HorasEfectivasMarcajes { dias }
  }

  /// Busca las horas trabajadas para un día específico del mes.
  ///
  /// Devuelve `None` si no existen registros para ese día.
  pub fn horas_efectivas(&self, dia: u32) -> Option<f64> {
    self.dias.get(&dia).copied()
  }
}

/// Entidad que representa los días o rangos de fechas en los que el usuario
/// no debe trabajar (vacaciones, bajas, festivos, etc.).
#[derive(Debug)]
pub struct DiasInhabiles {
  fechas: Vec<CalendarioFecha>,
}

impl DiasInhabiles {
  pub fn new(fechas: Vec<CalendarioFecha>) -> Self {
    DiasInhabiles { fechas }
  }

  /// Verifica si una fecha específica coincide con algún periodo inhábil.
  ///
  /// Busca en la lista de eventos de calendario si la fecha dada cae dentro
  /// del rango [fecha_inicio, fecha_fin] de algún evento.
  pub fn buscar(&self, fecha: NaiveDate) -> Option<&CalendarioFecha> {
    self
      .fechas
      .iter()
      .find(|f| fecha >= f.fecha_inicio && fecha <= f.fecha_fin)
  }
}

/// Entidad que contiene el historial de configuraciones horarias
/// aplicables a un usuario.
///
/// Permite reconstruir qué horario debía cumplir el usuario en cualquier fecha
/// contenida en el informe.
#[derive(Debug)]
pub struct HorariosUsuario {
  pub horarios: Vec<ConfigHorario>,
}

impl HorariosUsuario {
  pub fn new(horarios: Vec<ConfigHorario>) -> Self {
    HorariosUsuario { horarios }
  }

  /// Calcula las horas teóricas que el usuario debería trabajar para una fecha.
  ///
  /// Este cálculo identifica primero el bloque de configuración horaria vigente
  /// (aquel cuya fecha de creación es la más reciente anterior a la
  /// fecha consultada).
  /// Posteriormente, filtra por el día de la semana y verifica que el horario
  /// específico no haya caducado para esa fecha.
  pub fn horas_a_trabajar(&self, fecha: NaiveDate) -> f64 {
    let fecha_creacion = self
      .horarios
      .iter()
      .filter(|h| h.fecha_creacion < fecha)
      .map(|h| h.fecha_creacion)
      .max();

    if let Some(fecha_creacion) = fecha_creacion {
      let dia_letra = crate::horario::Dia::from(fecha.weekday()).letra();

      self
        .horarios
        .iter()
        .filter(|h| h.fecha_creacion == fecha_creacion)
        .filter(|h| h.horario.dia.letra() == dia_letra)
        .filter(|h| {
          let inicio_ok = h.caducidad_fecha_ini.is_none_or(|ini| fecha >= ini);
          let fin_ok = h.caducidad_fecha_fin.is_none_or(|fin| fecha <= fin);
          inicio_ok && fin_ok
        })
        .map(|h| h.horario.horas_a_trabajar())
        .sum()
    } else {
      0.0
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::horario::{Dia, Horario, TipoCalendarioFecha};
  use chrono::NaiveTime;

  #[test]
  fn test_dias_inhabiles_buscar() {
    let fecha_inicio = NaiveDate::from_ymd_opt(2023, 1, 1).unwrap();
    let fecha_fin = NaiveDate::from_ymd_opt(2023, 1, 5).unwrap();
    let cf = CalendarioFecha {
      id: 1,
      calendario: 1,
      fecha_inicio,
      fecha_fin,
      tipo: TipoCalendarioFecha::Vacaciones,
    };
    let dias_inhabiles = DiasInhabiles::new(vec![cf]);

    assert!(
      dias_inhabiles
        .buscar(NaiveDate::from_ymd_opt(2023, 1, 3).unwrap())
        .is_some()
    );
    assert!(dias_inhabiles.buscar(fecha_inicio).is_some());
    assert!(dias_inhabiles.buscar(fecha_fin).is_some());
    assert!(
      dias_inhabiles
        .buscar(NaiveDate::from_ymd_opt(2023, 1, 6).unwrap())
        .is_none()
    );
  }

  #[test]
  fn test_horarios_usuario_calculo_horas() {
    struct TestCase {
      descripcion: &'static str,
      configs: Vec<ConfigHorario>,
      fecha_consulta: NaiveDate,
      horas_esperadas: f64,
    }

    let h_base = |dia: Dia, h_ini: u32, h_fin: u32| Horario {
      id: 1,
      dia,
      hora_inicio: NaiveTime::from_hms_opt(h_ini, 0, 0).unwrap(),
      hora_fin: NaiveTime::from_hms_opt(h_fin, 0, 0).unwrap(),
    };

    let config = |horario: Horario,
                  creacion: NaiveDate,
                  ini: Option<NaiveDate>,
                  fin: Option<NaiveDate>| ConfigHorario {
      id: 1,
      usuario: 1,
      horario,
      fecha_creacion: creacion,
      caducidad_fecha_ini: ini,
      caducidad_fecha_fin: fin,
    };

    let fecha_creacion = NaiveDate::from_ymd_opt(2023, 1, 1).unwrap();

    let casos = vec![
      TestCase {
        descripcion: "Prueba: Consulta de un día con horario válido. Se espera: 8 horas.",
        configs: vec![config(
          h_base(Dia::Lunes, 8, 16),
          fecha_creacion,
          None,
          None,
        )],
        fecha_consulta: NaiveDate::from_ymd_opt(2023, 1, 2).unwrap(),
        horas_esperadas: 8.0,
      },
      TestCase {
        descripcion: "Prueba: Consulta de un día sin horario asignado. Se espera: 0 horas.",
        configs: vec![config(
          h_base(Dia::Lunes, 8, 16),
          fecha_creacion,
          None,
          None,
        )],
        fecha_consulta: NaiveDate::from_ymd_opt(2023, 1, 3).unwrap(),
        horas_esperadas: 0.0,
      },
      TestCase {
        descripcion: "Prueba: Consulta en fecha anterior a la creación del horario. Se espera: 0 horas.",
        configs: vec![config(
          h_base(Dia::Domingo, 8, 16),
          fecha_creacion,
          None,
          None,
        )],
        fecha_consulta: fecha_creacion,
        horas_esperadas: 0.0,
      },
      TestCase {
        descripcion: "Prueba: Consulta dentro del periodo de vigencia (caducidad). Se espera: 8 horas.",
        configs: vec![config(
          h_base(Dia::Lunes, 8, 16),
          fecha_creacion,
          Some(NaiveDate::from_ymd_opt(2023, 2, 1).unwrap()),
          Some(NaiveDate::from_ymd_opt(2023, 2, 28).unwrap()),
        )],
        fecha_consulta: NaiveDate::from_ymd_opt(2023, 2, 6).unwrap(),
        horas_esperadas: 8.0,
      },
      TestCase {
        descripcion: "Prueba: Consulta antes del periodo de vigencia. Se espera: 0 horas.",
        configs: vec![config(
          h_base(Dia::Lunes, 8, 16),
          fecha_creacion,
          Some(NaiveDate::from_ymd_opt(2023, 2, 1).unwrap()),
          Some(NaiveDate::from_ymd_opt(2023, 2, 28).unwrap()),
        )],
        fecha_consulta: NaiveDate::from_ymd_opt(2023, 1, 30).unwrap(),
        horas_esperadas: 0.0,
      },
      TestCase {
        descripcion: "Prueba: Consulta después del periodo de vigencia. Se espera: 0 horas.",
        configs: vec![config(
          h_base(Dia::Lunes, 8, 16),
          fecha_creacion,
          Some(NaiveDate::from_ymd_opt(2023, 2, 1).unwrap()),
          Some(NaiveDate::from_ymd_opt(2023, 2, 28).unwrap()),
        )],
        fecha_consulta: NaiveDate::from_ymd_opt(2023, 3, 6).unwrap(),
        horas_esperadas: 0.0,
      },
      TestCase {
        descripcion: "Prueba: Consulta en el límite inferior de vigencia. Se espera: 8 horas.",
        configs: vec![config(
          h_base(Dia::Miercoles, 8, 16),
          fecha_creacion,
          Some(NaiveDate::from_ymd_opt(2023, 2, 1).unwrap()),
          Some(NaiveDate::from_ymd_opt(2023, 2, 28).unwrap()),
        )],
        fecha_consulta: NaiveDate::from_ymd_opt(2023, 2, 1).unwrap(),
        horas_esperadas: 8.0,
      },
      TestCase {
        descripcion: "Prueba: Consulta en el límite superior de vigencia. Se espera: 8 horas.",
        configs: vec![config(
          h_base(Dia::Martes, 8, 16),
          fecha_creacion,
          Some(NaiveDate::from_ymd_opt(2023, 2, 1).unwrap()),
          Some(NaiveDate::from_ymd_opt(2023, 2, 28).unwrap()),
        )],
        fecha_consulta: NaiveDate::from_ymd_opt(2023, 2, 28).unwrap(),
        horas_esperadas: 8.0,
      },
      TestCase {
        descripcion: "Prueba: Cambio de configuración de horario. Se espera: Horas del nuevo horario (4).",
        configs: vec![
          config(h_base(Dia::Lunes, 8, 16), fecha_creacion, None, None),
          config(
            h_base(Dia::Lunes, 9, 13),
            NaiveDate::from_ymd_opt(2023, 2, 1).unwrap(),
            None,
            None,
          ),
        ],
        fecha_consulta: NaiveDate::from_ymd_opt(2023, 2, 6).unwrap(),
        horas_esperadas: 4.0,
      },
      TestCase {
        descripcion: "Prueba: Múltiples horarios para el mismo día. Se espera: Suma de horas (8).",
        configs: vec![
          config(h_base(Dia::Lunes, 8, 12), fecha_creacion, None, None),
          config(h_base(Dia::Lunes, 14, 18), fecha_creacion, None, None),
        ],
        fecha_consulta: NaiveDate::from_ymd_opt(2023, 1, 2).unwrap(),
        horas_esperadas: 8.0,
      },
    ];

    for caso in casos {
      let horarios_usuario = HorariosUsuario::new(caso.configs);
      assert_eq!(
        horarios_usuario.horas_a_trabajar(caso.fecha_consulta),
        caso.horas_esperadas,
        "Fallo en: {}",
        caso.descripcion
      );
    }
  }
}
