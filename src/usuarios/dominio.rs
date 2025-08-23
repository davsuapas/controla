use chrono::{NaiveTime, TimeDelta};

#[derive(Debug)]
pub struct Usuario {
  pub id: u64,
  pub nombre: String,
}

impl PartialEq for Usuario {
  fn eq(&self, other: &Self) -> bool {
    self.id == other.id
  }
}

impl Eq for Usuario {}

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
  pub fn letra(&self) -> char {
    match self {
      Dia::Lunes => 'L',
      Dia::Martes => 'M',
      Dia::Miercoles => 'X',
      Dia::Jueves => 'J',
      Dia::Viernes => 'V',
      Dia::Sabado => 'S',
      Dia::Domingo => 'D',
    }
  }
}

impl From<char> for Dia {
  fn from(dia: char) -> Self {
    match dia {
      'L' => Dia::Lunes,
      'M' => Dia::Martes,
      'X' => Dia::Miercoles,
      'J' => Dia::Jueves,
      'V' => Dia::Viernes,
      'S' => Dia::Sabado,
      'D' => Dia::Domingo,
      _ => panic!("Día no válido"),
    }
  }
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct Horario {
  pub id: u64,
  pub dia: Dia,
  pub hora_inicio: NaiveTime,
  pub hora_fin: NaiveTime,
}

impl Horario {
  #[inline]
  pub fn horas_a_trabajar(&self) -> TimeDelta {
    self.hora_fin - self.hora_inicio
  }
}
