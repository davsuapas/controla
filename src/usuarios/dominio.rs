use chrono::{NaiveTime, TimeDelta};

#[repr(u8)]
#[derive(Copy, Clone, Debug)]
pub enum Rol {
  Empleado = 1,
  Gestor = 2,
  Admin = 3,
  Director = 4,
  Registrador = 5,
  Inspector = 6,
}

impl From<u8> for Rol {
  fn from(value: u8) -> Self {
    match value {
      1 => Rol::Empleado,
      2 => Rol::Gestor,
      3 => Rol::Admin,
      4 => Rol::Director,
      5 => Rol::Registrador,
      6 => Rol::Inspector,
      _ => panic!("Valor de Rol no válido"),
    }
  }
}

#[derive(Debug)]
pub struct UsuarioNombre {
  pub id: u32,
  pub nombre: String,
}

impl PartialEq for UsuarioNombre {
  fn eq(&self, other: &Self) -> bool {
    self.id == other.id
  }
}

impl Eq for UsuarioNombre {}

#[derive(Debug)]
pub struct DescriptorUsuario {
  pub id: u32,
  pub nombre: String,
  pub primer_apellido: String,
  pub segundo_apellido: String,
}

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
  pub id: u32,
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
