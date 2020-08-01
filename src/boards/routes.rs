use actix_web::web;
use futures::future::join;

use super::db;
use super::models::BoardMessage;
use crate::error::Error;
use crate::firestore::FirestoreV1Client;
use crate::participants::db::*;
use crate::participants::models::Participant;

pub async fn list(
  firestore: web::Data<FirestoreV1Client>,
  participant: Participant,
) -> Result<web::HttpResponse, Error> {
  let firestore = &mut firestore.get_ref().clone();
  let boards = db::list(firestore, participant).await?;
  Ok(web::HttpResponse::Ok().json(boards))
}

pub async fn get(
  firestore: web::Data<FirestoreV1Client>,
  participant: Participant,
  board_id: web::Path<String>,
) -> Result<web::HttpResponse, Error> {
  let firestore = &mut firestore.get_ref().clone();
  let (_, board) = join(
    add_participant_board(&mut firestore.clone(), participant, board_id.clone()),
    db::get(firestore, board_id.to_string()),
  )
  .await;
  Ok(web::HttpResponse::Ok().json(board?))
}

pub async fn new(
  firestore: web::Data<FirestoreV1Client>,
  participant: Participant,
  board_message: web::Json<BoardMessage>,
) -> Result<web::HttpResponse, Error> {
  let mut board_message = board_message.into_inner();
  board_message.voting_open.get_or_insert(true);
  board_message.cards_open.get_or_insert(true);
  let firestore = &mut firestore.get_ref().clone();
  let board = db::new(firestore, participant.clone(), board_message).await?;
  add_participant_board(&mut firestore.clone(), participant, board.id.clone()).await?;
  Ok(web::HttpResponse::Ok().json(board))
}

pub async fn update(
  firestore: web::Data<FirestoreV1Client>,
  participant: Participant,
  board_id: web::Path<String>,
  board_message: web::Json<BoardMessage>,
) -> Result<web::HttpResponse, Error> {
  let firestore = &mut firestore.get_ref().clone();
  let board = db::get(firestore, board_id.to_string()).await?;
  if board.owner != participant.id {
    return Err(Error::Forbidden);
  }
  let board = db::update(firestore, board_id.to_string(), board_message.into_inner()).await?;
  Ok(web::HttpResponse::Ok().json(board))
}

pub async fn delete(
  firestore: web::Data<FirestoreV1Client>,
  participant: Participant,
  board_id: web::Path<String>,
) -> Result<web::HttpResponse, Error> {
  let firestore = &mut firestore.get_ref().clone();
  let board = db::get(firestore, board_id.to_string()).await?;
  if board.owner != participant.id {
    return Err(Error::Forbidden);
  }
  db::delete(firestore, board_id.to_string()).await?;
  Ok(web::HttpResponse::Ok().finish())
}
