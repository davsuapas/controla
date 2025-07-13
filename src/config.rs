use serde::Deserialize;
use std::fs::{self, File};
use std::io::BufReader;
use std::path::{Path, PathBuf};

#[derive(Deserialize, Debug)]
/// Representa la configuración de la base de datos.
pub struct DB {
  #[allow(unused)]
  /// url de la base de datos.
  pub url: String,
  /// password de la base de datos.
  /// Se obtiene de un fichero secreto.
  pub password: String,
}

/// Representa la configuración de la aplicación.
///
/// Consultar las estructuras internas para ver qué valores
/// se obtienen de un fichero secreto.
#[derive(Deserialize, Debug)]
pub struct Config {
  pub db: DB,
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

    config
  }
}

/// Representa un secreto que se obtiene de un fichero en una carpeta.
pub struct Secreto {
  ruta: PathBuf,
}

impl Secreto {
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
