use super::models::*;
use crate::error::Error;
use crate::firestore::v1::*;
use crate::firestore::FirestoreV1Client;

pub async fn new(firestore: &mut FirestoreV1Client) -> Result<Participant, Error> {
  let result = firestore
    .create_document(CreateDocumentRequest {
      parent: "projects/retrotools-284402/databases/(default)/documents".into(),
      collection_id: "participants".into(),
      ..Default::default()
    })
    .await?;
  Ok(result.into_inner().into())
}

pub async fn add_participant_board(
  firestore: &mut FirestoreV1Client,
  participant: Participant,
  board_id: String,
) -> Result<(), Error> {
  let participant_doc_id = to_participant_reference!("retrotools-284402", participant.id);
  let board_doc_id = to_board_reference!("retrotools-284402", board_id);
  firestore
    .batch_write(BatchWriteRequest {
      database: "projects/retrotools-284402/databases/(default)".into(),
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
  participant: Participant,
) -> Result<Vec<String>, Error> {
  let result = match firestore
    .get_document(GetDocumentRequest {
      name: to_participant_reference!("retrotools-284402", participant.id),
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
