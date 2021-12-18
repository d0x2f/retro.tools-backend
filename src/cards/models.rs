use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::convert::TryFrom;

use crate::config::Config;
use crate::error::Error;
use crate::firestore::v1::*;
use crate::participants::models::Participant;
use crate::columns::models::Column;

#[derive(Deserialize, Serialize)]
pub struct CardMessage {
  pub column_id: Option<String>,
  pub author: Option<String>,
  pub text: Option<String>,
  pub column: Option<String>,
}

#[derive(Deserialize, Serialize)]
pub struct ReactMessage {
  pub emoji: String,
}

#[derive(Deserialize, Serialize)]
pub struct Card {
  pub id: String,
  pub column: String,
  pub owner: String,
  pub author: String,
  pub text: String,
  pub created_at: i64,
  pub votes: Vec<String>,
  pub reactions: HashMap<String, Vec<String>>,
}

#[derive(Deserialize, Serialize)]
pub struct CardResponse {
  pub id: String,
  pub column: String,
  pub owner: bool,
  pub author: String,
  pub text: String,
  pub created_at: i64,
  pub votes: usize,
  pub voted: bool,
  pub reactions: HashMap<String, usize>,
  pub reacted: String,
}

#[derive(Serialize)]
pub struct CardCSVRow {
  pub column: String,
  pub author: String,
  pub text: String,
  pub created_at: i64,
  pub votes: usize,
}

impl CardCSVRow {
  pub fn from_card(card: Card, columns: &HashMap<String, Column>) -> CardCSVRow {
    CardCSVRow {
      column: match columns.get(&card.column) {
        Some(column) => column.name.clone().split('.').last().unwrap_or(&column.name).into(),
        _ => "Unknown Column".into()
      },
      author: card.author,
      text: card.text,
      created_at: card.created_at,
      votes: card.votes.len(),
    }
  }
}

impl CardResponse {
  pub fn from_card(config: &Config, card: Card, participant: &Participant) -> CardResponse {
    let participant_doc_id = to_participant_reference!(config.firestore_project, participant.id);
    CardResponse {
      id: card.id,
      column: card.column,
      owner: card.owner == participant.id,
      author: card.author,
      text: card.text,
      created_at: card.created_at,
      votes: card.votes.len(),
      voted: card.votes.contains(&participant_doc_id),
      reactions: card.reactions.clone().into_iter().map(|(k, v)| (k, v.len())).collect(),
      reacted: {
        let mut reaction: Option<String> = None;
        for (emoji, participants) in &card.reactions {
          if participants.contains(&participant_doc_id) {
            reaction = Some(emoji.to_string());
          }
        }
        match reaction {
          Some(emoji) => emoji,
          None => "".into()
        }
      }
    }
  }
}

impl TryFrom<Document> for Card {
  type Error = Error;

  fn try_from(document: Document) -> Result<Self, Self::Error> {
    Ok(Card {
      id: get_id!(document),
      column: from_reference!(get_reference_field!(document, "column")?).into(),
      owner: from_reference!(get_reference_field!(document, "owner")?).into(),
      author: get_string_field!(document, "author")?,
      text: get_string_field!(document, "text")?,
      created_at: get_create_time!(document),
      votes: match get_array_field!(document, "votes") {
        Ok(arr) => arr
          .values
          .clone()
          .into_iter()
          .map(|v| extract_string!(v.value_type))
          .partition::<Vec<Option<String>>, _>(Option::is_some)
          .0
          .into_iter()
          .map(Option::unwrap)
          .collect(),
        Err(_) => vec![],
      },
      reactions: match get_map_field!(document, "reactions") {
        Ok(map) => map
          .fields
          .clone()
          .into_iter()
          .map(|(k, v)| (k, extract_array!(v.value_type).unwrap()))
          .collect(),
        Err(_) => HashMap::new(),
      },
    })
  }
}

impl From<CardMessage> for Document {
  fn from(card: CardMessage) -> Document {
    let mut fields: HashMap<String, Value> = HashMap::new();
    if let Some(author) = card.author {
      fields.insert("author".into(), string_value!(author));
    }
    if let Some(text) = card.text {
      fields.insert("text".into(), string_value!(text));
    }
    if let Some(column) = card.column {
      fields.insert("column".into(), reference_value!(column));
    }
    Document {
      name: "".into(),
      fields,
      create_time: None,
      update_time: None,
    }
  }
}
