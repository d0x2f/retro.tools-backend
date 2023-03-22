pub mod db;
pub mod models;
pub mod routes;

use firestore::FirestoreDb;

use crate::error::Error;

pub async fn assert_cards_allowed(firestore: &FirestoreDb, board_id: String) -> Result<(), Error> {
  let board = db::get(firestore, board_id).await?;
  match board.cards_open {
    true => Ok(()),
    false => Err(Error::Forbidden),
  }
}

pub async fn assert_voting_allowed(firestore: &FirestoreDb, board_id: String) -> Result<(), Error> {
  let board = db::get(firestore, board_id).await?;
  match board.voting_open {
    true => Ok(()),
    false => Err(Error::Forbidden),
  }
}

pub async fn get_board(firestore: &FirestoreDb, board_id: String) -> Result<models::Board, Error> {
  db::get(firestore, board_id.to_string()).await
}
