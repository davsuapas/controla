use crate::{
  inc::{Incidencia, IncidenciaRepo},
  infra::ServicioError,
};

/// Servicio que gestiona las incidencias del usuario
pub struct IncidenciaServicio {
  repo: IncidenciaRepo,
}

impl IncidenciaServicio {
  pub fn new(repo: IncidenciaRepo) -> Self {
    IncidenciaServicio { repo }
  }
}

impl IncidenciaServicio {
  /// Añade una incidencia
  ///
  /// Si la incidencia ya existe devuelve un error
  /// gestionado por los propios constraint de la base
  /// de datos
  #[inline]
  pub async fn agregar(&self, inc: &Incidencia) -> Result<u32, ServicioError> {
    tracing::info!(
      incidencia = ?inc,
      "Se ha iniciado el servicio para crear una incidencia de marcaje");

    let id = match self.repo.agregar(inc).await {
      Ok(reg_id) => reg_id,
      Err(err) => {
        tracing::error!(
          incidencia = ?inc,
          error = %err,
          "Creando incidencia de marcaje"
        );
        return Err(ServicioError::from(err));
      }
    };

    tracing::debug!(
      incidencia = id,
      "Se ha completado satisfactoriamente la creación de la incidencia"
    );

    Ok(id)
  }
}
