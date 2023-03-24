pub mod db;
pub mod models;
pub mod routes;

use ::firestore::FirestoreDb;
use actix_identity::Identity;
use actix_web::cookie::Cookie;
use actix_web::cookie::CookieJar;
use actix_web::cookie::Key;
use actix_web::error::Error as ActixError;
use actix_web::HttpMessage;
use actix_web::HttpRequest;
use core::future::Future;

use crate::config::Config;
use crate::error::Error;
use models::Participant;

pub async fn new(
  config: &Config,
  firestore: &FirestoreDb,
  identity: impl Future<Output = Result<Identity, ActixError>>,
  req: HttpRequest,
) -> Result<Participant, Error> {
  let identity = identity.await;
  Ok(match identity {
    Ok(s) => Participant {
      id: s.id().unwrap(),
    },
    _ => {
      let participant;

      // check for a legacy session
      if let Some(legacy_session_cookie) = req.cookie("__session") {
        let legacy_session = legacy_session_cookie.value().to_string();
        let mut jar = CookieJar::new();
        jar.add(Cookie::new("__session", legacy_session));
        let key = Key::derive_from(&config.secret_key);
        participant = Participant {
          id: jar
            .private(&key)
            .get("__session")
            .unwrap()
            .value()
            .to_string(),
        };
      } else {
        // create a new participant
        participant = db::new(firestore).await?;
      }

      Identity::login(&req.extensions(), participant.id.clone())?;
      participant
    }
  })
}
