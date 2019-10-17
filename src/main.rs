#![feature(proc_macro_hygiene, decl_macro)]

extern crate openssl;
#[macro_use]
extern crate diesel_migrations;
#[macro_use]
extern crate rocket;
#[macro_use]
extern crate rocket_contrib;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate diesel;
extern crate env_logger;
#[macro_use]
extern crate log;
extern crate rand;

mod boards;
mod cards;
mod catchers;
mod guards;
mod models;
mod persistence;
mod ranks;
mod schema;
mod votes;

use rocket::config::{Config, Environment, Value};
use rocket::fairing::AdHoc;
use rocket::http::Method;
use rocket::*;
use rocket_cors;
use rocket_cors::Cors;
use rocket_cors::{AllowedOrigins, Error};
use std::collections::HashMap;
use std::env;

embed_migrations!();

fn run_db_migrations(rocket: Rocket) -> Result<Rocket, Rocket> {
  let conn = guards::DatabaseConnection::get_one(&rocket).expect("database connection");
  match embedded_migrations::run(&*conn) {
    Ok(()) => Ok(rocket),
    Err(e) => {
      error!("Failed to run database migrations: {:?}", e);
      Err(rocket)
    }
  }
}

// TODO:
//   - Only add localhost if environment isn't production.
//   - Take production origin as an environment var.
fn create_cors_fairing() -> Cors {
  let allowed_origins = AllowedOrigins::some_regex(&[
    "^http://127.0.0.1:(.*)$",
    "^http://localhost:(.*)$",
    "^https?://(.*).dyl.dog$",
  ]);

  rocket_cors::CorsOptions {
    allowed_origins,
    allowed_methods: vec![Method::Get, Method::Post, Method::Patch, Method::Delete]
      .into_iter()
      .map(From::from)
      .collect(),
    allow_credentials: true,
    ..Default::default()
  }
  .to_cors()
  .expect("cors object")
}

fn build_config() -> Config {
  let port = env::var("PORT")
    .unwrap_or_else(|_| "8000".to_owned())
    .parse()
    .unwrap();

  let connection_string = env::var("PSQL_CONNECTION_STRING")
    .unwrap_or_else(|_| "postgres://postgres:postgres@127.0.0.1/retrograde".to_owned());

  let connection_pool_size: i32 = env::var("PSQL_CONNECTION_POOL_SIZE")
    .unwrap_or_else(|_| "1".to_owned())
    .parse()
    .unwrap();

  // TODO: panic if in production mode and no key was given.
  let secret_key = env::var("SECRET_KEY")
    .unwrap_or_else(|_| "p5jimVesy/p+q3ZF5xwuiQ7G0mBEHmaVBBz7mWXqqqg=".to_owned());

  let environment = match env::var("ENVIRONMENT")
    .unwrap_or_else(|_| "development".to_owned())
    .as_str()
  {
    "production" => Environment::Production,
    _ => Environment::Development,
  };

  let mut database_config = HashMap::new();
  let mut databases = HashMap::new();

  database_config.insert("url", Value::from(connection_string));
  database_config.insert("pool_size", Value::from(connection_pool_size));
  databases.insert("postgres", Value::from(database_config));

  Config::build(environment)
    .address("0.0.0.0")
    .port(port)
    .secret_key(secret_key)
    .extra("databases", databases)
    .finalize()
    .unwrap()
}

fn rocket(config: Config) -> Rocket {
  rocket::custom(config)
    .attach(guards::DatabaseConnection::fairing())
    .attach(create_cors_fairing())
    .attach(AdHoc::on_attach("Database Migrations", run_db_migrations))
    .mount(
      "/",
      routes![
        boards::post_board,
        boards::get_boards,
        boards::get_board,
        boards::patch_board,
        boards::delete_board,
        ranks::post_rank,
        ranks::get_ranks,
        ranks::get_rank,
        ranks::patch_rank,
        ranks::delete_rank,
        cards::post_card,
        cards::get_board_cards,
        cards::get_rank_cards,
        cards::get_card,
        cards::patch_card,
        cards::delete_card,
        votes::get_votes,
        votes::post_vote,
        votes::delete_vote
      ],
    )
    .register(catchers![
      catchers::internal_error,
      catchers::unprocessible_entity,
      catchers::not_found,
      catchers::forbidden,
      catchers::unauthorised,
      catchers::bad_request
    ])
}

#[cfg(test)]
extern crate parking_lot;

#[cfg(test)]
use parking_lot::Mutex;
#[cfg(test)]
use rocket::local::Client;

#[cfg(test)]
static DB_LOCK: Mutex<()> = Mutex::new(());

#[cfg(test)]
fn test_cleanup() {
  use diesel::pg::PgConnection;
  use diesel::RunQueryDsl;
  use rocket_contrib::databases::diesel::Connection;
  use schema::{board, participant};

  let dbstring: String = "postgres://postgres:postgres@127.0.0.1/retrograde".into();
  let db = PgConnection::establish(&dbstring).expect("database connection");
  embedded_migrations::run(&db).expect("database migrations");

  diesel::delete(board::table)
    .execute(&db)
    .expect("database cleanup (board)");
  diesel::delete(participant::table)
    .execute(&db)
    .expect("database cleanup (participant)");
}

#[cfg(test)]
fn run_test<F>(test: F)
where
  F: FnOnce(Client),
{
  let _lock = DB_LOCK.lock();

  test_cleanup();

  let mut database_config = HashMap::new();
  let mut databases = HashMap::new();

  database_config.insert(
    "url",
    Value::from("postgres://postgres:postgres@127.0.0.1/retrograde"),
  );
  database_config.insert("pool_size", Value::from(1));
  databases.insert("postgres", Value::from(database_config));

  let config = Config::build(Environment::Development)
    .address("0.0.0.0")
    .port(80)
    .secret_key("apnUQicUZ8QRDN1+rlIGdvhfdabLCTg4aGd0MHFQIPQ=")
    .extra("databases", databases)
    .finalize()
    .unwrap();

  let client = Client::new(rocket(config)).expect("valid rocket instance");

  test(client);
}

fn main() -> Result<(), Error> {
  env_logger::init();
  rocket(build_config()).launch();
  Ok(())
}
