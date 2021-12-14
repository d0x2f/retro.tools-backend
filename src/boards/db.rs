use std::convert::TryFrom;
use std::convert::TryInto;

use super::models::*;
use crate::config::Config;
use crate::error::Error;
use crate::firestore::v1::*;
use crate::firestore::FirestoreV1Client;
use crate::participants::db::*;
use crate::participants::models::Participant;

pub async fn new(
  firestore: &mut FirestoreV1Client,
  config: &Config,
  participant: &Participant,
  board: BoardMessage,
) -> Result<Board, Error> {
  let mut document: Document = board.into();
  document.fields.insert(
    "owner".into(),
    reference_value!(to_participant_reference!(
      config.firestore_project,
      participant.id
    )),
  );
  let result = firestore
    .create_document(CreateDocumentRequest {
      parent: format!(
        "projects/{}/databases/(default)/documents",
        config.firestore_project
      ),
      collection_id: "boards".into(),
      document: Some(document),
      ..Default::default()
    })
    .await?;
  result.into_inner().try_into()
}

pub async fn list(
  firestore: &mut FirestoreV1Client,
  config: &Config,
  participant: &Participant,
) -> Result<Vec<Board>, Error> {
  let ids = get_participant_board_ids(firestore, config, participant).await?;
  let result = firestore
    .batch_get_documents(BatchGetDocumentsRequest {
      database: format!("projects/{}/databases/(default)", config.firestore_project),
      documents: ids,
      ..Default::default()
    })
    .await?;

  let mut result_stream = result.into_inner();
  let mut boards: Vec<Board> = vec![];
  while let Some(message) = result_stream.message().await? {
    if let Some(document) = message.result {
      if let Ok(board) = Board::try_from(document) {
        boards.push(board);
      }
    }
  }
  Ok(boards)
}

pub async fn get(
  firestore: &mut FirestoreV1Client,
  config: &Config,
  board_id: String,
) -> Result<Board, Error> {
  let result = firestore
    .get_document(GetDocumentRequest {
      name: format!(
        "projects/{}/databases/(default)/documents/boards/{}",
        config.firestore_project, board_id
      ),
      ..Default::default()
    })
    .await?;
  result.into_inner().try_into()
}

pub async fn update(
  firestore: &mut FirestoreV1Client,
  config: &Config,
  board_id: String,
  board: BoardMessage,
) -> Result<Board, Error> {
  let mut document: Document = board.into();
  document.name = format!(
    "projects/{}/databases/(default)/documents/boards/{}",
    config.firestore_project, board_id
  );
  let result = firestore
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
  firestore: &mut FirestoreV1Client,
  config: &Config,
  board_id: String,
) -> Result<(), Error> {
  let name = format!(
    "projects/{}/databases/(default)/documents/boards/{}",
    config.firestore_project, board_id
  );
  firestore
    .delete_document(DeleteDocumentRequest {
      name,
      ..Default::default()
    })
    .await?;
  Ok(())
}
