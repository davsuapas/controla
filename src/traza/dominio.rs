use chrono::NaiveDateTime;
use chrono_tz::Tz;

pub struct Traza {
  pub usuario_id: u64,
  pub fecha: NaiveDateTime,
  pub mensaje: String,
}

impl Traza {
  pub fn with_timezone(tz: Tz, user_id: u64, mensaje: String) -> Self {
    Traza {
      usuario_id: user_id,
      fecha: chrono::Utc::now().with_timezone(&tz).naive_local(),
      mensaje,
    }
  }
}
