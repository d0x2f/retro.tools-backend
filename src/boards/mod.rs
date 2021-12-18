pub mod db;
pub mod models;
pub mod routes;

use crate::config::Config;
use crate::error::Error;
use crate::firestore::FirestoreV1Client;

pub async fn assert_cards_allowed(
  firestore: &mut FirestoreV1Client,
  config: &Config,
  board_id: String,
) -> Result<(), Error> {
  let board = db::get(firestore, config, board_id).await?;
  match board.cards_open {
    true => Ok(()),
    false => Err(Error::Forbidden),
  }
}

pub async fn assert_voting_allowed(
  firestore: &mut FirestoreV1Client,
  config: &Config,
  board_id: String,
) -> Result<(), Error> {
  let board = db::get(firestore, config, board_id).await?;
  match board.voting_open {
    true => Ok(()),
    false => Err(Error::Forbidden),
  }
}

pub async fn get_board(
  firestore: &mut FirestoreV1Client,
  config: &Config,
  board_id: String
) -> Result<models::Board, Error> {
  db::get(firestore, &config, board_id.to_string()).await
}