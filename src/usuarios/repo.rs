use smallvec::SmallVec;
use sqlx::{Row, mysql::MySqlRow};

use chrono::{Datelike, NaiveDate, NaiveDateTime};

use crate::{
  infra::{
    DBError, Dni, Password, PoolConexion, ShortDateTimeFormat, Transaccion,
  },
  usuarios::{DescriptorUsuario, Dia, Horario, Rol, Usuario},
};

/// Implementación del repositorio de los usuarios y horarios.
pub struct UsuarioRepo {
  pool: PoolConexion,
}

impl UsuarioRepo {
  pub fn new(pool: PoolConexion) -> Self {
    UsuarioRepo { pool }
  }
}

impl UsuarioRepo {
  pub(in crate::usuarios) fn conexion(&self) -> &PoolConexion {
    &self.pool
  }

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

  /// Crea un nuevo usuario.
  ///
  /// El secreto es necesario para encriptar el DNI y la password.
  pub(in crate::usuarios) async fn crear_usuario(
    &self,
    trans: &mut Transaccion<'_>,
    secreto: &str,
    usuario: &Usuario,
  ) -> Result<u32, DBError> {
    const QUERY: &str = r"INSERT INTO usuarios 
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
    const QUERY: &str = r"UPDATE usuarios SET
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
    const QUERY: &str = r"UPDATE usuarios SET password = ? WHERE id = ?;";

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
    const QUERY: &str = r"UPDATE usuarios SET inicio = ? WHERE id = ?;";

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
    const QUERY: &str = r"SELECT CAST(COUNT(*) AS UNSIGNED) 
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
    const QUERY: &str = r"SELECT password
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
    const QUERY: &str = r"SELECT id, dni, email,
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
    const QUERY: &str = r"SELECT id, dni, email,
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
    const QUERY: &str = r"SELECT id, dni, email,
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
    const QUERY: &str = r"SELECT u.id, u.nombre,
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
    const QUERY: &str = r"SELECT rol 
      FROM roles_usuario 
      WHERE usuario = ?;";

    let rows = sqlx::query_scalar::<_, u8>(QUERY)
      .bind(usuario)
      .fetch_all(self.pool.conexion())
      .await
      .map_err(DBError::from_sqlx)?;

    Ok(rows.into_iter().map(Rol::from).collect())
  }

  /// Obtiene el horario más cercano a una hora dada para un usuario.
  ///
  /// Busca un horario que este entre las horas de inicio y fin
  /// del día de la semana y que no esté ya asignado a un marcaje horario.
  /// Si no encuentra un horario entre las horas de inicio y fin,
  /// devuelve el más cercano al inicio y que no esté ya asignado
  /// a un marcaje horario.
  /// Además, la hora que se busca, tiene que ser mayor de la hora
  /// final del marcaje del horario anterior asignado.
  pub(in crate::usuarios) async fn horario_cercano(
    &self,
    usuario: u32,
    hora: NaiveDateTime,
  ) -> Result<Horario, DBError> {
    let fecha_creacion = self.fecha_creacion_horario(usuario, hora).await?;
    let dia = crate::infra::letra_dia_semana(hora.weekday());

    // Busca un horario que esté entre las horas de inicio y fin
    // del día de la semana.
    const QUERY: &str = r"SELECT h.id, h.dia, h.hora_inicio, h.hora_fin
        FROM horarios h
         JOIN usuario_horarios uh ON h.id = uh.horario
        WHERE uh.usuario = ?
         AND uh.fecha_creacion = ?
         AND h.dia = ?
         AND ? BETWEEN h.hora_inicio AND h.hora_fin
         AND NOT EXISTS 
         (SELECT r.id
            FROM marcajes r
            WHERE r.usuario = uh.usuario AND r.fecha = ?
             AND r.horario = h.id
             AND modificado_por IS NULL AND eliminado IS NULL)
         AND ? > COALESCE(
         (SELECT MAX(r2.hora_fin)
            FROM marcajes r2
            JOIN horarios h2 ON r2.horario = h2.id
            WHERE r2.usuario = uh.usuario AND r2.fecha = ?
              AND h2.hora_inicio < h.hora_inicio
              AND modificado_por IS NULL AND eliminado IS NULL),
        '00:00:00')";

    let fecha = hora.date();
    let hora_buscar = hora.time();

    let result = sqlx::query(QUERY)
      .bind(usuario)
      .bind(fecha_creacion)
      .bind(dia)
      .bind(hora_buscar)
      .bind(fecha)
      .bind(hora_buscar)
      .bind(fecha)
      .fetch_optional(self.pool.conexion())
      .await
      .map_err(DBError::from_sqlx)?;

    if let Some(row) = result {
      Ok(UsuarioRepo::horario_from_row(&row))
    } else {
      // Si no encuentra un horario entre las horas de inicio y fin,
      // devuelve el más cercano al inicio.
      const QUERY: &str = r"SELECT h.id, h.dia, h.hora_inicio, h.hora_fin
            FROM horarios h
            JOIN usuario_horarios uh ON h.id = uh.horario
            WHERE uh.usuario = ?
             AND uh.fecha_creacion = ?
             AND h.dia = ?
             AND h.hora_inicio > ?
             AND NOT EXISTS 
             ( SELECT r.id
                 FROM marcajes r
                 WHERE r.usuario = uh.usuario AND r.fecha = ?
                  AND r.horario = h.id
                  AND modificado_por IS NULL AND eliminado IS NULL)
            AND ? > COALESCE(
            (SELECT MAX(r2.hora_fin)
                FROM marcajes r2
                JOIN horarios h2 ON r2.horario = h2.id
                WHERE r2.usuario = uh.usuario
                  AND r2.fecha = ?
                  AND h2.hora_inicio < h.hora_inicio
                  AND modificado_por IS NULL AND eliminado IS NULL),
            00:00:00')
            LIMIT 1;";

      let result = sqlx::query(QUERY)
        .bind(usuario)
        .bind(fecha_creacion)
        .bind(dia)
        .bind(hora_buscar)
        .bind(fecha)
        .bind(hora_buscar)
        .bind(fecha)
        .fetch_optional(self.pool.conexion())
        .await
        .map_err(DBError::from_sqlx)?;

      if let Some(row) = result {
        Ok(UsuarioRepo::horario_from_row(&row))
      } else {
        Err(DBError::registro_vacio(format!(
          "No se ha encontrado ningún horario próximo a la fecha: {}, \
           que no este ya asignado. \
           Verifique sus horarios creados en la fecha: {}",
          hora,
          fecha_creacion.formato_corto()
        )))
      }
    }
  }

  /// Obtiene el horario asignado al usuario para la fecha dada.
  ///
  /// Si ya se encuentra asignado se omite
  pub(in crate::usuarios) async fn horarios_usuario_sin_asignar(
    &self,
    usuario: u32,
    hora: NaiveDateTime,
  ) -> Result<Vec<Horario>, DBError> {
    let fecha_creacion = self.fecha_creacion_horario(usuario, hora).await?;
    let dia = crate::infra::letra_dia_semana(hora.weekday());

    const QUERY: &str = r"SELECT h.id, h.dia, h.hora_inicio, h.hora_fin
        FROM horarios h
         JOIN usuario_horarios uh ON h.id = uh.horario
        WHERE uh.usuario = ?
         AND uh.fecha_creacion = ?
         AND h.dia = ?
         AND NOT EXISTS 
         ( SELECT r.id
            FROM marcajes r
            WHERE r.usuario = uh.usuario AND r.fecha = ?
             AND r.horario = h.id
             AND modificado_por IS NULL AND eliminado IS NULL)
        ORDER BY h.hora_inicio;";

    let rows = sqlx::query(QUERY)
      .bind(usuario)
      .bind(fecha_creacion)
      .bind(dia)
      .bind(hora.date())
      .fetch_all(self.pool.conexion())
      .await
      .map_err(DBError::from_sqlx)?;

    Ok(
      rows
        .into_iter()
        .map(|row| UsuarioRepo::horario_from_row(&row))
        .collect(),
    )
  }

  /// Obtiene el número de marcajes horarios de un usuario
  pub(in crate::usuarios) async fn num_marcajes_horarios_usuario(
    &self,
    id: u32,
  ) -> Result<u32, DBError> {
    const QUERY: &str = r"SELECT CAST(COUNT(id) AS UNSIGNED) 
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

  /// Obtiene la fecha de creación del horario más reciente
  async fn fecha_creacion_horario(
    &self,
    usuario: u32,
    hora: NaiveDateTime,
  ) -> Result<NaiveDate, DBError> {
    const QUERY: &str = r"SELECT MAX(fecha_creacion) 
    FROM usuario_horarios 
    WHERE usuario = ? 
    AND fecha_creacion < ?";

    let fecha_creacion = sqlx::query_scalar::<_, Option<NaiveDate>>(QUERY)
      .bind(usuario)
      .bind(hora)
      .fetch_one(self.pool.conexion())
      .await
      .map_err(DBError::from_sqlx)?
      .ok_or_else(|| {
        DBError::registro_vacio(format!(
          "No se ha encontrado ningún horario configurado \
        para el usuario en la fecha: {}",
          hora.formato_corto()
        ))
      })?;

    Ok(fecha_creacion)
  }

  fn horario_from_row(row: &MySqlRow) -> Horario {
    Horario {
      id: row.get("id"),
      dia: Dia::from(row.get::<String, _>("dia").as_str()),
      hora_inicio: row.get("hora_inicio"),
      hora_fin: row.get("hora_fin"),
    }
  }

  async fn usuario_from_row(
    &self,
    row: &MySqlRow,
    secreto: &str,
  ) -> Result<Usuario, DBError> {
    let dni = Dni::from_encriptado(row.get("dni"), secreto)
      .map_err(DBError::cripto_from)?;
    let roles = self.roles_por_usuario(row.get("id")).await?;

    Ok(Usuario {
      id: row.get("id"),
      dni,
      email: row.get("email"),
      nombre: row.get("nombre"),
      primer_apellido: row.get("primer_apellido"),
      segundo_apellido: row.get("segundo_apellido"),
      password: None,
      activo: row.get("activo"),
      inicio: row.get("inicio"),
      roles,
    })
  }
}
