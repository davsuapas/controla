/// Representa una localización geográfica para acotar marcajes.
///
/// Contiene las coordenadas de latitud y longitud, así como la precisión
/// en metros de la ubicación obtenida mediante geolocalización.
#[derive(Debug, Clone)]
pub struct Localizacion {
  pub lat: f64,
  pub lng: f64,
  pub accuracy: f64,
}

/// Representa la configuración global de la aplicación.
///
/// Contiene los parámetros generales que se almacenan en la tabla `config`
/// de la base de datos. Actualmente solo incluye la localización geográfica
/// opcional para la acotación de marcajes.
#[derive(Debug, Clone)]
pub struct ConfigGlobal {
  pub localizacion: Option<Localizacion>,
  pub margen_recinto: Option<i32>,
}
