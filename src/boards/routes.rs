use super::db;
use crate::error::Error;
use crate::firestore::FirestoreV1Client;
use crate::participants::models::Participant;
use actix_web::web;

pub async fn get_boards(
  firestore: web::Data<FirestoreV1Client>,
  participant: Participant,
) -> Result<web::HttpResponse, Error> {
  let firestore = &mut firestore.get_ref().clone();
  let boards = db::list(firestore, participant).await?;
  Ok(web::HttpResponse::Ok().json(boards))
}

pub async fn get_board(
  firestore: web::Data<FirestoreV1Client>,
  participant: Participant,
  board_id: web::Path<String>,
) -> Result<web::HttpResponse, Error> {
  let firestore = &mut firestore.get_ref().clone();
  let board = db::get(firestore, participant, board_id.to_string()).await?;
  Ok(web::HttpResponse::Ok().json(board))
}
