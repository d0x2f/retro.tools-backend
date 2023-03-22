use futures::lock::Mutex;
use std::convert::TryFrom;
use std::convert::TryInto;
use std::sync::Arc;
use std::time::SystemTime;

use super::models::*;
use crate::config::Config;
use crate::error::Error;
use crate::firestore::v1::*;
use crate::firestore::FirestoreV1Client;

pub async fn new(
  firestore: Arc<Mutex<FirestoreV1Client>>,
  config: &Config,
  board_id: String,
  column: ColumnMessage,
) -> Result<Column, Error> {
  let mut document: Document = column.into();
  let now = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)?;
  document.fields.insert(
    "created_at".into(),
    timestamp_value!(now.as_secs() as i64, now.subsec_nanos() as i32),
  );
  let result = firestore
    .lock()
    .await
    .create_document(CreateDocumentRequest {
      parent: format!(
        "projects/{}/databases/(default)/documents/boards/{}",
        config.firestore_project, board_id
      ),
      collection_id: "columns".into(),
      document: Some(document),
      ..Default::default()
    })
    .await?;
  result.into_inner().try_into()
}

pub async fn list(
  firestore: Arc<Mutex<FirestoreV1Client>>,
  config: &Config,
  board_id: String,
) -> Result<Vec<Column>, Error> {
  let result = firestore
    .lock()
    .await
    .list_documents(ListDocumentsRequest {
      parent: format!(
        "projects/{}/databases/(default)/documents/boards/{}",
        config.firestore_project, board_id
      ),
      collection_id: "columns".into(),
      ..Default::default()
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
  firestore: Arc<Mutex<FirestoreV1Client>>,
  config: &Config,
  board_id: String,
  column_id: String,
) -> Result<Column, Error> {
  let result = firestore
    .lock()
    .await
    .get_document(GetDocumentRequest {
      name: format!(
        "projects/{}/databases/(default)/documents/boards/{}/columns/{}",
        config.firestore_project, board_id, column_id
      ),
      ..Default::default()
    })
    .await?;
  result.into_inner().try_into()
}

pub async fn update(
  firestore: Arc<Mutex<FirestoreV1Client>>,
  config: &Config,
  board_id: String,
  column_id: String,
  column: ColumnMessage,
) -> Result<Column, Error> {
  let mut document: Document = column.into();
  document.name = format!(
    "projects/{}/databases/(default)/documents/boards/{}/columns/{}",
    config.firestore_project, board_id, column_id
  );
  let result = firestore
    .lock()
    .await
    .update_document(UpdateDocumentRequest {
      document: Some(document.clone()),
      update_mask: Some(DocumentMask {
        field_paths: document.fields.keys().cloned().collect(),
      }),
      ..Default::default()
    })
    .await?;
  result.into_inner().try_into()
}

pub async fn delete(
  firestore: Arc<Mutex<FirestoreV1Client>>,
  config: &Config,
  board_id: String,
  column_id: String,
) -> Result<(), Error> {
  let name = format!(
    "projects/{}/databases/(default)/documents/boards/{}/columns/{}",
    config.firestore_project, board_id, column_id
  );
  firestore
    .lock()
    .await
    .delete_document(DeleteDocumentRequest {
      name,
      ..Default::default()
    })
    .await?;
  Ok(())
}
