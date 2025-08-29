use std::ops::Deref;

use data_encoding::HEXLOWER;
use sha2::{Digest, Sha256};

use crate::infra::{desencriptar, encriptar};

/// Tipo que representa un valor encriptado
#[derive(Debug)]
#[repr(transparent)]
pub struct Crypto(String);

impl Crypto {
  pub fn new(s: String) -> Self {
    Crypto(s)
  }
}

impl Crypto {
  /// Crea un nuevo valor desencriptando el valor proporcionado.
  ///
  /// Si el valor es None, se crea con una cadena vacía.
  #[inline]
  pub fn from_encriptado(
    dni_encrypted: Option<&str>,
    clave: &str,
  ) -> Result<Self, anyhow::Error> {
    match dni_encrypted {
      Some(d) => desencriptar(d, clave).map(Self),
      None => Ok(Self("".to_string())),
    }
  }

  /// Encripta el valor proporcionado y crea un nuevo valor.
  #[inline]
  pub fn encriptar(&self, clave: &str) -> Result<String, anyhow::Error> {
    encriptar(&self.0, clave)
  }

  /// Genera un hash con salt
  /// Misma entrada, misma salida (para detección de duplicados)
  pub fn hash_con_salt(&self, salt: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(self.0.as_bytes());
    hasher.update(salt.as_bytes());
    HEXLOWER.encode(&hasher.finalize())
  }

  /// Devuelve true si el valor es una cadena vacía.
  #[inline]
  pub fn is_empty(&self) -> bool {
    self.0.trim().is_empty()
  }
}

impl Deref for Crypto {
  type Target = String;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

pub type Password = Crypto;
pub type Dni = Crypto;
