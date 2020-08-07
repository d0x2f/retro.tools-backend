pub mod db;
pub mod models;
pub mod routes;

use crate::error::Error;
use crate::firestore::FirestoreV1Client;

pub async fn assert_cards_allowed(
  firestore: &mut FirestoreV1Client,
  board_id: String,
) -> Result<(), Error> {
  let board = db::get(firestore, board_id).await?;
  match board.cards_open {
    true => Ok(()),
    false => Err(Error::Forbidden),
  }
}

pub async fn assert_voting_allowed(
  firestore: &mut FirestoreV1Client,
  board_id: String,
) -> Result<(), Error> {
  let board = db::get(firestore, board_id).await?;
  match board.voting_open {
    true => Ok(()),
    false => Err(Error::Forbidden),
  }
}
