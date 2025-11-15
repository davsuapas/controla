//! Gestiona las incidencas producidas en los marcajes
//!
//! Cualquier modificación en el marcaje solo puede realizarse
//! a través de una incidencia y siempre mantiene todos
//! los marcajes originales, para no perder la trazabilidad.
//! Los cambios de estados quedan registrados a través
//! del módulo de trazas.
//!
//! Existen tres tipos de incidencias:
//! - Nuevo marcaje: Se solicita la creación de nuevo marcaje.
//! - Correción de la hora de salida: Teniendo en cuenta,
//!   que los marcajes son realizados a través de un botón
//!   y no es posible que el usuario manipule la hora de entrada,
//!   la única que puede haber, es porque el usuario se olvide
//!   marcar la hora de salida. Por tanto, solo es posible
//!   modificar la hora de salida.
//! - Eliminación del marcaje: Solicita eliminar un marcaje,
//!   pero solo es posible esta acción, por el rol registrador
//!   y el rol supervisor. El registrador solo puede solicitar
//!   la eliminación de marcajes realidos por él.
//!
//! Flujo de las incidencias:
//!  - El emmpleado, registrador o supervidor realiza una
//!    solicitud indicando el tipo de incidencia. Si el
//!    usuario es registrador puede eliminar marcajes
//!    que creo el. El supervidor puede actuar en nombre
//!    del registrador por si está de baja. El registrador
//!    solo puede ver los marcajes que ha hecho el mismo. 
//!    El supervisor puede ver todos los marcajes de todos
//!    los registradores, y el empleado solo puede ver los
//!    suyos, incluso si fueron hechos por registradores.
//!    Estas son todas aquellas cuyo campo usuario del marcaje
//!    difiere del campo usuario referencia (usuario registrador).
//!  - Los gestores de incidencias aceptan las
//!    solicitudes o las rechazan motivándolas. Cuando se acepta
//!    una solicitud, se crea un nuevo marcaje si el tipo
//!    es nuevo, se crea un marcaje asociado a el origen, en caso
//!    de ser una modificación, o se marca como eliminado.
//!    Si se producen errores de validación, la incidencia
//!    quedará en estado incosistente y el usuario podrá modficarla,
//!    para que posteriormente vuelva a ser aceptada. En caso
//!    de error interno, la incidencia quedará en un estado
//!    erróneo, para que el gestor o supervisor puedan volver
//!    a procesarla.
//!  - El usuario puede ver en todo momento el estado de
//!    su solicitud y actuar en función de su estado.
//!    El empleado y registrador solo puede ver sus solicitudes. 
//!    El supervisor puede ver todas las hechas por el registrador.
//!    Estas son todas aquellas que el campos usuario del marcaje
//!    difiere del campo creador de la incidenca
//!
//! Diagrama de estados:
//!   Solicitud -> Resolver, Rechazar
//!   Resolver -> Conflicto, ErrorResolver, Resuelta
//!   Conflicto -> Solicitud
//!   ErrorResolver -> Resolver
//!   Rechazar -> Rechazada
//!   Rechazada -> Solicitud

/// Módulo que gestiona toda las iteraciones sobre incidencias
/// con la base de datos
mod repo;

/// Módulo para manejar tipos genéricos del dominio
mod dominio;
/// Módulo que expone los servicios del usuario
mod servicio;

pub use dominio::*;
pub use repo::*;
pub use servicio::*;
