use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::convert::TryFrom;

use crate::error::Error;
use crate::firestore::v1::*;

#[derive(Deserialize, Serialize)]
pub struct CardMessage {
  pub column_id: Option<String>,
  pub author: Option<String>,
  pub text: Option<String>,
  pub column: Option<String>,
}

#[derive(Deserialize, Serialize)]
pub struct Card {
  pub id: String,
  pub column_id: String,
  pub owner: String,
  pub author: String,
  pub text: String,
  pub created_at: i64,
}

impl TryFrom<Document> for Card {
  type Error = Error;

  fn try_from(document: Document) -> Result<Self, Self::Error> {
    Ok(Card {
      id: get_id!(document),
      column_id: get_string_field!(document, "column_id")?,
      owner: from_reference!(get_reference_field!(document, "owner")?).into(),
      author: get_string_field!(document, "author")?,
      text: get_string_field!(document, "text")?,
      created_at: get_create_time!(document),
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
    if let Some(column_id) = card.column_id {
      fields.insert("column_id".into(), string_value!(column_id));
    }
    Document {
      name: "".into(),
      fields,
      create_time: None,
      update_time: None,
    }
  }
}
