use std::fmt::Debug;

use chrono::{NaiveDateTime, NaiveTime};
use smallvec::SmallVec;

use crate::infra::{Dni, Password};

// Si se cambia algún rol se debe actualizar en el web/src/modelos/usuarios.ts
#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Rol {
  /// Empleado con permisos para registrar y consultar marcajes.
  /// También puede enviar solicitudes de incidencias
  Empleado = 1,
  /// Permisos de gestión de incidencias. Puede aprobar o
  /// rechazar incidencias
  Gestor = 2,
  /// Permisos de administración de usuarios.
  Admin = 3,
  /// Permisos de dirección y generación de informes.
  Director = 4,
  /// Permisos para registrar marcajes en nombre del empleado.
  /// También puede realizar solicitudes de incidencias que
  /// haya previamente registrado
  Registrador = 5,
  /// Permisos para inspeccionar y auditar registros.
  Inspector = 6,
  /// Permisos que permite realizar acciones de un registrador
  /// o un gestor. Si un gestor o registrador deja la compañia,
  /// el supervisor podrá realizar sus tareas.
  Supervidor = 7,
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
      7 => Rol::Supervidor,
      _ => panic!("Valor de Rol no válido"),
    }
  }
}

#[derive(Debug)]
pub struct DescriptorUsuario {
  pub id: u32,
  pub nombre: String,
  pub primer_apellido: String,
  pub segundo_apellido: String,
}

pub struct Usuario {
  pub id: u32,
  pub dni: Dni,
  pub email: String,
  pub nombre: String,
  pub primer_apellido: String,
  pub segundo_apellido: String,
  pub password: Option<Password>,
  pub activo: Option<NaiveDateTime>,
  // Inicio es la fecha que el usuario se logea por primera vez
  pub inicio: Option<NaiveDateTime>,
  pub roles: SmallVec<[Rol; 7]>,
}

impl Usuario {
  pub fn eq_roles(&self, other: &Usuario) -> bool {
    if self.roles.len() != other.roles.len() {
      return false;
    }

    for rol in &self.roles {
      if !other.roles.contains(rol) {
        return false;
      }
    }

    true
  }

  pub fn nombre_completo(&self) -> String {
    format!(
      "{} {} {}",
      self.nombre, self.primer_apellido, self.segundo_apellido
    )
  }
}

impl Debug for Usuario {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("Usuario")
      .field("id", &self.id)
      .field("dni", &"[OCULTO]")
      .field("email", &self.email)
      .field("nombre", &self.nombre)
      .field("primer_apellido", &self.primer_apellido)
      .field("segundo_apellido", &self.segundo_apellido)
      .field(
        "password",
        if self.password.is_some() {
          &"[OCULTO]"
        } else {
          &"None"
        },
      )
      .field("activo", &self.activo)
      .field("inicio", &self.inicio)
      .field("roles", &self.roles)
      .finish()
  }
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
