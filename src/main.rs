#[macro_use]
extern crate log;

mod boards;
mod cards;
mod cloudrun;
mod columns;
mod config;
mod error;
mod participants;

use ::firestore::FirestoreDb;
use actix_cors::Cors;
use actix_identity::IdentityMiddleware;
use actix_session::config::PersistentSession;
use actix_session::{storage::CookieSessionStore, SessionMiddleware};
use actix_web::cookie::time::Duration;
use actix_web::cookie::Key;
use actix_web::{http, middleware as ActixMiddleware, web::Data, App, HttpServer};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
  env_logger::init();
  dotenv::dotenv().ok();

  let config = config::Config::from_env();
  let port = config.port;

  HttpServer::new(move || {
    let firestore_project = config.firestore_project.clone();
    let mut cors = Cors::default()
      .send_wildcard()
      .allowed_methods(vec!["GET", "POST", "PATCH", "PUT", "DELETE"])
      .allowed_header(http::header::CONTENT_TYPE)
      .supports_credentials()
      .max_age(60 * 60);

    for origin in &config.allowed_origins {
      cors = cors.allowed_origin(origin);
    }

    App::new()
      .data_factory(move || FirestoreDb::new(firestore_project.clone()))
      .app_data(Data::new(config.clone()))
      .wrap(ActixMiddleware::DefaultHeaders::new().add(("Cache-Control", "private")))
      .wrap(cors)
      .wrap(IdentityMiddleware::default())
      .wrap(
        SessionMiddleware::builder(
          CookieSessionStore::default(),
          Key::derive_from(&config.secret_key),
        )
        .cookie_secure(config.secure_cookie)
        .cookie_same_site(config.same_site)
        .cookie_name("id".into())
        .session_lifecycle(PersistentSession::default().session_ttl(
          Duration::seconds(60 * 60 * 24 * 30), // 30 days
        ))
        .build(),
      )
      .wrap(ActixMiddleware::Logger::default())
      .service(boards::routes::list)
      .service(boards::routes::new)
      .service(boards::routes::update)
      .service(boards::routes::get)
      .service(boards::routes::delete)
      .service(columns::routes::list)
      .service(columns::routes::new)
      .service(columns::routes::update)
      .service(columns::routes::get)
      .service(columns::routes::delete)
      .service(cards::routes::new)
      .service(cards::routes::list)
      .service(cards::routes::csv)
      .service(cards::routes::update)
      .service(cards::routes::get)
      .service(cards::routes::delete)
      .service(cards::routes::put_vote)
      .service(cards::routes::delete_vote)
      .service(cards::routes::put_reaction)
      .service(cards::routes::delete_reaction)
      .service(participants::routes::auth)
  })
  .bind(format!("0.0.0.0:{}", port))?
  .run()
  .await
}
