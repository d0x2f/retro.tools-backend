use std::convert::TryFrom;
use std::convert::TryInto;

use super::models::*;
use crate::error::Error;
use crate::firestore::v1::*;
use crate::firestore::FirestoreV1Client;

pub async fn new(
  firestore: &mut FirestoreV1Client,
  board_id: String,
  column: ColumnMessage,
) -> Result<Column, Error> {
  let document: Document = column.into();
  let result = firestore
    .create_document(CreateDocumentRequest {
      parent: format!(
        "projects/retrotools-284402/databases/(default)/documents/boards/{}",
        board_id
      ),
      collection_id: "columns".into(),
      document_id: "".into(),
      mask: None,
      document: Some(document),
    })
    .await?;
  Column::try_from(result.into_inner())
}

pub async fn list(
  firestore: &mut FirestoreV1Client,
  board_id: String,
) -> Result<Vec<Column>, Error> {
  let result = firestore
    .list_documents(ListDocumentsRequest {
      parent: format!(
        "projects/retrotools-284402/databases/(default)/documents/boards/{}",
        board_id
      ),
      collection_id: "columns".into(),
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
    .map(Column::try_from)
    .partition(Result::is_ok);
  Ok(valid_documents.into_iter().map(Result::unwrap).collect())
}

pub async fn get(
  firestore: &mut FirestoreV1Client,
  board_id: String,
  column_id: String,
) -> Result<Column, Error> {
  let result = firestore
    .get_document(GetDocumentRequest {
      name: format!(
        "projects/retrotools-284402/databases/(default)/documents/boards/{}/columns/{}",
        board_id, column_id
      )
      .into(),
      mask: None,
      consistency_selector: None,
    })
    .await?;
  result.into_inner().try_into()
}

pub async fn update(
  firestore: &mut FirestoreV1Client,
  board_id: String,
  column_id: String,
  column: ColumnMessage,
) -> Result<Column, Error> {
  let mut document: Document = column.into();
  document.name = format!(
    "projects/retrotools-284402/databases/(default)/documents/boards/{}/columns/{}",
    board_id, column_id
  )
  .into();
  let result = firestore
    .update_document(UpdateDocumentRequest {
      document: Some(document.clone()),
      mask: None,
      update_mask: Some(DocumentMask {
        field_paths: document.fields.keys().map(|k| k.clone()).collect(),
      }),
      current_document: None,
    })
    .await?;
  Column::try_from(result.into_inner())
}

pub async fn delete(
  firestore: &mut FirestoreV1Client,
  board_id: String,
  column_id: String,
) -> Result<(), Error> {
  let name = format!(
    "projects/retrotools-284402/databases/(default)/documents/boards/{}/columns/{}",
    board_id, column_id
  )
  .into();
  firestore
    .delete_document(DeleteDocumentRequest {
      name,
      current_document: None,
    })
    .await?;
  Ok(())
}
