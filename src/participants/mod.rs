pub mod db;
pub mod models;

use actix_http::error::Error as ActixError;
use actix_identity::Identity;
use futures::future::Ready;

use crate::error::Error;
use crate::firestore::FirestoreV1Client;
use models::Participant;

pub async fn new(
  mut firestore: FirestoreV1Client,
  identity: Ready<Result<Identity, ActixError>>,
) -> Result<Participant, Error> {
  let identity = identity.await?;
  Ok(match identity.identity() {
    Some(s) => Participant { id: s },
    None => {
      let participant = db::new(&mut firestore).await?;
      identity.remember(participant.id.clone());
      participant
    }
  })
}