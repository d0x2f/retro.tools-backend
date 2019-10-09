#![feature(proc_macro_hygiene, decl_macro)]

extern crate openssl;
#[macro_use]
extern crate rocket;
#[macro_use]
extern crate rocket_contrib;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate diesel;
extern crate env_logger;
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
use rocket::http::Method;
use rocket::*;
use rocket_cors;
use rocket_cors::{AllowedOrigins, Error};
use std::collections::HashMap;
use std::env;

fn main() -> Result<(), Error> {
  let allowed_origins = AllowedOrigins::some_exact(&["http://127.0.0.1:5000"]);

  let cors_fairing = rocket_cors::CorsOptions {
    allowed_origins,
    allowed_methods: vec![Method::Get, Method::Post, Method::Patch, Method::Delete]
      .into_iter()
      .map(From::from)
      .collect(),
    allow_credentials: true,
    ..Default::default()
  }
  .to_cors()?;

  let port = env::var("PORT")
    .unwrap_or("8000".to_owned())
    .parse()
    .unwrap();

  let connection_string = env::var("PSQL_CONNECTION_STRING")
    .unwrap_or("postgres://postgres:postgres@postgres/retrograde".to_owned());

  let environment = match env::var("ENVIRONMENT")
    .unwrap_or("development".to_owned())
    .as_str()
  {
    "production" => Environment::Production,
    _ => Environment::Development,
  };

  let mut database_config = HashMap::new();
  let mut databases = HashMap::new();

  database_config.insert("url", Value::from(connection_string));
  databases.insert("postgres", Value::from(database_config));

  let config = Config::build(environment)
    .address("0.0.0.0")
    .port(port)
    .extra("databases", databases)
    .finalize()
    .unwrap();

  println!("{}", port);

  rocket::custom(config)
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
        cards::get_cards,
        cards::get_card,
        cards::patch_card,
        cards::delete_card,
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
    .attach(guards::DatabaseConnection::fairing())
    .attach(cors_fairing)
    .launch();

  Ok(())
}
