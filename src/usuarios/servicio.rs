use chrono::{Utc};
use smallvec::SmallVec;

use crate::{
  agregar_traza, config::{BootAdmin, ConfigTrabajo},
   infra::{DBError, Dni, Password, ServicioError, dni_valido, validar_password},
   traza::{TipoTraza, TrazaBuilder, TrazaServicio},
   usuarios::{DescriptorUsuario, Rol, Usuario, UsuarioRepo}
};

///Servicio para manejar operaciones relacionadas con usuarios.
pub struct UsuarioServicio {
  cnfg: ConfigTrabajo,
  repo: UsuarioRepo,
  srv_traza: TrazaServicio,
}

impl UsuarioServicio {
  pub fn new(
    cnfg: ConfigTrabajo,
    repo: UsuarioRepo,
    srv_traza: TrazaServicio,
  ) -> Self {
    UsuarioServicio {
      cnfg,
      repo,
      srv_traza,
    }
  }
}

impl UsuarioServicio {

  /// Crear el usuario administrador inicial
  pub async fn crear_admin(
    &self,
    boot_admin: &BootAdmin,
  ) -> Result<(), ServicioError> {
    tracing::info!(
      admin = ?boot_admin,
      "Se intenta crear el usuario administrador inicial");

    let dni = Dni::new(boot_admin.dni.clone());
    let now = Utc::now()
          .with_timezone(&self.cnfg.zona_horaria)
          .naive_local();

    let usuario = Usuario {
      id: 0,
      email: "admin.inicial@controla.com".to_string(),
      nombre: "Modifique los datos del usuario".to_string(),
      primer_apellido: "Modifique los datos del usuario".to_string(),
      segundo_apellido: "Modifique los datos del usuario".to_string(),
      dni,
      activo: Some(now),
      inicio: None,
      roles: SmallVec::from_slice(&[Rol::Admin]),
      calendarios: vec![],
      password: Some(Password::new(boot_admin.password.clone())),
    };

    self.crear_usuario(0, &usuario).await?;

    Ok(())
  }

  /// Crea un nuevo usuario.
  /// 
  /// El usuario es creado por un usuario autor
  /// Si el usuario creador es cero, el usuario creador será
  /// mismo que el usuario que se va a crear.
  /// Valida los datos del usuario antes de proceder con la creación.
  /// Genera una traza de la operación.
  pub async fn crear_usuario(
    &self,
    creado_por: u32,
    usuario: &Usuario,
  ) -> Result<u32, ServicioError> {
    tracing::info!(
      usuario = ?usuario,
      "Se ha iniciado el servicio para crear un nuevo usuario");

    valida_ids_usuario(usuario)?;
    self.valida_password(usuario.id, usuario.password.as_ref().unwrap())?;
    self.valida_dni_duplicado(usuario).await?;

    let mut tr = self.repo.conexion().empezar_transaccion().await.map_err(
      |err| {
        tracing::error!(
           usuario = usuario.nombre_completo(), error = %err,
           "Iniciando transacción para creación de usuario");
        ServicioError::from(err)
      },
    )?;

    let id = match self
      .repo
      .crear_usuario(&mut tr, &self.cnfg.secreto, usuario)
      .await
    {
      Ok(id) => id,
      Err(err) => {
        tracing::error!(
          usuario = usuario.nombre_completo(), error = %err,
          error = %err,
          "Creando usuario");

        return Err(ServicioError::from(err));
      }
    };

    let cal_ids: Vec<u32> = usuario
      .calendarios
      .iter()
      .filter(|c| c.asignado)
      .map(|c| c.calendario)
      .collect();

    for &cal_id in &cal_ids {
      let fechas_conflictivas = self
        .repo
        .marcajes_conflictivos_asignacion_calendario(id, cal_id)
        .await?;
      if !fechas_conflictivas.is_empty() {
        let fechas_str: Vec<String> = fechas_conflictivas
          .into_iter()
          .map(|f| f.format("%d/%m/%Y").to_string())
          .collect();
        let mensaje_error = format!(
          "No se puede asignar el calendario porque el usuario tiene marcajes en las siguientes fechas que entran en conflicto: {}.",
          fechas_str.join(", ")
        );
        return Err(ServicioError::Validacion(mensaje_error));
      }
    }

    if let Err(err) = self
      .repo
      .agregar_roles(&mut tr, id, &usuario.roles)
      .await {
        tracing::error!( 
          usuario = id,
          roles = ?usuario.roles,
          error = %err,
          "Anádiendo roles a el de usuario");
        return Err(ServicioError::from(err));
    }

    if let Err(err) = self
      .repo
      .agregar_calendarios(&mut tr, id, &cal_ids)
      .await {
        tracing::error!(
          usuario = id,
          calendarios = ?cal_ids,
          error = %err,
          "Añadiendo calendarios al usuario");
        return Err(ServicioError::from(err));
    }

    let autor = if creado_por == 0 {
      id
    } else {
      creado_por
    };

    let traza = TrazaBuilder::with_usuario(
      TipoTraza::CreacionUsuario, id)
      .autor(Some(autor))
      .build(&self.cnfg.zona_horaria);

    agregar_traza!(
      self, tr, traza, "Creando traza creación de usuario", usuario = id);

    tr.commit().await.map_err(|err| {
      tracing::error!(
         usuario = id, error = %err,
        "Commit transacción para creación de usuario");
      ServicioError::from(err)
    })?;

    tracing::debug!(
      usuario = id,
      "Se ha completado satisfactoriamente la creación del usuario"
    );

    Ok(id)
  }

  /// Actualiza un usuario existente.
  ///
  /// El usuario es modificado por un usuario autor
  /// Valida los datos del usuario antes de proceder con la actualización.
  /// Genera trazas de las modificaciones.
  pub async fn actualizar_usuario(
    &self,
    modificado_por: u32,
    usuario: &Usuario,
    ) -> Result<(), ServicioError> {
    tracing::info!(
      usuario = ?usuario,
      "Se ha iniciado el servicio para actualizar un usuario");

    valida_ids_usuario(usuario)?;

    let usr_persistido = self.usuario(usuario.id).await?;

    let mut tr = self.repo.conexion().empezar_transaccion().await.map_err(
      |err| {
        tracing::error!(
           usuario = usuario.id , error = %err,
           "Iniciando transacción para actualización del usuario");
        ServicioError::from(err)
      },
    )?;

    if usr_persistido.nombre != usuario.nombre || 
      usr_persistido.primer_apellido != usuario.primer_apellido ||
      usr_persistido.segundo_apellido != usuario.segundo_apellido {
      tracing::debug!(
        usuario = usuario.id, "Ha cambiado el nombre del usuario");
   
      let traza = TrazaBuilder::with_usuario(
        TipoTraza::UsrNombreModificado, usuario.id)
        .autor(Some(modificado_por))
        .motivo(Some(format!(
          "Nombre cambiado de {} a {}",
          &usr_persistido.nombre_completo(), &usuario.nombre_completo()
        )))
        .build(&self.cnfg.zona_horaria);

      agregar_traza!(
        self, tr, traza,
        "Creando traza modificación de DNI", usuario = usuario.id);
    }

    let mut inicio_log = usr_persistido.inicio; 

    if usr_persistido.activo != usuario.activo {
      tracing::debug!(
        usuario = usuario.id, "Ha cambiado el campo activo del usuario");

      let traza = TrazaBuilder::with_usuario(
        TipoTraza::UsrActivoModificado, usuario.id)
        .autor(Some(modificado_por))
        .motivo(Some(format!(
          "Activo cambiado de {:?} a {:?}",
          usr_persistido.activo, usuario.activo
        )))
        .build(&self.cnfg.zona_horaria);

      agregar_traza!(
        self, tr, traza,
        "Creando traza modificación de activo", usuario = usuario.id);

      if usr_persistido.activo.is_none() && usuario.activo.is_some() {
        // Si se activa el usuario se resetea el valor
        // de inicio de log
        inicio_log = None;

        tracing::debug!(
          usuario = usuario.id,
          "Se ha activado el usuario. Se reinicia el inicio");
      }
    }

    if usr_persistido.dni != usuario.dni {
      tracing::debug!(
        usuario = usuario.id,
        "Ha cambiado el DNI del usuario");

      let reg_horarios = self.repo.num_marcajes_horarios_usuario(usuario.id)
        .await;

      match reg_horarios {
        Ok(num) => {
          if num > 0 {
            return Err(ServicioError::Usuario(
              "No se puede modificar el DNI si existen registros \
              horarios para este usuario. Consulte con el admistrador."
              .to_string()));
          }
        },
        Err(err) => {
          tracing::error!(
            usuario = usuario.id, error = %err,
            "Obteniendo el número de resgistros horarios del \
            usuario para validar el DNI");
          return Err(ServicioError::DB(err))
        }
      } 

      self.valida_dni_duplicado(usuario).await?;

      let traza = TrazaBuilder::with_usuario(
        TipoTraza::UsrDniModificado, usuario.id)
        .autor(Some(modificado_por))
        .build(&self.cnfg.zona_horaria);

      agregar_traza!(
        self, tr, traza,
        "Creando traza modificación de DNI", usuario = usuario.id);
    }

    if let Err(err) = self
      .repo
      .actualizar_usuario(&mut tr, &self.cnfg.secreto, usuario, inicio_log)
      .await {
        tracing::error!( 
          usuario = usuario.id, error = %err, "Actualizando usuario");
        return Err(ServicioError::from(err));
    }

    if !usr_persistido.eq_roles(usuario) {
      let traza = TrazaBuilder::with_usuario(
        TipoTraza::UsrRolesModificados, usuario.id)
        .autor(Some(modificado_por))
        .motivo(Some(format!(
          "Roles cambiados de {:?} a {:?}",
          usr_persistido.roles, usuario.roles
        )))
        .build(&self.cnfg.zona_horaria);

      agregar_traza!(
        self, tr, traza,
        "Creando traza modificación de roles", usuario = usuario.id);

      if let Err(err) = self
        .repo
        .agregar_roles(&mut tr, usuario.id, &usuario.roles)
        .await {
          tracing::error!( 
            usuario = usuario.id, error = %err,
            "Anádiendo roles al usuario");
          return Err(ServicioError::from(err));
      }
    }

    if !usr_persistido.eq_calendarios(usuario) {
      let cal_ids_nuevos: Vec<u32> = usuario
        .calendarios
        .iter()
        .filter(|c| c.asignado)
        .map(|c| c.calendario)
        .collect();

      let cal_ids_persistidos: std::collections::HashSet<u32> = usr_persistido
        .calendarios
        .iter()
        .filter(|c| c.asignado)
        .map(|c| c.calendario)
        .collect();

      for &cal_id in &cal_ids_nuevos {
        if !cal_ids_persistidos.contains(&cal_id) {
          // Es un calendario nuevo que se está asignando
          let fechas_conflictivas = self
            .repo
            .marcajes_conflictivos_asignacion_calendario(usuario.id, cal_id)
            .await?;
          if !fechas_conflictivas.is_empty() {
            let fechas_str: Vec<String> = fechas_conflictivas
              .into_iter()
              .map(|f| f.format("%d/%m/%Y").to_string())
              .collect();
            let mensaje_error = format!(
              "No se puede asignar el calendario porque el usuario tiene \
              marcajes en las siguientes fechas que entran en conflicto: {}.",
               fechas_str.join(", "));
            return Err(ServicioError::Validacion(mensaje_error));
          }
        }
      }

      let traza = TrazaBuilder::with_usuario(
        TipoTraza::UsrCalendariosModificados, usuario.id)
        .autor(Some(modificado_por))
        .motivo(Some(format!(
          "Calendarios cambiados de {:?} a {:?}",
          usr_persistido.calendarios
            .iter().map(|c| c.calendario).collect::<Vec<_>>(),
          cal_ids_nuevos
        )))
        .build(&self.cnfg.zona_horaria);

      agregar_traza!(
        self, tr, traza,
        "Creando traza modificación de calendarios", usuario = usuario.id);

      if let Err(err) = self.repo
        .agregar_calendarios(&mut tr, usuario.id, &cal_ids_nuevos).await {
        tracing::error!(
          usuario = usuario.id,
          error = %err,
          "Añadiendo calendarios al usuario");
        return Err(ServicioError::from(err));
      }
    }

    let traza = TrazaBuilder::with_usuario(
      TipoTraza::ActualizacionUsuario, usuario.id)
      .autor(Some(modificado_por))
      .build(&self.cnfg.zona_horaria);

    agregar_traza!(
      self, tr, traza,
      "Creando traza actualización de usuario", usuario = usuario.id);

    tr.commit().await.map_err(|err| {
      tracing::error!(
         usuario = usuario.id, error = %err,
        "Commit transacción para actualización de usuario");
      ServicioError::from(err)
    })?;

    tracing::debug!(
      usuario = usuario.id,
      "Se ha completado satisfactoriamente la actualización del usuario"
    );

    Ok(())
  }

  /// Actualiza la password de un usuario existente.
  ///
  /// Valida la password antes de proceder con la actualización.
  /// Genera trazas de las modificación.
  pub async fn actualizar_password(
    &self,
    usuario: u32,
    password: &Password,
    ) -> Result<(), ServicioError> {
    tracing::info!(
      usuario = usuario,
      "Se ha iniciado el servicio para actualizar las password de un usuario");
    
    self.valida_password(usuario, password)?;

    let mut tr = self.repo.conexion().empezar_transaccion().await.map_err(
      |err| {
        tracing::error!(
           usuario = usuario , error = %err,
           "Iniciando transacción para actualización del usuario");
        ServicioError::from(err)
      },
    )?;

    let traza = TrazaBuilder::with_usuario(
      TipoTraza::PasswordModificada, usuario)
      .build(&self.cnfg.zona_horaria);

    agregar_traza!(
      self, tr, traza,
      "Creando traza modificación password", usuario = usuario);

    if let Err(err) = self
      .repo
      .actualizar_password(&mut tr, &self.cnfg.secreto, usuario, password)
      .await {
        tracing::error!( 
          usuario = usuario, error = %err,
          "Actualizando password de usuario");
        return Err(ServicioError::from(err));
    }

    tr.commit().await.map_err(|err| {
      tracing::error!(
         usuario = usuario, error = %err,
        "Commit transacción para actualización de password");
      ServicioError::from(err)
    })?;


    tracing::debug!(
      usuario = usuario,
      "Se ha completado satisfactoriamente la actualización de la password"
    );

    Ok(())
  }
  
  async fn valida_dni_duplicado(
    &self, usuario: &Usuario) -> Result<(), ServicioError> {
    if self.repo.dni_duplicado(&self.cnfg.secreto, &usuario.dni)
      .await.map_err(|err| {
      tracing::error!(
        usuario = usuario.nombre_completo(),
        error = %err, "Validando DNI");
      ServicioError::from(err)
    })? {
      const VALIDA_DNI: &str = "El DNI del usuario ya existe. \
      No puede haber dos DNI iguales para usuarios diferentes";
  
      tracing::error!(usuario = ?usuario, VALIDA_DNI);
  
      return Err(ServicioError::Validacion(VALIDA_DNI.to_string()));
    }

    Ok(())
  }

  fn valida_password(
      &self, usuario: u32, password: &Password) -> Result<(), ServicioError> {
    if password.is_empty() {
      const VALIDA_PASS: &str = "La password del usuario no puede estar vacía";

      tracing::error!(usuario = usuario, VALIDA_PASS);

      return Err(ServicioError::Validacion(VALIDA_PASS.to_string()));
    }

    let res = validar_password(password, &self.cnfg.passw);

    if !res.es_valido {
      return Err(ServicioError::Validacion(res.to_string()));
    }

    Ok(())
  }

  /// Realiza el login de usuario
  /// 
  /// Si el usuario no inicio nunca sesión actualiza el inicio
  /// en la base de datos y añade una traza.
  /// 
  /// Devuelve si la password proporcionada es correcta.
  pub async fn login_usuario(
      &self, dni: &Dni, password: &Password
    ) -> Result<Option<Usuario>, ServicioError> {

    let result = self.repo.usuario_por_dni(&self.cnfg.secreto, dni).await;

    let usr = match result {
      Ok(u) => u,
      Err(DBError::RegistroVacio(_)) => {
        tracing::info!("No existe el usuario");
         return Ok(None);
      },
      Err(err) => { 
        tracing::error!(error = %err, "Obteniendo usuario por dni");
        return Err(ServicioError::from(err));
      }
    };

    tracing::info!(
      usuario = ?usr,
      "Se ha iniciado el servicio que valida el login de usuario");

    let result = self.repo.password(&self.cnfg.secreto, usr.id)
      .await.map_err(|err| {
      tracing::error!(error = %err, "Obteniendo password de usuario");
      ServicioError::from(err)
    })?;

    if let Some(passw) = result {
      if passw != *password {
        tracing::info!(usuario = usr.id, "La password es incorrecta");
        return Ok(None);
      }

      if usr.inicio.is_none() {
        let inicio = Utc::now()
          .with_timezone(&self.cnfg.zona_horaria)
          .naive_local();

        tracing::debug!(
          usuario = ?usr, nuevo_inicio = %inicio,
          "El usuario es el primer inicio de sesión que realiza");

        let mut tr = self.repo.conexion().empezar_transaccion().await.map_err(
          |err| {
            tracing::error!(
              usuario = ?usr, error = %err,
              "Iniciando transacción para atualizar el inicio sesión");
            ServicioError::from(err)
          },
        )?;

        if let Err(err) = self.repo.actualizar_inicio(
            &mut tr, usr.id, inicio).await {
          tracing::error!(
            usuario = ?usr, error = %err, "Actualizando inicio sesión");
         
          return Err(ServicioError::from(err))
        }

        let traza = TrazaBuilder::with_usuario(
          TipoTraza::PrimerInicio, usr.id)
          .build(&self.cnfg.zona_horaria);

        agregar_traza!(
          self, tr, traza, "Creando traza actualización inicio sesión",
          usuario = usr.id);

        tr.commit().await.map_err(|err| {
          tracing::error!(
            usuario = ?usr, error = %err,
            "Commit transacción para actualización inicio sesión");
          ServicioError::from(err)
        })?;
      }
      tracing::info!(
        usuario = ?usr,
        "Se ha completado satisfactoriamente el login de usuario");

      Ok(Some(usr))
    } else {
      tracing::info!(
        usuario = ?usr, "No existe el usuario o no esta activado");

      Ok(None)
    }
}

  /// Devuelve todos los usuarios existentes.
  pub async fn usuarios(&self) -> Result<Vec<Usuario>, ServicioError> {
    self.repo.usuarios(&self.cnfg.secreto).await.map_err(|err| {
      tracing::error!(error = %err, "Obteniendo usuarios");
      ServicioError::from(err)
    })
  }

  /// Devuelve un usuario por su ID.
  pub async fn usuario(&self, id: u32) -> Result<Usuario, ServicioError> {
    let mut usuario = self
      .repo
      .usuario(&self.cnfg.secreto, id)
      .await
      .map_err(|err| {
        tracing::error!(usuario = id, error = %err, "Obteniendo usuario");
        ServicioError::from(err)
      })?;
    
    usuario.calendarios = self.repo.calendarios_asignados_por_usuario(id).await?;
    Ok(usuario)
  }

  /// Devuelve un usuario por su ID con los calendarios asignados.
  /// 
  /// Si todos_los_calendarios es true se envían todos los calendarios
  /// se marcan los asignados. Si es false solo los asignados.
  pub async fn usuario_con_calendarios(
    &self,
    id: u32,
    todos_los_calendarios: bool,
  ) -> Result<Usuario, ServicioError> {
    let mut usuario = self
      .repo
      .usuario(&self.cnfg.secreto, id)
      .await
      .map_err(|err| {
        tracing::error!(usuario = id, error = %err, "Obteniendo usuario");
        ServicioError::from(err)
      })?;

    usuario.calendarios = if todos_los_calendarios {
      self.repo.todos_los_calendarios_con_asignacion(id).await?
    } else {
      self.repo.calendarios_asignados_por_usuario(id).await?
    };

    Ok(usuario)
  }

  pub async fn usuarios_por_rol(
    &self,
    rol: Rol,
  ) -> Result<Vec<DescriptorUsuario>, ServicioError> {
    self.repo.usuarios_por_rol(rol).await.map_err(|err| {
      tracing::error!(
          rol = ?rol,
          error = %err,
          "Obteniendo usuarios por rol");
        ServicioError::from(err)
      })
  }
}

fn valida_ids_usuario(
  usuario: &Usuario) -> Result<(), ServicioError> {
  // No uso Trim() para evitar que cree cadenas imnecesarias
  // ya que desde el interface de usuario se eliminan los espacios
  if usuario.email.is_empty() ||
    usuario.nombre.is_empty() ||
    usuario.primer_apellido.is_empty() ||
    usuario.segundo_apellido.is_empty() ||
    usuario.dni.is_empty() {
    const VALIDA_DESCRIPTORES: &str =
      "El email, nombre, apellidos o DNI del usuario no puede estar vacío";

    tracing::error!(usuario = ?usuario, VALIDA_DESCRIPTORES);

    return Err(ServicioError::Validacion(VALIDA_DESCRIPTORES.to_string()));
  }

  if !dni_valido(&usuario.dni) {
    const VALIDA_DNI: &str = "El DNI proporcionado no es válido";

    tracing::error!(usuario = ?usuario, VALIDA_DNI);

    return Err(ServicioError::Validacion(VALIDA_DNI.to_string()));
  }

  Ok(())
}
