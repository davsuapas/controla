//! Gestiona los horarios de los empleados.
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

/// Módulo para manejar los dominios sobre los horarios.
mod dominio;
/// Módulo que gestiona toda las iteraciones sobre el horario
/// con la base de datos
mod repo;
/// Módulo que expone los servicios del horario
mod servicio;

pub use dominio::*;
pub use repo::*;
pub use servicio::*;
