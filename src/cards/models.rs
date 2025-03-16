use chrono::Utc;
use firestore::{FirestoreReference, FirestoreTimestamp};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::convert::TryFrom;

use crate::columns::models::Column;
use crate::error::Error;

#[derive(Deserialize, Serialize)]
pub struct CardMessage {
  pub author: Option<String>,
  pub text: Option<String>,
  pub column: Option<String>,
}

#[derive(Deserialize, Serialize)]
pub struct CardChangeSet {
  #[serde(skip_serializing_if = "Option::is_none")]
  pub author: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub text: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub column: Option<FirestoreReference>,
}

#[derive(Deserialize, Serialize)]
pub struct ReactMessage {
  pub emoji: String,
}

#[derive(Deserialize, Serialize)]
pub struct Card {
  pub id: String,
  pub column: FirestoreReference,
  pub owner: FirestoreReference,
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

#[derive(Deserialize, Serialize)]
pub struct NewCard {
  pub created_at: FirestoreTimestamp,
  pub column: FirestoreReference,
  pub owner: Option<FirestoreReference>,
  pub author: String,
  pub text: String,
}

#[derive(Deserialize, Serialize)]
pub struct CardInFirestore {
  pub _firestore_id: String,
  pub _firestore_created: FirestoreTimestamp,
  pub created_at: Option<FirestoreTimestamp>,
  pub author: String,
  pub text: String,
  pub owner: FirestoreReference,
  pub column: FirestoreReference,
  pub votes: Option<Vec<String>>,
  pub reactions: Option<HashMap<String, Vec<String>>>,
}

#[derive(Serialize)]
pub struct CardCSVRow {
  pub column: String,
  pub author: String,
  pub text: String,
  pub created_at: i64,
  pub votes: usize,
}

impl TryFrom<CardMessage> for NewCard {
  type Error = Error;

  fn try_from(card: CardMessage) -> Result<Self, Self::Error> {
    Ok(NewCard {
      author: card.author.unwrap_or("".into()),
      text: card.text.unwrap_or("".into()),
      created_at: FirestoreTimestamp(Utc::now()),
      owner: None,
      column: FirestoreReference(
        card
          .column
          .ok_or(Error::BadRequest("column is required".into()))?,
      ),
    })
  }
}

impl From<CardInFirestore> for Card {
  fn from(card: CardInFirestore) -> Self {
    Card {
      id: card._firestore_id,
      created_at: card
        .created_at
        .unwrap_or(card._firestore_created)
        .0
        .timestamp(),
      owner: card.owner,
      column: card.column,
      author: card.author,
      text: card.text,
      votes: card.votes.unwrap_or_default(),
      reactions: card.reactions.unwrap_or_default(),
    }
  }
}

impl CardCSVRow {
  pub fn from_card(card: Card, columns: &HashMap<String, Column>) -> CardCSVRow {
    CardCSVRow {
      column: match columns.get(&card.column.0.split('/').next_back().unwrap().to_string()) {
        Some(column) => column
          .name
          .clone()
          .split('.')
          .next_back()
          .unwrap_or(&column.name)
          .into(),
        _ => "Unknown Column".into(),
      },
      author: card.author,
      text: card.text,
      created_at: card.created_at,
      votes: card.votes.len(),
    }
  }
}

impl CardResponse {
  pub fn from_card(card: Card, participant_id: &FirestoreReference) -> CardResponse {
    CardResponse {
      id: card.id,
      column: card.column.0.split('/').next_back().unwrap().to_string(),
      owner: &card.owner == participant_id,
      author: card.author,
      text: card.text,
      created_at: card.created_at,
      votes: card.votes.len(),
      voted: card.votes.contains(&participant_id.0),
      reactions: card
        .reactions
        .clone()
        .into_iter()
        .map(|(k, v)| (k, v.len()))
        .collect(),
      reacted: {
        let mut reaction: Option<String> = None;
        for (emoji, participants) in &card.reactions {
          if participants.contains(&participant_id.0) {
            reaction = Some(emoji.to_string());
          }
        }
        match reaction {
          Some(emoji) => emoji,
          None => "".into(),
        }
      },
    }
  }
}
