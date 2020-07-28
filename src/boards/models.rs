use serde::{Deserialize, Serialize};
use crate::firestore::v1::Document;
use crate::get_string_field;
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
    println!("{:?}", document);
    Ok(Board {
      id: document.name.rsplitn(2,'/').next().expect("document id").into(),
      name: get_string_field!(document, "name").ok_or(())?,
      cards_open: get_boolean_field!(document, "cards_open").ok_or(())?,
      voting_open: get_boolean_field!(document, "voting_open").ok_or(())?,
      created_at: document.create_time.ok_or(())?.seconds
    })
  }
}