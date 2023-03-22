use firestore::FirestoreDb;
use firestore::FirestoreReference;
use futures::lock::Mutex;
use futures::stream::BoxStream;
use futures::StreamExt;
use std::convert::TryInto;
use std::sync::Arc;

use super::models::*;
use crate::config::Config;
use crate::error::Error;
use crate::firestore::v1::*;
use crate::firestore::FirestoreV1Client;
use crate::participants::db::get_participant_board_ids;
use crate::participants::models::Participant;

pub async fn new(
  firestore: &FirestoreDb,
  participant: &Participant,
  board: BoardMessage,
) -> Result<Board, Error> {
  let mut new_board = Into::<NewBoard>::into(board);
  new_board.owner = Some(FirestoreReference(format!(
    "{}/participants/{}",
    firestore.get_documents_path(),
    participant.id
  )));
  firestore
    .fluent()
    .insert()
    .into("boards")
    .generate_document_id()
    .object(&new_board)
    .execute::<BoardInFirestore>()
    .await
    .map(|board| board.into())
    .map_err(|e| e.into())
}

pub async fn list(firestore: &FirestoreDb, participant: &Participant) -> Result<Vec<Board>, Error> {
  let boards = get_participant_board_ids(firestore, participant).await?;
  let mut object_stream: BoxStream<(_, Option<BoardInFirestore>)> = firestore
    .fluent()
    .select()
    .by_id_in("boards")
    .obj()
    .batch(boards)
    .await?;

  let mut boards: Vec<Board> = vec![];
  while let Some((_, Some(board))) = object_stream.next().await {
    boards.push(board.into());
  }
  Ok(boards)
}

pub async fn get(firestore: &FirestoreDb, board_id: String) -> Result<Board, Error> {
  firestore
    .fluent()
    .select()
    .by_id_in("boards")
    .obj::<BoardInFirestore>()
    .one(&board_id)
    .await?
    .ok_or(Error::NotFound)
    .map(|board| board.into())
}

pub async fn update(
  firestore: Arc<Mutex<FirestoreV1Client>>,
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
) -> Result<(), Error> {
  let name = format!(
    "projects/{}/databases/(default)/documents/boards/{}",
    config.firestore_project, board_id
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
