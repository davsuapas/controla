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
      DescriptorUsuarioDTO, DominiosWithCacheUsuarioDTO, HorarioOutDTO,
      IncidenciaDTO, MarcajeInDTO, MarcajeOutDTO, PasswordDniDTO,
      PasswordUsuarioDTO, UsuarioDTO, vec_dominio_to_dtos,
    },
  },
  infra::{Dni, Password},
  usuarios::Rol,
};

#[derive(Deserialize)]
struct UsuarioFechaParams {
  id: u32,
  fecha: NaiveDateTime,
}

#[derive(Deserialize)]
struct UsuarioFechaRegParams {
  id: u32,
  fecha: NaiveDateTime,
  usuario_reg: u32,
}

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
    .route(
      "/usuarios/{id}/marcajes/por/fecha/{fecha}",
      get(marcaje_por_fecha),
    )
    .route("/usuarios/{id}/ultimos_marcajes", get(ultimos_marcajes))
    .route(
      "/usuarios/{id}/horario/sin/asignar/{fecha}",
      get(horario_usuario_sin_asignar),
    )
    .route(
      "/usuarios/{id}/horario/cercano/{fecha}",
      get(horario_cercano),
    )
    .route(
      "/usuarios/{id}/marcajes/sin/inc/{fecha}",
      get(marcaje_sin_inc_por_fecha),
    )
    .route(
      "/usuarios/{id}/marcajes/sin/inc/{fecha}/registrador/{usuario_reg}",
      get(marcaje_sin_inc_por_fecha_reg),
    )
    .route("/roles/{id}/usuarios", get(usuarios_por_rol))
    .route("/marcajes", post(registrar))
    .route("/incidencias", post(crear_incidencia))
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

/// Api para obtener el marcaje sin incidencias por fecha
async fn marcaje_sin_inc_por_fecha(
  State(state): State<Arc<AppState>>,
  Path(param): Path<UsuarioFechaParams>,
) -> impl IntoResponse {
  state
    .marcaje_servicio
    .marcajes_inc_por_fecha_reg(param.id, param.fecha.date(), None)
    .await
    .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.mensaje_usuario()))
    .map(|regs| Json(DominiosWithCacheUsuarioDTO::<MarcajeOutDTO>::from(regs)))
}

/// Api para obtener el marcjae sin incidencias
/// por fecha y marcaje creado por un usuario registrador
async fn marcaje_sin_inc_por_fecha_reg(
  State(state): State<Arc<AppState>>,
  Path(param): Path<UsuarioFechaRegParams>,
) -> impl IntoResponse {
  state
    .marcaje_servicio
    .marcajes_inc_por_fecha_reg(
      param.id,
      param.fecha.date(),
      Some(param.usuario_reg),
    )
    .await
    .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.mensaje_usuario()))
    .map(|regs| Json(DominiosWithCacheUsuarioDTO::<MarcajeOutDTO>::from(regs)))
}

/// Api para obtener el registro por usuario y fecha.
async fn marcaje_por_fecha(
  State(state): State<Arc<AppState>>,
  Path(param): Path<UsuarioFechaParams>,
) -> impl IntoResponse {
  state
    .marcaje_servicio
    .marcaje_por_fecha(param.id, param.fecha.date())
    .await
    .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.mensaje_usuario()))
    .map(|regs| Json(DominiosWithCacheUsuarioDTO::<MarcajeOutDTO>::from(regs)))
}

/// Api para obtener los últimos marcajes horarios de un usuario.
async fn ultimos_marcajes(
  State(state): State<Arc<AppState>>,
  Path(usuario): Path<u32>,
) -> impl IntoResponse {
  state
    .marcaje_servicio
    .ultimos_marcajes(usuario)
    .await
    .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.mensaje_usuario()))
    .map(|regs| Json(DominiosWithCacheUsuarioDTO::<MarcajeOutDTO>::from(regs)))
}

/// Api para obtener el horario de un usuario completo.
async fn horario_usuario_sin_asignar(
  State(state): State<Arc<AppState>>,
  Path(params): Path<UsuarioFechaParams>,
) -> impl IntoResponse {
  state
    .usuario_servicio
    .horarios_usuario_sin_asignar(params.id, params.fecha)
    .await
    .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.mensaje_usuario()))
    .map(|horarios| Json(vec_dominio_to_dtos::<_, HorarioOutDTO>(horarios)))
}

/// Api para obtener el horario de un usuario máa próximo
async fn horario_cercano(
  State(state): State<Arc<AppState>>,
  Path(params): Path<UsuarioFechaParams>,
) -> impl IntoResponse {
  state
    .usuario_servicio
    .horario_usuario_cercano(params.id, params.fecha)
    .await
    .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.mensaje_usuario()))
    .map(|horarios| Json(vec_dominio_to_dtos::<_, HorarioOutDTO>(horarios)))
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

/// Api para crear un nuevo marcaje de empleado completo.
async fn registrar(
  State(state): State<Arc<AppState>>,
  Json(reg): Json<MarcajeInDTO>,
) -> impl IntoResponse {
  state
    .marcaje_servicio
    .agregar(&reg.into())
    .await
    .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.mensaje_usuario()))
    .map(|id| (StatusCode::CREATED, Json(id)))
}

// Api para crear una solicitud de incidencia por el usuario
async fn crear_incidencia(
  State(state): State<Arc<AppState>>,
  Json(solicitud): Json<IncidenciaDTO>,
) -> impl IntoResponse {
  state
    .inc_servicio
    .agregar(&solicitud.into())
    .await
    .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.mensaje_usuario()))
    .map(|id| (StatusCode::CREATED, Json(id)))
}
