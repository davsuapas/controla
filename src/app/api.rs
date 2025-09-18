use std::sync::Arc;

use axum::{
  Extension, Router,
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
      DescriptorUsuarioDTO, HorarioDTO, PasswordDniDTO, PasswordUsuarioDTO,
      RegistroInDTO, RegistroOutDTO, UsuarioDTO, vec_dominio_to_dtos,
    },
  },
  infra::{Dni, Password},
  usuarios::Rol,
};

/// Define las rutas de la aplicación.
pub fn rutas(app: Arc<AppState>) -> Router {
  // Rutas públicas (sin autenticación)
  let rutas_auth = Router::new().route("/usuarios/login", post(login));

  // Rutas seguras (con autenticación)
  let rutas_privadas = Router::new()
    .route("/usuarios/{id}/logout", get(logout))
    .route("/usuarios", post(crear_usuario))
    .route("/usuarios", put(actualizar_usuario))
    .route("/usuarios/password", put(actualizar_passw_usuario))
    .route("/usuarios", get(usuarios))
    .route("/usuarios/{id}", get(usuario))
    .route("/usuarios/{id}/ultimos_registros", get(ultimos_registros))
    .route("/usuarios/{id}/horario", get(horario_usuario))
    .route(
      "/usuarios/{id}/horario/{fecha}",
      get(horario_usuario_por_fecha),
    )
    .route("/roles/{id}/usuarios", get(usuarios_por_rol))
    .route("/registros", post(registrar))
    .layer(axum::middleware::from_fn(
      crate::infra::middleware::autenticacion,
    ));

  Router::new()
    .nest("/auth", rutas_auth)
    .nest("/api", rutas_privadas)
    .layer(Extension(app.manejador_sesion.clone()))
    .with_state(app)
}

/// Api para des-logear al usuario
///
/// Elimina la cookie del cliente.
async fn logout(
  State(state): State<Arc<AppState>>,
  Path(id): Path<u32>,
) -> impl IntoResponse {
  tracing::info!("Logout para el usuario: {}", id);

  (
    [(
      axum::http::header::SET_COOKIE,
      state.manejador_sesion.eliminar_sesion().to_string(),
    )],
    StatusCode::NO_CONTENT,
  )
}

/// Api para logear el usuario
///
/// Verifica que se cumpla la clave y si es correcto envía
/// la información del usuario y la cookie de sesión.
/// En caso contrario devuelve un estado: UNAUTHORIZED.
async fn login(
  State(state): State<Arc<AppState>>,
  Json(params): Json<PasswordDniDTO>,
) -> impl IntoResponse {
  let result = state
    .usuario_servicio
    .login_usuario(&Dni::new(params.dni), &Password::new(params.password))
    .await;

  match result {
    Ok(usuario) => {
      if let Some(usr) = usuario {
        // Crear token de sesión
        let token_cookie = match state.manejador_sesion.crear_sesion() {
          Ok(token) => token,
          Err(err) => {
            tracing::error!(
              usuario = ?usr, error = ?err,
              "Error al crear sesión");

            return Err((
              StatusCode::INTERNAL_SERVER_ERROR,
              "@@:Error al crear la sesión. \
              Intentelo de nuevo y si persiste el error \
              contacte con el administrador"
                .to_string(),
            ));
          }
        };

        // Devolver respuesta con cookie y datos del usuario
        Ok((
          [(axum::http::header::SET_COOKIE, token_cookie.to_string())],
          Json(UsuarioDTO::from(usr)),
        ))
      } else {
        Err((
          StatusCode::UNAUTHORIZED,
          "Usuario no autorizado".to_string(),
        ))
      }
    }
    Err(_) => Err((
      StatusCode::UNAUTHORIZED,
      "Usuario no autorizado".to_string(),
    )),
  }
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

/// Api para actualizar la password de un usuario existente
async fn actualizar_passw_usuario(
  State(state): State<Arc<AppState>>,
  Json(passw): Json<PasswordUsuarioDTO>,
) -> impl IntoResponse {
  state
    .usuario_servicio
    .actualizar_password(passw.id, &Password::new(passw.password))
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
