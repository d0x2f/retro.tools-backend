use firestore::paths;
use firestore::FirestoreDb;
use firestore::FirestoreReference;
use futures::stream::BoxStream;
use futures::StreamExt;

use super::models::*;
use crate::error::Error;
use crate::participants::db::get_participant_board_ids;
use crate::participants::models::Participant;

pub async fn new(
  firestore: &FirestoreDb,
  participant: &Participant,
  board: BoardMessage,
) -> Result<Board, Error> {
  let mut new_board: NewBoard = board.into();
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
  firestore: &FirestoreDb,
  board_id: String,
  board: BoardMessage,
) -> Result<Board, Error> {
  let serialised_board = serde_json::to_value(&board)?;
  firestore
    .fluent()
    .update()
    .fields(
      paths!(BoardMessage::{name, cards_open, voting_open, ice_breaking, data})
        .into_iter()
        .filter(|f| serialised_board.get(f).is_some()),
    )
    .in_col("boards")
    .document_id(&board_id)
    .object(&board)
    .execute::<BoardInFirestore>()
    .await
    .map(|board| board.into())
    .map_err(|e| e.into())
}

pub async fn delete(firestore: &FirestoreDb, board_id: String) -> Result<(), Error> {
  firestore
    .fluent()
    .delete()
    .from("boards")
    .document_id(&board_id)
    .execute()
    .await
    .map_err(|e| e.into())
}
