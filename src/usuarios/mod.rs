//! Gestiona toda la funcionalidad asociada a los usuarios
//! incluyendo la autenticación, autorización, gestión de horarios
//! y calendarios asociados a los empleados.
//!
//! Los perfiles de usuarios podrán ser:
//! - Administrador
//! - Empleado
//! - Gestor de incidencias
//! - Registrador
//! - Inspector
//! - Director
//! - Supervisor
//!
//! La definición de cada rol se encuentra en: [`dominio::Rol`]
//!
//! Los horarios de los empleados se gestionan por grupos. Cada nueva
//! configuración o grupo, se relacionan en base a la fecha de creación
//! en un instante en el tiempo, y todos estos horarios se agrupan
//! en torno a esta fecha de creación. Si se necesitará crear un
//! nuevo horario en base a otra fecha, sería necesario crear otra
//! nueva configuración (grupo) y si hay horarios que se heredan
//! se deberán a volver a incluir en este grupo.
//!
//! Los horarios para un grupo no pueden estar solapados para el
//! mismo día.
//!
//! El efecto de los horarios para un marcaje se aplica desde la
//! fecha de creación del grupo.
//!
//! Los horarios a parte de ser asignados a un usuario para una
//! determinada fecha, permiten caducidad. Esto es útil cuando un
//! empleado debe realizar un horario extra a su horario normal.
//! El administrador podrá asignar estas caducidad y tendrá vigencia
//! durante las fechas de inicio y fin de caducidad.

/// Módulo que gestiona el acceso a datos para los usuarios
mod repo;

/// Módulo que define el dominio de usuarios, roles y horarios
/// de los usuarios
mod dominio;
/// Módulo que expone los servicios del usuario
mod servicio;

pub use dominio::*;
pub use repo::*;
pub use servicio::*;
