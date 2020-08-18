use actix_web::web;
use futures::future::join;

use super::db;
use super::models::*;
use crate::config::Config;
use crate::error::Error;
use crate::firestore;
use crate::participants::db::*;
use crate::participants::models::Participant;

pub async fn new(
  config: web::Data<Config>,
  participant: Participant,
  board_message: web::Json<BoardMessage>,
) -> Result<web::HttpResponse, Error> {
  let mut firestore = firestore::get_client().await?;
  let mut board_message = board_message.into_inner();
  board_message.voting_open.get_or_insert(true);
  board_message.cards_open.get_or_insert(true);
  let board = db::new(&mut firestore, &config, &participant, board_message).await?;
  add_participant_board(
    &mut firestore.clone(),
    &config,
    &participant,
    board.id.clone(),
  )
  .await?;
  Ok(web::HttpResponse::Ok().json(BoardResponse::from_board(board, &participant)))
}

pub async fn list(
  config: web::Data<Config>,
  participant: Participant,
) -> Result<web::HttpResponse, Error> {
  let mut firestore = firestore::get_client().await?;
  let boards = db::list(&mut firestore, &config, &participant).await?;
  Ok(
    web::HttpResponse::Ok().json::<Vec<BoardResponse>>(
      boards
        .into_iter()
        .map(|board| BoardResponse::from_board(board, &participant))
        .collect(),
    ),
  )
}

pub async fn get(
  config: web::Data<Config>,
  participant: Participant,
  board_id: web::Path<String>,
) -> Result<web::HttpResponse, Error> {
  let mut firestore = firestore::get_client().await?;
  let (register, board) = join(
    add_participant_board(
      &mut firestore.clone(),
      &config,
      &participant,
      board_id.clone(),
    ),
    db::get(&mut firestore, &config, board_id.to_string()),
  )
  .await;
  register?;
  Ok(web::HttpResponse::Ok().json(BoardResponse::from_board(board?, &participant)))
}

pub async fn update(
  config: web::Data<Config>,
  participant: Participant,
  board_id: web::Path<String>,
  board_message: web::Json<BoardMessage>,
) -> Result<web::HttpResponse, Error> {
  let mut firestore = firestore::get_client().await?;
  let board = db::get(&mut firestore, &config, board_id.to_string()).await?;
  if board.owner != participant.id {
    return Err(Error::Forbidden);
  }
  let board = db::update(
    &mut firestore,
    &config,
    board_id.to_string(),
    board_message.into_inner(),
  )
  .await?;
  Ok(web::HttpResponse::Ok().json(BoardResponse::from_board(board, &participant)))
}

pub async fn delete(
  config: web::Data<Config>,
  participant: Participant,
  board_id: web::Path<String>,
) -> Result<web::HttpResponse, Error> {
  let mut firestore = firestore::get_client().await?;
  let board = db::get(&mut firestore, &config, board_id.to_string()).await?;
  if board.owner != participant.id {
    return Err(Error::Forbidden);
  }
  db::delete(&mut firestore, &config, board_id.to_string()).await?;
  Ok(web::HttpResponse::Ok().finish())
}
