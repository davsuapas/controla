use std::sync::Arc;

use axum::{
  Router,
  extract::{Json, Path, State},
  http::StatusCode,
  response::IntoResponse,
  routing::{get, post, put},
};
use chrono::NaiveDateTime;
use serde::Deserialize;

use crate::{
  app::{
    AppState,
    dto::{
      DescriptorUsuarioDTO, HorarioDTO, RegistroInDTO, RegistroOutDTO,
      UsuarioDTO, vec_dominio_to_dtos,
    },
  },
  infra::Password,
  usuarios::Rol,
};

/// Define las rutas de la aplicación.
pub fn rutas(app: Arc<AppState>) -> Router {
  let api_rutas = Router::new()
    .route("/usuarios", post(crear_usuario))
    .route("/usuarios", put(actualizar_usuario))
    .route(
      "/usuarios/{id}/password/{password}",
      put(actualizar_passw_usuario),
    )
    .route("/usuarios", get(usuarios))
    .route("/usuarios/{id}", get(usuario))
    .route("/usuarios/{id}/ultimos_registros", get(ultimos_registros))
    .route("/usuarios/{id}/horario", get(horario_usuario))
    .route(
      "/usuarios/{id}/horario/{fecha}",
      get(horario_usuario_por_fecha),
    )
    .route("/roles/{id}/usuarios", get(usuarios_por_rol))
    .route("/registros", post(registrar));

  Router::new().nest("/api", api_rutas).with_state(app)
}

/// Api para crear un nuevo usuario
async fn crear_usuario(
  State(state): State<Arc<AppState>>,
  Json(usuario): Json<UsuarioDTO>,
) -> impl IntoResponse {
  state
    .usuario_servicio
    .crear_usuario(usuario.autor, &usuario.into())
    .await
    .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.mensaje_usuario()))
    .map(|id| (StatusCode::CREATED, Json(id)))
}

/// Api para actualizar un usuario existente
async fn actualizar_usuario(
  State(state): State<Arc<AppState>>,
  Json(usuario): Json<UsuarioDTO>,
) -> impl IntoResponse {
  state
    .usuario_servicio
    .actualizar_usuario(usuario.autor, &usuario.into())
    .await
    .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.mensaje_usuario()))
    .map(|_| StatusCode::NO_CONTENT)
}

#[derive(Deserialize)]
struct PasswordUsuarioParams {
  id: u32,
  password: String,
}

/// Api para actualizar la password de un usuario existente
async fn actualizar_passw_usuario(
  State(state): State<Arc<AppState>>,
  Path(params): Path<PasswordUsuarioParams>,
) -> impl IntoResponse {
  state
    .usuario_servicio
    .actualizar_password(params.id, &Password::new(params.password))
    .await
    .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.mensaje_usuario()))
    .map(|_| StatusCode::NO_CONTENT)
}

/// Api para obtener todos los usuarios
async fn usuarios(State(state): State<Arc<AppState>>) -> impl IntoResponse {
  state
    .usuario_servicio
    .usuarios()
    .await
    .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.mensaje_usuario()))
    .map(|usrs| Json(vec_dominio_to_dtos::<_, UsuarioDTO>(usrs)))
}

async fn usuario(
  State(state): State<Arc<AppState>>,
  Path(id): Path<u32>,
) -> impl IntoResponse {
  state
    .usuario_servicio
    .usuario(id)
    .await
    .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.mensaje_usuario()))
    .map(|u| Json(UsuarioDTO::from(u)))
}

/// Api para obtener los últimos registros horarios de un usuario.
async fn ultimos_registros(
  State(state): State<Arc<AppState>>,
  Path(usuario): Path<u32>,
) -> impl IntoResponse {
  state
    .reg_servicio
    .ultimos_registros(usuario)
    .await
    .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.mensaje_usuario()))
    .map(|regs| Json(vec_dominio_to_dtos::<_, RegistroOutDTO>(regs)))
}

/// Api para obtener el horario de un usuario completo.
async fn horario_usuario(
  State(state): State<Arc<AppState>>,
  Path(usuario): Path<u32>,
) -> impl IntoResponse {
  state
    .usuario_servicio
    .horario_usuario(usuario, None)
    .await
    .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.mensaje_usuario()))
    .map(|horarios| Json(vec_dominio_to_dtos::<_, HorarioDTO>(horarios)))
}

#[derive(Deserialize)]
struct HorarioUsuarioParams {
  id: u32,
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
    .map(|horarios| Json(vec_dominio_to_dtos::<_, HorarioDTO>(horarios)))
}

/// Api para obtener los usuarios que tienen un rol específico.
async fn usuarios_por_rol(
  State(state): State<Arc<AppState>>,
  Path(id): Path<u8>,
) -> impl IntoResponse {
  state
    .usuario_servicio
    .usuarios_por_rol(Rol::from(id))
    .await
    .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.mensaje_usuario()))
    .map(|usrs| Json(vec_dominio_to_dtos::<_, DescriptorUsuarioDTO>(usrs)))
}

/// Api para crear un nuevo registro de empleado completo.
async fn registrar(
  State(state): State<Arc<AppState>>,
  Json(reg): Json<RegistroInDTO>,
) -> impl IntoResponse {
  state
    .reg_servicio
    .agregar(&reg.into())
    .await
    .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.mensaje_usuario()))
    .map(|id| (StatusCode::CREATED, Json(id)))
}
