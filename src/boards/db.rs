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

pub async fn get(firestore: &FirestoreDb, board_id: &String) -> Result<Board, Error> {
  firestore
    .fluent()
    .select()
    .by_id_in("boards")
    .obj::<BoardInFirestore>()
    .one(board_id)
    .await?
    .ok_or(Error::NotFound)
    .map(|board| board.into())
}

pub async fn update(
  firestore: &FirestoreDb,
  board_id: &String,
  board: BoardMessage,
) -> Result<Board, Error> {
  let serialised_board = serde_json::to_value(&board)?;
  firestore
    .fluent()
    .update()
    .fields(
      paths!(BoardMessage::{name, cards_open, voting_open, ice_breaking, data, anyone_is_owner})
        .into_iter()
        .filter(|f| serialised_board.get(f).is_some()),
    )
    .in_col("boards")
    .document_id(board_id)
    .object(&board)
    .execute::<BoardInFirestore>()
    .await
    .map(|board| board.into())
    .map_err(|e| e.into())
}

pub async fn delete(firestore: &FirestoreDb, board_id: &String) -> Result<(), Error> {
  firestore
    .fluent()
    .delete()
    .from("boards")
    .document_id(board_id)
    .execute()
    .await
    .map_err(|e| e.into())
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::boards::models::BoardMessage;
  use crate::participants::models::Participant;

  // Run with: FIRESTORE_EMULATOR_HOST=localhost:8080 cargo test -- --ignored
  async fn emulator_db() -> FirestoreDb {
    use chrono::Utc;
    use firestore::FirestoreDbOptions;
    use gcloud_sdk::{ExternalJwtFunctionSource, Token, TokenSourceType};
    // The emulator ignores auth entirely; supply a fake token to bypass ADC.
    let token_source = ExternalJwtFunctionSource::new(|| async {
      Ok(Token::new(
        "Bearer".to_string(),
        // Minimal JWT the emulator accepts: base64url(header).base64url(payload).
        // The emulator checks format but not signature.
        "eyJhbGciOiJub25lIiwidHlwIjoiSldUIn0.eyJzdWIiOiJ0ZXN0In0.".into(),
        Utc::now() + chrono::Duration::hours(1),
      ))
    });
    FirestoreDb::with_options_token_source(
      FirestoreDbOptions::new("test-project".to_string()),
      vec![],
      TokenSourceType::ExternalSource(Box::new(token_source)),
    )
    .await
    .unwrap()
  }

  fn test_participant() -> Participant {
    Participant { id: "integration-test-participant".to_string() }
  }

  fn board_msg(name: &str) -> BoardMessage {
    BoardMessage {
      name: Some(name.to_string()),
      cards_open: Some(true),
      voting_open: Some(false),
      ice_breaking: None,
      data: None,
      anyone_is_owner: None,
    }
  }

  #[tokio::test]
  #[ignore = "requires Firestore emulator: FIRESTORE_EMULATOR_HOST=localhost:8080"]
  async fn new_board_can_be_retrieved_by_id() {
    let db = emulator_db().await;
    let participant = test_participant();
    let board = new(&db, &participant, board_msg("Integration Test Board")).await.unwrap();
    let fetched = get(&db, &board.id).await.unwrap();
    assert_eq!(fetched.id, board.id);
    assert_eq!(fetched.name, "Integration Test Board");
    assert!(fetched.cards_open);
    assert!(!fetched.voting_open);
    delete(&db, &board.id).await.unwrap();
  }

  #[tokio::test]
  #[ignore = "requires Firestore emulator: FIRESTORE_EMULATOR_HOST=localhost:8080"]
  async fn get_nonexistent_board_returns_not_found() {
    let db = emulator_db().await;
    let result = get(&db, &"nonexistent-board-id-xyz".to_string()).await;
    assert!(matches!(result, Err(crate::error::Error::NotFound)));
  }

  #[tokio::test]
  #[ignore = "requires Firestore emulator: FIRESTORE_EMULATOR_HOST=localhost:8080"]
  async fn update_board_changes_fields() {
    let db = emulator_db().await;
    let participant = test_participant();
    let board = new(&db, &participant, board_msg("Before Update")).await.unwrap();
    let updated = update(
      &db,
      &board.id,
      BoardMessage {
        name: Some("After Update".to_string()),
        cards_open: Some(false),
        voting_open: None,
        ice_breaking: None,
        data: None,
        anyone_is_owner: None,
      },
    )
    .await
    .unwrap();
    assert_eq!(updated.name, "After Update");
    assert!(!updated.cards_open);
    delete(&db, &board.id).await.unwrap();
  }

  #[tokio::test]
  #[ignore = "requires Firestore emulator: FIRESTORE_EMULATOR_HOST=localhost:8080"]
  async fn new_board_with_anyone_is_owner_true_persists() {
    let db = emulator_db().await;
    let participant = test_participant();
    let board = new(
      &db,
      &participant,
      BoardMessage {
        name: Some("Anyone Is Owner Board".to_string()),
        cards_open: Some(true),
        voting_open: Some(true),
        ice_breaking: None,
        data: None,
        anyone_is_owner: Some(true),
      },
    )
    .await
    .unwrap();
    assert!(board.anyone_is_owner);
    let fetched = get(&db, &board.id).await.unwrap();
    assert!(fetched.anyone_is_owner);
    delete(&db, &board.id).await.unwrap();
  }

  #[tokio::test]
  #[ignore = "requires Firestore emulator: FIRESTORE_EMULATOR_HOST=localhost:8080"]
  async fn update_board_anyone_is_owner_persists() {
    let db = emulator_db().await;
    let participant = test_participant();
    let board = new(&db, &participant, board_msg("Toggle Anyone Is Owner")).await.unwrap();
    assert!(!board.anyone_is_owner);
    let updated = update(
      &db,
      &board.id,
      BoardMessage {
        name: None,
        cards_open: None,
        voting_open: None,
        ice_breaking: None,
        data: None,
        anyone_is_owner: Some(true),
      },
    )
    .await
    .unwrap();
    assert!(updated.anyone_is_owner);
    let fetched = get(&db, &board.id).await.unwrap();
    assert!(fetched.anyone_is_owner);
    delete(&db, &board.id).await.unwrap();
  }

  #[tokio::test]
  #[ignore = "requires Firestore emulator: FIRESTORE_EMULATOR_HOST=localhost:8080"]
  async fn update_board_anyone_is_owner_toggle_off_persists() {
    let db = emulator_db().await;
    let participant = test_participant();
    let board = new(
      &db,
      &participant,
      BoardMessage {
        name: Some("Anyone Is Owner Board".to_string()),
        cards_open: Some(true),
        voting_open: Some(true),
        ice_breaking: None,
        data: None,
        anyone_is_owner: Some(true),
      },
    )
    .await
    .unwrap();
    assert!(board.anyone_is_owner);
    let updated = update(
      &db,
      &board.id,
      BoardMessage {
        name: None,
        cards_open: None,
        voting_open: None,
        ice_breaking: None,
        data: None,
        anyone_is_owner: Some(false),
      },
    )
    .await
    .unwrap();
    assert!(!updated.anyone_is_owner);
    let fetched = get(&db, &board.id).await.unwrap();
    assert!(!fetched.anyone_is_owner);
    delete(&db, &board.id).await.unwrap();
  }

  #[tokio::test]
  #[ignore = "requires Firestore emulator: FIRESTORE_EMULATOR_HOST=localhost:8080"]
  async fn delete_board_makes_it_unretrievable() {
    let db = emulator_db().await;
    let participant = test_participant();
    let board = new(&db, &participant, board_msg("To Be Deleted")).await.unwrap();
    delete(&db, &board.id).await.unwrap();
    let result = get(&db, &board.id).await;
    assert!(matches!(result, Err(crate::error::Error::NotFound)));
  }
}
