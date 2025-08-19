#![allow(clippy::from_over_into)]

use chrono::{NaiveDate, NaiveTime};
use serde::Deserialize;

use crate::{registro::Registro, usuarios::Usuario};

/// Define la entidad de intercambio para el registro
#[derive(Deserialize)]
pub(in crate::app) struct RegistroDTO {
  pub usuario: Option<UsuarioDTO>,
  pub fecha: NaiveDate,
  pub hora_inicio: NaiveTime,
  pub hora_fin: Option<NaiveTime>,
}

impl RegistroDTO {
  pub(in crate::app) fn into_dominio(self, usuario: &UsuarioDTO) -> Registro {
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
pub(in crate::app) struct UsuarioDTO {
  pub id: u64,
  pub nombre: String,
}

impl Into<Usuario> for &UsuarioDTO {
  fn into(self) -> Usuario {
    Usuario {
      id: self.id,
      nombre: self.nombre.clone(),
    }
  }
}

impl Into<Usuario> for UsuarioDTO {
  fn into(self) -> Usuario {
    Usuario {
      id: self.id,
      nombre: self.nombre,
    }
  }
}
