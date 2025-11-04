use chrono::NaiveDate;

use crate::{
  config::ConfigTrabajo,
  infra::{
    DominiosWithCacheUsuario, ServicioError, ShortDateTimeFormat, Transaccion,
  },
  marcaje::{Marcaje, MarcajeRepo},
  usuarios::UsuarioServicio,
};

/// Servicio que gestiona los marcajes del usuario
pub struct MarcajeServicio {
  cnfg: ConfigTrabajo,
  repo: MarcajeRepo,
  usuario_servico: UsuarioServicio,
}

impl MarcajeServicio {
  pub fn new(
    cnfg: ConfigTrabajo,
    repo: MarcajeRepo,
    usuario_servico: UsuarioServicio,
  ) -> Self {
    MarcajeServicio {
      cnfg,
      repo,
      usuario_servico,
    }
  }
}

impl MarcajeServicio {
  #[inline]
  /// Añade un nuevo marcaje horario para el usuario.
  ///
  /// Para más detalles vea: [`agregar_with_trans`].
  pub async fn agregar(&self, reg: &Marcaje) -> Result<u32, ServicioError> {
    self.agregar_with_trans(None, reg, 0).await
  }
  /// Añade un nuevo marcaje horario para el usuario.
  ///
  /// Para calcular las horas a trabajar utiliza el horario más
  /// cercano a la hora de inicio del marcaje que todavía
  /// no haya sido asignado.
  ///
  /// Validaciones:
  /// * Si existen marcajes con alguna hora de fin sin registrar,
  /// * se devuelve un error.
  /// * Si el usuario no tiene un horario configurado, se devuelve un error.
  /// * Si la hora de inicio o fin ya están asignadas al usuario,
  ///   se devuelve un error.
  /// * El nuevo marcaje no se puede solapar con ningún otro marcaje.
  /// * La hora de inicio no puede ser anterior a la hora de fin
  ///   de un marcaje previo con un horario anterior al horario cercano
  ///   obtenido.
  ///
  /// Se puede excluir un marcaje pasado como parámetro
  /// Si no quiere excluir ningún marcaje use 0
  /// La exclusión puede ser muy útil cuando se quiere
  /// realizar una modificación de este marcaje
  ///
  /// Devuelve el ID del marcaje creado.
  pub async fn agregar_with_trans(
    &self,
    tr: Option<&mut Transaccion<'_>>,
    reg: &Marcaje,
    excluir_marcaje_id: u32,
  ) -> Result<u32, ServicioError> {
    tracing::info!(
      marcaje = ?reg,
      excluir_marcaje_id = excluir_marcaje_id,
      "Se ha iniciado el servicio para crear un marcaje horario de usuario");

    self.validar_agregacion(reg, excluir_marcaje_id).await?;

    let horario_cercano = self
      .usuario_servico
      .horario_cercano(
        reg.usuario,
        reg.hora_inicio_completa(),
        excluir_marcaje_id,
      )
      .await
      .inspect_err(|err| {
        tracing::error!(
          marcaje = ?reg,
          error = %err,
         "Buscando el horario más cercano cuando se añade un marcaje");
      })?;

    let horas_a_trabajar = horario_cercano.horas_a_trabajar();

    tracing::debug!(
      horario = ?horario_cercano,
      horas_a_trabajar = format!("{:.2}", horas_a_trabajar),
      "Horario más cercano a el marcajes horario del usuario");

    let id = match self.repo.agregar(tr, reg, horario_cercano.id).await {
      Ok(reg_id) => reg_id,
      Err(err) => {
        tracing::error!(
          marcaje = ?reg,
          error = %err,
          "Creando marcaje horario"
        );
        return Err(ServicioError::from(err));
      }
    };

    tracing::debug!(
      id_marcaje = id,
      "Se ha completado satisfactoriamente el marcaje horario"
    );

    Ok(id)
  }

  /// Actualiza el campo modificado_por del marcaje
  ///
  /// Devuelve True si se actualizo
  #[inline]
  pub async fn actualizar_modificado_por(
    &self,
    trans: &mut Transaccion<'_>,
    id: u32,
    modificar_por: u32,
  ) -> Result<bool, ServicioError> {
    self
      .repo
      .actualizar_modificado_por(trans, id, modificar_por)
      .await
      .map_err(|err| {
        tracing::error!(
          id_marcaje = id,
          modificar_por = modificar_por,
          error = %err,
          "Actualizando modificado_por del marcaje"
        );
        ServicioError::from(err)
      })
  }

  /// Marca un marcaje como eliminado
  ///
  /// Devuelve True si se actualizo
  #[inline]
  pub async fn marcar_marcaje_eliminado(
    &self,
    trans: &mut Transaccion<'_>,
    id: u32,
  ) -> Result<bool, ServicioError> {
    self
      .repo
      .marcar_marcaje_eliminado(trans, id)
      .await
      .map_err(|err| {
        tracing::error!(
          id_marcaje = id,
          error = %err,
          "Marcando el marcaje como eliminado"
        );
        ServicioError::from(err)
      })
  }

  /// Obtiene los marcaje dado el usuario y la fecha para
  /// el registrador que no tengan asigandas una incidencia
  #[inline]
  pub async fn marcajes_inc_por_fecha_reg(
    &self,
    usuario: u32,
    fecha: NaiveDate,
    usuario_reg: Option<u32>,
  ) -> Result<DominiosWithCacheUsuario<Marcaje>, ServicioError> {
    self
      .repo
      .marcajes_inc_por_fecha_reg(usuario, fecha, usuario_reg)
      .await
      .map_err(|err| {
        tracing::error!(
          usuario = usuario,
          fecha = ?fecha,
          usuario_reg = ?usuario_reg,
          error = %err,
          "Obteniendo los marcajes por fecha sin incidencias"
        );
        ServicioError::from(err)
      })
  }

  /// Obtiene el marcaje dado un usuario y la fecha
  #[inline]
  pub async fn marcaje_por_fecha(
    &self,
    usuario: u32,
    fecha: NaiveDate,
  ) -> Result<DominiosWithCacheUsuario<Marcaje>, ServicioError> {
    self
      .repo
      .marcajes_por_fecha(usuario, fecha)
      .await
      .map_err(|err| {
        tracing::error!(
          usuario = usuario,
          fecha = ?fecha,
          error = %err,
          "Obteniendo el marcaje por usuario y fecha"
        );
        ServicioError::from(err)
      })
  }

  /// Obtiene los últimos marcajes horarios de un usuario.
  #[inline]
  pub async fn ultimos_marcajes(
    &self,
    usuario: u32,
  ) -> Result<DominiosWithCacheUsuario<Marcaje>, ServicioError> {
    self
      .repo
      .ultimos_marcajes(
        usuario,
        Some(&self.cnfg.limites.ultimos_marcajes.to_string()),
      )
      .await
      .map_err(|err| {
        tracing::error!(
          usuario = usuario,
          error = %err,
          "Obteniendo los últimos marcaje horarios del usuario"
        );
        ServicioError::from(err)
      })
  }

  /// Valida añadir un nuevo marcaje
  ///
  /// Se puede excluir un marcaje pasado como parámetro
  /// Si no quiere excluir ningún marcaje use 0
  /// La exclusión puede ser muy útil cuando se quiere
  /// realizar una modificación de este marcaje
  async fn validar_agregacion(
    &self,
    reg: &Marcaje,
    excluir_marcaje_id: u32,
  ) -> Result<(), ServicioError> {
    if self
      .repo
      .hora_fin_vacia(reg.usuario, reg.fecha, excluir_marcaje_id)
      .await
      .map_err(ServicioError::from)?
    {
      return Err(ServicioError::Usuario(format!(
        "No puede se puede añadir un marcaje horario \
        con alguna hora de fin sin registrar \
        para el usuario: {} en la fecha: {}. \
        Por favor, registre antes la hora de fin.",
        &reg.usuario,
        &reg.fecha.formato_corto()
      )));
    }

    if self
      .repo
      .hora_asignada(
        reg.usuario,
        reg.fecha,
        reg.hora_inicio,
        excluir_marcaje_id,
      )
      .await
      .map_err(ServicioError::from)?
    {
      return Err(ServicioError::Usuario(format!(
        "La hora de inicio: {} se encuentra entre un rango de horas \
        ya registrado para el usuario: {} en la fecha: {}",
        reg.hora_inicio,
        &reg.usuario,
        &reg.fecha.formato_corto()
      )));
    }

    if let Some(hora_fin) = reg.hora_fin {
      let hora_asignada = self
        .repo
        .horas_solapadas(
          reg.usuario,
          reg.fecha,
          reg.hora_inicio,
          hora_fin,
          excluir_marcaje_id,
        )
        .await
        .map_err(ServicioError::from)?;

      if hora_asignada {
        return Err(ServicioError::Usuario(format!(
          "Ya existe un rango horario que se solapa con el \
          marcaje del usuario: {} en la fecha: {} desde: {} hasta: {}",
          &reg.usuario,
          &reg.fecha.formato_corto(),
          reg.hora_inicio,
          hora_fin
        )));
      }
    }

    Ok(())
  }
}
