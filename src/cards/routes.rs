use actix_web::web;

use super::db;
use super::models::CardMessage;
use crate::error::Error;
use crate::firestore::FirestoreV1Client;
use crate::participants::models::Participant;

pub async fn new(
  firestore: web::Data<FirestoreV1Client>,
  participant: Participant,
  params: web::Path<(String, String)>,
  card_message: web::Json<CardMessage>,
) -> Result<web::HttpResponse, Error> {
  let (board_id, column_id) = params.into_inner();
  let mut card_message = card_message.into_inner();
  card_message.author.get_or_insert("".into());
  card_message.column = Some(to_column_reference!(
    "retrotools-284402",
    board_id,
    column_id
  ));

  // Empty cards are not allowed
  if let Some(text) = &card_message.text {
    if text.is_empty() {
      return Err(Error::BadRequest("Empty cards are not allowed.".into()));
    }
  } else {
    return Err(Error::BadRequest("Card text must be provided.".into()));
  }

  let firestore = &mut firestore.get_ref().clone();
  let column = db::new(firestore, participant, board_id.to_string(), card_message).await?;
  Ok(web::HttpResponse::Ok().json(column))
}

pub async fn list(
  firestore: web::Data<FirestoreV1Client>,
  _participant: Participant,
  board_id: web::Path<String>,
) -> Result<web::HttpResponse, Error> {
  let firestore = &mut firestore.get_ref().clone();
  let cards = db::list(firestore, board_id.to_string()).await?;
  Ok(web::HttpResponse::Ok().json(cards))
}

pub async fn get(
  firestore: web::Data<FirestoreV1Client>,
  _participant: Participant,
  params: web::Path<(String, String)>,
) -> Result<web::HttpResponse, Error> {
  let (board_id, card_id) = params.into_inner();
  let firestore = &mut firestore.get_ref().clone();
  let column = db::get(firestore, board_id.to_string(), card_id.to_string()).await;
  Ok(web::HttpResponse::Ok().json(column?))
}

pub async fn update(
  firestore: web::Data<FirestoreV1Client>,
  participant: Participant,
  params: web::Path<(String, String)>,
  card_message: web::Json<CardMessage>,
) -> Result<web::HttpResponse, Error> {
  let (board_id, card_id) = params.into_inner();
  let mut card_message = card_message.into_inner();
  card_message.column = match card_message.column {
    Some(column) => Some(to_column_reference!(
      "retrotools-284402",
      board_id,
      column
    )),
    None => None,
  };

  let firestore = &mut firestore.get_ref().clone();
  let card = db::get(firestore, board_id.to_string(), card_id.to_string()).await?;
  if card.owner != participant.id {
    return Err(Error::Forbidden);
  }
  let card = db::update(
    firestore,
    board_id.to_string(),
    card_id.to_string(),
    card_message,
  )
  .await?;
  Ok(web::HttpResponse::Ok().json(card))
}

pub async fn delete(
  firestore: web::Data<FirestoreV1Client>,
  participant: Participant,
  params: web::Path<(String, String)>,
) -> Result<web::HttpResponse, Error> {
  let (board_id, card_id) = params.into_inner();
  let firestore = &mut firestore.get_ref().clone();
  let card = db::get(firestore, board_id.to_string(), card_id.to_string()).await?;
  if card.owner != participant.id {
    return Err(Error::Forbidden);
  }
  db::delete(firestore, board_id.to_string(), card_id.to_string()).await?;
  Ok(web::HttpResponse::Ok().finish())
}
