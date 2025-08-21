use std::sync::Arc;

use axum::{
  Router,
  extract::{Json, State},
  http::StatusCode,
  response::IntoResponse,
  routing::post,
};

use crate::app::{
  AppState,
  dto::{RegistroDTO, UsuarioDTO},
};

/// Define las rutas de la aplicación.
pub fn rutas(app: Arc<AppState>) -> Router {
  let api_rutas = Router::new().route("/registro", post(registrar));

  Router::new().nest("/api", api_rutas).with_state(app)
}

/// Ruta para crear un nuevo registro de empleado completo.
pub(in crate::app) async fn registrar(
  State(state): State<Arc<AppState>>,
  Json(reg): Json<RegistroDTO>,
) -> impl IntoResponse {
  let usuario_log = UsuarioDTO {
    id: 1,
    nombre: "David Suárez Pascual".to_string(),
  };

  let registro = reg.into_dominio(&usuario_log);

  state
    .reg_servicio
    .agregar(&usuario_log.into(), &registro)
    .await
    .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.mensaje_usuario()))
    .map(|_| (StatusCode::OK, ""))
}
