use firestore::{FirestoreDb, FirestoreReference};

use actix_web::{delete, get, patch, post, web, HttpResponse};

use super::db;
use super::models::ColumnMessage;
use crate::boards;
use crate::error::Error;
use crate::participants::models::Participant;

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
  if board.owner != participant_reference {
    return Err(Error::Forbidden);
  }
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
  if board.owner != participant_reference {
    return Err(Error::Forbidden);
  }
  db::delete(&firestore, &board_id, &column_id).await?;
  Ok(HttpResponse::Ok().finish())
}
