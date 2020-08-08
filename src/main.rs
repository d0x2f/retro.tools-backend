#[macro_use]
extern crate log;

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
use actix_identity::{CookieIdentityPolicy, IdentityService};
use actix_web::{http, middleware as ActixMiddleware, web, App, HttpServer};

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
  env_logger::init();
  dotenv::dotenv().ok();

  let config = config::Config::from_env().await;
  let port = config.port;
  let token = cloudrun::Token::new(config.firestore_token.clone()).expect("firestore token");
  cloudrun::Token::start_auto_renew(token.clone());

  HttpServer::new(move || {
    let token = token.clone();
    App::new()
      .data_factory(move || firestore::get_client(token.clone()))
      .data(config.clone())
      .wrap(
        Cors::new()
          .allowed_origin(&config.allowed_origin)
          .send_wildcard()
          .allowed_methods(vec!["GET", "POST", "PATCH", "PUT", "DELETE"])
          .allowed_header(http::header::CONTENT_TYPE)
          .supports_credentials()
          .max_age(60 * 60)
          .finish(),
      )
      .wrap(IdentityService::new(
        CookieIdentityPolicy::new(&config.secret_key)
          .name("__session")
          .secure(config.environment == config::Environment::Production)
          .max_age(30 * 24 * 60 * 60),
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
  })
  .bind(format!("0.0.0.0:{}", port))?
  .run()
  .await
}
