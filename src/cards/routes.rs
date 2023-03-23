use ::csv::Writer as CSVWriter;
use actix_web::http::header::{
  ContentDisposition, DispositionParam, DispositionType, CONTENT_DISPOSITION,
};
use actix_web::{delete, get, patch, post, put, web, HttpResponse};
use firestore::{FirestoreDb, FirestoreReference};

use super::db;
use super::models::*;
use crate::boards::*;
use crate::columns::get_columns;
use crate::error::Error;
use crate::participants::models::Participant;

#[post("boards/{board_id}/columns/{column_id}/cards")]
pub async fn new(
  firestore: web::Data<FirestoreDb>,
  participant: Participant,
  params: web::Path<(String, String)>,
  card_message: web::Json<CardMessage>,
) -> Result<HttpResponse, Error> {
  let (board_id, column_id) = params.into_inner();
  assert_cards_allowed(&firestore, board_id.to_string()).await?;
  let mut card_message = card_message.into_inner();
  card_message.author.get_or_insert("".into());
  card_message.column = Some(format!(
    "{}/columns/{}",
    firestore.parent_path("boards", &board_id)?,
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

  let card = db::new(&firestore, &participant, board_id.to_string(), card_message).await?;
  Ok(
    HttpResponse::Ok().json(CardResponse::from_card(
      card,
      &FirestoreReference(
        firestore
          .parent_path("participants", &participant.id)
          .unwrap()
          .into(),
      ),
    )),
  )
}

#[get("boards/{board_id}/cards")]
pub async fn list(
  firestore: web::Data<FirestoreDb>,
  participant: Participant,
  board_id: web::Path<String>,
) -> Result<HttpResponse, Error> {
  let cards = db::list(&firestore, board_id.to_string()).await?;
  Ok(
    HttpResponse::Ok().json(
      cards
        .into_iter()
        .map(|card| {
          CardResponse::from_card(
            card,
            &FirestoreReference(
              firestore
                .parent_path("participants", &participant.id)
                .unwrap()
                .into(),
            ),
          )
        })
        .collect::<Vec<CardResponse>>(),
    ),
  )
}

#[get("boards/{board_id}/cards/{card_id}")]
pub async fn get(
  firestore: web::Data<FirestoreDb>,
  participant: Participant,
  params: web::Path<(String, String)>,
) -> Result<HttpResponse, Error> {
  let (board_id, card_id) = params.into_inner();
  let card = db::get(&firestore, &board_id, &card_id).await?;
  Ok(
    HttpResponse::Ok().json(CardResponse::from_card(
      card,
      &FirestoreReference(
        firestore
          .parent_path("participants", &participant.id)
          .unwrap()
          .into(),
      ),
    )),
  )
}

#[patch("boards/{board_id}/cards/{card_id}")]
pub async fn update(
  firestore: web::Data<FirestoreDb>,
  participant: Participant,
  params: web::Path<(String, String)>,
  card_message: web::Json<CardMessage>,
) -> Result<HttpResponse, Error> {
  let (board_id, card_id) = params.into_inner();
  let mut card_message = card_message.into_inner();
  card_message.column = match card_message.column {
    Some(column) => Some(format!(
      "{}/{}",
      firestore.parent_path("boards", &board_id)?,
      column
    )),
    None => None,
  };

  let card = db::get(&firestore, &board_id, &card_id).await?;
  super::assert_card_owner(&firestore, &participant, &card, board_id.to_string()).await?;
  let card = db::update(
    &firestore,
    board_id.to_string(),
    card_id.to_string(),
    card_message,
  )
  .await?;
  Ok(
    HttpResponse::Ok().json(CardResponse::from_card(
      card,
      &FirestoreReference(
        firestore
          .parent_path("participants", &participant.id)
          .unwrap()
          .into(),
      ),
    )),
  )
}

#[delete("boards/{board_id}/cards/{card_id}")]
pub async fn delete(
  firestore: web::Data<FirestoreDb>,
  participant: Participant,
  params: web::Path<(String, String)>,
) -> Result<HttpResponse, Error> {
  let firestore = firestore.into_inner();
  let (board_id, card_id) = params.into_inner();
  let card = db::get(&firestore, &board_id, &card_id).await?;
  super::assert_card_owner(&firestore, &participant, &card, board_id.to_string()).await?;
  db::delete(&firestore, board_id.to_string(), card_id.to_string()).await?;
  Ok(HttpResponse::Ok().finish())
}

#[put("boards/{board_id}/cards/{card_id}/vote")]
pub async fn put_vote(
  firestore: web::Data<FirestoreDb>,
  participant: Participant,
  params: web::Path<(String, String)>,
) -> Result<HttpResponse, Error> {
  let firestore = firestore.into_inner();
  let (board_id, card_id) = params.into_inner();
  assert_voting_allowed(&firestore, board_id.to_string()).await?;
  db::put_vote(
    &firestore,
    &participant,
    board_id.to_string(),
    card_id.to_string(),
  )
  .await?;
  Ok(HttpResponse::Created().finish())
}

#[delete("boards/{board_id}/cards/{card_id}/vote")]
pub async fn delete_vote(
  firestore: web::Data<FirestoreDb>,
  participant: Participant,
  params: web::Path<(String, String)>,
) -> Result<HttpResponse, Error> {
  let firestore = firestore.into_inner();
  let (board_id, card_id) = params.into_inner();
  assert_voting_allowed(&firestore, board_id.to_string()).await?;
  db::delete_vote(
    &firestore,
    &participant,
    board_id.to_string(),
    card_id.to_string(),
  )
  .await?;
  Ok(HttpResponse::Created().finish())
}

#[put("boards/{board_id}/cards/{card_id}/react")]
pub async fn put_reaction(
  firestore: web::Data<FirestoreDb>,
  participant: Participant,
  params: web::Path<(String, String)>,
  react_message: web::Json<ReactMessage>,
) -> Result<HttpResponse, Error> {
  let firestore = firestore.into_inner();
  let (board_id, card_id) = params.into_inner();
  db::put_reaction(
    &firestore,
    &participant,
    &board_id,
    &card_id,
    &react_message.emoji,
  )
  .await?;
  Ok(HttpResponse::Created().finish())
}

#[delete("boards/{board_id}/cards/{card_id}/react")]
pub async fn delete_reaction(
  firestore: web::Data<FirestoreDb>,
  participant: Participant,
  params: web::Path<(String, String)>,
) -> Result<HttpResponse, Error> {
  let (board_id, card_id) = params.into_inner();
  db::delete_reaction(&firestore, &participant, &board_id, &card_id).await?;
  Ok(HttpResponse::Created().finish())
}

#[get("boards/{board_id}/csv")]
pub async fn csv(
  firestore: web::Data<FirestoreDb>,
  _participant: Participant,
  board_id: web::Path<String>,
) -> Result<HttpResponse, Error> {
  let firestore = firestore.into_inner();
  let board = get_board(&firestore, board_id.to_string()).await?;
  let columns = get_columns(&firestore, board_id.to_string()).await?;
  let mut cards = db::list(&firestore, board_id.to_string()).await?;
  cards.sort_by(|a, b| b.column.0.cmp(&a.column.0));
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
