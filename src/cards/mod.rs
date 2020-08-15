mod db;
pub mod models;
pub mod routes;

use crate::boards;
use crate::config::Config;
use crate::error::Error;
use crate::firestore::FirestoreV1Client;
use crate::participants::models::Participant;

pub async fn assert_card_owner(
  firestore: &mut FirestoreV1Client,
  config: &Config,
  participant: &Participant,
  card: &models::Card,
  board_id: String,
) -> Result<(), Error> {
  let board = boards::db::get(firestore, config, board_id).await?;
  if board.owner == participant.id || card.owner == participant.id {
    Ok(())
  } else {
    Err(Error::Forbidden)
  }
}
