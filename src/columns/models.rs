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
      id: document
        .name
        .rsplitn(2, '/')
        .next()
        .expect("document id")
        .into(),
      name: get_string_field!(document, "name")?,
      created_at: document
        .create_time
        .ok_or(Error::Other(
          "field `create_time` not set in document.".into(),
        ))?
        .seconds,
    })
  }
}

impl TryFrom<batch_get_documents_response::Result> for Column {
  type Error = Error;

  fn try_from(result: batch_get_documents_response::Result) -> Result<Self, Self::Error> {
    match result {
      batch_get_documents_response::Result::Missing(s) => {
        Err(Error::Other(format!("column not found: {}", s)))
      }
      batch_get_documents_response::Result::Found(d) => Self::try_from(d),
    }
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
      fields: fields,
      create_time: None,
      update_time: None,
    }
  }
}
