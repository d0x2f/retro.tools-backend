mod db;
pub mod models;
pub mod routes;

use firestore::FirestoreDb;

use crate::boards;
use crate::error::Error;
use crate::participants::models::Participant;

pub async fn assert_card_owner(
  firestore: &FirestoreDb,
  participant: &Participant,
  card: &models::Card,
  board_id: String,
) -> Result<(), Error> {
  let board = boards::db::get(firestore, board_id).await?;
  if board.owner == participant.id || card.owner == participant.id {
    Ok(())
  } else {
    Err(Error::Forbidden)
  }
}
