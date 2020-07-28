use std::cell::RefCell;
use std::convert::TryFrom;
use crate::error::Error;
use crate::firestore::FirestoreV1Client;
use crate::firestore::v1::ListDocumentsRequest;
use super::models::*;

pub async fn get_boards(firestore: &RefCell<FirestoreV1Client>) -> Result<Vec<Board>, Error> {
  let mut firestore = firestore.borrow_mut();
  let result = firestore.list_documents(ListDocumentsRequest {
    parent: "projects/retrotools-284402/databases/(default)/documents".into(),
    collection_id: "boards".into(),
    page_size: 10,
    page_token: "".into(),
    order_by: "".into(),
    mask: None,
    consistency_selector: None,
    show_missing: false
  }).await.map_err(|_| Error::Other)?;
  let documents = result.into_inner().documents;
  let (valid_documents, _): (Vec<_>, Vec<_>) = documents.into_iter().map(Board::try_from).partition(Result::is_ok);
  Ok(valid_documents.into_iter().map(Result::unwrap).collect())
}