use firestore::{FirestoreDb, FirestoreReference};

use actix_web::{delete, get, patch, post, web, HttpResponse};

use super::db;
use super::models::ColumnMessage;
use crate::boards;
use crate::boards::models::Board;
use crate::error::Error;
use crate::participants::models::Participant;

fn check_board_owner_permission(board: &Board, participant: &FirestoreReference) -> Result<(), Error> {
  if board.owner != *participant {
    Err(Error::Forbidden)
  } else {
    Ok(())
  }
}

#[post("boards/{board_id}/columns")]
pub async fn new(
  firestore: web::Data<FirestoreDb>,
  _participant: Participant,
  board_id: web::Path<String>,
  column_message: web::Json<ColumnMessage>,
) -> Result<HttpResponse, Error> {
  let column = db::new(&firestore, &board_id, column_message.into_inner()).await?;
  Ok(HttpResponse::Ok().json(column))
}

#[get("boards/{board_id}/columns")]
pub async fn list(
  firestore: web::Data<FirestoreDb>,
  _participant: Participant,
  board_id: web::Path<String>,
) -> Result<HttpResponse, Error> {
  let columns = db::list(&firestore, &board_id).await?;
  Ok(HttpResponse::Ok().json(columns))
}

#[get("boards/{board_id}/columns/{column_id}")]
pub async fn get(
  firestore: web::Data<FirestoreDb>,
  _participant: Participant,
  params: web::Path<(String, String)>,
) -> Result<HttpResponse, Error> {
  let (board_id, column_id) = params.into_inner();
  let column = db::get(&firestore, &board_id, &column_id).await;
  Ok(HttpResponse::Ok().json(column?))
}

#[patch("boards/{board_id}/columns/{column_id}")]
pub async fn update(
  firestore: web::Data<FirestoreDb>,
  participant: Participant,
  params: web::Path<(String, String)>,
  column_message: web::Json<ColumnMessage>,
) -> Result<HttpResponse, Error> {
  let (board_id, column_id) = params.into_inner();
  let board = boards::db::get(&firestore, &board_id).await?;
  let participant_reference = FirestoreReference(
    firestore
      .parent_path("participants", &participant.id)
      .unwrap()
      .into(),
  );
  check_board_owner_permission(&board, &participant_reference)?;
  let column = db::update(
    &firestore,
    &board_id,
    &column_id,
    column_message.into_inner(),
  )
  .await?;
  Ok(HttpResponse::Ok().json(column))
}

#[delete("boards/{board_id}/columns/{column_id}")]
pub async fn delete(
  firestore: web::Data<FirestoreDb>,
  participant: Participant,
  params: web::Path<(String, String)>,
) -> Result<HttpResponse, Error> {
  let (board_id, column_id) = params.into_inner();
  let participant_reference = FirestoreReference(
    firestore
      .parent_path("participants", &participant.id)
      .unwrap()
      .into(),
  );
  let board = boards::db::get(&firestore, &board_id).await?;
  check_board_owner_permission(&board, &participant_reference)?;
  db::delete(&firestore, &board_id, &column_id).await?;
  Ok(HttpResponse::Ok().finish())
}

#[cfg(test)]
mod tests {
  use super::*;
  use chrono::Utc;
  use serde_json::Map;

  fn ref_(s: &str) -> FirestoreReference {
    FirestoreReference(s.to_string())
  }

  fn make_board(owner: &str) -> Board {
    Board {
      id: "board1".to_string(),
      name: "Test".to_string(),
      cards_open: true,
      voting_open: true,
      ice_breaking: "".to_string(),
      created_at: Utc::now().timestamp(),
      owner: ref_(owner),
      anyone_is_owner: false,
      data: serde_json::Value::Object(Map::new()),
    }
  }

  #[test]
  fn board_owner_can_update_column() {
    let board = make_board("participants/owner");
    assert!(check_board_owner_permission(&board, &ref_("participants/owner")).is_ok());
  }

  #[test]
  fn non_owner_cannot_update_column() {
    let board = make_board("participants/owner");
    assert!(check_board_owner_permission(&board, &ref_("participants/other")).is_err());
  }

  #[test]
  fn non_owner_column_update_returns_forbidden() {
    let board = make_board("participants/owner");
    assert!(matches!(
      check_board_owner_permission(&board, &ref_("participants/other")),
      Err(Error::Forbidden)
    ));
  }

  #[test]
  fn board_owner_can_delete_column() {
    let board = make_board("participants/owner");
    assert!(check_board_owner_permission(&board, &ref_("participants/owner")).is_ok());
  }

  #[test]
  fn non_owner_cannot_delete_column() {
    let board = make_board("participants/owner");
    assert!(matches!(
      check_board_owner_permission(&board, &ref_("participants/other")),
      Err(Error::Forbidden)
    ));
  }
}
