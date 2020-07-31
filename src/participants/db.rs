use super::models::*;
use crate::error::Error;
use crate::firestore::v1::*;
use crate::firestore::FirestoreV1Client;

pub async fn new(firestore: &mut FirestoreV1Client) -> Result<Participant, Error> {
  let result = firestore
    .create_document(CreateDocumentRequest {
      parent: "projects/retrotools-284402/databases/(default)/documents".into(),
      collection_id: "participants".into(),
      document_id: "".into(),
      mask: None,
      document: None,
    })
    .await?;
  Ok(result.into_inner().into())
}

pub async fn add_participant_board(
  firestore: &mut FirestoreV1Client,
  participant: Participant,
  board_id: String,
) -> Result<Document, Error> {
  let result = firestore
    .create_document(CreateDocumentRequest {
      parent: format!(
        "projects/retrotools-284402/databases/(default)/documents/participants/{}",
        participant.id
      )
      .into(),
      collection_id: "boards".into(),
      document_id: board_id,
      mask: None,
      document: None,
    })
    .await?;
  Ok(result.into_inner())
}

pub async fn get_participant_board_ids(
  firestore: &mut FirestoreV1Client,
  participant: Participant,
) -> Result<Vec<String>, Error> {
  let result = firestore
    .list_documents(ListDocumentsRequest {
      parent: format!(
        "projects/retrotools-284402/databases/(default)/documents/participants/{}",
        participant.id
      )
      .into(),
      collection_id: "boards".into(),
      page_size: 10,
      page_token: "".into(),
      order_by: "".into(),
      mask: None,
      show_missing: false,
      consistency_selector: None,
    })
    .await?;
  let documents = result.into_inner().documents;
  Ok(
    documents
      .into_iter()
      .map(|d| {
        format!(
          "projects/retrotools-284402/databases/(default)/documents/boards/{}",
          d.name.rsplitn(2, '/').next().expect("document id")
        )
        .into()
      })
      .collect(),
  )
}
