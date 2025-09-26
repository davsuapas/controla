use base64::{Engine, engine::general_purpose::STANDARD};
use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use ring::{
  aead, hkdf,
  rand::{self, SecureRandom},
};
use thiserror::Error;

use crate::infra::DBError;

#[derive(Debug, Error)]
pub enum ServicioError {
  #[error("Error de acceso a la base de datos: {0}")]
  DB(#[from] DBError),
  #[error("Validación: {0}")]
  Validacion(String),
  #[error("{0}")]
  Usuario(String),
}

impl ServicioError {
  /// Si hay mensajes de error para el usuario devuelve la
  /// cadena con el mensaje si no devuelve una cadena vacía
  pub fn mensaje_usuario(&self) -> String {
    match self {
      ServicioError::Usuario(msg) => ServicioError::mensaje(msg),
      ServicioError::Validacion(msg) => ServicioError::mensaje(msg),
      ServicioError::DB(DBError::RegistroVacio(e)) => {
        ServicioError::mensaje(&e.to_string())
      }
      ServicioError::DB(_) => "".to_string(),
    }
  }

  fn mensaje(msg: &str) -> String {
    format!("@@:{}", msg)
  }
}

/// Dada una fecha y hora, devuelve el día de la semana como una letra.
/// Lunes es 'L', Martes es 'M', Miércoles es 'X' y así sucesivamente.
pub fn letra_dia_semana(dia_semana: chrono::Weekday) -> &'static str {
  match dia_semana {
    chrono::Weekday::Mon => "L",
    chrono::Weekday::Tue => "M",
    chrono::Weekday::Wed => "X",
    chrono::Weekday::Thu => "J",
    chrono::Weekday::Fri => "V",
    chrono::Weekday::Sat => "S",
    chrono::Weekday::Sun => "D",
  }
}

/// Dado el día devuelve el día en formato largo
pub fn dia_semana_formato_largo(dia_semana: chrono::Weekday) -> &'static str {
  match dia_semana {
    chrono::Weekday::Mon => "Lunes",
    chrono::Weekday::Tue => "Martes",
    chrono::Weekday::Wed => "Miércoles",
    chrono::Weekday::Thu => "Jueves",
    chrono::Weekday::Fri => "Viernes",
    chrono::Weekday::Sat => "Sábado",
    chrono::Weekday::Sun => "Domingo",
  }
}

pub trait ShortDateTimeFormat {
  /// Devuelve la fecha y hora en formato corto "dd/mm/yyyy HH:MM".
  fn formato_corto(&self) -> String;
}

impl ShortDateTimeFormat for NaiveDate {
  fn formato_corto(&self) -> String {
    self.format("%d/%m/%Y").to_string()
  }
}

impl ShortDateTimeFormat for NaiveDateTime {
  fn formato_corto(&self) -> String {
    self.format("%d/%m/%Y %H:%M").to_string()
  }
}

impl ShortDateTimeFormat for NaiveTime {
  fn formato_corto(&self) -> String {
    self.format("%H:%M").to_string()
  }
}

/// Encripta una cadena usando AES-GCM 256 con HKDF
pub fn encriptar(cadena: &str, clave: &str) -> Result<String, anyhow::Error> {
  // Derivar clave con HKDF
  let salt = hkdf::Salt::new(hkdf::HKDF_SHA256, &[]);
  let prk = salt.extract(clave.as_bytes());
  let key_material = prk
    .expand(&[b"encryption"], &aead::AES_256_GCM)
    .map_err(|_| anyhow::anyhow!("Error derivando clave".to_string()))?;

  let mut key_bytes = [0u8; 32];
  key_material
    .fill(&mut key_bytes)
    .map_err(|_| anyhow::anyhow!("Error creando clave".to_string()))?;

  // Crear sealing key
  let key = aead::UnboundKey::new(&aead::AES_256_GCM, &key_bytes)
    .map_err(|_| anyhow::anyhow!("Clave inválida".to_string()))?;
  let sealing_key = aead::LessSafeKey::new(key);

  // Generar nonce aleatorio
  let mut nonce_bytes = [0u8; 12];
  let rng = rand::SystemRandom::new();
  rng
    .fill(&mut nonce_bytes)
    .map_err(|e| anyhow::anyhow!(format!("Error generando nonce: {}", e)))?;
  let nonce = aead::Nonce::assume_unique_for_key(nonce_bytes);

  // Encriptar
  let mut data = cadena.as_bytes().to_vec();
  sealing_key
    .seal_in_place_append_tag(nonce, aead::Aad::empty(), &mut data)
    .map_err(|e| anyhow::anyhow!(format!("Error encriptando: {}", e)))?;

  // Combinar nonce + datos cifrados y codificar en Base64
  let mut resultado = Vec::with_capacity(nonce_bytes.len() + data.len());
  resultado.extend_from_slice(&nonce_bytes);
  resultado.extend_from_slice(&data);

  Ok(STANDARD.encode(&resultado))
}

/// Desencripta una cadena
pub fn desencriptar(
  texto_cifrado: &str,
  clave: &str,
) -> Result<String, anyhow::Error> {
  // Decodificar Base64
  let datos = STANDARD
    .decode(texto_cifrado)
    .map_err(|_| anyhow::anyhow!("Base64 inválido".to_string()))?;

  if datos.len() < 12 {
    return Err(anyhow::anyhow!("Datos insuficientes".to_string()));
  }

  // Separar nonce y datos cifrados
  let (nonce_bytes, datos_cifrados) = datos.split_at(12);
  let nonce = aead::Nonce::try_assume_unique_for_key(nonce_bytes)
    .map_err(|_| anyhow::anyhow!("Nonce inválido".to_string()))?;

  // Derivar clave (mismo proceso que encriptación)
  let salt = hkdf::Salt::new(hkdf::HKDF_SHA256, &[]);
  let prk = salt.extract(clave.as_bytes());
  let key_material = prk
    .expand(&[b"encryption"], &aead::AES_256_GCM)
    .map_err(|_| anyhow::anyhow!("Error derivando clave".to_string()))?;

  let mut key_bytes = [0u8; 32];
  key_material
    .fill(&mut key_bytes)
    .map_err(|_| anyhow::anyhow!("Error creando clave".to_string()))?;

  // Crear opening key
  let key = aead::UnboundKey::new(&aead::AES_256_GCM, &key_bytes)
    .map_err(|_| anyhow::anyhow!("Clave inválida".to_string()))?;
  let opening_key = aead::LessSafeKey::new(key);

  // Desencriptar
  let mut datos_vec = datos_cifrados.to_vec();
  let texto_plano = opening_key
    .open_in_place(nonce, aead::Aad::empty(), &mut datos_vec)
    .map_err(|e| anyhow::anyhow!(format!("Error desencriptando: {}", e)))?;

  String::from_utf8(texto_plano.to_vec())
    .map_err(|_| anyhow::anyhow!("UTF-8 inválido".to_string()))
}
