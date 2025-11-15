//! Gestiona los casos de uso para el marcaje horario por
//! parte de los empleados y los registradores.

/// Módulo que gestiona el acceso a datos para marcaje de los empleados
/// Los marcajes nunca pueden ser eliminados o modificados
/// físicamente, pero si virtualmente. Para ello,
/// existen los campos modificar_por y eliminado.
/// Solo los marcajes cuyo modificar_por y eliminado esten
/// a nulo se pueden tener en cuenta para evaluar registros
/// de cara a el empleado
///
/// Si el marcaje se realiza como rol registrador, el marcaje
/// se anota, no solo para el usuario para el que se realiza
/// el marcaje, si no para el registrador.
mod repo;

/// Módulo que define el dominio para el marcaje horario
mod dominio;
/// Módulo con los servicios para el marcaje horario
mod servicio;

pub use dominio::*;
pub use repo::*;
pub use servicio::*;
