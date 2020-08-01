#[macro_use]
extern crate log;

#[macro_use]
mod firestore;
mod boards;
mod error;
mod participants;

use actix_identity::{CookieIdentityPolicy, IdentityService};
use actix_web::{middleware as ActixMiddleware, web, App, HttpServer};
// use rand::Rng;

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
  env_logger::init();

  let private_key = [0; 32]; // rand::thread_rng().gen::<[u8; 32]>();
  HttpServer::new(move || {
    App::new()
      .data_factory(|| async { firestore::get_client().await })
      .wrap(IdentityService::new(
        CookieIdentityPolicy::new(&private_key)
          .name("__session")
          .secure(false),
      ))
      .wrap(ActixMiddleware::Logger::default())
      .service(
        web::scope("boards")
          .service(
            web::resource("")
              .route(web::get().to(boards::routes::list))
              .route(web::post().to(boards::routes::new)),
          )
          .service(
            web::resource("{board_id}")
              .route(web::patch().to(boards::routes::update))
              .route(web::get().to(boards::routes::get))
              .route(web::delete().to(boards::routes::delete)),
          ),
      )
  })
  .bind("127.0.0.1:8080")?
  .run()
  .await
}
