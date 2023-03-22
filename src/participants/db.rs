use firestore::path;
use firestore::FirestoreDb;
use firestore::FirestoreReference;
use futures::lock::Mutex;
use std::collections::HashMap;
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
) -> Result<Participant, Error> {
  let mut fields: HashMap<String, Value> = HashMap::new();
  let now = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)?;
  fields.insert(
    "created_at".into(),
    timestamp_value!(now.as_secs() as i64, now.subsec_nanos() as i32),
  );
  let result = firestore
    .lock()
    .await
    .create_document(CreateDocumentRequest {
      parent: format!(
        "projects/{}/databases/(default)/documents",
        config.firestore_project
      ),
      collection_id: "participants".into(),
      document: Some(Document {
        name: "".into(),
        fields,
        create_time: None,
        update_time: None,
      }),
      ..Default::default()
    })
    .await?;
  Ok(result.into_inner().into())
}

pub async fn add_participant_board(
  firestore: &FirestoreDb,
  participant: &Participant,
  board_id: String,
) -> Result<(), Error> {
  let mut transaction = firestore.begin_transaction().await?;
  firestore
    .fluent()
    .update()
    .in_col("participants")
    .document_id(participant.id.clone())
    .transforms(|t| {
      t.fields([t
        .field(path!(ParticipantBoardIds::boards))
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
  let result: Option<ParticipantBoardIds> = firestore
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
        .into_iter()
        .map(|id| id.split('/').last().unwrap().to_string())
        .collect(),
    )
  } else {
    Ok(vec![])
  }
}
