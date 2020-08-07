use std::convert::TryFrom;
use std::convert::TryInto;

use super::models::*;
use crate::config::Config;
use crate::error::Error;
use crate::firestore::v1::*;
use crate::firestore::FirestoreV1Client;
use crate::participants::models::Participant;

pub async fn new(
  firestore: &mut FirestoreV1Client,
  config: &Config,
  participant: &Participant,
  board_id: String,
  card: CardMessage,
) -> Result<Card, Error> {
  let mut document: Document = card.into();
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
        "projects/{}/databases/(default)/documents/boards/{}",
        config.firestore_project, board_id
      ),
      collection_id: "cards".into(),
      document: Some(document),
      ..Default::default()
    })
    .await?;
  Card::try_from(result.into_inner())
}

pub async fn list(
  firestore: &mut FirestoreV1Client,
  config: &Config,
  board_id: String,
) -> Result<Vec<Card>, Error> {
  let result = firestore
    .list_documents(ListDocumentsRequest {
      parent: format!(
        "projects/{}/databases/(default)/documents/boards/{}",
        config.firestore_project, board_id
      ),
      collection_id: "cards".into(),
      page_size: 200,
      ..Default::default()
    })
    .await?;
  let documents = result.into_inner().documents;
  let (valid_documents, _): (Vec<_>, Vec<_>) = documents
    .into_iter()
    .map(Card::try_from)
    .partition(Result::is_ok);
  Ok(valid_documents.into_iter().map(Result::unwrap).collect())
}

pub async fn get(
  firestore: &mut FirestoreV1Client,
  config: &Config,
  board_id: String,
  card_id: String,
) -> Result<Card, Error> {
  let result = firestore
    .get_document(GetDocumentRequest {
      name: format!(
        "projects/{}/databases/(default)/documents/boards/{}/cards/{}",
        config.firestore_project, board_id, card_id
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
  card_id: String,
  card: CardMessage,
) -> Result<Card, Error> {
  let mut document: Document = card.into();
  document.name = format!(
    "projects/{}/databases/(default)/documents/boards/{}/cards/{}",
    config.firestore_project, board_id, card_id
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
  card_id: String,
) -> Result<(), Error> {
  let name = format!(
    "projects/{}/databases/(default)/documents/boards/{}/cards/{}",
    config.firestore_project, board_id, card_id
  );
  firestore
    .delete_document(DeleteDocumentRequest {
      name,
      ..Default::default()
    })
    .await?;
  Ok(())
}

pub async fn put_vote(
  firestore: &mut FirestoreV1Client,
  config: &Config,
  participant: &Participant,
  board_id: String,
  card_id: String,
) -> Result<(), Error> {
  let participant_doc_id = to_participant_reference!(config.firestore_project, participant.id);
  let card_doc_id = to_card_reference!(config.firestore_project, board_id, card_id);
  firestore
    .batch_write(BatchWriteRequest {
      database: format!("projects/{}/databases/(default)", config.firestore_project),
      writes: vec![Write {
        operation: Some(write::Operation::Transform(DocumentTransform {
          document: card_doc_id,
          field_transforms: vec![document_transform::FieldTransform {
            field_path: "votes".into(),
            transform_type: Some(
              document_transform::field_transform::TransformType::AppendMissingElements(
                ArrayValue {
                  values: vec![reference_value!(participant_doc_id)],
                },
              ),
            ),
          }],
        })),
        ..Default::default()
      }],
      ..Default::default()
    })
    .await?
    .into_inner();
  Ok(())
}

pub async fn delete_vote(
  firestore: &mut FirestoreV1Client,
  config: &Config,
  participant: &Participant,
  board_id: String,
  card_id: String,
) -> Result<(), Error> {
  let participant_doc_id = to_participant_reference!(config.firestore_project, participant.id);
  let card_doc_id = to_card_reference!(config.firestore_project, board_id, card_id);
  firestore
    .batch_write(BatchWriteRequest {
      database: format!("projects/{}/databases/(default)", config.firestore_project),
      writes: vec![Write {
        operation: Some(write::Operation::Transform(DocumentTransform {
          document: card_doc_id,
          field_transforms: vec![document_transform::FieldTransform {
            field_path: "votes".into(),
            transform_type: Some(
              document_transform::field_transform::TransformType::RemoveAllFromArray(ArrayValue {
                values: vec![reference_value!(participant_doc_id)],
              }),
            ),
          }],
        })),
        ..Default::default()
      }],
      ..Default::default()
    })
    .await?
    .into_inner();
  Ok(())
}
