use std::sync::Arc;

use axum::{Router, extract::State, routing::get};

use crate::{
  app,
  config::ConfigTrabajo,
  infraestructura::PoolConexion,
  registro::{Registro, RegistroRepo, RegistroServicio, TrazaRepo},
  usuarios::{HorarioRepo, Usuario, UsuarioServicio},
};

/// Estructura principal de la aplicación que contiene los servicios.
pub struct AppState {
  pub reg_servicio: RegistroServicio,
  #[allow(dead_code)]
  pub usuario_servicio: UsuarioServicio,
}

impl AppState {
  /// Inicia la aplicación y devuelve una instancia de `App`.
  pub fn iniciar(cnf: ConfigTrabajo, pool: PoolConexion) -> Self {
    AppState {
      usuario_servicio: UsuarioServicio::new(HorarioRepo::new(pool.clone())),
      reg_servicio: RegistroServicio::new(
        cnf,
        RegistroRepo::new(pool.clone()),
        TrazaRepo::new(),
        UsuarioServicio::new(HorarioRepo::new(pool.clone())),
      ),
    }
  }
}

/// Define las rutas de la aplicación.
pub fn rutas(app: Arc<AppState>) -> Router {
  Router::new()
    .route("/", get(app::registrar))
    .with_state(app)
}

/// Ruta para crear un nuevo registro de empleado completo.
pub async fn registrar(State(state): State<Arc<AppState>>) -> &'static str {
  let usuario_log = Usuario {
    id: 2, // Aquí deberías obtener el ID del usuario logueado
    nombre: "Pepe Gomez".to_string(),
  };

  let usuario_reg = Usuario {
    id: 1, // Aquí deberías obtener el ID del usuario logueado
    nombre: "David Suárez Pascual".to_string(),
  };

  let registro = Registro {
    usuario: usuario_reg,
    fecha: chrono::Utc::now().naive_utc().date(),
    hora_inicio: chrono::NaiveTime::from_hms_opt(9, 1, 0).unwrap(),
    hora_fin: chrono::NaiveTime::from_hms_opt(10, 30, 0),
    horas_a_trabajar: 0,
  };

  state
    .reg_servicio
    .agregar(&usuario_log, registro)
    .await
    .expect("Error al agregar el registro");

  "Registro añadido correctamente"
}
