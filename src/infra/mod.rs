//! Gestiona la ifraestructura de la aplicación.
//! Servicios comunes

/// Módulo que contiene la loǵica de acceso a datos general
mod db;
/// Módulo para manejar tipos genéricos del dominio
mod dominio;
/// Módulo que contiene la loǵica de servicios general
mod servicio;

/// Módulo que contiene la loǵica de middlewares (Autenticación web, etc)
pub mod middleware;

//pub use app::*;
pub use db::*;
pub use dominio::*;
pub use servicio::*;

/// Macro de conveniencia para crear `MySqlArguments` con múltiples parámetros.
///
/// Este macro proporciona una forma limpia y type-safe de construir argumentos
/// para consultas MySQL sin tener que llamar manualmente a `add()` para cada
/// parámetro. Maneja automáticamente la creación del contenedor de argumentos
/// y añade todos los parámetros proporcionados en orden.
///
/// # Parámetros
/// - `$param:expr` - Expresiones que representan los valores a añadir como
///   parámetros.
///   Cada parámetro debe implementar los traits `sqlx::Encode` y
///   `sqlx::Type` para MySQL.
///   Los tipos soportados incluyen:
///   - Tipos primitivos: `i8`, `i16`, `i32`, `i64`, `u8`, `u16`, `u32`,
///     `u64`, `f32`, `f64`, `bool`
///   - Texto: `String`, `&str`, `char`
///   - Binarios: `Vec<u8>`, `&[u8]`
///   - Fechas y tiempos: `chrono::NaiveDate`,
///     `chrono::NaiveDateTime`, `chrono::DateTime<Utc>`
///   - Tipos opcionales: `Option<T>` donde `T` es un tipo soportado
///   - Colecciones: `Vec<T>`, arrays para tipos soportados
///   - JSON: `serde_json::Value`
///
/// # Consideraciones
/// - Los parámetros se añaden en el orden en que aparecen en la
///   invocación del macro
/// - La coma final es opcional para mayor flexibilidad en el formateo
/// - Todos los parámetros deben implementar `sqlx::Encode` y `sqlx::Type`
///   para MySQL
/// - El macro importa `sqlx::Arguments` y `sqlx::mysql::MySqlArguments`
///   en su ámbito
/// - Los parámetros se bindean a los placeholders `?` en la consulta SQL
///   en orden de aparición
///
/// # Ejemplos
///
/// ## Uso básico con diferentes tipos de datos
/// ```
/// # use sqlx::MySqlPool;
/// # async fn ejemplo() -> Result<(), sqlx::Error> {
/// # let pool: MySqlPool = unimplemented!();
///
/// let nombre = "Juan";
/// let edad = 30;
/// let activo = true;
/// let salario = 45000.50;
///
/// let args = mysql_params!(nombre => "nombre", edad => "edad");
///
/// // Los parámetros se bindean a ?1=nombre, ?2=edad
/// let query = sqlx::query_with(
///     "SELECT * FROM usuarios WHERE nombre = ? AND edad = ?",
///     args
/// );
/// # Ok(())
/// # }
/// ```
#[macro_export]
macro_rules! mysql_params {
    ($($param:expr => $name:literal),* $(,)?) => {{
        use sqlx::Arguments;
        let mut args = sqlx::mysql::MySqlArguments::default();
        $(
            args.add($param)
                .map_err(|_| DBError::Parametros($name))?;
        )*
        args
    }};
}
