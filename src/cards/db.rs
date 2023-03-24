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
