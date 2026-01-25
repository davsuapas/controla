use std::ops::Add;

use chrono::{Datelike, NaiveDate, NaiveDateTime};
use smallvec::SmallVec;
use sqlx::{Row, mysql::MySqlRow};

use crate::{
  infra::{
    DBError, DateOptional, Dni, NONE_DATE, Password, PoolConexion,
    ShortDateTimeFormat, Transaccion,
  },
  usuarios::{ConfigHorario, DescriptorUsuario, Dia, Horario, Rol, Usuario},
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

  /// Obtiene el horario más cercano a una hora dada para un usuario.
  ///
  /// Busca un horario que este entre las horas de inicio y fin
  /// del día de la semana y que no esté ya asignado a un marcaje horario.
  /// El horario puede tener caducidad. Si no esta caducado se selecciona.
  /// Si no encuentra un horario entre las horas de inicio y fin,
  /// devuelve el más cercano al inicio y que no esté ya asignado
  /// a un marcaje horario.
  /// Además, la hora que se busca, tiene que ser mayor de la hora
  /// final del marcaje del horario anterior asignado.
  ///
  /// Se puede excluir un marcaje pasado como parámetro
  /// Si no quiere excluir ningún marcaje use 0.
  /// La exclusión puede ser muy útil cuando se quiere
  /// realizar una modificación de este marcaje.
  ///
  /// Devuelve el identificador de usuario, horario y el horario.
  pub(in crate::usuarios) async fn horario_cercano(
    &self,
    usuario: u32,
    hora: NaiveDateTime,
    excluir_marcaje_id: u32,
  ) -> Result<(u32, Horario), DBError> {
    let fecha = hora.date();
    let dia = crate::infra::letra_dia_semana(hora.weekday());
    let hora_buscar = hora.time();

    let fecha_creacion = self.fecha_creacion_horario(usuario, fecha).await?;

    tracing::debug!(
      usuario = usuario,
      fecha = %fecha,
      hora = %hora_buscar,
      fecha_creacion = %fecha_creacion,
      dia = %dia,
      excluir_marcaje_id = excluir_marcaje_id,
      "Buscando el horario más cercano del usuario"
    );

    // Busca un horario que esté entre las horas de inicio y fin
    // del día de la semana siempre que no este asignado
    // y la hora inicio debe ser mayor que la hora de fin del
    // último marcaje previo.
    // También se comprueba los horarios con caducidad. Si esta
    // dentro del marcaje se selecciona.
    const QUERY: &str = "SELECT uh.id AS uh_id,
         h.id, h.dia, h.hora_inicio, h.hora_fin
        FROM usuario_horarios uh
         JOIN horarios h ON h.id = uh.horario
        WHERE uh.usuario = ?
         AND uh.fecha_creacion = ?
         AND (uh.caducidad_fecha_fin IS NULL OR ? >= uh.caducidad_fecha_ini)
         AND (uh.caducidad_fecha_fin IS NULL OR ? <= uh.caducidad_fecha_fin)         
         AND h.dia = ?
         AND ? BETWEEN h.hora_inicio AND h.hora_fin
         AND NOT EXISTS 
         (SELECT r.id
            FROM marcajes r
            WHERE r.usuario = uh.usuario AND r.fecha = ?
             AND r.id <> ? AND modificado_por IS NULL AND eliminado IS NULL
             AND r.usuario_horario = uh.id)
         AND ? > COALESCE(
         (SELECT MAX(r2.hora_fin)
            FROM marcajes r2
            JOIN usuario_horarios uh2 ON uh2.id = r2.usuario_horario
            JOIN horarios h2 ON h2.id = uh2.horario
            WHERE r2.usuario = uh.usuario AND r2.fecha = ?
              AND r2.id <> ? AND modificado_por IS NULL AND eliminado IS NULL
              AND h2.hora_inicio < h.hora_inicio),
        '00:00:00')";

    let result = sqlx::query(QUERY)
      .bind(usuario)
      .bind(fecha_creacion)
      .bind(fecha)
      .bind(fecha)
      .bind(dia)
      .bind(hora_buscar)
      .bind(fecha)
      .bind(excluir_marcaje_id)
      .bind(hora_buscar)
      .bind(fecha)
      .bind(excluir_marcaje_id)
      .fetch_optional(self.pool.conexion())
      .await
      .map_err(DBError::from_sqlx)?;

    if let Some(row) = result {
      Ok((row.get("uh_id"), UsuarioRepo::horario_from_row(&row)))
    } else {
      // Si no encuentra un horario mayor a la hora de inicio,
      // devuelve el más cercano al inicio siempre que no este asignado
      // y la hora inicio debe ser mayor que la hora de fin del
      // último marcaje previo.
      // También se comprueba los horarios con caducidad. Si esta
      // dentro del marcaje se selecciona.
      const QUERY: &str = "SELECT uh.id AS uh_id,
              h.id, h.dia, h.hora_inicio, h.hora_fin
            FROM horarios h
            JOIN usuario_horarios uh ON h.id = uh.horario
            WHERE uh.usuario = ?
             AND uh.fecha_creacion = ?
             AND (uh.caducidad_fecha_fin IS NULL OR ? >= uh.caducidad_fecha_ini)
             AND (uh.caducidad_fecha_fin IS NULL OR ? <= uh.caducidad_fecha_fin)         
             AND h.dia = ?
             AND h.hora_inicio > ?
             AND NOT EXISTS 
             ( SELECT r.id
                FROM marcajes r
                WHERE r.usuario = uh.usuario AND r.fecha = ?
                 AND r.id <> ? AND modificado_por IS NULL AND eliminado IS NULL
                 AND r.usuario_horario = uh.id)
            AND ? > COALESCE(
            (SELECT MAX(r2.hora_fin)
              FROM marcajes r2
                JOIN usuario_horarios uh2 ON uh2.id = r2.usuario_horario
                JOIN horarios h2 ON h2.id = uh2.horario
              WHERE r2.usuario = uh.usuario
               AND r2.fecha = ?
               AND r2.id <> ? AND modificado_por IS NULL AND eliminado IS NULL
               AND h2.hora_inicio < h.hora_inicio),
            '00:00:00')
            LIMIT 1;";

      let result = sqlx::query(QUERY)
        .bind(usuario)
        .bind(fecha_creacion)
        .bind(fecha)
        .bind(fecha)
        .bind(dia)
        .bind(hora_buscar)
        .bind(fecha)
        .bind(excluir_marcaje_id)
        .bind(hora_buscar)
        .bind(fecha)
        .bind(excluir_marcaje_id)
        .fetch_optional(self.pool.conexion())
        .await
        .map_err(DBError::from_sqlx)?;

      if let Some(row) = result {
        Ok((row.get("uh_id"), UsuarioRepo::horario_from_row(&row)))
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

  /// Obtiene los horarios sin asignar para un usuario en una fecha dada.
  ///
  /// También tiene en cuenta las caducidades de los horarios.
  /// Si se encuentra asignado se omite
  pub(in crate::usuarios) async fn horarios_usuario_sin_asignar(
    &self,
    usuario: u32,
    fecha: NaiveDate,
  ) -> Result<Vec<Horario>, DBError> {
    let fecha_creacion = self.fecha_creacion_horario(usuario, fecha).await?;
    let dia = crate::infra::letra_dia_semana(fecha.weekday());

    const QUERY: &str = "SELECT h.id, h.dia, h.hora_inicio, h.hora_fin
        FROM usuario_horarios uh
         JOIN horarios h ON h.id = uh.horario
        WHERE uh.usuario = ?
         AND uh.fecha_creacion = ?
         AND (uh.caducidad_fecha_fin IS NULL OR ? >= uh.caducidad_fecha_ini)
         AND (uh.caducidad_fecha_fin IS NULL OR ? <= uh.caducidad_fecha_fin)         
         AND h.dia = ?
         AND NOT EXISTS 
         ( SELECT r.id
            FROM marcajes r
            WHERE r.usuario = uh.usuario AND r.fecha = ?
             AND r.usuario_horario = uh.id
             AND modificado_por IS NULL AND eliminado IS NULL)
        ORDER BY h.hora_inicio;";

    let rows = sqlx::query(QUERY)
      .bind(usuario)
      .bind(fecha_creacion)
      .bind(fecha)
      .bind(fecha)
      .bind(dia)
      .bind(fecha)
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

  /// Crea una nueva configuración de horario para un usuario.
  pub(in crate::usuarios) async fn agregar_config_usuario(
    &self,
    trans: &mut Transaccion<'_>,
    config: &ConfigHorario,
    id_horario: u32,
  ) -> Result<u32, DBError> {
    const QUERY: &str = "INSERT INTO usuario_horarios
      (usuario, horario, fecha_creacion, 
      caducidad_fecha_ini, caducidad_fecha_fin)
      VALUES (?, ?, ?, ?, ?);";

    let cad_fecha_ini = config.caducidad_fecha_ini.convert_to_date();

    let res = sqlx::query(QUERY)
      .bind(config.usuario)
      .bind(id_horario)
      .bind(config.fecha_creacion)
      .bind(cad_fecha_ini)
      .bind(config.caducidad_fecha_fin)
      .execute(&mut **trans.deref_mut())
      .await
      .map_err(DBError::from_sqlx)?;

    Ok(res.last_insert_id() as u32)
  }

  /// Modifica una configuración de horario para un usuario.
  pub(in crate::usuarios) async fn modificar_config_usuario(
    &self,
    trans: &mut Transaccion<'_>,
    config: &ConfigHorario,
    id_horario: u32,
  ) -> Result<(), DBError> {
    const QUERY: &str = "UPDATE usuario_horarios SET
        horario = ?, caducidad_fecha_ini = ?, caducidad_fecha_fin = ?
        WHERE id = ?;";

    let cad_fecha_ini = config.caducidad_fecha_ini.convert_to_date();

    let res = sqlx::query(QUERY)
      .bind(id_horario)
      .bind(cad_fecha_ini)
      .bind(config.caducidad_fecha_fin)
      .bind(config.id)
      .execute(&mut **trans.deref_mut())
      .await
      .map_err(DBError::from_sqlx)?;

    if res.rows_affected() == 0 {
      Err(DBError::registro_vacio(
        "Modificando configuración de horario".to_string(),
      ))
    } else {
      Ok(())
    }
  }

  /// Elimina una configuración de horario para un usuario.
  pub(in crate::usuarios) async fn eliminar_config_usuario(
    &self,
    trans: &mut Transaccion<'_>,
    id: u32,
  ) -> Result<(), DBError> {
    const QUERY: &str = "DELETE FROM usuario_horarios WHERE id = ?;";

    let res = sqlx::query(QUERY)
      .bind(id)
      .execute(&mut **trans.deref_mut())
      .await
      .map_err(DBError::from_sqlx)?;

    if res.rows_affected() == 0 {
      Err(DBError::registro_vacio(
        "Eliminando configuración de horario".to_string(),
      ))
    } else {
      Ok(())
    }
  }

  /// Duplica la configuración de un horario.
  ///
  /// Dado el id de usuario y una fecha de creación como
  /// parámetro, se obtiene la última fecha de creación
  /// del empleado y se crea un nuevo grupo de horarios
  /// para el usuario dado y la fecha de creación dadas
  /// como parámetros y el grupo de horarios obtenidos de
  /// la última fecha de creación y usuario existentes en la
  /// base de datos.
  ///
  /// Los registros con caducidad no se duplican porque
  /// están sujetos a un rango que ya no se encontrará
  /// vigente
  ///
  /// Se verifica mediante un índice único si se intenta
  /// duplicar para la misma fecha de creación.
  pub(in crate::usuarios) async fn duplicar_config_horario(
    &self,
    usuario: u32,
    nueva_fecha_creacion: NaiveDate,
  ) -> Result<(), DBError> {
    // Se añade un día para que tenga en cuenta el día
    // de hoy. Ver [`Self::fecha_creacion_horario`]
    let fecha_creacion = self
      .fecha_creacion_horario(
        usuario,
        nueva_fecha_creacion.add(chrono::Duration::days(1)),
      )
      .await?;

    const QUERY: &str = "INSERT INTO usuario_horarios
      (usuario, horario, fecha_creacion,
       caducidad_fecha_ini, caducidad_fecha_fin)
      SELECT usuario, horario, ?, caducidad_fecha_ini, caducidad_fecha_fin
      FROM usuario_horarios
      WHERE usuario = ? AND fecha_creacion = ?
      AND caducidad_fecha_fin IS NULL;";

    sqlx::query(QUERY)
      .bind(nueva_fecha_creacion)
      .bind(usuario)
      .bind(fecha_creacion)
      .execute(self.pool.conexion())
      .await
      .map_err(DBError::from_sqlx)?;

    Ok(())
  }

  /// Obtiene un horario configurado dado el id.
  pub(in crate::usuarios) async fn config_horario_por_id(
    &self,
    id: u32,
  ) -> Result<ConfigHorario, DBError> {
    const QUERY: &str = "SELECT h.id, uh.id AS uh_id,
        uh.usuario, uh.fecha_creacion,
        uh.caducidad_fecha_ini, uh.caducidad_fecha_fin,
        h.dia, h.hora_inicio, h.hora_fin
      FROM usuario_horarios uh
      JOIN horarios h ON uh.horario = h.id
      WHERE uh.id = ?";

    let row = sqlx::query(QUERY)
      .bind(id)
      .fetch_optional(self.pool.conexion())
      .await
      .map_err(DBError::from_sqlx)?;

    if let Some(row) = row {
      Ok(UsuarioRepo::config_horario_from_row(&row))
    } else {
      Err(DBError::registro_vacio(format!(
        "No se ha encontrado ningún horario configurado con id: {}",
        id
      )))
    }
  }

  /// Obtiene una lista de horarios configurados para un usuario
  ///
  /// Primero obtiene la última fecha de configuración
  /// desde [`Self::fecha_creacion_horario`] y a continuación
  /// recupera de la base de datos todos los horarios para el
  /// empleado pasado como parámetro y la última fecha de configuración
  pub(in crate::usuarios) async fn config_horario(
    &self,
    usuario: u32,
    fecha_actual: NaiveDate,
  ) -> Result<Vec<ConfigHorario>, DBError> {
    // Se añade un día para que tenga en cuenta el día
    // de hoy. Ver [`Self::fecha_creacion_horario`]
    let fecha_creacion = match self
      .fecha_creacion_horario(
        usuario,
        fecha_actual.add(chrono::Duration::days(1)),
      )
      .await
    {
      Ok(fecha) => fecha,
      Err(DBError::RegistroVacio(_)) => return Ok(vec![]),
      Err(e) => return Err(e),
    };

    const QUERY: &str = "SELECT h.id, uh.id AS uh_id,
        uh.usuario, uh.fecha_creacion,
        uh.caducidad_fecha_ini, uh.caducidad_fecha_fin,
        h.dia, h.hora_inicio, h.hora_fin
      FROM usuario_horarios uh
      JOIN horarios h ON uh.horario = h.id
      WHERE uh.usuario = ? AND uh.fecha_creacion = ?
      ORDER BY h.dia, h.hora_inicio;";

    let rows = sqlx::query(QUERY)
      .bind(usuario)
      .bind(fecha_creacion)
      .fetch_all(self.pool.conexion())
      .await
      .map_err(DBError::from_sqlx)?;

    rows
      .iter()
      .map(|row| Ok(UsuarioRepo::config_horario_from_row(row)))
      .collect()
  }

  /// Verifica que una configuración no se solape con otras para el mismo día.
  ///
  /// El solapamiento se verifica para el mismo usuario y fecha creación.
  /// Se siguen las siguientes reglas para la configuración que se envía
  /// como parámetro:
  ///
  /// - Para la verificación se excluye la configuración horaria
  ///   que se envía como parámetro (referenciada por el id).
  /// - No puede haber otro horario en la bd entre la hora de inicio y fin.
  /// - Para las configuraciones con fechas de caducidad:
  ///   - Se pueden solapar con otras con caducidad si no se solapan las
  ///     fechas de caducidad.
  ///   - No se pueden solapar con otras que no tienen caducidad.
  ///
  /// En definitiva, excepto para las fecha con caducida no puede haber
  /// entre una hora de inicio y otra fin ninguna otra hora.
  ///
  /// Devuelve true si existe solapamiento.
  pub(in crate::usuarios) async fn config_horario_solape(
    &self,
    config_horario: &ConfigHorario,
  ) -> Result<bool, DBError> {
    const QUERY: &str = "SELECT CAST(COUNT(*) AS UNSIGNED) 
      FROM usuario_horarios uh
      JOIN horarios h ON uh.horario = h.id
      WHERE uh.usuario = ?
      AND uh.fecha_creacion = ?
      AND uh.id <> ?
      AND h.dia = ?
      AND h.hora_inicio < ?
      AND h.hora_fin > ?
      AND (
        uh.caducidad_fecha_fin IS NULL 
        OR ? IS NULL 
        OR (uh.caducidad_fecha_ini <= ? AND uh.caducidad_fecha_fin >= ?)
      );";

    let cad_fecha_ini = config_horario.caducidad_fecha_ini.convert_to_date();
    let cad_fecha_fin = config_horario.caducidad_fecha_fin;

    let count: u32 = sqlx::query_scalar(QUERY)
      .bind(config_horario.usuario)
      .bind(config_horario.fecha_creacion)
      .bind(config_horario.id)
      .bind(config_horario.horario.dia.letra())
      .bind(config_horario.horario.hora_fin)
      .bind(config_horario.horario.hora_inicio)
      .bind(cad_fecha_fin)
      .bind(cad_fecha_fin)
      .bind(cad_fecha_ini)
      .fetch_one(self.pool.conexion())
      .await
      .map_err(DBError::from_sqlx)?;

    Ok(count > 0)
  }

  /// Busca un horario por su día y horas.
  pub(in crate::usuarios) async fn horario_por_dia_horas(
    &self,
    horario: &Horario,
  ) -> Result<Option<u32>, DBError> {
    const QUERY: &str = "SELECT id FROM horarios 
      WHERE dia = ? AND hora_inicio = ? AND hora_fin = ?";

    let row = sqlx::query_scalar(QUERY)
      .bind(horario.dia.letra())
      .bind(horario.hora_inicio)
      .bind(horario.hora_fin)
      .fetch_optional(self.pool.conexion())
      .await
      .map_err(DBError::from_sqlx)?;

    Ok(row)
  }

  /// Verifica si un horario está en uso por otra configuración.
  pub(in crate::usuarios) async fn es_horario_usado_excepto(
    &self,
    id_horario: u32,
    id_config_excluida: u32,
  ) -> Result<bool, DBError> {
    const QUERY: &str = "SELECT CAST(COUNT(*) AS UNSIGNED) 
      FROM usuario_horarios 
      WHERE horario = ? AND id <> ?;";

    let count: u32 = sqlx::query_scalar(QUERY)
      .bind(id_horario)
      .bind(id_config_excluida)
      .fetch_one(self.pool.conexion())
      .await
      .map_err(DBError::from_sqlx)?;

    Ok(count > 0)
  }

  /// Busca si el horario de un usuario se encuentra referenciado en el marcaje
  pub(in crate::usuarios) async fn esta_horario_en_marcaje(
    &self,
    usuario_horario: u32,
  ) -> Result<bool, DBError> {
    const QUERY: &str = "SELECT CAST(COUNT(*) AS UNSIGNED) 
      FROM marcajes 
      WHERE usuario_horario = ?;";

    let count: u32 = sqlx::query_scalar(QUERY)
      .bind(usuario_horario)
      .fetch_one(self.pool.conexion())
      .await
      .map_err(DBError::from_sqlx)?;

    Ok(count > 0)
  }

  /// Crea un nuevo horario.
  pub(in crate::usuarios) async fn crear_horario(
    &self,
    trans: &mut Transaccion<'_>,
    horario: &Horario,
  ) -> Result<u32, DBError> {
    const QUERY: &str = "INSERT INTO horarios (dia, hora_inicio, hora_fin)
       VALUES (?, ?, ?);";

    let res = sqlx::query(QUERY)
      .bind(horario.dia.letra())
      .bind(horario.hora_inicio)
      .bind(horario.hora_fin)
      .execute(&mut **trans.deref_mut())
      .await
      .map_err(DBError::from_sqlx)?;

    Ok(res.last_insert_id() as u32)
  }

  /// Elimina un horario.
  pub(in crate::usuarios) async fn eliminar_horario(
    &self,
    trans: &mut Transaccion<'_>,
    id_horario: u32,
  ) -> Result<(), DBError> {
    const QUERY: &str = "DELETE FROM horarios WHERE id = ?;";

    sqlx::query(QUERY)
      .bind(id_horario)
      .execute(&mut **trans.deref_mut())
      .await
      .map_err(DBError::from_sqlx)?;

    Ok(())
  }

  /// Obtiene la fecha de creación más reciente del horario.
  ///
  /// Se pretende obtener el grupo de horarios compacto
  /// Todos los horarios de un empleado se agrupan entorno a
  /// una fecha de creación, incluso aunque se hayan creado a posteriori
  /// Esto implica que si se quiere crear un nuevo horario después
  /// de haber creado su grupo, se aplicará desde la fecha de creación
  /// del grupo. Por tanto, si se necesita crear un horario a partir
  /// de este nuevo instante se debe crear obligatoriamente otro grupo
  /// y copiar todos los horarios anteriores (si se requieren).
  async fn fecha_creacion_horario(
    &self,
    usuario: u32,
    fecha: NaiveDate,
  ) -> Result<NaiveDate, DBError> {
    const QUERY: &str = "SELECT MAX(fecha_creacion) 
    FROM usuario_horarios 
    WHERE usuario = ? 
    AND fecha_creacion < ?";

    let fecha_creacion = sqlx::query_scalar::<_, Option<NaiveDate>>(QUERY)
      .bind(usuario)
      .bind(fecha)
      .fetch_one(self.pool.conexion())
      .await
      .map_err(DBError::from_sqlx)?
      .ok_or_else(|| {
        DBError::registro_vacio(format!(
          "No se ha encontrado ningún horario configurado \
        para el usuario en la fecha: {}",
          fecha.formato_corto()
        ))
      })?;

    Ok(fecha_creacion)
  }

  fn config_horario_from_row(row: &MySqlRow) -> ConfigHorario {
    ConfigHorario {
      id: row.get("uh_id"),
      usuario: row.get("usuario"),
      horario: UsuarioRepo::horario_from_row(row),
      fecha_creacion: row.get("fecha_creacion"),
      caducidad_fecha_ini: {
        // 01/01/1900 es equivalente a nulo, pero no se utiliza
        // nulo porque se encuentra en un índice
        let fecha: NaiveDate = row.get("caducidad_fecha_ini");
        if fecha == NONE_DATE {
          None
        } else {
          Some(fecha)
        }
      },
      caducidad_fecha_fin: row.get("caducidad_fecha_fin"),
    }
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
