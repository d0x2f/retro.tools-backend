#[macro_use]
extern crate log;

#[allow(warnings)]
#[macro_use]
mod firestore;
mod boards;
mod cards;
mod cloudrun;
mod columns;
mod config;
mod error;
mod participants;

use actix_cors::Cors;
use actix_http::cookie::SameSite;
use actix_identity::{CookieIdentityPolicy, IdentityService};
use actix_web::{http, middleware as ActixMiddleware, web, App, HttpServer};
use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
  env_logger::init();
  dotenv::dotenv().ok();

  // Hack: If GCP_SERVICE_ACCOUNT_JSON is set, create a json file with that content
  // and set GOOGLE_APPLICATION_CREDENTIALS to point to that file
  if let Ok(base64) = env::var("GCP_SERVICE_ACCOUNT_JSON") {
    info!("Reading from GCP_SERVICE_ACCOUNT_JSON");
    if let Ok(json) = base64::decode(base64) {
      info!("Using credentials in GCP_SERVICE_ACCOUNT_JSON");
      let mut file = File::create(Path::new("/tmp/gcp-service-account.json"))?;
      file.write_all(&json)?;
      env::set_var(
        "GOOGLE_APPLICATION_CREDENTIALS",
        "/tmp/gcp-service-account.json",
      );
    } else {
      error!("Failed to parse GCP_SERVICE_ACCOUNT_JSON");
    }
  }

  let config = config::Config::from_env().await;
  let port = config.port;

  HttpServer::new(move || {
    App::new()
      .data(config.clone())
      .wrap(ActixMiddleware::DefaultHeaders::new().header("Cache-Control", "private"))
      .wrap(
        Cors::default()
          .allowed_origin(&config.allowed_origin)
          .send_wildcard()
          .allowed_methods(vec!["GET", "POST", "PATCH", "PUT", "DELETE"])
          .allowed_header(http::header::CONTENT_TYPE)
          .supports_credentials()
          .max_age(60 * 60),
      )
      .wrap(IdentityService::new(
        CookieIdentityPolicy::new(&config.secret_key)
          .name("__session")
          .secure(config.environment == config::Environment::Production)
          .max_age(30 * 24 * 60 * 60)
          .same_site(SameSite::Strict),
      ))
      .wrap(ActixMiddleware::Logger::default())
      .service(
        web::resource("boards")
          .route(web::get().to(boards::routes::list))
          .route(web::post().to(boards::routes::new)),
      )
      .service(
        web::resource("boards/{board_id}")
          .route(web::patch().to(boards::routes::update))
          .route(web::get().to(boards::routes::get))
          .route(web::delete().to(boards::routes::delete)),
      )
      .service(
        web::resource("boards/{board_id}/columns")
          .route(web::get().to(columns::routes::list))
          .route(web::post().to(columns::routes::new)),
      )
      .service(
        web::resource("boards/{board_id}/columns/{column_id}")
          .route(web::patch().to(columns::routes::update))
          .route(web::get().to(columns::routes::get))
          .route(web::delete().to(columns::routes::delete)),
      )
      .service(
        web::resource("boards/{board_id}/columns/{column_id}/cards")
          .route(web::post().to(cards::routes::new)),
      )
      .service(web::resource("boards/{board_id}/cards").route(web::get().to(cards::routes::list)))
      .service(web::resource("boards/{board_id}/csv").route(web::get().to(cards::routes::csv)))
      .service(
        web::resource("boards/{board_id}/cards/{card_id}")
          .route(web::patch().to(cards::routes::update))
          .route(web::get().to(cards::routes::get))
          .route(web::delete().to(cards::routes::delete)),
      )
      .service(
        web::resource("boards/{board_id}/cards/{card_id}/vote")
          .route(web::put().to(cards::routes::put_vote))
          .route(web::delete().to(cards::routes::delete_vote)),
      )
      .service(
        web::resource("boards/{board_id}/cards/{card_id}/react")
          .route(web::put().to(cards::routes::put_reaction))
          .route(web::delete().to(cards::routes::delete_reaction)),
      )
  })
  .bind(format!("0.0.0.0:{}", port))?
  .run()
  .await
}
