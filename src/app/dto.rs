#![allow(clippy::from_over_into)]

use chrono::{NaiveDate, NaiveTime};
use serde::{Deserialize, Serialize};

use crate::{
  registro::Registro,
  usuarios::{DescriptorUsuario, Horario, UsuarioNombre},
};

/// Define la entidad de intercambio para el horario
#[derive(Serialize)]
pub struct HorarioDTO {
  pub dia: char,
  pub hora_inicio: NaiveTime,
  pub hora_fin: NaiveTime,
}

impl From<Horario> for HorarioDTO {
  fn from(horario: Horario) -> Self {
    HorarioDTO {
      dia: horario.dia.letra(),
      hora_inicio: horario.hora_inicio,
      hora_fin: horario.hora_fin,
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

/// Define la entidad de intercambio para el registro
#[derive(Deserialize)]
pub(in crate::app) struct RegistroDTO {
  pub usuario: Option<UsuarioNombreDTO>,
  pub fecha: NaiveDate,
  pub hora_inicio: NaiveTime,
  pub hora_fin: Option<NaiveTime>,
}

impl RegistroDTO {
  pub(in crate::app) fn into_dominio(
    self,
    usuario: &UsuarioNombreDTO,
  ) -> Registro {
    Registro {
      usuario: self.usuario.map_or_else(|| usuario.into(), |u| u.into()),
      fecha: self.fecha,
      hora_inicio: self.hora_inicio,
      hora_fin: self.hora_fin,
    }
  }
}

/// Define la entidad de intercambio para el usuario
#[derive(Deserialize)]
pub(in crate::app) struct UsuarioNombreDTO {
  pub id: u64,
  pub nombre: String,
}

impl Into<UsuarioNombre> for &UsuarioNombreDTO {
  fn into(self) -> UsuarioNombre {
    UsuarioNombre {
      id: self.id,
      nombre: self.nombre.clone(),
    }
  }
}

impl Into<UsuarioNombre> for UsuarioNombreDTO {
  fn into(self) -> UsuarioNombre {
    UsuarioNombre {
      id: self.id,
      nombre: self.nombre,
    }
  }
}

#[derive(Serialize)]
pub(in crate::app) struct DescriptorUsuarioDTO {
  pub id: u64,
  pub nombre: String,
  pub primer_apellido: String,
  pub segundo_apellido: String,
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
