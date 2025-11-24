use std::collections::{HashMap, HashSet};

use std::fmt::{self, Formatter};
use std::{fmt::Display, ops::Deref};

use data_encoding::HEXLOWER;
use sha2::{Digest, Sha256};

use crate::infra::{desencriptar, encriptar};
use crate::usuarios::DescriptorUsuario;

/// Estructura que representa un array de entidades
/// que contiene descriptores de usuarios
/// y una colección de descriptores de usuario
/// cacheados que corresponde con todos los usuarios de
/// la entidades
pub struct DominioWithCacheUsuario<T> {
  pub items: Vec<T>,
  pub cache: HashMap<u32, DescriptorUsuario>,
}

impl<T> DominioWithCacheUsuario<T> {
  pub fn new(capacidad_entidad: usize) -> Self {
    Self {
      items: Vec::with_capacity(capacidad_entidad),
      cache: HashMap::with_capacity(capacidad_entidad / 2),
    }
  }

  /// Agregar una entidad
  pub fn push_entidad(&mut self, item: T) {
    self.items.push(item);
  }

  /// Agregar un descriptor de usuario asociado
  ///
  /// Si ya existe en la caché, no se añade de nuevo
  pub fn push_usuario(&mut self, usuario: DescriptorUsuario) {
    self.cache.entry(usuario.id).or_insert(usuario);
  }
}

/// Tipo que representa un valor encriptado
#[derive(Debug, Eq, PartialEq)]
#[repr(transparent)]
pub struct Crypto(String);

impl Crypto {
  pub fn new(s: String) -> Self {
    Crypto(s)
  }
}

impl Deref for Crypto {
  type Target = String;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl From<Crypto> for String {
  fn from(dni: Dni) -> String {
    dni.0
  }
}

impl Display for Crypto {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.0)
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
    self.0.is_empty()
  }
}

pub type Password = Crypto;
pub type Dni = Crypto;

// Valida si un DNI español es correcto
pub fn dni_valido(dni: &Dni) -> bool {
  if dni.len() != 9 {
    return false;
  }

  // Separar número (8 dígitos) y letra (1 carácter)
  let (numero_str, letra) = dni.split_at(8);
  let letra = letra.to_uppercase();

  // Verificar que los primeros 8 caracteres son dígitos
  if !numero_str.chars().all(|c| c.is_ascii_digit()) {
    return false;
  }

  let numero: u32 = match numero_str.parse() {
    Ok(n) => n,
    Err(_) => return false,
  };

  // Calcular la letra que debería tener
  let letras = "TRWAGMYFPDXBNJZSQVHLCKE";
  let letra_calculada = letras.chars().nth((numero % 23) as usize);

  // Comparar con la letra proporcionada
  match letra_calculada {
    Some(l) => l.to_string() == letra,
    None => false,
  }
}

/// Configuración para la validación de contraseñas
#[derive(Debug, Clone)]
pub struct PasswordLimites {
  pub longitud_minima: usize,
  pub mayusculas: bool,
  pub minusculas: bool,
  pub digitos: bool,
  pub caracteres_especiales: bool,
  pub special_chars: &'static str,
  pub password_comunes: HashSet<String>,
}

impl PasswordLimites {
  pub fn new(
    len: usize,
    mayus: bool,
    minus: bool,
    digitos: bool,
    chars: bool,
  ) -> Self {
    Self {
      longitud_minima: len,
      mayusculas: mayus,
      minusculas: minus,
      digitos,
      caracteres_especiales: chars,
      special_chars: "!@#$%^&*()_+-=[]{}|;:,.<>?",
      password_comunes: HashSet::from([
        "clave".to_string(),
        "password".to_string(),
        "123456".to_string(),
        "qwerty".to_string(),
        "admin".to_string(),
        "12345678".to_string(),
        "87654321".to_string(),
      ]),
    }
  }
}

/// Resultado de la validación de contraseña
#[derive(Debug, PartialEq)]
pub struct PasswordValidationResult {
  pub es_valido: bool,
  pub errors: Vec<String>,
  pub score: u8, // 0-100
}

impl Display for PasswordValidationResult {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    // Estado de validación
    let estado = if self.es_valido {
      "✅ VÁLIDO"
    } else {
      "❌ INVÁLIDO"
    };

    // Encabezado
    writeln!(f, "{} - Puntuación: {}/100", estado, self.score)?;
    writeln!(f, "Nivel de seguridad: {}", self.nivel_seguridad())?;

    // Errores si los hay
    if !self.errors.is_empty() {
      writeln!(f, "\nErrores encontrados:")?;
      for error in &self.errors {
        writeln!(f, "• {}", error)?;
      }
    }

    Ok(())
  }
}

impl PasswordValidationResult {
  /// Función para obtener una descripción del nivel de seguridad
  pub fn nivel_seguridad(&self) -> &'static str {
    match self.score {
      0..=39 => "Muy débil",
      40..=59 => "Débil",
      60..=79 => "Moderada",
      80..=89 => "Fuerte",
      90..=100 => "Muy fuerte",
      _ => "Desconocido",
    }
  }
}

/// Valida la seguridad de una contraseña
/// según la configuración proporcionada
pub fn validar_password(
  password: &Password,
  config: &PasswordLimites,
) -> PasswordValidationResult {
  let mut errors = Vec::new();
  let mut score = 0;

  // Verificar longitud mínima
  if password.len() < config.longitud_minima {
    errors.push(format!(
      "La contraseña debe tener al menos {} caracteres",
      config.longitud_minima
    ));
  } else {
    score += 20;
  }

  // Verificar letras mayúsculas
  if config.mayusculas && !password.chars().any(|c| c.is_ascii_uppercase()) {
    errors.push("Debe contener al menos una letra mayúscula".to_string());
  } else if config.mayusculas {
    score += 15;
  }

  // Verificar letras minúsculas
  if config.minusculas && !password.chars().any(|c| c.is_ascii_lowercase()) {
    errors.push("Debe contener al menos una letra minúscula".to_string());
  } else if config.minusculas {
    score += 15;
  }

  // Verificar dígitos
  if config.digitos && !password.chars().any(|c| c.is_ascii_digit()) {
    errors.push("Debe contener al menos un dígito".to_string());
  } else if config.digitos {
    score += 15;
  }

  // Verificar caracteres especiales
  if config.caracteres_especiales
    && !password.chars().any(|c| config.special_chars.contains(c))
  {
    errors.push(format!(
      "Debe contener al menos un carácter especial: {}",
      config.special_chars
    ));
  } else if config.caracteres_especiales {
    score += 20;
  }

  // Verificar contraseñas comunes
  if config.password_comunes.contains(&password.to_lowercase()) {
    errors.push("La contraseña es demasiado común".to_string());
  } else {
    score += 15;
  }

  // Ajustar score basado en longitud
  if password.len() >= 12 {
    score += 10;
  } else if password.len() >= 16 {
    score += 20;
  }

  // Asegurar que el score esté entre 0-100
  score = score.min(100);

  PasswordValidationResult {
    es_valido: errors.is_empty(),
    errors,
    score,
  }
}

// Tests
#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_dni_valido() {
    assert!(dni_valido(&Dni::new("12345678Z".to_string()))); // DNI válido
    assert!(dni_valido(&Dni::new("00000000T".to_string()))); // Caso límite
    assert!(dni_valido(&Dni::new("99999999R".to_string()))); // Caso límite
  }

  #[test]
  fn test_dni_invalido() {
    assert!(!dni_valido(&Dni::new("12345678A".to_string()))); // Letra incorrec
    assert!(!dni_valido(&Dni::new("1234567Z".to_string()))); // Muy corto
    assert!(!dni_valido(&Dni::new("123456789Z".to_string()))); // Muy largo
    assert!(!dni_valido(&Dni::new("ABCD5678Z".to_string()))); // Número no váli
    assert!(!dni_valido(&Dni::new("12345678".to_string()))); // Falta letra
  }

  #[test]
  fn test_dni_valido_minusculas() {
    assert!(dni_valido(&Dni::new("12345678z".to_string()))); // Letra en minús
  }

  #[test]
  fn test_password_valida() {
    let config = PasswordLimites::new(8, true, true, true, true);
    let result =
      validar_password(&Password::new("Secure123!".to_string()), &config);
    assert!(result.es_valido);
    assert!(result.score >= 80);
  }

  #[test]
  fn test_password_corta() {
    let config = PasswordLimites::new(8, true, true, true, true);
    let result =
      validar_password(&Password::new("Short1!".to_string()), &config);
    assert!(!result.es_valido);
    assert!(result.errors.iter().any(|e| e.contains("al menos 8")));
  }

  #[test]
  fn test_password_sin_mayusculas() {
    let config = PasswordLimites::new(8, true, true, true, true);
    let result =
      validar_password(&Password::new("password123!".to_string()), &config);
    assert!(!result.es_valido);
    assert!(result.errors.iter().any(|e| e.contains("mayúscula")));
  }

  #[test]
  fn test_password_sin_minúsculas() {
    let config = PasswordLimites::new(8, true, true, true, true);
    let result =
      validar_password(&Password::new("DEDF4!WSD!&".to_string()), &config);
    assert!(!result.es_valido);
    assert!(result.errors.iter().any(|e| e.contains("minúscula")));
  }

  #[test]
  fn test_password_sin_digito() {
    let config = PasswordLimites::new(8, true, true, true, true);
    let result =
      validar_password(&Password::new("AwsDef$r&fr".to_string()), &config);
    assert!(!result.es_valido);
    assert!(result.errors.iter().any(|e| e.contains("dígito")));
  }

  #[test]
  fn test_password_sin_especiales() {
    let config = PasswordLimites::new(8, true, true, true, true);
    let result =
      validar_password(&Password::new("F3dEfrTGfRf43".to_string()), &config);
    assert!(!result.es_valido);
    assert!(result.errors.iter().any(|e| e.contains("especial")));
  }

  #[test]
  fn test_password_comun() {
    let config = PasswordLimites::new(8, true, true, true, true);
    let result =
      validar_password(&Password::new("password".to_string()), &config);
    assert!(!result.es_valido);
    assert!(result.errors.iter().any(|e| e.contains("común")));
  }
}
