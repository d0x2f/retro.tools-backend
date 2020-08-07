use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::convert::TryFrom;

use crate::error::Error;
use crate::firestore::v1::*;
use crate::participants::models::Participant;

#[derive(Deserialize, Serialize)]
pub struct BoardMessage {
  pub name: Option<String>,
  pub cards_open: Option<bool>,
  pub voting_open: Option<bool>,
  pub data: Option<serde_json::Value>,
}

#[derive(Deserialize, Serialize)]
pub struct Board {
  pub id: String,
  pub name: String,
  pub cards_open: bool,
  pub voting_open: bool,
  pub created_at: i64,
  pub owner: String,
  pub data: serde_json::Value,
}

#[derive(Deserialize, Serialize)]
pub struct BoardResponse {
  pub id: String,
  pub name: String,
  pub cards_open: bool,
  pub voting_open: bool,
  pub created_at: i64,
  pub owner: bool,
  pub data: serde_json::Value,
}

impl BoardResponse {
  pub fn from_board(board: Board, participant: &Participant) -> BoardResponse {
    BoardResponse {
      id: board.id,
      name: board.name,
      cards_open: board.cards_open,
      voting_open: board.voting_open,
      created_at: board.created_at,
      owner: board.owner == participant.id,
      data: board.data,
    }
  }
}

impl TryFrom<Document> for Board {
  type Error = Error;

  fn try_from(document: Document) -> Result<Self, Self::Error> {
    Ok(Board {
      id: get_id!(document),
      name: get_string_field!(document, "name")?,
      cards_open: get_boolean_field!(document, "cards_open")?,
      voting_open: get_boolean_field!(document, "voting_open")?,
      created_at: get_create_time!(document),
      owner: from_reference!(get_reference_field!(document, "owner")?).into(),
      data: serde_json::from_str(get_string_field!(document, "data")?.as_str())?,
    })
  }
}

impl TryFrom<batch_get_documents_response::Result> for Board {
  type Error = Error;

  fn try_from(result: batch_get_documents_response::Result) -> Result<Self, Self::Error> {
    match result {
      batch_get_documents_response::Result::Missing(s) => {
        Err(Error::Other(format!("board not found: {}", s)))
      }
      batch_get_documents_response::Result::Found(d) => Self::try_from(d),
    }
  }
}

impl From<BoardMessage> for Document {
  fn from(board: BoardMessage) -> Document {
    let mut fields: HashMap<String, Value> = HashMap::new();
    if let Some(name) = board.name {
      fields.insert("name".into(), string_value!(name));
    }
    if let Some(cards_open) = board.cards_open {
      fields.insert("cards_open".into(), boolean_value!(cards_open));
    }
    if let Some(voting_open) = board.voting_open {
      fields.insert("voting_open".into(), boolean_value!(voting_open));
    }
    if let Some(data) = board.data {
      fields.insert("data".into(), string_value!(data.to_string()));
    }
    Document {
      name: "".into(),
      fields,
      create_time: None,
      update_time: None,
    }
  }
}
