pub mod db;
pub mod models;
pub mod routes;

use actix_identity::Identity;
use actix_web::error::Error as ActixError;
use actix_web::HttpMessage;
use actix_web::HttpRequest;
use core::future::Future;
use futures::lock::Mutex;
use gcp_auth::AuthenticationManager;
use std::sync::Arc;

use crate::config::Config;
use crate::error::Error;
use crate::firestore;
use models::Participant;

pub async fn new(
  config: Config,
  identity: impl Future<Output = Result<Identity, ActixError>>,
  req: HttpRequest,
) -> Result<Participant, Error> {
  let auth = AuthenticationManager::new().await.unwrap();
  let firestore = firestore::get_client(auth).await?;
  let identity = identity.await;
  Ok(match identity {
    Ok(s) => Participant {
      id: s.id().unwrap(),
    },
    _ => {
      let participant = db::new(Arc::new(Mutex::new(firestore)), &config).await?;
      Identity::login(&req.extensions(), participant.id.clone())?;
      participant
    }
  })
}
