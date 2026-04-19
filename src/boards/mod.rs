pub mod db;
pub mod models;
pub mod routes;

use firestore::FirestoreDb;

use crate::error::Error;

pub fn check_cards_allowed(board: &models::Board) -> Result<(), Error> {
  if board.cards_open { Ok(()) } else { Err(Error::Forbidden) }
}

pub fn check_voting_allowed(board: &models::Board) -> Result<(), Error> {
  if board.voting_open { Ok(()) } else { Err(Error::Forbidden) }
}

pub async fn assert_cards_allowed(firestore: &FirestoreDb, board_id: &String) -> Result<(), Error> {
  check_cards_allowed(&db::get(firestore, board_id).await?)
}

pub async fn assert_voting_allowed(
  firestore: &FirestoreDb,
  board_id: &String,
) -> Result<(), Error> {
  check_voting_allowed(&db::get(firestore, board_id).await?)
}

pub async fn get_board(firestore: &FirestoreDb, board_id: &String) -> Result<models::Board, Error> {
  db::get(firestore, board_id).await
}

#[cfg(test)]
mod tests {
  use super::*;
  use chrono::Utc;
  use firestore::FirestoreReference;
  use serde_json::Map;

  fn make_board(cards_open: bool, voting_open: bool) -> models::Board {
    models::Board {
      id: "board1".to_string(),
      name: "Test".to_string(),
      cards_open,
      voting_open,
      ice_breaking: "".to_string(),
      created_at: Utc::now().timestamp(),
      owner: FirestoreReference("participants/owner".to_string()),
      anyone_is_owner: false,
      data: serde_json::Value::Object(Map::new()),
    }
  }

  #[test]
  fn cards_open_permits_card_creation() {
    assert!(check_cards_allowed(&make_board(true, true)).is_ok());
  }

  #[test]
  fn cards_closed_blocks_card_creation() {
    assert!(check_cards_allowed(&make_board(false, true)).is_err());
  }

  #[test]
  fn cards_closed_returns_forbidden() {
    assert!(matches!(check_cards_allowed(&make_board(false, true)), Err(Error::Forbidden)));
  }

  #[test]
  fn voting_open_permits_voting() {
    assert!(check_voting_allowed(&make_board(true, true)).is_ok());
  }

  #[test]
  fn voting_closed_blocks_voting() {
    assert!(check_voting_allowed(&make_board(true, false)).is_err());
  }

  #[test]
  fn voting_closed_returns_forbidden() {
    assert!(matches!(check_voting_allowed(&make_board(true, false)), Err(Error::Forbidden)));
  }
}
