use crate::firestore::v1::batch_get_documents_response;
use crate::firestore::v1::Document;
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;

#[derive(Deserialize, Serialize)]
pub struct Board {
  pub id: String,
  pub name: String,
  pub cards_open: bool,
  pub voting_open: bool,
  pub created_at: i64,
}

impl TryFrom<Document> for Board {
  type Error = ();

  fn try_from(document: Document) -> Result<Self, Self::Error> {
    Ok(Board {
      id: document
        .name
        .rsplitn(2, '/')
        .next()
        .expect("document id")
        .into(),
      name: get_string_field!(document, "name").ok_or(())?,
      cards_open: get_boolean_field!(document, "cards_open").ok_or(())?,
      voting_open: get_boolean_field!(document, "voting_open").ok_or(())?,
      created_at: document.create_time.ok_or(())?.seconds,
    })
  }
}

impl TryFrom<batch_get_documents_response::Result> for Board {
  type Error = ();

  fn try_from(result: batch_get_documents_response::Result) -> Result<Self, Self::Error> {
    match result {
      batch_get_documents_response::Result::Missing(_) => Err(()),
      batch_get_documents_response::Result::Found(d) => Self::try_from(d),
    }
  }
}
