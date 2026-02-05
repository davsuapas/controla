use chrono::{NaiveDate, NaiveDateTime};
use smallvec::SmallVec;
use sqlx::{Row, mysql::MySqlRow};

use crate::{
  infra::{DBError, Dni, Password, PoolConexion, Transaccion},
  usuarios::{DescriptorUsuario, Rol, Usuario, UsuarioCalendario},
};

/// Implementación del repositorio de los usuarios y horarios.
pub struct UsuarioRepo {
  pool: PoolConexion,
}

impl UsuarioRepo {
  pub fn new(pool: PoolConexion) -> Self {
    UsuarioRepo { pool }
  }

  pub(in crate::usuarios) fn conexion(&self) -> &PoolConexion {
    &self.pool
  }
}

impl UsuarioRepo {
  /// Añadir roles a un usuario.
  ///
  /// Si el usuario ya tiene roles, se eliminan antes de añadir los nuevos.
  pub(in crate::usuarios) async fn agregar_roles(
    &self,
    trans: &mut Transaccion<'_>,
    usuario: u32,
    roles: &[Rol],
  ) -> Result<(), DBError> {
    const DELETE_QUERY: &str = "DELETE FROM roles_usuario
       WHERE usuario = ?;";

    sqlx::query(DELETE_QUERY)
      .bind(usuario)
      .execute(&mut **trans.deref_mut())
      .await
      .map_err(DBError::from_sqlx)?;

    const QUERY: &str = "INSERT INTO roles_usuario (usuario, rol)
       VALUES (?, ?);";

    for rol in roles {
      sqlx::query(QUERY)
        .bind(usuario)
        .bind(*rol as u32)
        .execute(&mut **trans.deref_mut())
        .await
        .map_err(DBError::from_sqlx)?;
    }

    Ok(())
  }

  /// Añadir calendarios a un usuario.
  ///
  /// Si el usuario ya tiene calendarios, se eliminan antes de añadir los nuevos.
  pub(in crate::usuarios) async fn agregar_calendarios(
    &self,
    trans: &mut Transaccion<'_>,
    usuario: u32,
    calendarios: &[u32],
  ) -> Result<(), DBError> {
    const DELETE_QUERY: &str = "DELETE FROM calendarios_usuario
       WHERE usuario = ?;";

    sqlx::query(DELETE_QUERY)
      .bind(usuario)
      .execute(&mut **trans.deref_mut())
      .await
      .map_err(DBError::from_sqlx)?;

    const QUERY: &str = "INSERT INTO calendarios_usuario (usuario, calendario)
       VALUES (?, ?);";

    for calendario in calendarios {
      sqlx::query(QUERY)
        .bind(usuario)
        .bind(calendario)
        .execute(&mut **trans.deref_mut())
        .await
        .map_err(DBError::from_sqlx)?;
    }
    Ok(())
  }

  /// Crea un nuevo usuario.
  ///
  /// El secreto es necesario para encriptar el DNI y la password.
  pub(in crate::usuarios) async fn crear_usuario(
    &self,
    trans: &mut Transaccion<'_>,
    secreto: &str,
    usuario: &Usuario,
  ) -> Result<u32, DBError> {
    const QUERY: &str = "INSERT INTO usuarios 
      (dni, dni_hash, email, nombre, primer_apellido, segundo_apellido,
      password, activo, inicio) 
      VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?);";

    let dni = usuario
      .dni
      .encriptar(secreto)
      .map_err(DBError::cripto_from)?;

    let password = usuario
      .password
      .as_ref()
      .unwrap()
      .encriptar(secreto)
      .map_err(DBError::cripto_from)?;

    let result = sqlx::query(QUERY)
      .bind(&dni)
      .bind(usuario.dni.hash_con_salt(secreto))
      .bind(&usuario.email)
      .bind(&usuario.nombre)
      .bind(&usuario.primer_apellido)
      .bind(&usuario.segundo_apellido)
      .bind(&password)
      .bind(usuario.activo)
      .bind(usuario.inicio)
      .execute(&mut **trans.deref_mut())
      .await
      .map_err(DBError::from_sqlx)?;

    Ok(result.last_insert_id() as u32)
  }

  /// Actualiza un usuario.
  ///
  /// Solo se puede actualizar el DNI, nombre, apellidos y activo.
  ///
  /// El secreto es necesario para encriptar el DNI.
  pub(in crate::usuarios) async fn actualizar_usuario(
    &self,
    trans: &mut Transaccion<'_>,
    secreto: &str,
    usuario: &Usuario,
    inicio: Option<NaiveDateTime>,
  ) -> Result<(), DBError> {
    const QUERY: &str = "UPDATE usuarios SET
      dni = ?, dni_hash = ?, email = ?, nombre = ?,
      primer_apellido = ?, segundo_apellido = ?,
      activo = ?, inicio = ?
      WHERE id = ?;";

    let dni = usuario
      .dni
      .encriptar(secreto)
      .map_err(DBError::cripto_from)?;

    let res = sqlx::query(QUERY)
      .bind(&dni)
      .bind(usuario.dni.hash_con_salt(secreto))
      .bind(&usuario.email)
      .bind(&usuario.nombre)
      .bind(&usuario.primer_apellido)
      .bind(&usuario.segundo_apellido)
      .bind(usuario.activo)
      .bind(inicio)
      .bind(usuario.id)
      .execute(&mut **trans.deref_mut())
      .await
      .map_err(DBError::from_sqlx)?;

    if res.rows_affected() == 0 {
      Err(DBError::registro_vacio("Actualizando usuario".to_string()))
    } else {
      Ok(())
    }
  }

  /// Actualiza la password.
  ///
  /// El secreto es necesario para encriptar la password.
  pub(in crate::usuarios) async fn actualizar_password(
    &self,
    trans: &mut Transaccion<'_>,
    secreto: &str,
    usuario: u32,
    password: &Password,
  ) -> Result<(), DBError> {
    const QUERY: &str = "UPDATE usuarios SET password = ? WHERE id = ?;";

    let pass = password.encriptar(secreto).map_err(DBError::cripto_from)?;

    let res = sqlx::query(QUERY)
      .bind(&pass)
      .bind(usuario)
      .execute(&mut **trans.deref_mut())
      .await
      .map_err(DBError::from_sqlx)?;

    if res.rows_affected() == 0 {
      Err(DBError::registro_vacio("Actualizando password".to_string()))
    } else {
      Ok(())
    }
  }

  /// Actualizar la sesión de inicio
  pub(in crate::usuarios) async fn actualizar_inicio(
    &self,
    trans: &mut Transaccion<'_>,
    usuario: u32,
    inicio: NaiveDateTime,
  ) -> Result<(), DBError> {
    const QUERY: &str = "UPDATE usuarios SET inicio = ? WHERE id = ?;";

    let res = sqlx::query(QUERY)
      .bind(inicio)
      .bind(usuario)
      .execute(&mut **trans.deref_mut())
      .await
      .map_err(DBError::from_sqlx)?;

    if res.rows_affected() == 0 {
      Err(DBError::registro_vacio(
        "Actualizando inicio de usuario".to_string(),
      ))
    } else {
      Ok(())
    }
  }

  /// Verifica que no exista un dni duplicado.
  pub(in crate::usuarios) async fn dni_duplicado(
    &self,
    secreto: &str,
    dni: &Dni,
  ) -> Result<bool, DBError> {
    const QUERY: &str = "SELECT CAST(COUNT(*) AS UNSIGNED) 
      FROM usuarios 
      WHERE dni_hash = ?;";

    let count: u32 = sqlx::query_scalar(QUERY)
      .bind(dni.hash_con_salt(secreto))
      .fetch_one(self.pool.conexion())
      .await
      .map_err(DBError::from_sqlx)?;

    Ok(count > 0)
  }

  /// Obtiene la password de un usuario
  ///
  /// La clave sirve para desencriptar las password
  pub(in crate::usuarios) async fn password(
    &self,
    clave: &str,
    usuario: u32,
  ) -> Result<Option<Password>, DBError> {
    const QUERY: &str = "SELECT password
        FROM usuarios
        WHERE id = ? AND activo IS NOT NULL";

    let row = sqlx::query(QUERY)
      .bind(usuario)
      .fetch_optional(self.pool.conexion())
      .await
      .map_err(DBError::from_sqlx)?;

    if let Some(r) = row {
      let p: String = r.get("password");
      Ok(Some(
        Password::from_encriptado(Some(&p), clave)
          .map_err(DBError::cripto_from)?,
      ))
    } else {
      Ok(None)
    }
  }

  /// Obtiene todos los usuarios.
  ///
  /// El secreto es necesario para desencriptar el DNI.
  pub(in crate::usuarios) async fn usuarios(
    &self,
    secreto: &str,
  ) -> Result<Vec<Usuario>, DBError> {
    const QUERY: &str = "SELECT id, dni, email,
      nombre, primer_apellido, segundo_apellido,
      activo, inicio 
      FROM usuarios;";

    let rows = sqlx::query(QUERY)
      .fetch_all(self.pool.conexion())
      .await
      .map_err(DBError::from_sqlx)?;

    let mut usuarios = Vec::with_capacity(rows.len());

    for row in rows {
      usuarios.push(self.usuario_from_row(&row, secreto).await?);
    }
    Ok(usuarios)
  }

  /// Obtiene un usuario dado el id.
  ///
  /// El secreto es necesario para desencriptar el DNI.
  pub(in crate::usuarios) async fn usuario(
    &self,
    secreto: &str,
    id: u32,
  ) -> Result<Usuario, DBError> {
    const QUERY: &str = "SELECT id, dni, email,
      nombre, primer_apellido, segundo_apellido,
      activo, inicio 
      FROM usuarios
      WHERE id = ?;";

    let row = sqlx::query(QUERY)
      .bind(id)
      .fetch_optional(self.pool.conexion())
      .await
      .map_err(DBError::from_sqlx)?;

    if let Some(row) = row {
      self.usuario_from_row(&row, secreto).await
    } else {
      Err(DBError::registro_vacio(format!(
        "No se ha encontrado ningún usuario con id: {}",
        id
      )))
    }
  }

  /// Obtiene un usuario dado el dni.
  ///
  /// El secreto es necesario para desencriptar el DNI.
  pub(in crate::usuarios) async fn usuario_por_dni(
    &self,
    secreto: &str,
    dni: &Dni,
  ) -> Result<Usuario, DBError> {
    const QUERY: &str = "SELECT id, dni, email,
      nombre, primer_apellido, segundo_apellido,
      activo, inicio 
      FROM usuarios
      WHERE dni_hash = ?;";

    let row = sqlx::query(QUERY)
      .bind(dni.hash_con_salt(secreto))
      .fetch_optional(self.pool.conexion())
      .await
      .map_err(DBError::from_sqlx)?;

    if let Some(row) = row {
      self.usuario_from_row(&row, secreto).await
    } else {
      Err(DBError::registro_vacio(format!(
        "No se ha encontrado ningún usuario con dni: {}",
        &dni
      )))
    }
  }

  /// Obtiene los usuarios que tienen un rol específico.
  pub(in crate::usuarios) async fn usuarios_por_rol(
    &self,
    rol: Rol,
  ) -> Result<Vec<DescriptorUsuario>, DBError> {
    const QUERY: &str = "SELECT u.id, u.nombre,
          u.primer_apellido, u.segundo_apellido 
          FROM usuarios u
          JOIN roles_usuario ru ON u.id = ru.usuario
          WHERE ru.rol = ?;";

    let rows = sqlx::query(QUERY)
      .bind(rol as u32)
      .fetch_all(self.pool.conexion())
      .await
      .map_err(DBError::from_sqlx)?;

    Ok(
      rows
        .into_iter()
        .map(|row| DescriptorUsuario {
          id: row.get("id"),
          nombre: row.get("nombre"),
          primer_apellido: row.get("primer_apellido"),
          segundo_apellido: row.get("segundo_apellido"),
        })
        .collect(),
    )
  }

  pub(in crate::usuarios) async fn roles_por_usuario(
    &self,
    usuario: u32,
  ) -> Result<SmallVec<[Rol; 7]>, DBError> {
    const QUERY: &str = "SELECT rol 
      FROM roles_usuario 
      WHERE usuario = ?;";

    let rows = sqlx::query_scalar::<_, u8>(QUERY)
      .bind(usuario)
      .fetch_all(self.pool.conexion())
      .await
      .map_err(DBError::from_sqlx)?;

    Ok(rows.into_iter().map(Rol::from).collect())
  }

  pub(in crate::usuarios) async fn calendarios_asignados_por_usuario(
    &self,
    usuario: u32,
  ) -> Result<Vec<UsuarioCalendario>, DBError> {
    const QUERY: &str = "SELECT c.id, c.nombre
        FROM calendarios c
        JOIN calendarios_usuario cu ON c.id = cu.calendario
        WHERE cu.usuario = ? ORDER BY c.nombre;";

    let rows = sqlx::query(QUERY)
      .bind(usuario)
      .fetch_all(self.pool.conexion())
      .await
      .map_err(DBError::from_sqlx)?;

    Ok(
      rows
        .into_iter()
        .map(|row| UsuarioCalendario {
          calendario: row.get("id"),
          nombre: row.get("nombre"),
          asignado: true,
        })
        .collect(),
    )
  }

  pub(in crate::usuarios) async fn todos_los_calendarios_con_asignacion(
    &self,
    usuario: u32,
  ) -> Result<Vec<UsuarioCalendario>, DBError> {
    const QUERY: &str = "SELECT c.id, c.nombre, 
    EXISTS(
      SELECT 1 
      FROM calendarios_usuario cu 
      WHERE cu.calendario = c.id AND cu.usuario = ?) as asignado
       FROM calendarios c ORDER BY c.nombre";

    let rows = sqlx::query(QUERY)
      .bind(usuario)
      .fetch_all(self.pool.conexion())
      .await
      .map_err(DBError::from_sqlx)?;

    Ok(
      rows
        .into_iter()
        .map(|row| UsuarioCalendario {
          calendario: row.get("id"),
          nombre: row.get("nombre"),
          asignado: row.get("asignado"),
        })
        .collect(),
    )
  }

  /// Obtiene el número de marcajes horarios de un usuario
  pub(in crate::usuarios) async fn num_marcajes_horarios_usuario(
    &self,
    id: u32,
  ) -> Result<u32, DBError> {
    const QUERY: &str = "SELECT CAST(COUNT(id) AS UNSIGNED) 
        FROM marcajes
        WHERE usuario = ?";

    Ok(
      sqlx::query_scalar(QUERY)
        .bind(id)
        .fetch_one(self.pool.conexion())
        .await
        .map_err(DBError::from_sqlx),
    )?
  }

  /// Obtiene las fechas de los marcajes de un usuario
  /// que entran en conflicto con las fechas de un calendario.
  pub(in crate::usuarios) async fn marcajes_conflictivos_asignacion_calendario(
    &self,
    usuario: u32,
    calendario: u32,
  ) -> Result<Vec<NaiveDate>, DBError> {
    const QUERY: &str = "SELECT DISTINCT m.fecha
        FROM marcajes m
        JOIN calendario_fechas cf
         ON m.fecha BETWEEN cf.fecha_inicio AND cf.fecha_fin
        WHERE m.usuario = ?
          AND cf.calendario = ?
          AND modificado_por IS NULL AND eliminado IS NULL
        ORDER BY m.fecha";

    sqlx::query_scalar(QUERY)
      .bind(usuario)
      .bind(calendario)
      .fetch_all(self.pool.conexion())
      .await
      .map_err(DBError::from_sqlx)
  }

  async fn usuario_from_row(
    &self,
    row: &MySqlRow,
    secreto: &str,
  ) -> Result<Usuario, DBError> {
    let dni = Dni::from_encriptado(row.get("dni"), secreto)
      .map_err(DBError::cripto_from)?;
    let id: u32 = row.get("id");
    let roles = self.roles_por_usuario(id).await?;

    Ok(Usuario {
      id,
      dni,
      email: row.get("email"),
      nombre: row.get("nombre"),
      primer_apellido: row.get("primer_apellido"),
      segundo_apellido: row.get("segundo_apellido"),
      password: None,
      activo: row.get("activo"),
      inicio: row.get("inicio"),
      roles,
      calendarios: vec![],
    })
  }
}
