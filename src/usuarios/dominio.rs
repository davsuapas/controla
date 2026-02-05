use std::fmt::Debug;

use chrono::NaiveDateTime;
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UsuarioCalendario {
  pub calendario: u32,
  pub nombre: String,
  pub asignado: bool,
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
  pub calendarios: Vec<UsuarioCalendario>,
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

  pub fn eq_calendarios(&self, other: &Usuario) -> bool {
    let self_cals: Vec<_> = self
      .calendarios
      .iter()
      .filter(|c| c.asignado)
      .map(|c| c.calendario)
      .collect();

    if self_cals.len()
      != other.calendarios.iter().filter(|c| c.asignado).count()
    {
      return false;
    }

    let other_cals: Vec<_> = other
      .calendarios
      .iter()
      .filter(|c| c.asignado)
      .map(|c| c.calendario)
      .collect();

    for cal in &self_cals {
      if !other_cals.contains(cal) {
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
      .field("calendarios", &self.calendarios)
      .finish()
  }
}
