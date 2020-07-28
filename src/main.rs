mod error;
#[macro_use]
mod firestore;
mod boards;

use actix_web::{middleware, web, App, HttpServer};
use std::cell::RefCell;

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
  std::env::set_var("RUST_LOG", "actix_web=info");
  env_logger::init();

  HttpServer::new(|| {
    App::new()
      .data_factory(|| async { firestore::get_client().await.map(RefCell::new) })
      .wrap(middleware::Logger::default())
      .service(web::resource("/boards").to(boards::routes::get_boards))
  })
  .bind("127.0.0.1:8080")?
  .run()
  .await
}