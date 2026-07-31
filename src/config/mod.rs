//! Gestiona la configuración global de la aplicación almacenada en base de datos
//! y la configuración inicial cargada desde archivo.
//!
//! La configuración global (DB) permite almacenar parámetros generales de la
//! aplicación, como la localización geográfica para acotar los marcajes. Esta
//! configuración se almacena en la tabla `config` y solo contiene un registro.
//!
//! Cuando un usuario activa la opción de acotar marcajes, el sistema utiliza
//! las coordenadas de localización para restringir la creación de marcajes a
//! una zona geográfica determinada. Si no se ha configurado localización, la
//! acotación no tiene efecto.
//!
//! La configuración inicial (archivo) se carga al arrancar la aplicación desde
//! un fichero JSON y contiene parámetros como conexión a base de datos, zona
//! horaria, límites de consultas, etc.

/// Módulo para manejar los dominios sobre la configuración global.
mod dominio;
/// Módulo que gestiona las iteraciones sobre la configuración global con la base de datos.
pub mod repo;
/// Módulo que expone los servicios de la configuración global.
mod servicio;
/// Módulo que contiene la configuración inicial cargada desde archivo JSON.
mod setup;

pub use dominio::*;
pub use repo::*;
pub use servicio::*;
pub use setup::*;
