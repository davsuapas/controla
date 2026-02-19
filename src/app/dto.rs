use std::collections::HashMap;

use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use serde::{Deserialize, Serialize};

use crate::horario::{
  Calendario, CalendarioFecha, ConfigHorario, Dia, Horario, TipoCalendarioFecha,
};
use crate::informes::{CumplimientoHorario, InformeCumplimiento};
use crate::{
  inc::{
    EstadoIncidencia, Incidencia, IncidenciaProceso, IncidenciaSolictud,
    TipoIncidencia,
  },
  infra::{Dni, DominioWithCacheUsuario, Password, ShortDateTimeFormat},
  marcaje::{DescriptorMarcaje, Marcaje},
  usuarios::{DescriptorUsuario, Rol, Usuario},
};

#[derive(Deserialize)]
pub struct IncidenciasFiltroParams {
  pub fecha_inicio: Option<NaiveDate>,
  pub fecha_fin: Option<NaiveDate>,
  pub estados: Vec<u8>,
  pub supervisor: bool,
  pub usuario: Option<u32>,
}

#[derive(Serialize)]
pub(in crate::app) struct UsuarioCalendarioDTO {
  pub calendario: u32,
  pub nombre: String,
  pub asignado: bool,
}

impl From<&crate::usuarios::UsuarioCalendario> for UsuarioCalendarioDTO {
  fn from(value: &crate::usuarios::UsuarioCalendario) -> Self {
    Self {
      calendario: value.calendario,
      nombre: value.nombre.clone(),
      asignado: value.asignado,
    }
  }
}

/// Define la entidad de intercambio para el usuario
#[derive(Deserialize)]
pub(in crate::app) struct UsuarioBodyDTO {
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
  #[serde(default)]
  pub calendarios: Vec<u32>,
}

#[derive(Serialize)]
pub(in crate::app) struct UsuarioOutDTO {
  pub id: u32,
  pub dni: String,
  pub email: String,
  pub nombre: String,
  pub primer_apellido: String,
  pub segundo_apellido: String,
  pub activo: Option<NaiveDateTime>,
  pub inicio: Option<NaiveDateTime>,
  pub roles: Vec<u8>,
  pub calendarios: Vec<UsuarioCalendarioDTO>,
}

impl From<Usuario> for UsuarioOutDTO {
  fn from(usr: Usuario) -> Self {
    UsuarioOutDTO {
      id: usr.id,
      dni: usr.dni.into(),
      email: usr.email,
      nombre: usr.nombre,
      primer_apellido: usr.primer_apellido,
      segundo_apellido: usr.segundo_apellido,
      activo: usr.activo,
      inicio: usr.inicio,
      roles: usr.roles.iter().map(|r| *r as u8).collect(),
      calendarios: usr
        .calendarios
        .iter()
        .map(UsuarioCalendarioDTO::from)
        .collect(),
    }
  }
}

impl From<UsuarioBodyDTO> for Usuario {
  fn from(usr: UsuarioBodyDTO) -> Self {
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
      calendarios: usr
        .calendarios
        .into_iter()
        .map(|c| crate::usuarios::UsuarioCalendario {
          calendario: c,
          nombre: "".to_string(),
          asignado: true,
        })
        .collect(),
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
#[derive(Serialize, Deserialize)]
pub(in crate::app) struct HorarioDTO {
  pub id: u32,
  pub dia: String,
  pub hora_inicio: NaiveTime,
  pub hora_fin: NaiveTime,
  pub horas_a_trabajar: f64,
}

impl From<Horario> for HorarioDTO {
  fn from(horario: Horario) -> Self {
    HorarioDTO {
      id: horario.id,
      dia: horario.dia.letra().to_string(),
      hora_inicio: horario.hora_inicio,
      hora_fin: horario.hora_fin,
      horas_a_trabajar: horario.horas_a_trabajar(),
    }
  }
}

impl From<HorarioDTO> for Horario {
  fn from(dto: HorarioDTO) -> Self {
    Horario {
      id: dto.id,
      dia: Dia::from(dto.dia.as_str()),
      hora_inicio: dto.hora_inicio,
      hora_fin: dto.hora_fin,
    }
  }
}

#[derive(Serialize, Deserialize)]
pub(in crate::app) struct ConfigHorarioDTO {
  pub id: u32,
  pub usuario: u32,
  pub horario: HorarioDTO,
  pub fecha_creacion: NaiveDate,
  pub caducidad_fecha_ini: Option<NaiveDate>,
  pub caducidad_fecha_fin: Option<NaiveDate>,
}

impl From<ConfigHorario> for ConfigHorarioDTO {
  fn from(config: ConfigHorario) -> Self {
    ConfigHorarioDTO {
      id: config.id,
      usuario: config.usuario,
      horario: config.horario.into(),
      fecha_creacion: config.fecha_creacion,
      caducidad_fecha_ini: config.caducidad_fecha_ini,
      caducidad_fecha_fin: config.caducidad_fecha_fin,
    }
  }
}

impl From<ConfigHorarioDTO> for ConfigHorario {
  fn from(dto: ConfigHorarioDTO) -> Self {
    ConfigHorario {
      id: dto.id,
      usuario: dto.usuario,
      horario: dto.horario.into(),
      fecha_creacion: dto.fecha_creacion,
      caducidad_fecha_ini: dto.caducidad_fecha_ini,
      caducidad_fecha_fin: dto.caducidad_fecha_fin,
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
  pub horario: HorarioDTO,
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

// Define la entidad de intercambio para incidencias tipo solicitud.
#[derive(Deserialize)]
pub struct IncidenciaSolictudDTO {
  pub id: u32,
  pub estado: u8,
  pub motivo_solicitud: Option<String>,
  pub fecha_solicitud: NaiveDateTime,
  pub hora_inicio: Option<NaiveTime>,
  pub hora_fin: Option<NaiveTime>,
  pub usuario_creador: u32,
}

impl From<IncidenciaSolictudDTO> for IncidenciaSolictud {
  fn from(inc: IncidenciaSolictudDTO) -> Self {
    IncidenciaSolictud {
      id: inc.id,
      estado: EstadoIncidencia::from(inc.estado),
      motivo_solicitud: inc.motivo_solicitud,
      fecha_solicitud: inc.fecha_solicitud,
      hora_inicio: inc.hora_inicio,
      hora_fin: inc.hora_fin,
      usuario_creador: inc.usuario_creador,
    }
  }
}

// Define la entidad de intercambio para el calendario.
#[derive(Serialize, Deserialize)]
pub(in crate::app) struct CalendarioDTO {
  pub id: u32,
  pub nombre: String,
  pub descripcion: String,
}

impl From<Calendario> for CalendarioDTO {
  fn from(c: Calendario) -> Self {
    CalendarioDTO {
      id: c.id,
      nombre: c.nombre,
      descripcion: c.descripcion,
    }
  }
}

impl From<CalendarioDTO> for Calendario {
  fn from(dto: CalendarioDTO) -> Self {
    Calendario {
      id: dto.id,
      nombre: dto.nombre,
      descripcion: dto.descripcion,
    }
  }
}

// Define la entidad de intercambio para las fechas del calendario.
#[derive(Serialize, Deserialize)]
pub(in crate::app) struct CalendarioFechaDTO {
  pub id: u32,
  pub calendario: u32,
  pub fecha_inicio: NaiveDate,
  pub fecha_fin: NaiveDate,
  pub tipo: u8,
}

impl From<CalendarioFecha> for CalendarioFechaDTO {
  fn from(f: CalendarioFecha) -> Self {
    CalendarioFechaDTO {
      id: f.id,
      calendario: f.calendario,
      fecha_inicio: f.fecha_inicio,
      fecha_fin: f.fecha_fin,
      tipo: f.tipo.into(),
    }
  }
}

impl From<CalendarioFechaDTO> for CalendarioFecha {
  fn from(dto: CalendarioFechaDTO) -> Self {
    CalendarioFecha {
      id: dto.id,
      calendario: dto.calendario,
      fecha_inicio: dto.fecha_inicio,
      fecha_fin: dto.fecha_fin,
      tipo: TipoCalendarioFecha::from(dto.tipo),
    }
  }
}

#[derive(Serialize)]
pub struct CumplimientoHorarioDTO {
  pub fecha: NaiveDate,
  pub horas_trabajo_efectivo: f64,
  pub horas_trabajadas: f64,
  pub horas_a_trabajar: f64,
  pub saldo: f64,
  pub nota: String,
}

impl From<CumplimientoHorario> for CumplimientoHorarioDTO {
  fn from(value: CumplimientoHorario) -> Self {
    Self {
      fecha: value.fecha,
      horas_trabajo_efectivo: value.horas_trabajo_efectivo,
      horas_trabajadas: value.horas_trabajadas,
      horas_a_trabajar: value.horas_a_trabajar,
      saldo: value.saldo,
      nota: value.nota,
    }
  }
}

#[derive(Serialize)]
pub struct InformeCumplimientoDTO {
  pub lineas: Vec<CumplimientoHorarioDTO>,
  #[serde(rename = "total_saldo")]
  pub total_saldo: f64,
}

impl From<InformeCumplimiento> for InformeCumplimientoDTO {
  fn from(value: InformeCumplimiento) -> Self {
    Self {
      lineas: value
        .lineas
        .into_iter()
        .map(CumplimientoHorarioDTO::from)
        .collect(),
      total_saldo: value.total_saldo,
    }
  }
}

// DTO genérico para DominiosWithCacheUsuario
#[derive(Serialize)]
pub(in crate::app) struct DominiosWithCacheUsuarioDTO<T> {
  pub items: Vec<T>,
  pub cache: HashMap<u32, DescriptorUsuarioDTO>,
}

impl<T, U> From<DominioWithCacheUsuario<T>> for DominiosWithCacheUsuarioDTO<U>
where
  U: From<T>,
{
  fn from(domain: DominioWithCacheUsuario<T>) -> Self {
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
