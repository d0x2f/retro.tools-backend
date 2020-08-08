use super::models::*;
use crate::config::Config;
use crate::error::Error;
use crate::firestore::v1::*;
use crate::firestore::FirestoreV1Client;

pub async fn new(firestore: &mut FirestoreV1Client, config: &Config) -> Result<Participant, Error> {
  let result = firestore
    .create_document(CreateDocumentRequest {
      parent: format!(
        "projects/{}/databases/(default)/documents",
        config.firestore_project
      ),
      collection_id: "participants".into(),
      ..Default::default()
    })
    .await?;
  Ok(result.into_inner().into())
}

pub async fn get(
  firestore: &mut FirestoreV1Client,
  config: &Config,
  id: &str,
) -> Result<Participant, Error> {
  let result = firestore
    .get_document(GetDocumentRequest {
      name: format!(
        "projects/{}/databases/(default)/documents/participants/{}",
        config.firestore_project, id
      ),
      mask: Some(DocumentMask {
        field_paths: vec![],
      }),
      ..Default::default()
    })
    .await?;
  Ok(result.into_inner().into())
}

pub async fn add_participant_board(
  firestore: &mut FirestoreV1Client,
  config: &Config,
  participant: &Participant,
  board_id: String,
) -> Result<(), Error> {
  let participant_doc_id = to_participant_reference!(config.firestore_project, participant.id);
  let board_doc_id = to_board_reference!(config.firestore_project, board_id);
  firestore
    .batch_write(BatchWriteRequest {
      database: format!("projects/{}/databases/(default)", config.firestore_project),
      writes: vec![Write {
        operation: Some(write::Operation::Transform(DocumentTransform {
          document: participant_doc_id,
          field_transforms: vec![document_transform::FieldTransform {
            field_path: "boards".into(),
            transform_type: Some(
              document_transform::field_transform::TransformType::AppendMissingElements(
                ArrayValue {
                  values: vec![reference_value!(board_doc_id)],
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

pub async fn get_participant_board_ids(
  firestore: &mut FirestoreV1Client,
  config: &Config,
  participant: &Participant,
) -> Result<Vec<String>, Error> {
  let result = match firestore
    .get_document(GetDocumentRequest {
      name: to_participant_reference!(config.firestore_project, participant.id),
      ..Default::default()
    })
    .await
  {
    Ok(r) => r,
    Err(error) => {
      return match error.code() {
        tonic::Code::NotFound => Ok(vec![]),
        _ => Err(error.into()),
      }
    }
  };
  let document = result.into_inner();
  match get_array_field!(document, "boards") {
    Err(_) => Ok(vec![]),
    Ok(boards) => {
      let (valid_boards, _): (Vec<_>, Vec<_>) = boards
        .values
        .clone()
        .into_iter()
        .map(|b| extract_string!(b.value_type))
        .partition(Option::is_some);
      Ok(valid_boards.into_iter().map(Option::unwrap).collect())
    }
  }
}
