use chrono::Utc;
use firestore::{FirestoreReference, FirestoreTimestamp};
use serde::{Deserialize, Serialize};
use serde_json::Map;

#[derive(Deserialize, Serialize)]
pub struct BoardMessage {
  #[serde(skip_serializing_if = "Option::is_none")]
  pub name: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub cards_open: Option<bool>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub voting_open: Option<bool>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub ice_breaking: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub data: Option<serde_json::Value>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub anyone_is_owner: Option<bool>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Board {
  pub id: String,
  pub name: String,
  pub cards_open: bool,
  pub voting_open: bool,
  pub ice_breaking: String,
  pub created_at: i64,
  pub owner: FirestoreReference,
  pub anyone_is_owner: bool,
  pub data: serde_json::Value,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct NewBoard {
  pub name: String,
  pub cards_open: bool,
  pub voting_open: bool,
  pub ice_breaking: Option<String>,
  pub created_at: FirestoreTimestamp,
  pub owner: Option<FirestoreReference>,
  pub anyone_is_owner: bool,
  pub data: serde_json::Value,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct BoardInFirestore {
  pub _firestore_id: String,
  pub _firestore_created: FirestoreTimestamp,
  pub name: String,
  pub cards_open: bool,
  pub voting_open: bool,
  pub ice_breaking: Option<String>,
  pub created_at: Option<FirestoreTimestamp>,
  pub owner: FirestoreReference,
  pub anyone_is_owner: Option<bool>,
  pub data: serde_json::Value,
}

impl From<BoardMessage> for NewBoard {
  fn from(board: BoardMessage) -> Self {
    NewBoard {
      name: board.name.unwrap_or_else(|| "".into()),
      cards_open: board.cards_open.unwrap_or(true),
      voting_open: board.voting_open.unwrap_or(true),
      ice_breaking: board.ice_breaking,
      created_at: FirestoreTimestamp(Utc::now()),
      owner: None,
      anyone_is_owner: board.anyone_is_owner.unwrap_or(false),
      data: board
        .data
        .unwrap_or_else(|| serde_json::Value::Object(Map::new())),
    }
  }
}

impl From<BoardInFirestore> for Board {
  fn from(board: BoardInFirestore) -> Self {
    Board {
      id: board._firestore_id,
      name: board.name,
      cards_open: board.cards_open,
      voting_open: board.voting_open,
      ice_breaking: board.ice_breaking.unwrap_or_else(|| "".into()),
      created_at: board
        .created_at
        .unwrap_or(board._firestore_created)
        .0
        .timestamp(),
      owner: board.owner,
      anyone_is_owner: board.anyone_is_owner.unwrap_or(false),
      data: board.data,
    }
  }
}

#[derive(Deserialize, Serialize)]
pub struct BoardResponse {
  pub id: String,
  pub name: String,
  pub cards_open: bool,
  pub voting_open: bool,
  pub ice_breaking: String,
  pub created_at: i64,
  pub owner: bool,
  pub anyone_is_owner: bool,
  pub data: serde_json::Value,
}

impl BoardResponse {
  pub fn from_board(board: Board, participant_id: &FirestoreReference) -> BoardResponse {
    BoardResponse {
      id: board.id,
      name: board.name,
      cards_open: board.cards_open,
      voting_open: board.voting_open,
      ice_breaking: board.ice_breaking,
      created_at: board.created_at,
      owner: board.anyone_is_owner || &board.owner == participant_id,
      anyone_is_owner: board.anyone_is_owner,
      data: board.data,
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use chrono::Utc;
  use firestore::{FirestoreReference, FirestoreTimestamp};

  fn ref_(path: &str) -> FirestoreReference {
    FirestoreReference(path.to_string())
  }

  fn board_in_firestore(id: &str, owner: &str) -> BoardInFirestore {
    BoardInFirestore {
      _firestore_id: id.to_string(),
      _firestore_created: FirestoreTimestamp(Utc::now()),
      name: "Test Board".to_string(),
      cards_open: true,
      voting_open: false,
      ice_breaking: Some("How are you?".to_string()),
      created_at: None,
      owner: ref_(owner),
      anyone_is_owner: None,
      data: serde_json::Value::Object(serde_json::Map::new()),
    }
  }

  #[test]
  fn board_message_all_none_uses_defaults() {
    let msg = BoardMessage {
      name: None,
      cards_open: None,
      voting_open: None,
      ice_breaking: None,
      data: None,
      anyone_is_owner: None,
    };
    let b: NewBoard = msg.into();
    assert_eq!(b.name, "");
    assert!(b.cards_open);
    assert!(b.voting_open);
    assert!(b.ice_breaking.is_none());
    assert_eq!(b.data, serde_json::Value::Object(serde_json::Map::new()));
    assert!(b.owner.is_none());
  }

  #[test]
  fn board_message_explicit_values_preserved() {
    let msg = BoardMessage {
      name: Some("My Retro".to_string()),
      cards_open: Some(false),
      voting_open: Some(false),
      ice_breaking: Some("Icebreaker!".to_string()),
      data: Some(serde_json::json!({"key": "value"})),
      anyone_is_owner: None,
    };
    let b: NewBoard = msg.into();
    assert_eq!(b.name, "My Retro");
    assert!(!b.cards_open);
    assert!(!b.voting_open);
    assert_eq!(b.ice_breaking, Some("Icebreaker!".to_string()));
    assert_eq!(b.data, serde_json::json!({"key": "value"}));
  }

  #[test]
  fn board_in_firestore_ice_breaking_preserved() {
    let b: Board = board_in_firestore("b1", "participants/user1").into();
    assert_eq!(b.id, "b1");
    assert_eq!(b.ice_breaking, "How are you?");
    assert!(!b.voting_open);
  }

  #[test]
  fn board_in_firestore_ice_breaking_none_defaults_to_empty() {
    let mut raw = board_in_firestore("b2", "participants/user1");
    raw.ice_breaking = None;
    let b: Board = raw.into();
    assert_eq!(b.ice_breaking, "");
  }

  #[test]
  fn board_response_owner_true_when_refs_match() {
    let participant = ref_("participants/user1");
    let board: Board = board_in_firestore("b1", "participants/user1").into();
    let resp = BoardResponse::from_board(board, &participant);
    assert!(resp.owner);
  }

  #[test]
  fn board_response_owner_false_when_refs_differ() {
    let participant = ref_("participants/user2");
    let board: Board = board_in_firestore("b1", "participants/user1").into();
    let resp = BoardResponse::from_board(board, &participant);
    assert!(!resp.owner);
  }

  #[test]
  fn board_response_owner_true_when_anyone_is_owner() {
    let participant = ref_("participants/user2");
    let mut raw = board_in_firestore("b1", "participants/user1");
    raw.anyone_is_owner = Some(true);
    let board: Board = raw.into();
    let resp = BoardResponse::from_board(board, &participant);
    assert!(resp.owner);
    assert!(resp.anyone_is_owner);
  }

  #[test]
  fn board_response_anyone_is_owner_false_by_default() {
    let participant = ref_("participants/user1");
    let board: Board = board_in_firestore("b1", "participants/user1").into();
    let resp = BoardResponse::from_board(board, &participant);
    assert!(!resp.anyone_is_owner);
  }

  #[test]
  fn board_in_firestore_anyone_is_owner_none_defaults_to_false() {
    let raw = board_in_firestore("b1", "participants/user1");
    let board: Board = raw.into();
    assert!(!board.anyone_is_owner);
  }

  #[test]
  fn board_message_anyone_is_owner_none_defaults_to_false() {
    let msg = BoardMessage {
      name: None,
      cards_open: None,
      voting_open: None,
      ice_breaking: None,
      data: None,
      anyone_is_owner: None,
    };
    let b: NewBoard = msg.into();
    assert!(!b.anyone_is_owner);
  }

  #[test]
  fn board_message_anyone_is_owner_true_is_preserved() {
    let msg = BoardMessage {
      name: None,
      cards_open: None,
      voting_open: None,
      ice_breaking: None,
      data: None,
      anyone_is_owner: Some(true),
    };
    let b: NewBoard = msg.into();
    assert!(b.anyone_is_owner);
  }

  #[test]
  fn board_response_fields_match_board() {
    let participant = ref_("participants/user1");
    let board: Board = board_in_firestore("b1", "participants/user1").into();
    let resp = BoardResponse::from_board(board, &participant);
    assert_eq!(resp.id, "b1");
    assert_eq!(resp.name, "Test Board");
    assert!(resp.cards_open);
    assert!(!resp.voting_open);
    assert_eq!(resp.ice_breaking, "How are you?");
  }
}
