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
      owner: &board.owner == participant_id,
      data: board.data,
    }
  }
}
