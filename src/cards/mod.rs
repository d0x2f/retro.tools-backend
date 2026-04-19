mod db;
pub mod models;
pub mod routes;

use firestore::{FirestoreDb, FirestoreReference};

use crate::boards;
use crate::error::Error;
use crate::participants::models::Participant;

pub fn check_card_owner(
  board: &boards::models::Board,
  card: &models::Card,
  participant_ref: &FirestoreReference,
) -> Result<(), Error> {
  if board.owner == *participant_ref || card.owner == *participant_ref {
    Ok(())
  } else {
    Err(Error::Forbidden)
  }
}

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
  check_card_owner(&board, card, &participant_reference)
}

#[cfg(test)]
mod tests {
  use super::*;
  use chrono::Utc;
  use serde_json::Map;
  use std::collections::HashMap;

  fn ref_(s: &str) -> FirestoreReference {
    FirestoreReference(s.to_string())
  }

  fn make_board(owner: &str) -> boards::models::Board {
    boards::models::Board {
      id: "board1".to_string(),
      name: "Test".to_string(),
      cards_open: true,
      voting_open: true,
      ice_breaking: "".to_string(),
      created_at: Utc::now().timestamp(),
      owner: ref_(owner),
      anyone_is_owner: false,
      data: serde_json::Value::Object(Map::new()),
    }
  }

  fn make_card(owner: &str) -> models::Card {
    models::Card {
      id: "card1".to_string(),
      column: ref_("boards/b1/columns/col1"),
      owner: ref_(owner),
      author: "Alice".to_string(),
      text: "Some text".to_string(),
      created_at: Utc::now().timestamp(),
      votes: vec![],
      reactions: HashMap::new(),
    }
  }

  #[test]
  fn board_owner_can_edit_any_card() {
    let board = make_board("participants/owner");
    let card = make_card("participants/other");
    assert!(check_card_owner(&board, &card, &ref_("participants/owner")).is_ok());
  }

  #[test]
  fn card_owner_can_edit_own_card() {
    let board = make_board("participants/owner");
    let card = make_card("participants/author");
    assert!(check_card_owner(&board, &card, &ref_("participants/author")).is_ok());
  }

  #[test]
  fn non_owner_non_card_owner_is_forbidden() {
    let board = make_board("participants/owner");
    let card = make_card("participants/author");
    assert!(matches!(
      check_card_owner(&board, &card, &ref_("participants/stranger")),
      Err(Error::Forbidden)
    ));
  }

  #[test]
  fn board_owner_who_is_also_card_owner_can_edit() {
    let board = make_board("participants/owner");
    let card = make_card("participants/owner");
    assert!(check_card_owner(&board, &card, &ref_("participants/owner")).is_ok());
  }
}
