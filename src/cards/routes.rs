use ::csv::Writer as CSVWriter;
use actix_web::http::header::{
  ContentDisposition, DispositionParam, DispositionType, CONTENT_DISPOSITION,
};
use actix_web::{delete, get, patch, post, put, web, HttpResponse};
use firestore::FirestoreDb;
use futures::lock::Mutex;

use super::db;
use super::models::*;
use crate::boards::*;
use crate::columns::get_columns;
use crate::config::Config;
use crate::error::Error;
use crate::firestore::FirestoreV1Client;
use crate::participants::models::Participant;

#[post("boards/{board_id}/columns/{column_id}/cards")]
pub async fn new(
  config: web::Data<Config>,
  firestore: web::Data<Mutex<FirestoreV1Client>>,
  firestore2: web::Data<FirestoreDb>,
  participant: Participant,
  params: web::Path<(String, String)>,
  card_message: web::Json<CardMessage>,
) -> Result<HttpResponse, Error> {
  let firestore = firestore.into_inner();
  let (board_id, column_id) = params.into_inner();
  assert_cards_allowed(&firestore2, board_id.to_string()).await?;
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
  Ok(HttpResponse::Ok().json(CardResponse::from_card(&config, card, &participant)))
}

#[get("boards/{board_id}/cards")]
pub async fn list(
  config: web::Data<Config>,
  firestore: web::Data<Mutex<FirestoreV1Client>>,
  participant: Participant,
  board_id: web::Path<String>,
) -> Result<HttpResponse, Error> {
  let firestore = firestore.into_inner();
  let cards = db::list(firestore, &config, board_id.to_string()).await?;
  Ok(
    HttpResponse::Ok().json(
      cards
        .into_iter()
        .map(|card| CardResponse::from_card(&config, card, &participant))
        .collect::<Vec<CardResponse>>(),
    ),
  )
}

#[get("boards/{board_id}/cards/{card_id}")]
pub async fn get(
  config: web::Data<Config>,
  firestore: web::Data<Mutex<FirestoreV1Client>>,
  participant: Participant,
  params: web::Path<(String, String)>,
) -> Result<HttpResponse, Error> {
  let firestore = firestore.into_inner();
  let (board_id, card_id) = params.into_inner();
  let card = db::get(
    firestore,
    &config,
    board_id.to_string(),
    card_id.to_string(),
  )
  .await?;
  Ok(HttpResponse::Ok().json(CardResponse::from_card(&config, card, &participant)))
}

#[patch("boards/{board_id}/cards/{card_id}")]
pub async fn update(
  config: web::Data<Config>,
  firestore: web::Data<Mutex<FirestoreV1Client>>,
  firestore2: web::Data<FirestoreDb>,
  participant: Participant,
  params: web::Path<(String, String)>,
  card_message: web::Json<CardMessage>,
) -> Result<HttpResponse, Error> {
  let firestore = firestore.into_inner();
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

  let card = db::get(
    firestore.clone(),
    &config,
    board_id.to_string(),
    card_id.to_string(),
  )
  .await?;
  super::assert_card_owner(&firestore2, &participant, &card, board_id.to_string()).await?;
  let card = db::update(
    firestore,
    &config,
    board_id.to_string(),
    card_id.to_string(),
    card_message,
  )
  .await?;
  Ok(HttpResponse::Ok().json(CardResponse::from_card(&config, card, &participant)))
}

#[delete("boards/{board_id}/cards/{card_id}")]
pub async fn delete(
  config: web::Data<Config>,
  firestore: web::Data<Mutex<FirestoreV1Client>>,
  firestore2: web::Data<FirestoreDb>,
  participant: Participant,
  params: web::Path<(String, String)>,
) -> Result<HttpResponse, Error> {
  let firestore = firestore.into_inner();
  let (board_id, card_id) = params.into_inner();
  let card = db::get(
    firestore.clone(),
    &config,
    board_id.to_string(),
    card_id.to_string(),
  )
  .await?;
  super::assert_card_owner(&firestore2, &participant, &card, board_id.to_string()).await?;
  db::delete(
    firestore,
    &config,
    board_id.to_string(),
    card_id.to_string(),
  )
  .await?;
  Ok(HttpResponse::Ok().finish())
}

#[put("boards/{board_id}/cards/{card_id}/vote")]
pub async fn put_vote(
  config: web::Data<Config>,
  firestore: web::Data<Mutex<FirestoreV1Client>>,
  firestore2: web::Data<FirestoreDb>,
  participant: Participant,
  params: web::Path<(String, String)>,
) -> Result<HttpResponse, Error> {
  let firestore = firestore.into_inner();
  let (board_id, card_id) = params.into_inner();
  assert_voting_allowed(&firestore2, board_id.to_string()).await?;
  db::put_vote(
    firestore,
    &config,
    &participant,
    board_id.to_string(),
    card_id.to_string(),
  )
  .await?;
  Ok(HttpResponse::Created().finish())
}

#[delete("boards/{board_id}/cards/{card_id}/vote")]
pub async fn delete_vote(
  config: web::Data<Config>,
  firestore: web::Data<Mutex<FirestoreV1Client>>,
  firestore2: web::Data<FirestoreDb>,
  participant: Participant,
  params: web::Path<(String, String)>,
) -> Result<HttpResponse, Error> {
  let firestore = firestore.into_inner();
  let (board_id, card_id) = params.into_inner();
  assert_voting_allowed(&firestore2, board_id.to_string()).await?;
  db::delete_vote(
    firestore,
    &config,
    &participant,
    board_id.to_string(),
    card_id.to_string(),
  )
  .await?;
  Ok(HttpResponse::Created().finish())
}

#[put("boards/{board_id}/cards/{card_id}/react")]
pub async fn put_reaction(
  config: web::Data<Config>,
  firestore: web::Data<Mutex<FirestoreV1Client>>,
  participant: Participant,
  params: web::Path<(String, String)>,
  react_message: web::Json<ReactMessage>,
) -> Result<HttpResponse, Error> {
  let firestore = firestore.into_inner();
  let (board_id, card_id) = params.into_inner();
  db::put_reaction(
    firestore,
    &config,
    &participant,
    board_id.to_string(),
    card_id.to_string(),
    react_message.emoji.clone(),
  )
  .await?;
  Ok(HttpResponse::Created().finish())
}

#[delete("boards/{board_id}/cards/{card_id}/react")]
pub async fn delete_reaction(
  config: web::Data<Config>,
  firestore: web::Data<Mutex<FirestoreV1Client>>,
  participant: Participant,
  params: web::Path<(String, String)>,
) -> Result<HttpResponse, Error> {
  let firestore = firestore.into_inner();
  let (board_id, card_id) = params.into_inner();
  db::delete_reaction(
    firestore,
    &config,
    &participant,
    board_id.to_string(),
    card_id.to_string(),
  )
  .await?;
  Ok(HttpResponse::Created().finish())
}

#[get("boards/{board_id}/csv")]
pub async fn csv(
  config: web::Data<Config>,
  firestore: web::Data<Mutex<FirestoreV1Client>>,
  firestore2: web::Data<FirestoreDb>,
  _participant: Participant,
  board_id: web::Path<String>,
) -> Result<HttpResponse, Error> {
  let firestore = firestore.into_inner();
  let board = get_board(&firestore2, board_id.to_string()).await?;
  let columns = get_columns(firestore.clone(), &config, board_id.to_string()).await?;
  let mut cards = db::list(firestore, &config, board_id.to_string()).await?;
  cards.sort_by(|a, b| b.column.cmp(&a.column));
  let mut csv_writer = CSVWriter::from_writer(vec![]);
  for card in cards.into_iter() {
    csv_writer.serialize(CardCSVRow::from_card(card, &columns))?;
  }
  Ok(
    HttpResponse::Ok()
      .insert_header((
        CONTENT_DISPOSITION,
        ContentDisposition {
          disposition: DispositionType::Attachment,
          parameters: vec![DispositionParam::Filename(format!(
            "{}-{}.csv",
            board.name, board.created_at
          ))],
        },
      ))
      .body(String::from_utf8(csv_writer.into_inner()?)?),
  )
}
