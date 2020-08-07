use actix_web::web;

use super::db;
use super::models::*;
use crate::boards::*;
use crate::config::Config;
use crate::error::Error;
use crate::firestore::FirestoreV1Client;
use crate::participants::models::Participant;

pub async fn new(
  firestore: web::Data<FirestoreV1Client>,
  config: web::Data<Config>,
  participant: Participant,
  params: web::Path<(String, String)>,
  card_message: web::Json<CardMessage>,
) -> Result<web::HttpResponse, Error> {
  let firestore = &mut firestore.get_ref().clone();
  let (board_id, column_id) = params.into_inner();
  assert_cards_allowed(firestore, &config, board_id.to_string()).await?;
  let mut card_message = card_message.into_inner();
  card_message.author.get_or_insert("".into());
  card_message.column = Some(to_column_reference!(
    config.firestore_project,
    board_id.to_string(),
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

  let card = db::new(
    firestore,
    &config,
    &participant,
    board_id.to_string(),
    card_message,
  )
  .await?;
  Ok(web::HttpResponse::Ok().json(CardResponse::from_card(&config, card, &participant)))
}

pub async fn list(
  firestore: web::Data<FirestoreV1Client>,
  config: web::Data<Config>,
  participant: Participant,
  board_id: web::Path<String>,
) -> Result<web::HttpResponse, Error> {
  let firestore = &mut firestore.get_ref().clone();
  let cards = db::list(firestore, &config, board_id.to_string()).await?;
  Ok(
    web::HttpResponse::Ok().json::<Vec<CardResponse>>(
      cards
        .into_iter()
        .map(|card| CardResponse::from_card(&config, card, &participant))
        .collect(),
    ),
  )
}

pub async fn get(
  firestore: web::Data<FirestoreV1Client>,
  config: web::Data<Config>,
  participant: Participant,
  params: web::Path<(String, String)>,
) -> Result<web::HttpResponse, Error> {
  let (board_id, card_id) = params.into_inner();
  let firestore = &mut firestore.get_ref().clone();
  let card = db::get(
    firestore,
    &config,
    board_id.to_string(),
    card_id.to_string(),
  )
  .await?;
  Ok(web::HttpResponse::Ok().json(CardResponse::from_card(&config, card, &participant)))
}

pub async fn update(
  firestore: web::Data<FirestoreV1Client>,
  config: web::Data<Config>,
  participant: Participant,
  params: web::Path<(String, String)>,
  card_message: web::Json<CardMessage>,
) -> Result<web::HttpResponse, Error> {
  let (board_id, card_id) = params.into_inner();
  let mut card_message = card_message.into_inner();
  card_message.column = match card_message.column {
    Some(column) => Some(to_column_reference!(
      config.firestore_project,
      board_id,
      column
    )),
    None => None,
  };

  let firestore = &mut firestore.get_ref().clone();
  let card = db::get(
    firestore,
    &config,
    board_id.to_string(),
    card_id.to_string(),
  )
  .await?;
  if card.owner != participant.id {
    return Err(Error::Forbidden);
  }
  let card = db::update(
    firestore,
    &config,
    board_id.to_string(),
    card_id.to_string(),
    card_message,
  )
  .await?;
  Ok(web::HttpResponse::Ok().json(CardResponse::from_card(&config, card, &participant)))
}

pub async fn delete(
  firestore: web::Data<FirestoreV1Client>,
  config: web::Data<Config>,
  participant: Participant,
  params: web::Path<(String, String)>,
) -> Result<web::HttpResponse, Error> {
  let (board_id, card_id) = params.into_inner();
  let firestore = &mut firestore.get_ref().clone();
  let card = db::get(
    firestore,
    &config,
    board_id.to_string(),
    card_id.to_string(),
  )
  .await?;
  if card.owner != participant.id {
    return Err(Error::Forbidden);
  }
  db::delete(
    firestore,
    &config,
    board_id.to_string(),
    card_id.to_string(),
  )
  .await?;
  Ok(web::HttpResponse::Ok().finish())
}

pub async fn put_vote(
  firestore: web::Data<FirestoreV1Client>,
  config: web::Data<Config>,
  participant: Participant,
  params: web::Path<(String, String)>,
) -> Result<web::HttpResponse, Error> {
  let (board_id, card_id) = params.into_inner();
  let firestore = &mut firestore.get_ref().clone();
  assert_voting_allowed(firestore, &config, board_id.to_string()).await?;
  db::put_vote(
    firestore,
    &config,
    &participant,
    board_id.to_string(),
    card_id.to_string(),
  )
  .await?;
  Ok(web::HttpResponse::Created().finish())
}

pub async fn delete_vote(
  firestore: web::Data<FirestoreV1Client>,
  config: web::Data<Config>,
  participant: Participant,
  params: web::Path<(String, String)>,
) -> Result<web::HttpResponse, Error> {
  let (board_id, card_id) = params.into_inner();
  let firestore = &mut firestore.get_ref().clone();
  assert_voting_allowed(firestore, &config, board_id.to_string()).await?;
  db::delete_vote(
    firestore,
    &config,
    &participant,
    board_id.to_string(),
    card_id.to_string(),
  )
  .await?;
  Ok(web::HttpResponse::Created().finish())
}
