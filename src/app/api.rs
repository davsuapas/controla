use std::sync::Arc;

use axum::{
  Router,
  extract::{Json, Path, State},
  http::StatusCode,
  response::IntoResponse,
  routing::{get, post},
};
use chrono::NaiveDateTime;
use serde::Deserialize;

use crate::app::{
  AppState,
  dto::{RegistroDTO, UsuarioDTO, horarios_to_dtos},
};

/// Define las rutas de la aplicación.
pub fn rutas(app: Arc<AppState>) -> Router {
  let api_rutas = Router::new()
    .route("/registro", post(registrar))
    .route("/usuario/{id}/horario", get(horario_usuario))
    .route(
      "/usuario/{id}/horario/{fecha}",
      get(horario_usuario_por_fecha),
    );

  Router::new().nest("/api", api_rutas).with_state(app)
}

/// Api para crear un nuevo registro de empleado completo.
async fn registrar(
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

/// Api para obtener el horario de un usuario completo.
async fn horario_usuario(
  State(state): State<Arc<AppState>>,
  Path(usuario): Path<u64>,
) -> impl IntoResponse {
  state
    .usuario_servicio
    .horario_usuario(usuario, None)
    .await
    .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.mensaje_usuario()))
    .map(|horarios| Json(horarios_to_dtos(horarios)))
}

#[derive(Deserialize)]
struct HorarioUsuarioParams {
  id: u64,
  fecha: NaiveDateTime,
}

/// Api para obtener el horario de un usuario dada una fecha y hora.
async fn horario_usuario_por_fecha(
  State(state): State<Arc<AppState>>,
  Path(params): Path<HorarioUsuarioParams>,
) -> impl IntoResponse {
  state
    .usuario_servicio
    .horario_usuario(params.id, Some(params.fecha))
    .await
    .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.mensaje_usuario()))
    .map(|horarios| Json(horarios_to_dtos(horarios)))
}
