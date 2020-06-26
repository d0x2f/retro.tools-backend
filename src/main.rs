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
extern crate time;
#[macro_use]
extern crate lazy_static;
extern crate nanoid;
extern crate serde;
extern crate serde_json;

use nanoid::nanoid;

#[cfg(test)]
mod testing;

#[macro_use]
mod macros;

mod boards;
mod cards;
mod catchers;
mod fairings;
mod guards;
mod metrics;
mod models;
mod persistence;
mod ranks;
mod schema;
mod votes;

use dotenv;
use rocket::config::{Config as RocketConfig, Environment, Value};
use rocket::fairing::AdHoc;
use rocket::http::Method;
use rocket::*;
use rocket_cors;
use rocket_cors::Cors;
use rocket_cors::{AllowedOrigins, Error};
use rocket_prometheus::{prometheus::Registry, PrometheusMetrics};
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

pub struct Config {
  port: u16,
  secret_key: String,
  connection_string: String,
  connection_pool_size: u32,
  environment: Environment,
  allowed_origins: String,
}

impl Config {
  #[cfg(test)]
  fn from_values(
    port: u16,
    secret_key: String,
    connection_string: String,
    connection_pool_size: u32,
    environment: Environment,
    allowed_origins: String,
  ) -> Config {
    Config {
      port,
      secret_key,
      connection_string,
      connection_pool_size,
      environment,
      allowed_origins,
    }
  }

  fn from_env() -> Config {
    let environment = match env::var("ENVIRONMENT")
      .unwrap_or_else(|_| "production".to_owned())
      .as_str()
    {
      "production" => Environment::Production,
      _ => Environment::Development,
    };

    let secret_key = match env::var("SECRET_KEY") {
      Err(_) => match environment {
        Environment::Production => {
          panic!("No secret key provided despite being in production mode!")
        }
        _ => "".to_owned(),
      },
      Ok(s) => s,
    };

    Config {
      port: env::var("PORT").expect("port").parse().expect("integer"),
      secret_key,
      connection_string: env::var("PSQL_CONNECTION_STRING").expect("postgres connection string"),
      connection_pool_size: env::var("PSQL_CONNECTION_POOL_SIZE")
        .expect("postgres connection pool size")
        .parse()
        .expect("integer"),
      environment,
      allowed_origins: env::var("ALLOWED_ORIGINS").expect("allowed origin regex"),
    }
  }
}

fn create_prometheus_fairing() -> PrometheusMetrics {
  let mut labels = HashMap::new();
  labels.insert("instance_id".to_string(), nanoid!(16));
  let registry = Registry::new_custom(None, Some(labels)).expect("valid prometheus registry");
  registry
    .register(Box::new(metrics::PARTICIPANT_COUNT.clone()))
    .expect("metric registration");
  registry
    .register(Box::new(metrics::BOARDS_COUNT.clone()))
    .expect("metric registration");
  registry
    .register(Box::new(metrics::BOARD_PARTICIPANT_COUNT.clone()))
    .expect("metric registration");
  registry
    .register(Box::new(metrics::RANK_COUNT.clone()))
    .expect("metric registration");
  registry
    .register(Box::new(metrics::CARD_COUNT.clone()))
    .expect("metric registration");
  PrometheusMetrics::with_registry(registry)
}

fn create_cors_fairing(config: &Config) -> Cors {
  let allowed_origins = AllowedOrigins::some_regex(&[&config.allowed_origins]);

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

fn build_rocket_config(config: &Config) -> RocketConfig {
  let mut database_config = HashMap::new();
  let mut databases = HashMap::new();

  database_config.insert("url", Value::from(config.connection_string.clone()));
  database_config.insert("pool_size", Value::from(config.connection_pool_size));
  databases.insert("postgres", Value::from(database_config));

  RocketConfig::build(config.environment)
    .address("0.0.0.0")
    .port(config.port)
    .secret_key(&config.secret_key)
    .extra("databases", databases)
    .finalize()
    .unwrap()
}

fn rocket(config: Config) -> Rocket {
  let prometheus = create_prometheus_fairing();
  rocket::custom(build_rocket_config(&config))
    .attach(prometheus.clone())
    .attach(guards::DatabaseConnection::fairing())
    .attach(create_cors_fairing(&config))
    .attach(fairings::GlobalHeaders {})
    .attach(AdHoc::on_attach("Database Migrations", run_db_migrations))
    .mount("/_metrics", prometheus)
    .mount(
      "/",
      routes![
        boards::post_board,
        boards::get_boards,
        boards::get_board,
        boards::patch_board,
        boards::delete_board,
        boards::export_csv,
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

fn main() -> Result<(), Error> {
  env_logger::init();
  dotenv::dotenv().ok();
  rocket(Config::from_env()).launch();
  Ok(())
}
