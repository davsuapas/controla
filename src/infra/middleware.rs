use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use axum::{
  Extension, extract::Request, http::StatusCode, middleware::Next,
  response::IntoResponse,
};
use axum_extra::extract::cookie::{Cookie, CookieJar, SameSite};
use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use sha2::Sha256;

const NOMBRE_COOKIE_SESION: &str = "token";

/// Datos de la sesión que se almacenarán
#[derive(Debug, Serialize, Deserialize)]
pub struct DatosSesion {
  pub id: String,
  pub caduca_en: u64, // timestamp UNIX
}

/// Manejador de sesiones con caducidad
pub struct ManejadorSesion {
  clave_secreta: String,
  duracion_sesion: Duration,
  produccion: bool,
}

/// Errores de sesión
#[derive(Debug)]
pub enum ErrorSesion {
  TokenInvalido,
  SesionExpirada,
  ErrorEncriptacion,
}

type HmacSha256 = Hmac<Sha256>;

impl ManejadorSesion {
  pub fn new(
    clave_secreta: String,
    duracion_sesion: Duration,
    produccion: bool,
  ) -> Self {
    Self {
      clave_secreta,
      duracion_sesion,
      produccion,
    }
  }

  /// Crea un nuevo token de sesión firmado
  pub fn crear_sesion(&self) -> Result<Cookie<'_>, ErrorSesion> {
    let caduca_en =
      self.obtener_timestamp_actual() + self.duracion_sesion.as_secs();

    let datos_sesion = DatosSesion {
      id: uuid::Uuid::new_v4().to_string(),
      caduca_en,
    };

    let token = self.crear_token_hmac(&datos_sesion)?;

    // Crear cookie segura
    Ok(
      Cookie::build((NOMBRE_COOKIE_SESION, token))
        .path("/")
        .http_only(true)
        .secure(self.produccion)
        .same_site(self.lax())
        .max_age(time::Duration::seconds(
          self.duracion_sesion.as_secs() as i64
        ))
        .build(),
    )
  }

  /// Elimina la cookie de sesión del cliente
  pub fn eliminar_sesion(&self) -> Cookie<'_> {
    Cookie::build((NOMBRE_COOKIE_SESION, ""))
      .path("/")
      .http_only(true)
      .secure(self.produccion)
      .same_site(self.lax())
      .max_age(time::Duration::seconds(0))
      .build()
  }

  fn lax(&self) -> SameSite {
    if self.produccion {
      SameSite::Strict
    } else {
      SameSite::Lax
    }
  }

  /// Valida un token de sesión
  pub fn validar_sesion(
    &self,
    token: &str,
  ) -> Result<DatosSesion, ErrorSesion> {
    let datos_sesion = self.verificar_token_hmac(token)?;

    // Verificar caducidad
    if datos_sesion.caduca_en < self.obtener_timestamp_actual() {
      return Err(ErrorSesion::SesionExpirada);
    }

    Ok(datos_sesion)
  }

  /// Crea token HMAC firmado
  fn crear_token_hmac(
    &self,
    datos: &DatosSesion,
  ) -> Result<String, ErrorSesion> {
    let json_datos = serde_json::to_string(datos)
      .map_err(|_| ErrorSesion::ErrorEncriptacion)?;

    let mut mac = HmacSha256::new_from_slice(self.clave_secreta.as_bytes())
      .map_err(|_| ErrorSesion::ErrorEncriptacion)?;

    mac.update(json_datos.as_bytes());
    let resultado = mac.finalize();
    let firma = hex::encode(resultado.into_bytes());

    Ok(format!("{}.{}", json_datos, firma))
  }

  /// Verifica token HMAC
  fn verificar_token_hmac(
    &self,
    token: &str,
  ) -> Result<DatosSesion, ErrorSesion> {
    let partes: Vec<&str> = token.splitn(2, '.').collect();
    if partes.len() != 2 {
      return Err(ErrorSesion::TokenInvalido);
    }

    let json_datos = partes[0];
    let firma = partes[1];

    // Verificar HMAC
    let mut mac = HmacSha256::new_from_slice(self.clave_secreta.as_bytes())
      .map_err(|_| ErrorSesion::ErrorEncriptacion)?;

    mac.update(json_datos.as_bytes());

    let bytes_firma =
      hex::decode(firma).map_err(|_| ErrorSesion::TokenInvalido)?;

    mac
      .verify_slice(&bytes_firma)
      .map_err(|_| ErrorSesion::TokenInvalido)?;

    // Deserializar datos
    let datos_sesion: DatosSesion = serde_json::from_str(json_datos)
      .map_err(|_| ErrorSesion::TokenInvalido)?;

    Ok(datos_sesion)
  }

  /// Obtiene el timestamp actual
  fn obtener_timestamp_actual(&self) -> u64 {
    SystemTime::now()
      .duration_since(UNIX_EPOCH)
      .unwrap()
      .as_secs()
  }
}

/// Middleware que verifica si la sesión es válida y no ha expirado
pub async fn autenticacion(
  cookiejar: CookieJar,
  Extension(manejador_sesiones): Extension<Arc<ManejadorSesion>>,
  solicitud: Request,
  siguiente: Next,
) -> Result<impl IntoResponse, StatusCode> {
  let token = cookiejar
    .get(NOMBRE_COOKIE_SESION)
    .map(|cookie| cookie.value().to_string())
    .ok_or_else(|| {
      tracing::error!("Cookie de sesión no encontrada");
      StatusCode::UNAUTHORIZED
    })?;

  match manejador_sesiones.validar_sesion(&token) {
    Ok(_) => Ok(siguiente.run(solicitud).await),
    Err(err) => {
      tracing::error!(error = ?err, "Middleware de autenticación");
      Err(StatusCode::UNAUTHORIZED)
    }
  }
}
