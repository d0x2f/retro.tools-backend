pub mod db;
pub mod models;

use actix_http::error::Error as ActixError;
use actix_identity::Identity;
use futures::future::Ready;

use crate::config::Config;
use crate::error::Error;
use crate::firestore;
use models::Participant;

pub async fn new(
  config: Config,
  identity: Ready<Result<Identity, ActixError>>,
  legacy_id: Option<String>,
) -> Result<Participant, Error> {
  let mut firestore = firestore::get_client().await?;
  let identity = identity.await?;
  Ok(match identity.identity() {
    Some(s) => Participant { id: s },
    None => {
      if let Some(legacy_id) = legacy_id {
        if let Ok(participant) = db::get(&mut firestore, &config, &legacy_id).await {
          identity.remember(participant.id.clone());
          return Ok(participant);
        }
      }
      let participant = db::new(&mut firestore, &config).await?;
      identity.remember(participant.id.clone());
      participant
    }
  })
}
