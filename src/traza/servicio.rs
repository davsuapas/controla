use crate::{
  infra::{ServicioError, Transaccion},
  traza::{Traza, TrazaRepo},
};

pub struct TrazaServicio {
  traza_repo: TrazaRepo,
}

impl TrazaServicio {
  pub fn new(traza_repo: TrazaRepo) -> Self {
    TrazaServicio { traza_repo }
  }
}

impl TrazaServicio {
  /// Agrega una nueva traza al sistema.
  ///
  /// Las trazas se suelen agregar dentro de
  /// una transacción que afecta a otros datos.
  /// Es necesario proporcionar una transacción
  /// para asegurar la consistencia de los datos.
  #[inline]
  pub async fn agregar(
    &self,
    trans: &mut Transaccion<'_>,
    traza: &Traza,
  ) -> Result<u32, ServicioError> {
    self.traza_repo.agregar(trans, traza).await.map_err(|err| {
      tracing::error!(
        traza = ?traza,
        error = %err,
        "Agregando una nueva traza al sistema"
      );
      ServicioError::from(err)
    })
  }
}
