use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use serde::{Deserialize, Serialize};

use crate::{
  infra::{Dni, Password, ShortDateTimeFormat},
  registro::Registro,
  usuarios::{DescriptorUsuario, Horario, Rol, Usuario},
};

/// Define la entidad de intercambio para el usuario
#[derive(Serialize, Deserialize)]
pub(in crate::app) struct UsuarioDTO {
  pub id: u32,
  pub autor: u32, // Es el usuario que lo manipula y sirve para trazas
  pub dni: String,
  pub email: String,
  pub nombre: String,
  pub primer_apellido: String,
  pub segundo_apellido: String,
  pub password: Option<String>,
  pub activo: Option<NaiveDateTime>,
  pub inicio: Option<NaiveDateTime>,
  pub roles: Vec<u8>,
}

impl From<Usuario> for UsuarioDTO {
  fn from(usr: Usuario) -> Self {
    UsuarioDTO {
      id: usr.id,
      autor: 0, // El autor solo tiene efecto en las trazas
      dni: usr.dni.into(),
      email: usr.email,
      nombre: usr.nombre,
      primer_apellido: usr.primer_apellido,
      segundo_apellido: usr.segundo_apellido,
      password: None, // Nunca se envía la contraseña
      activo: usr.activo,
      inicio: usr.inicio,
      roles: usr.roles.iter().map(|r| *r as u8).collect(),
    }
  }
}

impl From<UsuarioDTO> for Usuario {
  fn from(usr: UsuarioDTO) -> Self {
    Usuario {
      id: usr.id,
      dni: Dni::new(usr.dni),
      email: usr.email,
      nombre: usr.nombre,
      primer_apellido: usr.primer_apellido,
      segundo_apellido: usr.segundo_apellido,
      password: usr.password.map(Password::new),
      activo: usr.activo,
      inicio: usr.inicio,
      roles: usr.roles.into_iter().map(Rol::from).collect(),
    }
  }
}

#[derive(Serialize, Deserialize)]
pub(in crate::app) struct DescriptorUsuarioDTO {
  pub id: u32,
  pub nombre: String,
  pub primer_apellido: String,
  pub segundo_apellido: String,
}

impl From<DescriptorUsuarioDTO> for DescriptorUsuario {
  fn from(usr: DescriptorUsuarioDTO) -> Self {
    DescriptorUsuario {
      id: usr.id,
      nombre: usr.nombre,
      primer_apellido: usr.primer_apellido,
      segundo_apellido: usr.segundo_apellido,
    }
  }
}

impl From<DescriptorUsuario> for DescriptorUsuarioDTO {
  fn from(usr: DescriptorUsuario) -> Self {
    DescriptorUsuarioDTO {
      id: usr.id,
      nombre: usr.nombre,
      primer_apellido: usr.primer_apellido,
      segundo_apellido: usr.segundo_apellido,
    }
  }
}

/// Define la entidad de intercambio para el cambio de contraseña
#[derive(Deserialize)]
pub struct PasswordDniDTO {
  pub dni: String,
  pub password: String,
}

/// Define la entidad de intercambio para el cambio de contraseña
#[derive(Deserialize)]
pub struct PasswordUsuarioDTO {
  pub id: u32,
  pub password: String,
}

/// Define la entidad de intercambio para el horario
#[derive(Serialize)]
pub(in crate::app) struct HorarioOutDTO {
  pub dia: &'static str,
  pub hora_inicio: String,
  pub hora_fin: String,
  pub horas_a_trabajar: f64,
}

impl From<Horario> for HorarioOutDTO {
  fn from(horario: Horario) -> Self {
    HorarioOutDTO {
      dia: horario.dia.letra(),
      hora_inicio: horario.hora_inicio.formato_corto(),
      hora_fin: horario.hora_fin.formato_corto(),
      horas_a_trabajar: horario.horas_a_trabajar(),
    }
  }
}

/// Define la entidad de intercambio para el registro
#[derive(Deserialize)]
pub(in crate::app) struct RegistroInDTO {
  pub usuario: u32,
  pub usuario_reg: Option<DescriptorUsuarioDTO>,
  pub fecha: NaiveDate,
  pub hora_inicio: NaiveTime,
  pub hora_fin: Option<NaiveTime>,
}

impl From<RegistroInDTO> for Registro {
  fn from(reg: RegistroInDTO) -> Self {
    Registro {
      usuario: reg.usuario,
      usuario_reg: reg.usuario_reg.map(Into::into),
      fecha: reg.fecha,
      hora_inicio: reg.hora_inicio,
      hora_fin: reg.hora_fin,
      horario: None,
    }
  }
}

/// Define la entidad de intercambio para el registro
#[derive(Serialize)]
pub(in crate::app) struct RegistroOutDTO {
  pub usuario_reg: Option<DescriptorUsuarioDTO>,
  pub horario: HorarioOutDTO,
  pub fecha: NaiveDate,
  pub hora_inicio: String,
  pub hora_fin: Option<String>,
  pub hora_trabajadas: Option<f64>,
}

impl From<Registro> for RegistroOutDTO {
  fn from(reg: Registro) -> Self {
    let horas_trabajadas = reg.horas_trabajadas();

    RegistroOutDTO {
      usuario_reg: reg.usuario_reg.map(Into::into),
      horario: reg.horario.expect("Registro debe tener horario").into(),
      fecha: reg.fecha,
      hora_inicio: reg.hora_inicio.formato_corto(),
      hora_fin: reg.hora_fin.map(|hf| hf.formato_corto()),
      hora_trabajadas: horas_trabajadas,
    }
  }
}

/// Convierte un vector de entidades de dominio a un vector de DTOs.
pub(in crate::app) fn vec_dominio_to_dtos<T, U>(entidad: Vec<T>) -> Vec<U>
where
  U: From<T>,
{
  entidad.into_iter().map(U::from).collect()
}
