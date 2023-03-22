use actix_web::{delete, get, patch, post, web, HttpResponse};
use firestore::FirestoreDb;
use futures::future::join;

use super::db;
use super::models::*;
use crate::error::Error;
use crate::participants::db::*;
use crate::participants::models::Participant;

#[post("boards")]
pub async fn new(
  firestore: web::Data<FirestoreDb>,
  participant: Participant,
  board_message: web::Json<BoardMessage>,
) -> Result<HttpResponse, Error> {
  let mut board_message = board_message.into_inner();
  board_message.voting_open.get_or_insert(true);
  board_message.cards_open.get_or_insert(true);
  let board = db::new(&firestore, &participant, board_message).await?;
  add_participant_board(&firestore, &participant, board.id.clone()).await?;
  Ok(HttpResponse::Ok().json(BoardResponse::from_board(board, &participant)))
}

#[get("boards")]
pub async fn list(
  firestore: web::Data<FirestoreDb>,
  participant: Participant,
) -> Result<HttpResponse, Error> {
  let boards = db::list(&firestore, &participant).await?;
  Ok(
    HttpResponse::Ok().json(
      boards
        .into_iter()
        .map(|board| BoardResponse::from_board(board, &participant))
        .collect::<Vec<BoardResponse>>(),
    ),
  )
}

#[get("boards/{board_id}")]
pub async fn get(
  firestore: web::Data<FirestoreDb>,
  participant: Participant,
  board_id: web::Path<String>,
) -> Result<HttpResponse, Error> {
  let (register, board) = join(
    add_participant_board(&firestore, &participant, board_id.clone()),
    db::get(&firestore, board_id.to_string()),
  )
  .await;
  register?;
  Ok(HttpResponse::Ok().json(BoardResponse::from_board(board?, &participant)))
}

#[patch("boards/{board_id}")]
pub async fn update(
  firestore: web::Data<FirestoreDb>,
  participant: Participant,
  board_id: web::Path<String>,
  board_message: web::Json<BoardMessage>,
) -> Result<HttpResponse, Error> {
  let firestore = firestore.into_inner();
  let board = db::get(&firestore, board_id.to_string()).await?;
  if board.owner != participant.id {
    return Err(Error::Forbidden);
  }
  let board = db::update(&firestore, board_id.to_string(), board_message.into_inner()).await?;
  Ok(HttpResponse::Ok().json(BoardResponse::from_board(board, &participant)))
}

#[delete("boards/{board_id}")]
pub async fn delete(
  firestore: web::Data<FirestoreDb>,
  participant: Participant,
  board_id: web::Path<String>,
) -> Result<HttpResponse, Error> {
  let firestore = firestore.into_inner();
  let board = db::get(&firestore, board_id.to_string()).await?;
  if board.owner != participant.id {
    return Err(Error::Forbidden);
  }
  db::delete(&firestore, board_id.to_string()).await?;
  Ok(HttpResponse::Ok().finish())
}
