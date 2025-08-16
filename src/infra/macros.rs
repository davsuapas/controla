/// Macro para generar un getter para el pool de conexiones en un repo.
///
/// # Ejemplo
/// ```
/// #[db_pool_getter]
/// struct MiRepo { pool: PoolConexion }
/// let repo = MiRepo { pool: obtener_pool() };
/// let _pool = repo.pool(); // Getter automático
/// ```
#[macro_export]
macro_rules! db_pool_getter {
  ($struct:ident) => {
    impl $struct {
      #[doc = "Obtiene una referencia al pool de conexiones"]
      #[inline]
      pub fn conexion(&self) -> &PoolConexion {
        &self.pool
      }
    }
  };
}
