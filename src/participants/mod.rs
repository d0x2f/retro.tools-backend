pub mod db;
pub mod models;
pub mod routes;

use ::firestore::FirestoreDb;
use actix_identity::Identity;
use actix_web::cookie::Cookie;
use actix_web::cookie::CookieJar;
use actix_web::cookie::Key;
use actix_web::error::Error as ActixError;
use actix_web::web::Data;
use actix_web::HttpMessage;
use actix_web::HttpRequest;
use core::future::Future;

use crate::config::Config;
use crate::error::Error;
use models::Participant;

fn extract_legacy_session(req: &HttpRequest) -> Option<Participant> {
  let config = req.app_data::<Data<Config>>().unwrap();
  let legacy_session_cookie = req.cookie("__session")?;
  let legacy_session = legacy_session_cookie.value().to_string();
  let mut jar = CookieJar::new();
  jar.add(Cookie::new("__session", legacy_session));
  let key = Key::derive_from(&config.secret_key);
  Some(Participant {
    id: jar.private(&key).get("__session")?.value().to_string(),
  })
}

pub async fn new(
  identity: impl Future<Output = Result<Identity, ActixError>>,
  req: HttpRequest,
) -> Result<Participant, Error> {
  let identity = identity.await;
  Ok(match identity {
    Ok(s) => Participant {
      id: s.id().unwrap(),
    },
    _ => {
      let firestore = req.app_data::<Data<FirestoreDb>>().unwrap();
      let participant = extract_legacy_session(&req).unwrap_or(db::new(firestore).await?);
      Identity::login(&req.extensions(), participant.id.clone())?;
      participant
    }
  })
}
