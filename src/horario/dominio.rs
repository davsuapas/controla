use chrono::{NaiveDate, NaiveTime};

#[derive(Debug)]
pub enum Dia {
  Lunes,
  Martes,
  Miercoles,
  Jueves,
  Viernes,
  Sabado,
  Domingo,
}

impl Dia {
  pub fn letra(&self) -> &'static str {
    match self {
      Dia::Lunes => "L",
      Dia::Martes => "M",
      Dia::Miercoles => "X",
      Dia::Jueves => "J",
      Dia::Viernes => "V",
      Dia::Sabado => "S",
      Dia::Domingo => "D",
    }
  }
}

impl From<&str> for Dia {
  fn from(dia: &str) -> Self {
    match dia {
      "L" => Dia::Lunes,
      "M" => Dia::Martes,
      "X" => Dia::Miercoles,
      "J" => Dia::Jueves,
      "V" => Dia::Viernes,
      "S" => Dia::Sabado,
      "D" => Dia::Domingo,
      _ => panic!("Día no válido"),
    }
  }
}

#[derive(Debug)]
pub struct Horario {
  pub id: u32,
  pub dia: Dia,
  pub hora_inicio: NaiveTime,
  pub hora_fin: NaiveTime,
}

impl Horario {
  #[inline]
  pub fn horas_a_trabajar(&self) -> f64 {
    let diferencia = self.hora_fin - self.hora_inicio;
    diferencia.num_milliseconds() as f64 / 3_600_000.0
  }
}

#[derive(Debug)]
pub struct ConfigHorario {
  pub id: u32,
  pub usuario: u32,
  pub horario: Horario,
  pub fecha_creacion: NaiveDate,
  pub caducidad_fecha_ini: Option<NaiveDate>,
  pub caducidad_fecha_fin: Option<NaiveDate>,
}

#[derive(Debug)]
pub struct Calendario {
  pub id: u32,
  pub nombre: String,
  pub descripcion: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum TipoCalendarioFecha {
  Baja = 0,
  Vacaciones = 1,
  DiasPropios = 2,
  Permiso = 3,
  Festivo = 4,
  Cierre = 5,
  Otros = 6,
}

impl From<u8> for TipoCalendarioFecha {
  fn from(val: u8) -> Self {
    match val {
      0 => TipoCalendarioFecha::Baja,
      1 => TipoCalendarioFecha::Vacaciones,
      2 => TipoCalendarioFecha::DiasPropios,
      3 => TipoCalendarioFecha::Permiso,
      4 => TipoCalendarioFecha::Festivo,
      5 => TipoCalendarioFecha::Cierre,
      6 => TipoCalendarioFecha::Otros,
      _ => TipoCalendarioFecha::Otros,
    }
  }
}

impl TipoCalendarioFecha {
  pub fn as_str(&self) -> &'static str {
    match self {
      TipoCalendarioFecha::Baja => "Baja",
      TipoCalendarioFecha::Vacaciones => "Vacaciones",
      TipoCalendarioFecha::DiasPropios => "Días Propios",
      TipoCalendarioFecha::Permiso => "Permiso",
      TipoCalendarioFecha::Festivo => "Festivo",
      TipoCalendarioFecha::Cierre => "Cierre de empresa",
      TipoCalendarioFecha::Otros => "Otros",
    }
  }
}

impl From<TipoCalendarioFecha> for u8 {
  fn from(val: TipoCalendarioFecha) -> Self {
    val as u8
  }
}

#[derive(Debug)]
pub struct CalendarioFecha {
  pub id: u32,
  pub calendario: u32,
  pub fecha_inicio: NaiveDate,
  pub fecha_fin: NaiveDate,
  pub tipo: TipoCalendarioFecha,
}
