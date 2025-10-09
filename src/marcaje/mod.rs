//! Gestiona los casos de uso para el marcaje horario por
//! parte de los empleados y la gestión de incidencias

/// Módulo que gestiona el acceso a datos para marcaje de los empleados
/// Los marcajes nunca pueden ser eliminados o modificados
/// físicamente, pero si virtualmente. Para ello,
/// existen los campos modificar_por y eliminado.
/// Solo los marcajes cuyo modificar_por y eliminado esten
/// a nulo se pueden tener en cuenta para evaluar registros
/// de cara a el empleado
mod repo;

/// Módulo que define el dominio para el marcaje horario
mod dominio;
/// Módulo con los servicios para el marcaje horario
mod servicio;

pub use dominio::*;
pub use repo::*;
pub use servicio::*;
