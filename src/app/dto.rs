use std::collections::HashMap;

use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use serde::{Deserialize, Serialize};

use crate::{
  inc::{EstadoIncidencia, Incidencia, IncidenciaProceso, TipoIncidencia},
  infra::{Dni, DominiosWithCacheUsuario, Password, ShortDateTimeFormat},
  marcaje::{DescriptorMarcaje, Marcaje},
  usuarios::{DescriptorUsuario, Horario, Rol, Usuario},
};

#[derive(Deserialize)]
pub struct IncidenciasFiltroParams {
  pub fecha_inicio: Option<NaiveDate>,
  pub fecha_fin: Option<NaiveDate>,
  pub estados: Vec<u8>,
  pub usuario: Option<u32>,
}

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

/// Define la entidad de intercambio de salida para el horario
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

/// Define la entidad de intercambio con el mínimo de info del marcaje
#[derive(Serialize, Deserialize)]
pub struct DescriptorMarcajeDTO {
  pub id: u32,
  pub hora_inicio: Option<NaiveTime>,
  pub hora_fin: Option<NaiveTime>,
}

impl From<DescriptorMarcajeDTO> for DescriptorMarcaje {
  fn from(marcaje: DescriptorMarcajeDTO) -> Self {
    DescriptorMarcaje {
      id: marcaje.id,
      hora_inicio: marcaje.hora_inicio,
      hora_fin: marcaje.hora_fin,
    }
  }
}

impl From<DescriptorMarcaje> for DescriptorMarcajeDTO {
  fn from(marcaje: DescriptorMarcaje) -> Self {
    DescriptorMarcajeDTO {
      id: marcaje.id,
      hora_inicio: marcaje.hora_inicio,
      hora_fin: marcaje.hora_fin,
    }
  }
}

/// Define la entidad de intercambio de entrada para el marcaje
#[derive(Deserialize)]
pub(in crate::app) struct MarcajeInDTO {
  pub usuario: u32,
  pub usuario_reg: Option<u32>,
  pub fecha: NaiveDate,
  pub hora_inicio: NaiveTime,
  pub hora_fin: Option<NaiveTime>,
}

impl From<MarcajeInDTO> for Marcaje {
  fn from(reg: MarcajeInDTO) -> Self {
    Marcaje {
      id: 0, // Es auto incremental
      usuario: reg.usuario,
      usuario_reg: reg.usuario_reg,
      fecha: reg.fecha,
      hora_inicio: reg.hora_inicio,
      hora_fin: reg.hora_fin,
      horario: None,
    }
  }
}

/// Define la entidad de intercambio de salida para el marcaje
#[derive(Serialize)]
pub(in crate::app) struct MarcajeOutDTO {
  pub id: u32,
  pub usuario: u32,
  pub usuario_reg: Option<u32>,
  pub horario: HorarioOutDTO,
  pub fecha: NaiveDate,
  pub hora_inicio: String,
  pub hora_fin: Option<String>,
  pub hora_trabajadas: Option<f64>,
}

impl From<Marcaje> for MarcajeOutDTO {
  fn from(reg: Marcaje) -> Self {
    let horas_trabajadas = reg.horas_trabajadas();

    MarcajeOutDTO {
      id: reg.id,
      usuario: reg.usuario,
      usuario_reg: reg.usuario_reg,
      horario: reg.horario.expect("Marcaje debe tener horario").into(),
      fecha: reg.fecha,
      hora_inicio: reg.hora_inicio.formato_corto(),
      hora_fin: reg.hora_fin.map(|hf| hf.formato_corto()),
      hora_trabajadas: horas_trabajadas,
    }
  }
}

// Define la entidad de intercambio para el proceso de incidencias.
#[derive(Deserialize)]
pub(in crate::app) struct IncidenciaInProcesoDTO {
  pub usuario_gestor: u32,
  pub param_filtro_inc: IncidenciasFiltroParams,
  pub incidencias: Vec<IncidenciaProcesoDTO>,
}

impl From<IncidenciaProcesoDTO> for IncidenciaProceso {
  fn from(inc: IncidenciaProcesoDTO) -> Self {
    IncidenciaProceso {
      id: inc.id,
      estado: EstadoIncidencia::from(inc.estado),
      motivo_rechazo: inc.motivo_rechazo,
    }
  }
}

// Define la entidad de retorno para el proceso de incidencias.
#[derive(Serialize)]
pub(in crate::app) struct IncidenciaOutProcesoDTO {
  pub incidencias_erroneas: Vec<u32>,
  pub incidencias: DominiosWithCacheUsuarioDTO<IncidenciaDTO>,
}

// Define la entidad de intercambio para el proceso de incidencias.
#[derive(Deserialize)]
pub(in crate::app) struct IncidenciaProcesoDTO {
  pub id: u32,
  pub estado: u8,
  pub motivo_rechazo: Option<String>,
}

// Define la entidad de intercambio para las incidencias.
#[derive(Serialize, Deserialize)]
pub(in crate::app) struct IncidenciaDTO {
  pub id: u32,
  pub tipo: u8,
  pub usuario: u32,
  pub fecha_solicitud: NaiveDateTime,
  pub fecha_resolucion: Option<NaiveDateTime>,
  pub fecha: NaiveDate,
  pub hora_inicio: Option<NaiveTime>,
  pub hora_fin: Option<NaiveTime>,
  pub marcaje: Option<DescriptorMarcajeDTO>,
  pub estado: u8,
  pub fecha_estado: Option<NaiveDateTime>,
  pub error: Option<String>,
  pub usuario_creador: u32,
  pub usuario_gestor: Option<u32>,
  pub motivo_solicitud: Option<String>,
  pub motivo_rechazo: Option<String>,
}

impl From<IncidenciaDTO> for Incidencia {
  fn from(inc: IncidenciaDTO) -> Self {
    Incidencia {
      id: inc.id,
      tipo: TipoIncidencia::from(inc.tipo),
      usuario: inc.usuario,
      fecha_solicitud: inc.fecha_solicitud,
      fecha_resolucion: inc.fecha_resolucion,
      fecha: inc.fecha,
      hora_inicio: inc.hora_inicio,
      hora_fin: inc.hora_fin,
      marcaje: inc.marcaje.map(DescriptorMarcaje::from),
      estado: EstadoIncidencia::from(inc.estado),
      fecha_estado: inc.fecha_estado,
      error: inc.error,
      usuario_creador: inc.usuario_creador,
      usuario_gestor: inc.usuario_gestor,
      motivo_solicitud: inc.motivo_solicitud,
      motivo_rechazo: inc.motivo_rechazo,
    }
  }
}

impl From<Incidencia> for IncidenciaDTO {
  fn from(inc: Incidencia) -> Self {
    IncidenciaDTO {
      id: inc.id,
      tipo: inc.tipo as u8,
      fecha_solicitud: inc.fecha_solicitud,
      fecha_resolucion: inc.fecha_resolucion,
      usuario: inc.usuario,
      fecha: inc.fecha,
      hora_inicio: inc.hora_inicio,
      hora_fin: inc.hora_fin,
      marcaje: inc.marcaje.map(DescriptorMarcajeDTO::from),
      estado: inc.estado as u8,
      fecha_estado: inc.fecha_estado,
      error: inc.error,
      usuario_creador: inc.usuario_creador,
      usuario_gestor: inc.usuario_gestor,
      motivo_solicitud: inc.motivo_solicitud,
      motivo_rechazo: inc.motivo_rechazo,
    }
  }
}

// DTO genérico para DominiosWithCacheUsuario
#[derive(Serialize)]
pub(in crate::app) struct DominiosWithCacheUsuarioDTO<T> {
  pub items: Vec<T>,
  pub cache: HashMap<u32, DescriptorUsuarioDTO>,
}

impl<T, U> From<DominiosWithCacheUsuario<T>> for DominiosWithCacheUsuarioDTO<U>
where
  U: From<T>,
{
  fn from(domain: DominiosWithCacheUsuario<T>) -> Self {
    DominiosWithCacheUsuarioDTO {
      items: domain.items.into_iter().map(U::from).collect(),
      cache: domain
        .cache
        .into_iter()
        .map(|(id, user)| (id, user.into()))
        .collect(),
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
