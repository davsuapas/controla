use chrono::{NaiveDate, NaiveDateTime, Utc};
use smallvec::SmallVec;

use crate::{
  agregar_traza, config::{BootAdmin, ConfigTrabajo},
   infra::{DBError, Dni, Password, ServicioError, ShortDateTimeFormat, Transaccion, dni_valido, validar_password},
   traza::{TipoTraza, TrazaBuilder, TrazaServicio},
   usuarios::{ConfigHorario, DescriptorUsuario, Horario, Rol, Usuario, UsuarioRepo}
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
          usuario = usuario.id, error = %err,
          "Actualizando usuario");
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
    self
      .repo
      .usuario(&self.cnfg.secreto, id)
      .await
      .map_err(|err| {
        tracing::error!(usuario = id, error = %err, "Obteniendo usuario");
        ServicioError::from(err)
      })
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

  /// Devuelve los horarios del usuario sin asingnar.
  ///
  /// Si no se proporciona una hora, devuelve el horario del día actual.
  /// simpre que no este asignado.
  /// Si se proporciona una hora, devuelve el horario más cercano a esa hora.
  pub async fn horarios_usuario_sin_asignar(
    &self,
    usuario: u32,
    fecha: NaiveDate,
  ) -> Result<Vec<Horario>, ServicioError> {
     self
        .repo
        .horarios_usuario_sin_asignar(usuario, fecha)
        .await
        .map_err(|err| {
          tracing::error!(
          usuario = usuario,
          hora = ?fecha,
          error = %err,
         "Obteniendo horario del usuario sin asignar");
          ServicioError::from(err)
        })
  }

  /// Devuelve el horario del usuario más cercano.
  /// 
  /// Si no devuelve ningún horario se tracea el error y
  /// se devuelve una array vacío
  pub async fn horario_usuario_cercano(
    &self,
    usuario: u32,
    hora: NaiveDateTime,
  ) -> Result<Vec<Horario>, ServicioError> {
    tracing::debug!(
      usuario = usuario,
      hora = %hora,
      "Consultando el horario más cercano del usuario");

    let res = self.repo
        .horario_cercano(usuario, hora, 0)
        .await
        .map(|(_, horario)| vec![horario]);
    match res {
      Ok(horarios) => Ok(horarios),
      Err(err) => match err {
        DBError::RegistroVacio(e) => {
          tracing::warn!(
            error = %e,
           "Consulta horario cercano");
          Ok(vec![])
        },
        _ => {
          tracing::error!(
            usuario = usuario,
            hora = ?hora,
            error = %err,
          "Consulta horario cercano");
          Err(ServicioError::from(err))
        } 
      },
    }
  }

  /// Devuelve el horario más cercano al usuario.
  ///
  /// Se puede excluir un marcaje pasado como parámetro
  /// Si no quiere excluir ningún marcaje use 0
  /// La exclusión puede ser muy útil cuando se quiere
  /// realizar una modificación de este marcaje
  /// 
  /// Devuelve el identificador de usuario, horario y el horario.
  pub async fn horario_cercano(
    &self,
    usuario: u32,
    hora: NaiveDateTime,
    excluir_marcaje_id: u32,
  ) -> Result<(u32, Horario), ServicioError> {
    self
      .repo
      .horario_cercano(usuario, hora, excluir_marcaje_id)
      .await
      .map_err(ServicioError::from)
  }

  /// Devuelve la configuración de horarios de un usuario.
  ///
  /// Recupera la lista de horarios configurados para el usuario especificado.
  pub async fn config_horario(
    &self,
    usuario: u32,
  ) -> Result<Vec<ConfigHorario>, ServicioError> {
    tracing::debug!(
      usuario = usuario,
      "Obteniendo configuración de horario del usuario");

    let fecha_actual = Utc::now()
      .with_timezone(&self.cnfg.zona_horaria)
      .naive_local().date();

    self.repo.config_horario(usuario, fecha_actual).await.map_err(|err| {
      tracing::error!(
        usuario = usuario,
        error = %err,
        "Obteniendo configuración de horario del usuario"
      );
      ServicioError::from(err)
    })
  }
  
  /// Obtiene el horario dado el id.
  pub async fn config_horario_por_id(&self, id: u32) -> Result<ConfigHorario, ServicioError> {
    tracing::debug!(
      id = id,
      "Obteniendo horario por id");

    self.repo.config_horario_por_id(id).await.map_err(|err| {
      tracing::error!(
        id = id,
        error = %err,
        "Obteniendo horario por id"
      );
      ServicioError::from(err)
    })
  }

  /// Duplica la configuración de un horario
  /// 
  /// Devuelve la configuración del horario duplicada
  pub async fn duplicar_config_horario(
    &self,
    usuario: u32,
    nueva_fecha_creacion: NaiveDate,
  ) -> Result<Vec<ConfigHorario>, ServicioError> {
    tracing::info!(
      usuario = usuario,
      nueva_fecha_creacion = %nueva_fecha_creacion,
      "Se duplica la configuración del horario");

    self.repo.duplicar_config_horario(usuario, nueva_fecha_creacion).await.map_err(|err| {
      tracing::error!(
        usuario = usuario,
        nueva_fecha_creacion = %nueva_fecha_creacion,
        error = %err,
        "Duplicando grupo horario."
      );
      ServicioError::from(err)
    })?;

    self.config_horario(usuario).await
  }

  /// Añade una nueva configuración de horario.
  /// 
  /// Verifica que no exista solapamiento con otros horarios.
  /// Si existe solapamiento se envía un error al usuario.
  /// Si no existe solapamiento se llama a [`Self::actualizar_horario`] 
  /// para actualizar el maestro del horario y devolve el ide de horario.
  /// Por último se crea la configuración del horario.
  pub async fn agregar_config_horario(
    &self,
    config_horario: &ConfigHorario,
  ) -> Result<u32, ServicioError> {
    tracing::info!(
      usuario = config_horario.usuario,
      fecha = %config_horario.fecha_creacion,
      "Se ha iniciado el servicio para agregar una configuración de horario");

    if self
      .repo
      .config_horario_solape(config_horario)
      .await
      .map_err(|err| {
        tracing::error!(
          usuario = config_horario.usuario,
          error = %err,
          "Verificando solapamiento de horario"
        );
        ServicioError::from(err)
      })?
    {
      return Err(ServicioError::Validacion(format!(
        "El horario se solapa con otro existente para el usuario: {} en la fecha: {}",
        config_horario.usuario,
        config_horario.fecha_creacion.formato_corto()
      )));
    }

    let mut tr = self.repo.conexion().empezar_transaccion().await.map_err(
      |err| {
        tracing::error!(
           usuario = config_horario.usuario, error = %err,
           "Iniciando transacción para agregar configuración de horario");
        ServicioError::from(err)
      },
    )?;

    let id_horario = self
      .obtener_o_crear_horario(&mut tr, &config_horario.horario)
      .await?;

    let id_config = match self
      .repo
      .agregar_config_usuario(&mut tr, config_horario, id_horario)
      .await
    {
      Ok(id) => id,
      Err(err) => {
        tracing::error!(
          usuario = config_horario.usuario,
          error = %err,
          "Agregando configuración de horario en bd"
        );

        return Err(ServicioError::from(err));
      }
    };

    tr.commit().await.map_err(|err| {
      tracing::error!(
         usuario = config_horario.usuario, error = %err,
        "Commit transacción para agregar configuración de horario");
      ServicioError::from(err)
    })?;

    tracing::debug!(
      id_config = id_config,
      "Se ha completado satisfactoriamente la agregación \
      de la configuración de horario"
    );

    Ok(id_config)
  }

  /// Modifica una configuración de horario.
  ///
  /// Verifica que no esté referenciada en un marcaje.
  /// Verifica que no exista solapamiento con otros horarios.
  /// Si no existe solapamiento se llama a [`Self::actualizar_horario`]
  /// para actualizar el maestro del horario y por último se
  /// modifica la configuración del horario.
  pub async fn modificar_config_horario(
    &self,
    config_horario: &ConfigHorario,
  ) -> Result<(), ServicioError> {
    tracing::info!(
      config_horario = ?config_horario,
      "Se ha iniciado el servicio para modificar una configuración de horario");

    if self
      .repo
      .esta_horario_en_marcaje(config_horario.id)
      .await
      .map_err(|err| {
        tracing::error!(
          config_horario_id = config_horario.id,
          error = %err,
          "Verificando si la configuración de horario está en un marcaje"
        );
        ServicioError::from(err)
      })?
    {
      return Err(ServicioError::Validacion(
        "No se puede modificar la configuración del horario porque \
        ya está referenciada en un marcaje."
          .to_string(),
      ));
    }

    if self
      .repo
      .config_horario_solape(config_horario)
      .await
      .map_err(|err| {
        tracing::error!(
          usuario = config_horario.usuario,
          error = %err,
          "Verificando solapamiento de horario"
        );
        ServicioError::from(err)
      })?
    {
      return Err(ServicioError::Validacion(format!(
        "El horario se solapa con otro existente para el usuario: {} en la fecha: {}",
        config_horario.usuario,
        config_horario.fecha_creacion.formato_corto()
      )));
    }

    let mut tr = self.repo.conexion().empezar_transaccion().await.map_err(
      |err| {
        tracing::error!(
           usuario = config_horario.usuario, error = %err,
           "Iniciando transacción para modificar configuración de horario");
        ServicioError::from(err)
      },
    )?;

    let id_horario_antiguo = config_horario.horario.id;
    let id_nuevo_horario = self
      .obtener_o_crear_horario(&mut tr, &config_horario.horario)
      .await?;

    if let Err(err) = self
      .repo
      .modificar_config_usuario(&mut tr, config_horario, id_nuevo_horario)
      .await
    {
      tracing::error!(
        config_horario = ?config_horario,
        error = %err,
        "Modificando configuración de horario en bd"
      );
      return Err(ServicioError::from(err));
    }

    if id_horario_antiguo != id_nuevo_horario {
      self
        .eliminar_horario(&mut tr, id_horario_antiguo, config_horario.id)
        .await?;
    }

    tr.commit().await.map_err(|err| {
      tracing::error!(
         usuario = config_horario.usuario, error = %err,
        "Commit transacción para modificar configuración de horario");
      ServicioError::from(err)
    })?;

    tracing::debug!(
      id_config = config_horario.id,
      "Se ha completado satisfactoriamente la modificación \
      de la configuración de horario"
    );

    Ok(())
  }

  /// Elimina una configuración de horario.
  ///
  /// Verifica que no esté referenciada en un marcaje.
  /// Elimina la configuración y si el horario asociado no se usa
  /// en ninguna otra configuración, también se elimina.
  pub async fn eliminar_config_horario(
    &self,
    id_config: u32,
  ) -> Result<(), ServicioError> {
    tracing::info!(
      id_config = id_config,
      "Se ha iniciado el servicio para eliminar una configuración de horario");

    if self
      .repo
      .esta_horario_en_marcaje(id_config)
      .await
      .map_err(|err| {
        tracing::error!(
          id_config = id_config,
          error = %err,
          "Verificando si la configuración de horario está en un marcaje"
        );
        ServicioError::from(err)
      })?
    {
      return Err(ServicioError::Validacion(
        "No se puede eliminar la configuración del horario porque ya está referenciada en un marcaje."
          .to_string(),
      ));
    }

    let config = self.config_horario_por_id(id_config).await?;

    let mut tr = self.repo.conexion().empezar_transaccion().await.map_err(
      |err| {
        tracing::error!(
           id_config = id_config, error = %err,
           "Iniciando transacción para eliminar configuración de horario");
        ServicioError::from(err)
      },
    )?;

    if let Err(err) = self
      .repo
      .eliminar_config_usuario(&mut tr, id_config)
      .await
    {
      tracing::error!(
        id_config = id_config,
        error = %err,
        "Eliminando configuración de horario en bd"
      );
      return Err(ServicioError::from(err));
    }

    self
      .eliminar_horario(&mut tr, config.horario.id, id_config)
      .await?;

    tr.commit().await.map_err(|err| {
      tracing::error!(
         id_config = id_config, error = %err,
        "Commit transacción para eliminar configuración de horario");
      ServicioError::from(err)
    })?;

    tracing::debug!(
      id_config = id_config,
      "Se ha completado satisfactoriamente la eliminación de la configuración de horario"
    );

    Ok(())
  }

  /// Obtiene el id de un horario existente o crea uno nuevo.
  ///
  /// Si existe ya un horario (mismo día y fechas) devuelve el id.
  /// Si no existe lo crea y devuelve el nuevo id.
  async fn obtener_o_crear_horario(
    &self,
    trans: &mut Transaccion<'_>,
    horario: &Horario,
  ) -> Result<u32, ServicioError> {
    tracing::info!(
      horario = ?horario,
      "Se ha iniciado el servicio para obtener o crear un horario");

    if let Some(id) = self
      .repo
      .horario_por_dia_horas(horario)
      .await
      .map_err(|err| {
        tracing::error!(
          horario = ?horario,
          error = %err,
          "Buscando horario por día y horas"
        );
        ServicioError::from(err)
      })?
    {
      Ok(id)
    } else {
      tracing::debug!("No existe el horario maestro. Se crea uno nuevo");
      match self
        .repo
        .crear_horario(trans, horario)
        .await
        {
          Ok(id) => Ok(id),
          Err(err) => {
            tracing::error!(
              horario = ?horario,
              error = %err,
              "Creando nuevo horario maestro"
            );
            Err(ServicioError::from(err))
          }
        }
    }
  }

  /// Elimina el horario si no es usado por ninguna otra configuración.
  async fn eliminar_horario(
    &self,
    trans: &mut Transaccion<'_>,
    id_horario: u32,
    id_config_excluida: u32,
  ) -> Result<(), ServicioError> {
    if !self
      .repo
      .es_horario_usado_excepto(id_horario, id_config_excluida)
      .await
      .map_err(|err| {
        tracing::error!(
          id_horario = id_horario,
          id_config_excluida = id_config_excluida,
          error = %err,
          "Verificando si el horario está en uso"
        );
        ServicioError::from(err)
      })?
    {
      tracing::debug!(
        id_horario = id_horario,
        "El horario ya no se referencia. Se elimina"
      );
      self.repo.eliminar_horario(trans, id_horario).await.map_err(|err| {
        tracing::error!(
          id_horario = id_horario,
          error = %err,
          "Eliminando horario no referenciado"
        );
        ServicioError::from(err)
      })?;
    }
    Ok(())
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
