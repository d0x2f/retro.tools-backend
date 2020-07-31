#[macro_use]
extern crate log;

#[macro_use]
mod firestore;
mod boards;
mod error;
mod participants;

use actix_identity::{CookieIdentityPolicy, IdentityService};
use actix_web::{middleware as ActixMiddleware, web, App, HttpServer};
use rand::Rng;

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
  std::env::set_var("RUST_LOG", "actix_web=info");
  env_logger::init();

  let private_key = rand::thread_rng().gen::<[u8; 32]>();
  HttpServer::new(move || {
    App::new()
      .data_factory(|| async { firestore::get_client().await })
      .wrap(IdentityService::new(
        CookieIdentityPolicy::new(&private_key)
          .name("__session")
          .secure(false),
      ))
      .wrap(ActixMiddleware::Logger::default())
      .service(web::resource("/boards").to(boards::routes::get_boards))
      .service(web::resource("/boards/{board_id}").to(boards::routes::get_board))
  })
  .bind("127.0.0.1:8080")?
  .run()
  .await
}
