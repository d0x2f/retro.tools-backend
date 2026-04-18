use firestore::path;
use firestore::paths;
use firestore::FirestoreDb;
use firestore::FirestoreReference;
use futures::stream::BoxStream;
use futures::StreamExt;
use std::convert::TryInto;

use super::models::*;
use crate::error::Error;
use crate::participants::models::Participant;

pub async fn new(
  firestore: &FirestoreDb,
  participant: &Participant,
  board_id: &String,
  card: CardMessage,
) -> Result<Card, Error> {
  let mut new_card: NewCard = card.try_into()?;
  new_card.owner = Some(FirestoreReference(format!(
    "{}/participants/{}",
    firestore.get_documents_path(),
    participant.id
  )));
  firestore
    .fluent()
    .insert()
    .into("cards")
    .generate_document_id()
    .parent(firestore.parent_path("boards", board_id)?)
    .object(&new_card)
    .execute::<CardInFirestore>()
    .await
    .map(|card| card.into())
    .map_err(|e| e.into())
}

pub async fn list(firestore: &FirestoreDb, board_id: &String) -> Result<Vec<Card>, Error> {
  let mut object_stream: BoxStream<Option<CardInFirestore>> = firestore
    .fluent()
    .list()
    .from("cards")
    .parent(firestore.parent_path("boards", board_id)?)
    .obj::<Option<CardInFirestore>>()
    .stream_all()
    .await?;

  let mut cards: Vec<Card> = vec![];
  while let Some(Some(card)) = object_stream.next().await {
    cards.push(card.into());
  }
  Ok(cards)
}

pub async fn get(
  firestore: &FirestoreDb,
  board_id: &String,
  card_id: &String,
) -> Result<Card, Error> {
  firestore
    .fluent()
    .select()
    .by_id_in("cards")
    .parent(firestore.parent_path("boards", board_id)?)
    .obj::<CardInFirestore>()
    .one(&card_id)
    .await?
    .ok_or(Error::NotFound)
    .map(|card| card.into())
}

pub async fn update(
  firestore: &FirestoreDb,
  board_id: &String,
  card_id: &String,
  card: CardMessage,
) -> Result<Card, Error> {
  let change_set = CardChangeSet {
    author: card.author,
    text: card.text,
    column: card.column.map(|c| {
      FirestoreReference(format!(
        "{}/columns/{}",
        firestore.parent_path("boards", board_id).unwrap(),
        c
      ))
    }),
  };
  let serialised_card = serde_json::to_value(&change_set)?;
  firestore
    .fluent()
    .update()
    .fields(
      paths!(CardMessage::{column, author, text})
        .into_iter()
        .filter(|f| serialised_card.get(f).is_some()),
    )
    .in_col("cards")
    .document_id(card_id)
    .parent(firestore.parent_path("boards", board_id)?)
    .object(&change_set)
    .execute::<CardInFirestore>()
    .await
    .map(|card| card.into())
    .map_err(|e| e.into())
}

pub async fn delete(
  firestore: &FirestoreDb,
  board_id: &String,
  card_id: &String,
) -> Result<(), Error> {
  firestore
    .fluent()
    .delete()
    .from("cards")
    .document_id(card_id)
    .parent(firestore.parent_path("boards", board_id)?)
    .execute()
    .await
    .map_err(|e| e.into())
}

pub async fn put_vote(
  firestore: &FirestoreDb,
  participant: &Participant,
  board_id: &String,
  card_id: &String,
) -> Result<(), Error> {
  let mut transaction = firestore.begin_transaction().await?;
  firestore
    .fluent()
    .update()
    .in_col("cards")
    .document_id(card_id)
    .parent(firestore.parent_path("boards", board_id)?)
    .transforms(|t| {
      t.fields([t
        .field(path!(CardInFirestore::votes))
        .append_missing_elements([FirestoreReference(
          firestore
            .parent_path("participants", &participant.id)
            .unwrap()
            .into(),
        )])])
    })
    .only_transform()
    .add_to_transaction(&mut transaction)?;
  transaction.commit().await?;
  Ok(())
}

pub async fn delete_vote(
  firestore: &FirestoreDb,
  participant: &Participant,
  board_id: &String,
  card_id: &String,
) -> Result<(), Error> {
  let mut transaction = firestore.begin_transaction().await?;
  firestore
    .fluent()
    .update()
    .in_col("cards")
    .document_id(card_id)
    .parent(firestore.parent_path("boards", board_id)?)
    .transforms(|t| {
      t.fields([t
        .field(path!(CardInFirestore::votes))
        .remove_all_from_array([FirestoreReference(
          firestore
            .parent_path("participants", &participant.id)
            .unwrap()
            .into(),
        )])])
    })
    .only_transform()
    .add_to_transaction(&mut transaction)?;
  transaction.commit().await?;
  Ok(())
}

pub async fn put_reaction(
  firestore: &FirestoreDb,
  participant: &Participant,
  board_id: &String,
  card_id: &String,
  emoji: &String,
) -> Result<(), Error> {
  // Delete an existing reaction first
  delete_reaction(firestore, participant, board_id, card_id).await?;

  let mut transaction = firestore.begin_transaction().await?;
  firestore
    .fluent()
    .update()
    .in_col("cards")
    .document_id(card_id)
    .parent(firestore.parent_path("boards", board_id)?)
    .transforms(|t| {
      t.fields([t
        .field(format!("reactions.`{}`", emoji))
        .append_missing_elements([FirestoreReference(
          firestore
            .parent_path("participants", &participant.id)
            .unwrap()
            .into(),
        )])])
    })
    .only_transform()
    .add_to_transaction(&mut transaction)?;
  transaction.commit().await?;
  Ok(())
}

pub async fn delete_reaction(
  firestore: &FirestoreDb,
  participant: &Participant,
  board_id: &String,
  card_id: &String,
) -> Result<(), Error> {
  let card = get(firestore, board_id, card_id).await?;
  let participant_reference = FirestoreReference(
    firestore
      .parent_path("participants", &participant.id)
      .unwrap()
      .into(),
  );

  // Find which emoji the participant has reacted with
  let mut reaction: Option<String> = None;
  for (emoji, participants) in card.reactions {
    if participants.contains(&participant_reference.0) {
      reaction = Some(emoji);
    }
  }

  let Some(emoji) = reaction else {
    return Ok(());
  };

  let mut transaction = firestore.begin_transaction().await?;
  firestore
    .fluent()
    .update()
    .in_col("cards")
    .document_id(card_id)
    .parent(firestore.parent_path("boards", board_id)?)
    .transforms(|t| {
      t.fields([t
        .field(format!("reactions.`{}`", emoji))
        .remove_all_from_array([FirestoreReference(
          firestore
            .parent_path("participants", &participant.id)
            .unwrap()
            .into(),
        )])])
    })
    .only_transform()
    .add_to_transaction(&mut transaction)?;
  transaction.commit().await?;
  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::boards;
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

  async fn setup_board(db: &FirestoreDb) -> String {
    boards::db::new(
      db,
      &test_participant(),
      BoardMessage {
        name: Some("Card Integration Test Board".to_string()),
        cards_open: Some(true),
        voting_open: Some(true),
        ice_breaking: None,
        data: None,
      },
    )
    .await
    .unwrap()
    .id
  }

  fn card_msg(column_path: &str) -> CardMessage {
    CardMessage {
      author: Some("Test Author".to_string()),
      text: Some("Test card text".to_string()),
      column: Some(column_path.to_string()),
    }
  }

  #[tokio::test]
  #[ignore = "requires Firestore emulator: FIRESTORE_EMULATOR_HOST=localhost:8080"]
  async fn new_card_can_be_retrieved_by_id() {
    let db = emulator_db().await;
    let board_id = setup_board(&db).await;
    let column_ref = format!("{}/boards/{}/columns/col1", db.get_documents_path(), board_id);
    let card = new(&db, &test_participant(), &board_id, card_msg(&column_ref)).await.unwrap();
    let fetched = get(&db, &board_id, &card.id).await.unwrap();
    assert_eq!(fetched.id, card.id);
    assert_eq!(fetched.text, "Test card text");
    assert_eq!(fetched.author, "Test Author");
    boards::db::delete(&db, &board_id).await.unwrap();
  }

  #[tokio::test]
  #[ignore = "requires Firestore emulator: FIRESTORE_EMULATOR_HOST=localhost:8080"]
  async fn get_nonexistent_card_returns_not_found() {
    let db = emulator_db().await;
    let board_id = setup_board(&db).await;
    let result = get(&db, &board_id, &"no-such-card".to_string()).await;
    assert!(matches!(result, Err(crate::error::Error::NotFound)));
    boards::db::delete(&db, &board_id).await.unwrap();
  }

  #[tokio::test]
  #[ignore = "requires Firestore emulator: FIRESTORE_EMULATOR_HOST=localhost:8080"]
  async fn put_vote_adds_participant_to_votes() {
    let db = emulator_db().await;
    let participant = test_participant();
    let board_id = setup_board(&db).await;
    let column_ref = format!("{}/boards/{}/columns/col1", db.get_documents_path(), board_id);
    let card = new(&db, &participant, &board_id, card_msg(&column_ref)).await.unwrap();
    assert!(card.votes.is_empty());
    put_vote(&db, &participant, &board_id, &card.id).await.unwrap();
    let after = get(&db, &board_id, &card.id).await.unwrap();
    assert_eq!(after.votes.len(), 1);
    boards::db::delete(&db, &board_id).await.unwrap();
  }

  #[tokio::test]
  #[ignore = "requires Firestore emulator: FIRESTORE_EMULATOR_HOST=localhost:8080"]
  async fn put_vote_is_idempotent() {
    let db = emulator_db().await;
    let participant = test_participant();
    let board_id = setup_board(&db).await;
    let column_ref = format!("{}/boards/{}/columns/col1", db.get_documents_path(), board_id);
    let card = new(&db, &participant, &board_id, card_msg(&column_ref)).await.unwrap();
    put_vote(&db, &participant, &board_id, &card.id).await.unwrap();
    put_vote(&db, &participant, &board_id, &card.id).await.unwrap();
    let after = get(&db, &board_id, &card.id).await.unwrap();
    assert_eq!(after.votes.len(), 1, "duplicate vote should not be recorded");
    boards::db::delete(&db, &board_id).await.unwrap();
  }

  #[tokio::test]
  #[ignore = "requires Firestore emulator: FIRESTORE_EMULATOR_HOST=localhost:8080"]
  async fn delete_vote_removes_participant_from_votes() {
    let db = emulator_db().await;
    let participant = test_participant();
    let board_id = setup_board(&db).await;
    let column_ref = format!("{}/boards/{}/columns/col1", db.get_documents_path(), board_id);
    let card = new(&db, &participant, &board_id, card_msg(&column_ref)).await.unwrap();
    put_vote(&db, &participant, &board_id, &card.id).await.unwrap();
    delete_vote(&db, &participant, &board_id, &card.id).await.unwrap();
    let after = get(&db, &board_id, &card.id).await.unwrap();
    assert!(after.votes.is_empty());
    boards::db::delete(&db, &board_id).await.unwrap();
  }

  #[tokio::test]
  #[ignore = "requires Firestore emulator: FIRESTORE_EMULATOR_HOST=localhost:8080"]
  async fn put_reaction_records_emoji_for_participant() {
    let db = emulator_db().await;
    let participant = test_participant();
    let board_id = setup_board(&db).await;
    let column_ref = format!("{}/boards/{}/columns/col1", db.get_documents_path(), board_id);
    let card = new(&db, &participant, &board_id, card_msg(&column_ref)).await.unwrap();
    put_reaction(&db, &participant, &board_id, &card.id, &"👍".to_string()).await.unwrap();
    let after = get(&db, &board_id, &card.id).await.unwrap();
    assert!(after.reactions.get("👍").is_some_and(|v| !v.is_empty()));
    boards::db::delete(&db, &board_id).await.unwrap();
  }

  #[tokio::test]
  #[ignore = "requires Firestore emulator: FIRESTORE_EMULATOR_HOST=localhost:8080"]
  async fn put_reaction_replaces_prior_reaction() {
    let db = emulator_db().await;
    let participant = test_participant();
    let board_id = setup_board(&db).await;
    let column_ref = format!("{}/boards/{}/columns/col1", db.get_documents_path(), board_id);
    let card = new(&db, &participant, &board_id, card_msg(&column_ref)).await.unwrap();
    put_reaction(&db, &participant, &board_id, &card.id, &"👍".to_string()).await.unwrap();
    put_reaction(&db, &participant, &board_id, &card.id, &"❤️".to_string()).await.unwrap();
    let after = get(&db, &board_id, &card.id).await.unwrap();
    let thumbs_up = after.reactions.get("👍").map(|v| v.len()).unwrap_or(0);
    let heart = after.reactions.get("❤️").map(|v| v.len()).unwrap_or(0);
    assert_eq!(thumbs_up, 0, "old reaction should be removed");
    assert_eq!(heart, 1, "new reaction should be recorded");
    boards::db::delete(&db, &board_id).await.unwrap();
  }

  #[tokio::test]
  #[ignore = "requires Firestore emulator: FIRESTORE_EMULATOR_HOST=localhost:8080"]
  async fn delete_reaction_removes_emoji_for_participant() {
    let db = emulator_db().await;
    let participant = test_participant();
    let board_id = setup_board(&db).await;
    let column_ref = format!("{}/boards/{}/columns/col1", db.get_documents_path(), board_id);
    let card = new(&db, &participant, &board_id, card_msg(&column_ref)).await.unwrap();
    put_reaction(&db, &participant, &board_id, &card.id, &"🎉".to_string()).await.unwrap();
    delete_reaction(&db, &participant, &board_id, &card.id).await.unwrap();
    let after = get(&db, &board_id, &card.id).await.unwrap();
    let count = after.reactions.get("🎉").map(|v| v.len()).unwrap_or(0);
    assert_eq!(count, 0);
    boards::db::delete(&db, &board_id).await.unwrap();
  }
}
