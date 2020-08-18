use actix_web::web;

use super::db;
use super::models::ColumnMessage;
use crate::boards;
use crate::config::Config;
use crate::error::Error;
use crate::firestore;
use crate::participants::models::Participant;

pub async fn new(
  config: web::Data<Config>,
  _participant: Participant,
  board_id: web::Path<String>,
  column_message: web::Json<ColumnMessage>,
) -> Result<web::HttpResponse, Error> {
  let mut firestore = firestore::get_client().await?;
  let column = db::new(
    &mut firestore,
    &config,
    board_id.to_string(),
    column_message.into_inner(),
  )
  .await?;
  Ok(web::HttpResponse::Ok().json(column))
}

pub async fn list(
  config: web::Data<Config>,
  _participant: Participant,
  board_id: web::Path<String>,
) -> Result<web::HttpResponse, Error> {
  let mut firestore = firestore::get_client().await?;
  let columns = db::list(&mut firestore, &config, board_id.to_string()).await?;
  Ok(web::HttpResponse::Ok().json(columns))
}

pub async fn get(
  config: web::Data<Config>,
  _participant: Participant,
  params: web::Path<(String, String)>,
) -> Result<web::HttpResponse, Error> {
  let mut firestore = firestore::get_client().await?;
  let (board_id, column_id) = params.into_inner();
  let column = db::get(
    &mut firestore,
    &config,
    board_id.to_string(),
    column_id.to_string(),
  )
  .await;
  Ok(web::HttpResponse::Ok().json(column?))
}

pub async fn update(
  config: web::Data<Config>,
  participant: Participant,
  params: web::Path<(String, String)>,
  column_message: web::Json<ColumnMessage>,
) -> Result<web::HttpResponse, Error> {
  let mut firestore = firestore::get_client().await?;
  let (board_id, column_id) = params.into_inner();
  let board = boards::db::get(&mut firestore, &config, board_id.to_string()).await?;
  if board.owner != participant.id {
    return Err(Error::Forbidden);
  }
  let column = db::update(
    &mut firestore,
    &config,
    board_id.to_string(),
    column_id.to_string(),
    column_message.into_inner(),
  )
  .await?;
  Ok(web::HttpResponse::Ok().json(column))
}

pub async fn delete(
  config: web::Data<Config>,
  participant: Participant,
  params: web::Path<(String, String)>,
) -> Result<web::HttpResponse, Error> {
  let mut firestore = firestore::get_client().await?;
  let (board_id, column_id) = params.into_inner();
  let board = boards::db::get(&mut firestore, &config, board_id.to_string()).await?;
  if board.owner != participant.id {
    return Err(Error::Forbidden);
  }
  db::delete(
    &mut firestore,
    &config,
    board_id.to_string(),
    column_id.to_string(),
  )
  .await?;
  Ok(web::HttpResponse::Ok().finish())
}
