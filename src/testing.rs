extern crate parking_lot;

use super::models::{Board, Card, NewBoard, NewCard, NewRank, Rank};
use super::persistence;
use super::schema::{board, participant};
use super::Config;
use super::{embedded_migrations, guards, rocket};
use parking_lot::Mutex;
use rocket::config::Environment;
use rocket::http::ContentType;
use rocket::local::Client;

static DB_LOCK: Mutex<()> = Mutex::new(());

use diesel::pg::PgConnection;
use diesel::prelude::*;

fn test_cleanup(db: &PgConnection) {
  embedded_migrations::run(db).expect("database migrations");

  diesel::delete(board::table)
    .execute(db)
    .expect("database cleanup (board)");
  diesel::delete(participant::table)
    .execute(db)
    .expect("database cleanup (participant)");
}

pub fn create_new_client() -> Client {
  let config = Config::from_values(
    80,
    "apnUQicUZ8QRDN1+rlIGdvhfdabLCTg4aGd0MHFQIPQ=".to_owned(),
    "postgres://postgres:postgres@127.0.0.1/retrograde".to_owned(),
    2,
    Environment::Development,
    ".*".to_owned(),
  );

  let rocket = rocket(config);
  Client::new(rocket).expect("valid rocket instance")
}

pub fn run_test<F>(test: F)
where
  F: FnOnce(Client, &PgConnection),
{
  let _lock = DB_LOCK.lock();

  let client = create_new_client();
  let db = guards::DatabaseConnection::get_one(client.rocket()).expect("database connection");

  test_cleanup(&db);
  test(client, &db);
}

/// Create a board
pub fn create_board(client: &Client, board: &NewBoard) -> (Board, String) {
  let mut response = client
    .post("/boards")
    .header(ContentType::JSON)
    .body(serde_json::to_string(board).unwrap())
    .dispatch();
  (
    serde_json::from_str(response.body_string().unwrap().as_str()).unwrap(),
    response.cookies()[0].value().to_owned(),
  )
}

/// Create a rank
pub fn create_rank(db: &PgConnection, rank: NewRank) -> Rank {
  persistence::ranks::put_rank(db, rank).unwrap()
}

/// Create a card
pub fn create_card(db: &PgConnection, card: NewCard) -> Card {
  persistence::cards::put_card(db, card, "").unwrap()
}
