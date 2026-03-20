use std::sync::Arc;

use axum::{
  Extension, Router,
  extract::{Json, Path, State},
  http::StatusCode,
  response::IntoResponse,
  routing::{delete, get, post, put},
};
use chrono::{NaiveDate, NaiveDateTime};
use serde::Deserialize;

use crate::{
  app::{
    AppState,
    dto::{
      CalendarioDTO, CalendarioFechaDTO, ConfigHorarioDTO,
      DescriptorUsuarioDTO, DominiosWithCacheUsuarioDTO, HorarioDTO,
      IncidenciaDTO, IncidenciaInProcesoDTO, IncidenciaOutProcesoDTO,
      IncidenciaSolictudDTO, IncidenciasFiltroParams, InformeCumplimientoDTO,
      MarcajeInDTO, MarcajeOutDTO, PasswordDniDTO, PasswordUsuarioDTO,
      UsuarioBodyDTO, UsuarioOutDTO, vec_dominio_to_dtos,
    },
  },
  inc::{EstadoIncidencia, IncidenciaProceso},
  infra::{Dni, Password},
  usuarios::Rol,
};

#[derive(Deserialize)]
pub struct FiltroParams {
  pub usuario: u32,
  pub fecha_inicio: Option<NaiveDate>,
  pub fecha_fin: Option<NaiveDate>,
  pub usuario_reg: Option<u32>,
}

#[derive(Deserialize)]
pub struct InformeCumplimientoParams {
  #[serde(rename = "empleadoId")]
  empleado_id: u32,
  mes: u32,
  anio: i32,
}

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

#[derive(Deserialize)]
pub struct UsuarioParams {
  todos_los_calendarios: bool,
}

#[derive(Deserialize)]
pub struct CalendarioFechaFiltroParams {
  pub fecha_inicio: Option<NaiveDate>,
  pub fecha_fin: Option<NaiveDate>,
}

/// Define las rutas de la aplicación.
///
/// Recibe el código de la aplicación para el tenant.
pub fn rutas(cod_app: &str, app: Arc<AppState>) -> Router {
  // Rutas públicas (sin autenticación)
  let rutas_auth = Router::new().route("/usuarios/login", post(login));

  // Rutas seguras (con autenticación)
  let rutas_privadas = Router::new()
    .route("/usuarios/{id}/logout", get(logout))
    .route("/usuarios", post(crear_usuario))
    .route("/usuarios", put(actualizar_usuario))
    .route("/usuarios/password", put(actualizar_passw_usuario))
    .route(
      "/usuarios/{id}/finalizar/marcaje/{fecha}",
      put(marcaje_finalizar),
    )
    .route("/usuarios", get(usuarios))
    .route("/usuarios/{id}", get(usuario))
    .route(
      "/usuarios/{id}/marcajes/por/fecha/{fecha}",
      get(marcaje_por_fecha),
    )
    .route("/usuarios/{id}/ultimos_marcajes", get(ultimos_marcajes))
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
    .route(
      "/usuarios/{id}/marcajes/fecha/{fecha}/sin/finalizar",
      get(marcaje_sin_finalizar),
    )
    .route("/usuarios/{id}/horarios", get(config_horarios))
    .route(
      "/usuarios/{id}/horarios/duplicar/{fecha}",
      post(duplicar_config_horario),
    )
    .route("/horarios/{id}", get(horario))
    .route("/horarios", post(crear_horario))
    .route("/horarios", put(modificar_config_horario))
    .route("/horarios/{id}", delete(eliminar_config_horario))
    .route("/roles/{id}/usuarios", get(usuarios_por_rol))
    .route("/marcajes", post(registrar))
    .route("/marcajes/entre/fechas", post(marcajes_entre_fechas))
    .route("/incidencias", post(crear_incidencia))
    .route(
      "/incidencias/cambiar/a/solicitud",
      put(cambiar_incidencia_solicitud),
    )
    .route("/incidencias/procesar", post(procesar_incidencias))
    .route("/incidencias/por/fechas", post(incidencias_por_fechas))
    .route("/calendarios", get(calendarios))
    .route("/calendarios/{id}", get(calendario))
    .route("/calendarios", post(crear_calendario))
    .route("/calendarios", put(actualizar_calendario))
    .route("/calendarios/{id}", delete(eliminar_calendario))
    .route("/calendarios/{id}/fechas", get(fechas_calendario))
    .route("/calendarios/fechas/{id}", get(fecha_calendario))
    .route("/calendarios/fechas", post(crear_fecha_calendario))
    .route("/calendarios/fechas", put(actualizar_fecha_calendario))
    .route(
      "/calendarios/fechas/{id}",
      delete(eliminar_fecha_calendario),
    )
    .route(
      "/informes/cumplimiento/horario",
      get(informe_cumplimiento_horario),
    )
    .layer(axum::middleware::from_fn(
      crate::infra::middleware::autenticacion,
    ));

  let ruta_app = if cod_app.is_empty() {
    String::new()
  } else {
    format!("/{}", cod_app)
  };

  Router::new()
    .nest(format!("{}/auth", ruta_app).as_str(), rutas_auth)
    .nest(format!("{}/api", ruta_app).as_str(), rutas_privadas)
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
/// Verifica que la clave sea correcta y si es correcto envía
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
          Json(UsuarioOutDTO::from(usr)),
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
  Json(usuario): Json<UsuarioBodyDTO>,
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
  Json(usuario): Json<UsuarioBodyDTO>,
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

/// Api que finaliza un marcaje de un usuario en una fecha determinada
async fn marcaje_finalizar(
  State(state): State<Arc<AppState>>,
  Path(param): Path<UsuarioFechaParams>,
) -> impl IntoResponse {
  state
    .marcaje_servicio
    .finalizar_marcaje(param.id, param.fecha)
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
    .map(|usrs| Json(vec_dominio_to_dtos::<_, UsuarioOutDTO>(usrs)))
}

async fn usuario(
  State(state): State<Arc<AppState>>,
  Path(id): Path<u32>,
  axum::extract::Query(params): axum::extract::Query<UsuarioParams>,
) -> impl IntoResponse {
  state
    .usuario_servicio
    .usuario_con_calendarios(id, params.todos_los_calendarios)
    .await
    .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.mensaje_usuario()))
    .map(|u| Json(UsuarioOutDTO::from(u)))
}

/// Api para obtener los marcajes entre fechas para un usuario
async fn marcajes_entre_fechas(
  State(state): State<Arc<AppState>>,
  Json(param): Json<FiltroParams>,
) -> impl IntoResponse {
  state
    .marcaje_servicio
    .marcajes_entre_fechas_reg(
      param.usuario,
      param.fecha_inicio,
      param.fecha_fin,
      param.usuario_reg,
    )
    .await
    .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.mensaje_usuario()))
    .map(|regs| Json(DominiosWithCacheUsuarioDTO::<MarcajeOutDTO>::from(regs)))
}

/// Api para obtener los marcajes sin incidencias por fecha
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

/// Api para obtener el marcaje sin incidencias
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

/// Api que verifica si un marcaje tiene su hora fin sin marcar para un usuario
///
/// Si no esta finalizado devuelve true sino false
async fn marcaje_sin_finalizar(
  State(state): State<Arc<AppState>>,
  Path(param): Path<UsuarioFechaParams>,
) -> impl IntoResponse {
  state
    .marcaje_servicio
    .hora_fin_vacia(param.id, param.fecha.date())
    .await
    .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.mensaje_usuario()))
    .map(|vacia| {
      if vacia {
        StatusCode::OK
      } else {
        StatusCode::NO_CONTENT
      }
    })
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

/// Api para obtener el horario de un usuario más próximo
async fn horario_cercano(
  State(state): State<Arc<AppState>>,
  Path(params): Path<UsuarioFechaParams>,
) -> impl IntoResponse {
  match state
    .horario_servicio
    .horario_usuario_cercano(params.id, params.fecha.date())
    .await
  {
    Ok(Some(horario)) => Json(HorarioDTO::from(horario)).into_response(),
    Ok(None) => StatusCode::NOT_FOUND.into_response(),
    Err(err) => {
      (StatusCode::INTERNAL_SERVER_ERROR, err.mensaje_usuario()).into_response()
    }
  }
}

/// Api para obtener la configuración de horarios de un usuario.
async fn config_horarios(
  State(state): State<Arc<AppState>>,
  Path(id): Path<u32>,
) -> impl IntoResponse {
  state
    .horario_servicio
    .config_horario(id)
    .await
    .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.mensaje_usuario()))
    .map(|configs| Json(vec_dominio_to_dtos::<_, ConfigHorarioDTO>(configs)))
}

/// Api para duplicar la configuración de horarios de un usuario.
async fn duplicar_config_horario(
  State(state): State<Arc<AppState>>,
  Path((id, fecha)): Path<(u32, NaiveDate)>,
) -> impl IntoResponse {
  state
    .horario_servicio
    .duplicar_config_horario(id, fecha)
    .await
    .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.mensaje_usuario()))
    .map(|configs| Json(vec_dominio_to_dtos::<_, ConfigHorarioDTO>(configs)))
}

/// Api para obtener el horario dado el id.
async fn horario(
  State(state): State<Arc<AppState>>,
  Path(id): Path<u32>,
) -> impl IntoResponse {
  state
    .horario_servicio
    .config_horario_por_id(id)
    .await
    .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.mensaje_usuario()))
    .map(|h| Json(ConfigHorarioDTO::from(h)))
}

/// Api para crear una nueva configuración de horario.
async fn crear_horario(
  State(state): State<Arc<AppState>>,
  Json(config): Json<ConfigHorarioDTO>,
) -> impl IntoResponse {
  state
    .horario_servicio
    .agregar_config_horario(&config.into())
    .await
    .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.mensaje_usuario()))
    .map(|id| (StatusCode::CREATED, Json(id)))
}

/// Api para modificar una configuración de horario.
async fn modificar_config_horario(
  State(state): State<Arc<AppState>>,
  Json(config): Json<ConfigHorarioDTO>,
) -> impl IntoResponse {
  state
    .horario_servicio
    .modificar_config_horario(&config.into())
    .await
    .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.mensaje_usuario()))
    .map(|_| StatusCode::NO_CONTENT)
}

/// Api para eliminar una configuración de horario.
async fn eliminar_config_horario(
  State(state): State<Arc<AppState>>,
  Path(id): Path<u32>,
) -> impl IntoResponse {
  state
    .horario_servicio
    .eliminar_config_horario(id)
    .await
    .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.mensaje_usuario()))
    .map(|_| StatusCode::NO_CONTENT)
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

/// Api para crear una solicitud de incidencia por el usuario
async fn crear_incidencia(
  State(state): State<Arc<AppState>>,
  Json(inc): Json<IncidenciaDTO>,
) -> impl IntoResponse {
  state
    .inc_servicio
    .agregar(&inc.into())
    .await
    .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.mensaje_usuario()))
    .map(|id| (StatusCode::CREATED, Json(id)))
}

/// Api para crear una solicitud desde un estado previo.
///
/// El estado previo viene en la propia incidencia
///
/// Devuelve las incidencias modificada
async fn cambiar_incidencia_solicitud(
  State(state): State<Arc<AppState>>,
  Json(solicitud): Json<IncidenciaSolictudDTO>,
) -> impl IntoResponse {
  let id = solicitud.id;

  state
    .inc_servicio
    .cambiar_estado_a_solicitud(&solicitud.into())
    .await
    .map_err(|err| {
      (StatusCode::INTERNAL_SERVER_ERROR, err.mensaje_usuario())
    })?;

  state
    .inc_servicio
    .incidencias(Some(id), None, None, &[], false, None)
    .await
    .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.mensaje_usuario()))
    .map(|regs| Json(DominiosWithCacheUsuarioDTO::<IncidenciaDTO>::from(regs)))
}

/// Api para obtener el informe de cumplimiento horario
async fn informe_cumplimiento_horario(
  State(state): State<Arc<AppState>>,
  axum::extract::Query(params): axum::extract::Query<InformeCumplimientoParams>,
) -> impl IntoResponse {
  state
    .informe_servicio
    .cumplimiento_horario(params.empleado_id, params.mes, params.anio)
    .await
    .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.mensaje_usuario()))
    .map(|informe| Json(InformeCumplimientoDTO::from(informe)))
}

/// Api para obtener todos los calendarios laborales.
async fn calendarios(State(state): State<Arc<AppState>>) -> impl IntoResponse {
  state
    .horario_servicio
    .calendarios()
    .await
    .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.mensaje_usuario()))
    .map(|cals| Json(vec_dominio_to_dtos::<_, CalendarioDTO>(cals)))
}

/// Api para obtener un calendario laboral por su id.
async fn calendario(
  State(state): State<Arc<AppState>>,
  Path(id): Path<u32>,
) -> impl IntoResponse {
  state
    .horario_servicio
    .calendario(id)
    .await
    .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.mensaje_usuario()))
    .map(|c| Json(CalendarioDTO::from(c)))
}

/// Api para crear un nuevo calendario laboral.
async fn crear_calendario(
  State(state): State<Arc<AppState>>,
  Json(dto): Json<CalendarioDTO>,
) -> impl IntoResponse {
  state
    .horario_servicio
    .crear_calendario(&dto.into())
    .await
    .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.mensaje_usuario()))
    .map(|id| (StatusCode::CREATED, Json(id)))
}

/// Api para actualizar un calendario laboral existente.
async fn actualizar_calendario(
  State(state): State<Arc<AppState>>,
  Json(dto): Json<CalendarioDTO>,
) -> impl IntoResponse {
  state
    .horario_servicio
    .actualizar_calendario(&dto.into())
    .await
    .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.mensaje_usuario()))
    .map(|_| StatusCode::NO_CONTENT)
}

/// Api para eliminar un calendario laboral.
async fn eliminar_calendario(
  State(state): State<Arc<AppState>>,
  Path(id): Path<u32>,
) -> impl IntoResponse {
  state
    .horario_servicio
    .eliminar_calendario(id)
    .await
    .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.mensaje_usuario()))
    .map(|_| StatusCode::NO_CONTENT)
}

/// Api para obtener las fechas señaladas de un calendario.
///
/// Permite filtrar por fecha de inicio y fin.
async fn fechas_calendario(
  State(state): State<Arc<AppState>>,
  Path(id): Path<u32>,
  axum::extract::Query(params): axum::extract::Query<
    CalendarioFechaFiltroParams,
  >,
) -> impl IntoResponse {
  state
    .horario_servicio
    .calendario_fechas(id, params.fecha_inicio, params.fecha_fin)
    .await
    .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.mensaje_usuario()))
    .map(|fechas| Json(vec_dominio_to_dtos::<_, CalendarioFechaDTO>(fechas)))
}

/// Api para obtener una fecha señalada por su id.
async fn fecha_calendario(
  State(state): State<Arc<AppState>>,
  Path(id): Path<u32>,
) -> impl IntoResponse {
  state
    .horario_servicio
    .calendario_fecha(id)
    .await
    .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.mensaje_usuario()))
    .map(|f| Json(CalendarioFechaDTO::from(f)))
}

/// Api para crear una nueva fecha señalada en un calendario.
async fn crear_fecha_calendario(
  State(state): State<Arc<AppState>>,
  Json(dto): Json<CalendarioFechaDTO>,
) -> impl IntoResponse {
  state
    .horario_servicio
    .crear_calendario_fecha(&dto.into())
    .await
    .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.mensaje_usuario()))
    .map(|id| (StatusCode::CREATED, Json(id)))
}

/// Api para actualizar una fecha señalada existente.
async fn actualizar_fecha_calendario(
  State(state): State<Arc<AppState>>,
  Json(dto): Json<CalendarioFechaDTO>,
) -> impl IntoResponse {
  state
    .horario_servicio
    .actualizar_calendario_fecha(&dto.into())
    .await
    .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.mensaje_usuario()))
    .map(|_| StatusCode::NO_CONTENT)
}

/// Api para eliminar una fecha señalada.
async fn eliminar_fecha_calendario(
  State(state): State<Arc<AppState>>,
  Path(id): Path<u32>,
) -> impl IntoResponse {
  state
    .horario_servicio
    .eliminar_calendario_fecha(id)
    .await
    .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.mensaje_usuario()))
    .map(|_| StatusCode::NO_CONTENT)
}

/// Api para procesar las incidencias.
///
/// Devuelve las incidencias según el filtro como parámetro
/// y un array de incidencias con errores fatales.
async fn procesar_incidencias(
  State(state): State<Arc<AppState>>,
  Json(entrada): Json<IncidenciaInProcesoDTO>,
) -> Result<(StatusCode, Json<IncidenciaOutProcesoDTO>), (StatusCode, String)> {
  let incidencias_vec: Vec<IncidenciaProceso> =
    entrada.incidencias.into_iter().map(|i| i.into()).collect();

  let incs_erroneas = match state
    .inc_servicio
    .procesar_incidencias(entrada.usuario_gestor, incidencias_vec.as_slice())
    .await
  {
    Ok(v) => v,
    Err(err) => {
      return Err((StatusCode::INTERNAL_SERVER_ERROR, err.mensaje_usuario()));
    }
  };

  let estados_vec: Vec<EstadoIncidencia> = entrada
    .param_filtro_inc
    .estados
    .into_iter()
    .map(EstadoIncidencia::from)
    .collect();

  let incs = match state
    .inc_servicio
    .incidencias(
      None,
      entrada.param_filtro_inc.fecha_inicio,
      entrada.param_filtro_inc.fecha_fin,
      estados_vec.as_slice(),
      entrada.param_filtro_inc.supervisor,
      entrada.param_filtro_inc.usuario,
    )
    .await
  {
    Ok(regs) => DominiosWithCacheUsuarioDTO::<IncidenciaDTO>::from(regs),
    Err(err) => {
      return Err((StatusCode::INTERNAL_SERVER_ERROR, err.mensaje_usuario()));
    }
  };

  Ok((
    StatusCode::CREATED,
    Json(IncidenciaOutProcesoDTO {
      incidencias_erroneas: incs_erroneas,
      incidencias: incs,
    }),
  ))
}

/// Devuelve las incidencias filtradas por una serie de filtros
async fn incidencias_por_fechas(
  State(state): State<Arc<AppState>>,
  Json(param): Json<IncidenciasFiltroParams>,
) -> impl IntoResponse {
  let estados_vec: Vec<EstadoIncidencia> = param
    .estados
    .into_iter()
    .map(EstadoIncidencia::from)
    .collect();

  state
    .inc_servicio
    .incidencias(
      None,
      param.fecha_inicio,
      param.fecha_fin,
      estados_vec.as_slice(),
      param.supervisor,
      param.usuario,
    )
    .await
    .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.mensaje_usuario()))
    .map(|regs| Json(DominiosWithCacheUsuarioDTO::<IncidenciaDTO>::from(regs)))
}
