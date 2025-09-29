//! Módulo para gestionar la auditoría del sistema.
//!
//! Las trazas registran los eventos provocados por los usuarios.
//! Cada traza contiene información sobre el tipo de evento,
//! la fecha en que ocurrió, el usuario que lo provocó, y
//! opcionalmente, el horario y los marcajes afectados, así como
//! un motivo descriptivo del evento.

/// Módulo que gestiona el repositorio de las trazas.
mod repo;

/// Módulo que gestiona el dominio de las trazas.
mod dominio;
/// Módulo que gestiona el servicio de las trazas.
mod servicio;

pub use dominio::*;
pub use repo::*;
pub use servicio::*;

/// Macro para agregar una traza y manejar errores de forma consistente.
///
/// Parámetros:
/// - `$self`: Referencia al servicio que contiene `srv_traza.
/// - `tr`: Referencia mutable a la transacción actual.
/// - `$traza`: Referencia a la traza a agregar.
/// - `$mensaje`: Mensaje de error a registrar en caso de fallo.
/// - `$( $key:ident = $value:expr ),*`: Pares clave-valor adicionales
///    para el registro de errores.
///
/// Si la adición de la traza falla, se registra un error con el mensaje y
/// los pares clave-valor proporcionados, se revierte la transacción y se
/// retorna el error.
///
/// ## Ejemplo de uso:
/// ```
/// agregar_traza!(
///   self, tr, traza, "Creando traza creación de usuario", usuario = id);
/// ```
#[macro_export]
macro_rules! agregar_traza {
    (
        $self:expr,
        $tr:expr,
        $traza:expr,
        $mensaje:expr,
        $( $key:ident = $value:expr ),*
    ) => {
        if let Err(err) = $self
            .srv_traza
            .agregar(&mut $tr, &$traza)
            .await
        {
            tracing::error!(
                $( $key = $value, )*
                error = %err,
                $mensaje
            );
            $tr.rollback().await.map_err(ServicioError::from)?;

            return Err(err);
        }
    };
}
