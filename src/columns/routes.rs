use firestore::FirestoreDb;

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
  let firestore = firestore.into_inner();
  let column = db::new(
    &firestore,
    board_id.to_string(),
    column_message.into_inner(),
  )
  .await?;
  Ok(HttpResponse::Ok().json(column))
}

#[get("boards/{board_id}/columns")]
pub async fn list(
  firestore: web::Data<FirestoreDb>,
  _participant: Participant,
  board_id: web::Path<String>,
) -> Result<HttpResponse, Error> {
  let firestore = firestore.into_inner();
  let columns = db::list(&firestore, board_id.to_string()).await?;
  Ok(HttpResponse::Ok().json(columns))
}

#[get("boards/{board_id}/columns/{column_id}")]
pub async fn get(
  firestore: web::Data<FirestoreDb>,
  _participant: Participant,
  params: web::Path<(String, String)>,
) -> Result<HttpResponse, Error> {
  let firestore = firestore.into_inner();
  let (board_id, column_id) = params.into_inner();
  let column = db::get(&firestore, board_id.to_string(), column_id.to_string()).await;
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
  let board = boards::db::get(&firestore, board_id.to_string()).await?;
  if board.owner != participant.id {
    return Err(Error::Forbidden);
  }
  let column = db::update(
    &firestore,
    board_id.to_string(),
    column_id.to_string(),
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
  let firestore = firestore.into_inner();
  let (board_id, column_id) = params.into_inner();
  let board = boards::db::get(&firestore, board_id.to_string()).await?;
  if board.owner != participant.id {
    return Err(Error::Forbidden);
  }
  db::delete(&firestore, board_id.to_string(), column_id.to_string()).await?;
  Ok(HttpResponse::Ok().finish())
}
