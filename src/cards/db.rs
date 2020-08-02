use std::convert::TryFrom;
use std::convert::TryInto;

use super::models::*;
use crate::error::Error;
use crate::firestore::v1::*;
use crate::firestore::FirestoreV1Client;
use crate::participants::models::Participant;

pub async fn new(
  firestore: &mut FirestoreV1Client,
  participant: Participant,
  board_id: String,
  card: CardMessage,
) -> Result<Card, Error> {
  let mut document: Document = card.into();
  document
    .fields
    .insert("owner".into(), string_value!(participant.id.clone()));
  let result = firestore
    .create_document(CreateDocumentRequest {
      parent: format!(
        "projects/retrotools-284402/databases/(default)/documents/boards/{}",
        board_id
      ),
      collection_id: "cards".into(),
      document_id: "".into(),
      mask: None,
      document: Some(document),
    })
    .await?;
  Card::try_from(result.into_inner())
}

pub async fn list(
  firestore: &mut FirestoreV1Client,
  board_id: String,
) -> Result<Vec<Card>, Error> {
  let result = firestore
    .list_documents(ListDocumentsRequest {
      parent: format!(
        "projects/retrotools-284402/databases/(default)/documents/boards/{}",
        board_id
      ),
      collection_id: "cards".into(),
      page_size: 0,
      page_token: "".into(),
      order_by: "".into(),
      mask: None,
      show_missing: false,
      consistency_selector: None,
    })
    .await?;
  let documents = result.into_inner().documents;
  let (valid_documents, _): (Vec<_>, Vec<_>) = documents
    .into_iter()
    .map(Card::try_from)
    .partition(Result::is_ok);
  Ok(valid_documents.into_iter().map(Result::unwrap).collect())
}

pub async fn get(
  firestore: &mut FirestoreV1Client,
  board_id: String,
  card_id: String,
) -> Result<Card, Error> {
  let result = firestore
    .get_document(GetDocumentRequest {
      name: format!(
        "projects/retrotools-284402/databases/(default)/documents/boards/{}/cards/{}",
        board_id, card_id
      ),
      mask: None,
      consistency_selector: None,
    })
    .await?;
  result.into_inner().try_into()
}

pub async fn update(
  firestore: &mut FirestoreV1Client,
  board_id: String,
  card_id: String,
  card: CardMessage,
) -> Result<Card, Error> {
  let mut document: Document = card.into();
  document.name = format!(
    "projects/retrotools-284402/databases/(default)/documents/boards/{}/cards/{}",
    board_id, card_id
  );
  let result = firestore
    .update_document(UpdateDocumentRequest {
      document: Some(document.clone()),
      mask: None,
      update_mask: Some(DocumentMask {
        field_paths: document.fields.keys().cloned().collect(),
      }),
      current_document: None,
    })
    .await?;
  result.into_inner().try_into()
}

pub async fn delete(firestore: &mut FirestoreV1Client, board_id: String, card_id: String) -> Result<(), Error> {
  let name = format!(
    "projects/retrotools-284402/databases/(default)/documents/boards/{}/cards/{}",
    board_id,
    card_id
  );
  firestore
    .delete_document(DeleteDocumentRequest {
      name,
      current_document: None,
    })
    .await?;
  Ok(())
}