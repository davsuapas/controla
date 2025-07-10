//! Este módulo define la estructura de configuración de la aplicación
//!
//! El fichero de configuración debe estar en formato JSON

use serde::Deserialize;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

#[derive(Deserialize)]
pub struct DB {
  pub url: String,
  pub password: String,
}

#[derive(Deserialize)]
pub struct Config {
  pub db: DB,
}

impl Config {
  pub fn desde_archivo(archivo: &Path) -> Self {
    let fichero = File::open(archivo)
      .expect("No se pudo abrir el archivo de configuración");

    let reader = BufReader::new(fichero);
    let config: Self = serde_json::from_reader(reader)
      .expect("No se pudo deserializar el archivo de configuración");

    config
  }
}
