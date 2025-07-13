//! controla gestiona el registro horario del personal de una organización
//!
//! El sistema debe cumplir el Real Decreto-ley 8/2019, de 8 de marzo.
//! Puntos principales:
//! - Registro inmutable de la jornada laboral.
//! - Tener en cuenta la flexibilidad horaria.
//! - Resguardo de los datos durante al menos 4 años.
//! - Acceso desde cualquier lugar por parte de los inspectores.
//!
//! El sistema expone una API REST que consume el cliente web. Se pretende
//! dar cabida a los siguientes casos de uso:
//! - Sistema de identificación de usuarios.
//! - Marcaje de entrada y salida.
//! - Saldo horario: diferencia entre horas marcadas y horas trabajadas.
//! - Cumplimiento horario: horas trabajadas frente a horas previstas.
//! - Solicitud de marcaje: Solicitud para gestionar errores de marcaje.
//! - Gestión de las incidencias de marcajes.
//! - Administración de usuarios y empleados
//! - Informes horrarios y de incidencias.
//! - Registro automático y manual por el controlador.
//! - Consultas por partede de un inspector.
//!
//! Se usará una base de datos mysql para almacenar los datos de la aplicación.
//! y AWS Cognito para la autenticación de usuarios.
//!
//! # Ejecución:
//! ```bash
//! controla -fichero_config=config.json -carpeta_secretos=secretos
//! ```
//! # Configuración:
//! La carpeta de secretos debe contener un fichero por cada secreto
//! que se quiera usar en la configuración.

mod config;

mod registro;
mod usuarios;

use config::*;

use std::path::PathBuf;
use std::{env, path::Path};
use tracing::level_filters::LevelFilter;
use tracing_subscriber::fmt;

fn main() {
  let args: Vec<String> = env::args().collect();

  let fichero_config = obtener_argumento(&args, "-fichero_config=")
    .map(Path::new)
    .expect("Falta el argumento 'fichero_config='");

  let carpeta_secretos = obtener_argumento(&args, "-carpeta_secretos=")
    .map(PathBuf::from)
    .expect("Falta el argumento 'carpeta_secretos='");

  fmt::Subscriber::builder()
    .with_max_level(LevelFilter::INFO)
    .pretty()
    .with_target(false)
    .init();

  tracing::info!("Iniciando la aplicación controla...");

  let config =
    Config::desde_archivo(fichero_config, Secreto::new(carpeta_secretos));

  tracing::info!("Configuración cargada: {:?}", config);
}

fn obtener_argumento<'a>(args: &'a [String], prefijo: &str) -> Option<&'a str> {
  args
    .iter()
    .find(|arg| arg.starts_with(prefijo))
    .map(|arg| &arg[prefijo.len()..])
}
