use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::convert::TryFrom;

use crate::error::Error;
use crate::firestore::v1::*;

#[derive(Deserialize, Serialize)]
pub struct ColumnMessage {
  pub name: Option<String>,
}

#[derive(Deserialize, Serialize)]
pub struct Column {
  pub id: String,
  pub name: String,
  pub created_at: i64,
}

impl TryFrom<Document> for Column {
  type Error = Error;

  fn try_from(document: Document) -> Result<Self, Self::Error> {
    Ok(Column {
      id: get_id!(document),
      name: get_string_field!(document, "name")?,
      created_at: get_create_time!(document),
    })
  }
}

impl From<ColumnMessage> for Document {
  fn from(column: ColumnMessage) -> Document {
    let mut fields: HashMap<String, Value> = HashMap::new();
    if let Some(name) = column.name {
      fields.insert("name".into(), string_value!(name));
    }
    Document {
      name: "".into(),
      fields,
      create_time: None,
      update_time: None,
    }
  }
}
