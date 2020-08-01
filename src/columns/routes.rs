use actix_web::web;

use super::db;
use crate::boards;
use super::models::ColumnMessage;
use crate::error::Error;
use crate::firestore::FirestoreV1Client;
use crate::participants::models::Participant;

pub async fn new(
  firestore: web::Data<FirestoreV1Client>,
  _participant: Participant,
  board_id: web::Path<String>,
  column_message: web::Json<ColumnMessage>,
) -> Result<web::HttpResponse, Error> {
  let firestore = &mut firestore.get_ref().clone();
  let column = db::new(firestore, board_id.to_string(), column_message.into_inner()).await?;
  Ok(web::HttpResponse::Ok().json(column))
}

pub async fn list(
  firestore: web::Data<FirestoreV1Client>,
  _participant: Participant,
  board_id: web::Path<String>,
) -> Result<web::HttpResponse, Error> {
  let firestore = &mut firestore.get_ref().clone();
  let columns = db::list(firestore, board_id.to_string()).await?;
  Ok(web::HttpResponse::Ok().json(columns))
}

pub async fn get(
  firestore: web::Data<FirestoreV1Client>,
  _participant: Participant,
  params: web::Path<(String, String)>,
) -> Result<web::HttpResponse, Error> {
  let (board_id, column_id) = params.into_inner();
  let firestore = &mut firestore.get_ref().clone();
  let column = db::get(firestore, board_id.to_string(), column_id.to_string()).await;
  Ok(web::HttpResponse::Ok().json(column?))
}

pub async fn update(
  firestore: web::Data<FirestoreV1Client>,
  participant: Participant,
  params: web::Path<(String, String)>,
  column_message: web::Json<ColumnMessage>,
) -> Result<web::HttpResponse, Error> {
  let (board_id, column_id) = params.into_inner();
  let firestore = &mut firestore.get_ref().clone();
  let board = boards::db::get(firestore, board_id.to_string()).await?;
  if board.owner != participant.id {
    return Err(Error::Forbidden);
  }
  let column = db::update(firestore, board_id.to_string(), column_id.to_string(), column_message.into_inner()).await?;
  Ok(web::HttpResponse::Ok().json(column))
}

pub async fn delete(
  firestore: web::Data<FirestoreV1Client>,
  participant: Participant,
  params: web::Path<(String, String)>,
) -> Result<web::HttpResponse, Error> {
  let (board_id, column_id) = params.into_inner();
  let firestore = &mut firestore.get_ref().clone();
  let board = boards::db::get(firestore, board_id.to_string()).await?;
  if board.owner != participant.id {
    return Err(Error::Forbidden);
  }
  db::delete(firestore, board_id.to_string(), column_id.to_string()).await?;
  Ok(web::HttpResponse::Ok().finish())
}