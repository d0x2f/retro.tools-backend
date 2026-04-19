mod db;
pub mod models;
pub mod routes;

use firestore::{FirestoreDb, FirestoreReference};

use crate::boards;
use crate::error::Error;
use crate::participants::models::Participant;

pub async fn assert_card_owner(
  firestore: &FirestoreDb,
  participant: &Participant,
  card: &models::Card,
  board_id: &String,
) -> Result<(), Error> {
  let board = boards::db::get(firestore, board_id).await?;
  let participant_reference = FirestoreReference(format!(
    "{}/participants/{}",
    firestore.get_documents_path(),
    participant.id
  ));
  if board.owner == participant_reference || card.owner == participant_reference {
    Ok(())
  } else {
    Err(Error::Forbidden)
  }
}
