pub mod db;
pub mod models;

use actix_http::error::Error as ActixError;
use actix_identity::Identity;
use futures::future::Ready;

use crate::config::Config;
use crate::error::Error;
use crate::firestore::FirestoreV1Client;
use models::Participant;

pub async fn new(
  mut firestore: FirestoreV1Client,
  config: Config,
  identity: Ready<Result<Identity, ActixError>>,
) -> Result<Participant, Error> {
  let identity = identity.await?;
  Ok(match identity.identity() {
    Some(s) => Participant { id: s },
    None => {
      let participant = db::new(&mut firestore, &config).await?;
      identity.remember(participant.id.clone());
      participant
    }
  })
}
