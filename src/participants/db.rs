use chrono::Utc;
use firestore::path;
use firestore::FirestoreDb;
use firestore::FirestoreReference;

use super::models::*;
use crate::error::Error;

pub async fn new(firestore: &FirestoreDb) -> Result<Participant, Error> {
  let new_participant = NewParticipant {
    created_at: Utc::now().into(),
  };

  firestore
    .fluent()
    .insert()
    .into("participants")
    .generate_document_id()
    .object(&new_participant)
    .execute::<ParticipantInFirestore>()
    .await
    .map(|participant| participant.into())
    .map_err(|e| e.into())
}

pub async fn add_participant_board(
  firestore: &FirestoreDb,
  participant: &Participant,
  board_id: &String,
) -> Result<(), Error> {
  let mut transaction = firestore.begin_transaction().await?;
  firestore
    .fluent()
    .update()
    .in_col("participants")
    .document_id(&participant.id)
    .transforms(|t| {
      t.fields([t
        .field(path!(ParticipantInFirestore::boards))
        .append_missing_elements([FirestoreReference(format!(
          "{}/boards/{}",
          firestore.get_documents_path(),
          board_id
        ))])])
    })
    .only_transform()
    .add_to_transaction(&mut transaction)?;
  transaction.commit().await?;
  Ok(())
}

pub async fn get_participant_board_ids(
  firestore: &FirestoreDb,
  participant: &Participant,
) -> Result<Vec<String>, Error> {
  let result: Option<ParticipantInFirestore> = firestore
    .fluent()
    .select()
    .by_id_in("participants")
    .obj()
    .one(&participant.id)
    .await?;

  if let Some(participant) = result {
    Ok(
      participant
        .boards
        .unwrap_or(vec![])
        .into_iter()
        .map(|id| id.split('/').last().unwrap().to_string())
        .collect(),
    )
  } else {
    Ok(vec![])
  }
}
