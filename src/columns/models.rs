use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::convert::TryFrom;

use crate::error::Error;
use crate::firestore::v1::*;

#[derive(Deserialize, Serialize)]
pub struct ColumnMessage {
  pub name: Option<String>,
  pub data: Option<serde_json::Value>,
  pub position: Option<i64>,
}

#[derive(Deserialize, Serialize)]
pub struct Column {
  pub id: String,
  pub name: String,
  pub created_at: i64,
  pub data: serde_json::Value,
  pub position: i64,
}

impl TryFrom<Document> for Column {
  type Error = Error;

  fn try_from(document: Document) -> Result<Self, Self::Error> {
    Ok(Column {
      id: get_id!(document),
      name: get_string_field!(document, "name").unwrap_or_else(|_| "".into()),
      created_at: get_create_time!(document),
      data: serde_json::from_str(
        get_string_field!(document, "data")
          .unwrap_or_else(|_| "".into())
          .as_str(),
      )?,
      position: get_integer_field!(document, "position").unwrap_or(0),
    })
  }
}

impl From<ColumnMessage> for Document {
  fn from(column: ColumnMessage) -> Document {
    let mut fields: HashMap<String, Value> = HashMap::new();
    if let Some(name) = column.name {
      fields.insert("name".into(), string_value!(name));
    }
    if let Some(data) = column.data {
      fields.insert("data".into(), string_value!(data.to_string()));
    }
    if let Some(position) = column.position {
      fields.insert("position".into(), integer_value!(position));
    }
    Document {
      name: "".into(),
      fields,
      create_time: None,
      update_time: None,
    }
  }
}
