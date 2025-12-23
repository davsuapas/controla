//! controla gestiona el marcaje horario del personal de una organización
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
//! - Administración de usuarios y empleados.
//! - Gestión de horarios múltiples del empleado.
//! - Gestión de los calendarios de los empleados.
//! - Marcaje de entrada y salida por el empleado.
//! - Marcaje manual por el registrador en nombre del empleado.
//! - Consulta de los registros de marcajes.
//! - Informes de saldo horario: Diferencia entre horas marcadas y horas trabajadas.
//! - Informes del cumplimiento horario (resumen mensual): Horas trabajadas frente a horas previstas.
//! - Solicitud de incidencias: Solicitud para gestionar errores de marcaje.
//! - Gestión de las incidencias de marcajes por el gestor.
//! - Informes horarios e incidencias.
//! - Consultas por parte de los inspectores.
//! - Auditoría de acciones realizadas en el sistema.
//!
//! Se usará una base de datos mysql para almacenar los datos de la aplicación.
//! Los campos pk auto-incrementales deben empezar en uno.
//!
//! # Ejecución:
//! ```bash
//! cargo run -- -fichero_config= -carpeta_secretos=
//! ```
//! # Configuración:
//! La carpeta de secretos debe contener un fichero por cada secreto
//! que se quiera usar en la configuración.

mod app;
/// Gestiona la configuración de la aplicación.
mod config;
mod infra;

mod inc;
mod marcaje;
mod traza;
mod usuarios;

use config::*;
use sqlx::mysql::{MySqlConnectOptions, MySqlPoolOptions};

use std::path::PathBuf;
use std::sync::Arc;
use std::{env, path::Path};
use tracing::level_filters::LevelFilter;
use tracing_subscriber::fmt;

use crate::app::{AppState, lanzar_procesos_inicio, rutas};
use crate::infra::PoolConexion;

#[tokio::main]
async fn main() {
  let args: Vec<String> = env::args().collect();

  // Carga la configuración desde el fichero indicado en los argumentos.
  let fichero_config = obtener_argumento(&args, "-fichero_config=")
    .map(Path::new)
    .expect("Falta el argumento 'fichero_config='");

  let carpeta_secretos = obtener_argumento(&args, "-carpeta_secretos=")
    .map(PathBuf::from)
    .expect("Falta el argumento 'carpeta_secretos='");

  let config =
    Config::desde_archivo(fichero_config, Secreto::new(carpeta_secretos));

  // Configura el logger.
  fmt::Subscriber::builder()
    .with_max_level(obtener_nivel_log(&config))
    .pretty()
    .with_target(false)
    .init();

  eprintln!("🚀 Iniciando aplicación controla...");
  eprintln!("🛠️ Configuración cargada: {:?}", config);

  eprintln!("📊 Conectando a la base de datos...");

  // Crea el pool de conexiones a la base de datos.
  // Por ahora solo se conecta por socket.
  let conn = MySqlConnectOptions::new()
    .socket(&config.db.ruta_socket)
    .username(&config.db.usuario)
    .database(&config.db.nombre);

  let pool = MySqlPoolOptions::new()
    .max_connections(config.db.max_conexiones)
    .connect_with(conn)
    .await
    .unwrap_or_else(|err| {
      panic!(
        "No se pudo conectar a la base de datos: {}. Error: {}",
        config.db.nombre, err
      )
    });

  eprintln!("🌐 Preparando los servicios de aplicación...");

  let app = Arc::new(AppState::iniciar(
    &config.config_trabajo(),
    PoolConexion::new(pool),
  ));

  eprintln!("🌱 Lanzando los procesos de inicio...");

  lanzar_procesos_inicio(&config, &app).await;

  eprintln!("📡 Iniciando el servidor web...");

  let direccion =
    format!("{}:{}", config.servidor.host, config.servidor.puerto);

  let listener = tokio::net::TcpListener::bind(&direccion).await.unwrap();

  print_banner();

  eprintln!(
    "✨ Aplicación iniciada y ecuchando en {}. CTRL+C para salir.",
    direccion.as_str()
  );

  axum::serve(listener, rutas(&config.servidor.app, app))
    .await
    .unwrap();
}

fn obtener_nivel_log(config: &Config) -> LevelFilter {
  match config.log.level.as_str() {
    "trace" => LevelFilter::TRACE,
    "debug" => LevelFilter::DEBUG,
    "info" => LevelFilter::INFO,
    "warn" => LevelFilter::WARN,
    "error" => LevelFilter::ERROR,
    _ => LevelFilter::INFO,
  }
}

fn obtener_argumento<'a>(args: &'a [String], prefijo: &str) -> Option<&'a str> {
  args
    .iter()
    .find(|arg| arg.starts_with(prefijo))
    .map(|arg| &arg[prefijo.len()..])
}

fn print_banner() {
  eprintln!(
    r#"
 ██████╗ ██████╗ ███╗   ██╗████████╗██████╗  ██████╗ ██╗      █████╗ 
██╔════╝██╔═══██╗████╗  ██║╚══██╔══╝██╔══██╗██╔═══██╗██║     ██╔══██╗
██║     ██║   ██║██╔██╗ ██║   ██║   ██████╔╝██║   ██║██║     ███████║
██║     ██║   ██║██║╚██╗██║   ██║   ██╔══██╗██║   ██║██║     ██╔══██║
╚██████╗╚██████╔╝██║ ╚████║   ██║   ██║  ██║╚██████╔╝███████╗██║  ██║
 ╚═════╝ ╚═════╝ ╚═╝  ╚═══╝   ╚═╝   ╚═╝  ╚═╝ ╚═════╝ ╚══════╝╚═╝  ╚═╝
    "#
  );
  eprintln!("    🔥 Sistema de Control de Horarios 🔥");
  eprintln!("    ══════════════════════════════════════");
  eprintln!();
}
