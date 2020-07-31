use futures::future::join;
use std::convert::TryFrom;
use std::convert::TryInto;

use super::models::Board;
use crate::error::Error;
use crate::firestore::v1::*;
use crate::firestore::FirestoreV1Client;
use crate::participants::db::*;
use crate::participants::models::Participant;

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

pub async fn get(
  firestore: &mut FirestoreV1Client,
  participant: Participant,
  board_id: String,
) -> Result<Board, Error> {
  let (_, result) = join(
    add_participant_board(&mut firestore.clone(), participant, board_id.clone()),
    firestore.get_document(GetDocumentRequest {
      name: format!(
        "projects/retrotools-284402/databases/(default)/documents/boards/{}",
        board_id
      )
      .into(),
      mask: None,
      consistency_selector: None,
    }),
  )
  .await;
  result?.into_inner().try_into().map_err(|_| Error::Other)
}
