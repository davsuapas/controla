//! Gestiona los casos de uso para el marcaje horario por
//! parte de los empleados, registradores y supervisores.
//!
//! Los empleados y registradores pueden realizar marcajes
//! de forma manual (solo registradores) o automáticamente
//! (empleados y registradores).
//!
//! Los marcajes manuales permiten elegir el usuario contra
//! el que se va realizar el marcaje. Los automáticos la
//! entrada y salida se marca según la fecha y hora en ese
//! momento temporal.
//!
//! Los marcajes nunca pueden ser eliminados o modificados
//! físicamente, pero si virtualmente. Para ello,
//! existen los campos modificar_por y eliminado.
//! Solo los marcajes cuyo modificar_por y eliminado esten
//! a nulo se pueden tener en cuenta para evaluar registros
//! de cara a el empleado.
//!
//! Si el marcaje se realiza como rol registrador, el marcaje
//! se anota, no solo para el usuario para el que se realiza
//! el marcaje, si no para el registrador.
//!
//! El módulo permite la consulta de los registros marcados con
//! las siguientes restricciones:
//!
//! Si la fechas de bisuqeda son Nones se filtra solos por usuario.
//! Si el usuario_reg es igual a 0, significa que es supervisor
//! y puede ver todos los marcajes de caulquier usuario.
//! Si el usuario es diferente el usuario_reg, significa
//! que es usuario registrador y por tanto puede ver solo
//! los marcajes que registro el.
//! Si son iguales el usuario es empleado y solo puede ver
//! sus marcajes.
mod repo;

/// Módulo que define el dominio para el marcaje horario
mod dominio;
/// Módulo con los servicios para el marcaje horario
mod servicio;

pub use dominio::*;
pub use repo::*;
pub use servicio::*;
