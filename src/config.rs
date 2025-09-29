use chrono_tz::Tz;
use serde::Deserialize;
use std::fs::{self, File};
use std::io::BufReader;
use std::path::{Path, PathBuf};

use crate::infra::PasswordLimites;

#[derive(Deserialize, Debug, Clone, Copy)]
pub struct PasswordConfig {
  pub longitud_minima: usize,
  pub mayusculas: bool,
  pub minusculas: bool,
  pub digitos: bool,
  pub caracteres_especiales: bool,
}

impl From<PasswordConfig> for PasswordLimites {
  fn from(pass: PasswordConfig) -> Self {
    PasswordLimites::new(
      pass.longitud_minima,
      pass.mayusculas,
      pass.minusculas,
      pass.digitos,
      pass.caracteres_especiales,
    )
  }
}

/// Representa los límites del número de registros
/// que se pueden obtener en las consultas.
#[derive(Deserialize, Debug, Clone, Copy)]
pub struct Limites {
  /// Número máximo de los últimos marcajes horarios a mostrar
  pub ultimos_marcajes: u8,
}

#[derive(Deserialize)]
/// Representa la configuración de la base de datos.
pub struct DB {
  /// url de la base de datos.
  pub url: String,
  /// usuario de la base de datos.
  pub usuario: String,
  /// password de la base de datos.
  /// Se obtiene de un fichero secreto.
  pub password: String,
  /// Número máximo de conexiones a la base de datos.
  pub max_conexiones: u32,
  /// Límites de las consultas.
  pub limites: Limites,
}

impl std::fmt::Debug for DB {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("DB")
      .field("url", &self.url)
      .field("usuario", &self.usuario)
      .field("password", &"[OCULTA]")
      .field("max_conexiones", &self.max_conexiones)
      .field("limites", &self.limites)
      .finish()
  }
}

/// Representa la configuración de los logs.
#[derive(Deserialize, Debug)]
pub struct Log {
  /// Nivel de log.
  pub level: String,
}

/// Representa la configuración del servidor
#[derive(Deserialize, Debug)]
pub struct Servidor {
  /// Host del servidor
  pub host: String,
  /// Puerto
  pub puerto: u32,
  /// Producción o desarrollo
  pub produccion: bool,
}

/// Representa la configuración de la aplicación.
///
/// Consultar las estructuras internas para ver qué valores
/// se obtienen de un fichero secreto.
#[derive(Deserialize)]
pub struct Config {
  pub db: DB,
  pub log: Log,
  pub servidor: Servidor,
  pub password: PasswordConfig,
  pub zona_horaria: Tz,
  pub secreto: String,
  // Duración en segundos de la sesión cuando un usuario autentica
  pub caducidad_sesion: u64,
}

impl std::fmt::Debug for Config {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("Config")
      .field("db", &self.db)
      .field("log", &self.log)
      .field("servidor", &self.servidor)
      .field("password", &self.password)
      .field("zona_horaria", &self.zona_horaria)
      .field("secreto", &"[OCULTO]")
      .field("caducidad_sesion", &self.caducidad_sesion)
      .finish()
  }
}

/// Representa la configuración del trabajo.
///
/// Se propaga a través de la aplicación
#[derive(Clone)]
pub struct ConfigTrabajo {
  pub zona_horaria: Tz,
  pub secreto: String,
  pub limites: Limites,
  pub passw: PasswordLimites,
  pub caducidad_sesion: u64,
  pub produccion: bool,
}

impl Config {
  /// Carga la configuración desde un archivo JSON.
  ///
  /// Existen valores que se obtienen de un fichero secreto.
  /// Por ejemplo, la contraseña de la base de datos.
  /// El fichero secreto se obtiene de la carpeta indicada.
  /// Para ver que valores se obtienen de un fichero secreto
  /// se debe consultar la documentación de las estructuras
  /// internas de [`Config`].
  pub fn desde_archivo(archivo: &Path, secreto: Secreto) -> Self {
    let fichero = File::open(archivo)
      .expect("No se pudo abrir el archivo de configuración");

    let reader = BufReader::new(fichero);
    let mut config: Self = serde_json::from_reader(reader)
      .expect("No se pudo deserializar el archivo de configuración");

    config.db.password = secreto.get(&config.db.password);
    config.secreto = secreto.get(&config.secreto);

    config
  }

  /// Genera la configuración para las aplicaciones que gestionan el trabajo.
  pub fn config_trabajo(&self) -> ConfigTrabajo {
    ConfigTrabajo {
      secreto: self.secreto.clone(),
      zona_horaria: self.zona_horaria,
      limites: self.db.limites,
      passw: self.password.into(),
      caducidad_sesion: self.caducidad_sesion,
      produccion: self.servidor.produccion,
    }
  }
}

/// Representa un secreto que se obtiene de un fichero en una carpeta.
pub struct Secreto {
  ruta: PathBuf,
}

impl Secreto {
  /// Crea un nuevo objeto `Secreto` con la ruta a la carpeta
  /// donde se encuentran los ficheros secretos.
  pub fn new(ruta: PathBuf) -> Self {
    Self { ruta }
  }

  /// Obtiene el contenido del fichero secreto con el código dado.
  ///
  /// El código es el nombre del fichero sin la extensión.
  pub fn get(&self, codigo: &str) -> String {
    let fichero = self.ruta.join(codigo);
    fs::read_to_string(fichero).unwrap_or_else(|err| {
      panic!("No se pudo leer el fichero secreto: {} ({}) ", codigo, err)
    })
  }
}
