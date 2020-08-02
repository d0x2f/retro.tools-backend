use std::convert::TryFrom;
use std::convert::TryInto;

use super::models::*;
use crate::error::Error;
use crate::firestore::v1::*;
use crate::firestore::FirestoreV1Client;
use crate::participants::db::*;
use crate::participants::models::Participant;

pub async fn new(
  firestore: &mut FirestoreV1Client,
  participant: Participant,
  board: BoardMessage,
) -> Result<Board, Error> {
  let mut document: Document = board.into();
  document
    .fields
    .insert("owner".into(), reference_value!(to_participant_reference!("retrotools-284402", participant.id)));
  let result = firestore
    .create_document(CreateDocumentRequest {
      parent: "projects/retrotools-284402/databases/(default)/documents".into(),
      collection_id: "boards".into(),
      document_id: "".into(),
      mask: None,
      document: Some(document),
    })
    .await?;
  result.into_inner().try_into()
}

pub async fn list(
  firestore: &mut FirestoreV1Client,
  participant: Participant,
) -> Result<Vec<Board>, Error> {
  let ids = get_participant_board_ids(firestore, participant).await?;
  let result = firestore
    .batch_get_documents(BatchGetDocumentsRequest {
      database: "projects/retrotools-284402/databases/(default)".into(),
      documents: ids,
      mask: None,
      consistency_selector: None,
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

pub async fn get(firestore: &mut FirestoreV1Client, board_id: String) -> Result<Board, Error> {
  let result = firestore
    .get_document(GetDocumentRequest {
      name: format!(
        "projects/retrotools-284402/databases/(default)/documents/boards/{}",
        board_id
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
  board: BoardMessage,
) -> Result<Board, Error> {
  let mut document: Document = board.into();
  document.name = format!(
    "projects/retrotools-284402/databases/(default)/documents/boards/{}",
    board_id
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

pub async fn delete(firestore: &mut FirestoreV1Client, board_id: String) -> Result<(), Error> {
  let name = format!(
    "projects/retrotools-284402/databases/(default)/documents/boards/{}",
    board_id
  );
  firestore
    .delete_document(DeleteDocumentRequest {
      name,
      current_document: None,
    })
    .await?;
  Ok(())
}
