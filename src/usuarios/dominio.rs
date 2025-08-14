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
  pub fn desde_str(dia: &str) -> Option<Self> {
    match dia {
      "L" => Some(Dia::Lunes),
      "M" => Some(Dia::Martes),
      "X" => Some(Dia::Miercoles),
      "J" => Some(Dia::Jueves),
      "V" => Some(Dia::Viernes),
      "S" => Some(Dia::Sabado),
      "D" => Some(Dia::Domingo),
      _ => None,
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
