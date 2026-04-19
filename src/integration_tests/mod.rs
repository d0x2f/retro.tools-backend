// Run with: FIRESTORE_EMULATOR_HOST=localhost:8080 cargo test -- --ignored

mod board_tests;
mod card_tests;
mod column_tests;

use actix_web::cookie::{Cookie, SameSite};
use actix_web::test::{self};
use chrono::Utc;
use firestore::{FirestoreDb, FirestoreDbOptions};
use gcloud_sdk::{ExternalJwtFunctionSource, Token, TokenSourceType};
use serde_json::Value;

use crate::config::{Config, Environment, GoogleAccountKey};

pub(super) async fn emulator_db() -> FirestoreDb {
  // "owner" is the Firebase emulator's magic token that bypasses security rules,
  // matching the credential used by Firebase Admin SDKs in emulator mode.
  let token_source = ExternalJwtFunctionSource::new(|| async {
    Ok(Token::new(
      "Bearer".to_string(),
      "owner".into(),
      Utc::now() + chrono::Duration::hours(1),
    ))
  });
  FirestoreDb::with_options_token_source(
    FirestoreDbOptions::new("test-project".to_string()),
    vec![],
    TokenSourceType::ExternalSource(Box::new(token_source)),
  )
  .await
  .unwrap()
}

pub(super) fn test_config() -> Config {
  Config {
    port: 8000,
    secret_key: vec![0_u8; 64],
    environment: Environment::Development,
    allowed_origins: vec![],
    firestore_project: "test-project".to_string(),
    firebase_credentials: GoogleAccountKey {
      private_key: String::new(),
      client_email: String::new(),
    },
    secure_cookie: false,
    same_site: SameSite::Lax,
  }
}

// Builds an initialised actix test service with the full middleware stack.
// The macro avoids the complex return-type annotation of test::init_service.
macro_rules! make_app {
  ($db:expr) => {{
    let key = actix_web::cookie::Key::derive_from(&[0_u8; 64]);
    actix_web::test::init_service(
      actix_web::App::new()
        .app_data(actix_web::web::Data::new($db))
        .app_data(actix_web::web::Data::new(crate::integration_tests::test_config()))
        .wrap(actix_identity::IdentityMiddleware::default())
        .wrap(
          actix_session::SessionMiddleware::builder(
            actix_session::storage::CookieSessionStore::default(),
            key,
          )
          .cookie_secure(false)
          .cookie_name("__session".into())
          .build(),
        )
        .service(crate::boards::routes::list)
        .service(crate::boards::routes::new)
        .service(crate::boards::routes::update)
        .service(crate::boards::routes::get)
        .service(crate::boards::routes::delete)
        .service(crate::columns::routes::list)
        .service(crate::columns::routes::new)
        .service(crate::columns::routes::update)
        .service(crate::columns::routes::get)
        .service(crate::columns::routes::delete)
        .service(crate::cards::routes::new)
        .service(crate::cards::routes::list)
        .service(crate::cards::routes::csv)
        .service(crate::cards::routes::update)
        .service(crate::cards::routes::get)
        .service(crate::cards::routes::delete)
        .service(crate::cards::routes::put_vote)
        .service(crate::cards::routes::delete_vote)
        .service(crate::cards::routes::put_reaction)
        .service(crate::cards::routes::delete_reaction),
    )
    .await
  }};
}

pub(super) use make_app;

pub(super) fn session_cookie(
  resp: &actix_web::dev::ServiceResponse<impl actix_web::body::MessageBody>,
) -> Cookie<'static> {
  for val in resp.headers().get_all("set-cookie") {
    if let Ok(s) = val.to_str() {
      if let Ok(c) = Cookie::parse_encoded(s) {
        if c.name() == "__session" {
          return c.into_owned();
        }
      }
    }
  }
  panic!("no __session cookie in response");
}

pub(super) async fn body_json(
  resp: actix_web::dev::ServiceResponse<impl actix_web::body::MessageBody>,
) -> Value {
  let bytes = test::read_body(resp).await;
  serde_json::from_slice(&bytes).expect("response body should be valid JSON")
}

pub(super) async fn setup_board(
  app: &impl actix_web::dev::Service<
    actix_http::Request,
    Response = actix_web::dev::ServiceResponse<impl actix_web::body::MessageBody>,
    Error = actix_web::Error,
  >,
) -> (String, Cookie<'static>) {
  use actix_web::test::TestRequest;
  use serde_json::json;
  let resp = test::call_service(
    app,
    TestRequest::post().uri("/boards").set_json(json!({})).to_request(),
  )
  .await;
  let cookie = session_cookie(&resp);
  let board_id = body_json(resp).await["id"].as_str().unwrap().to_string();
  (board_id, cookie)
}

pub(super) async fn setup_board_and_column(
  app: &impl actix_web::dev::Service<
    actix_http::Request,
    Response = actix_web::dev::ServiceResponse<impl actix_web::body::MessageBody>,
    Error = actix_web::Error,
  >,
) -> (String, String, Cookie<'static>) {
  use actix_web::test::TestRequest;
  use serde_json::json;
  let board_resp = test::call_service(
    app,
    TestRequest::post().uri("/boards").set_json(json!({})).to_request(),
  )
  .await;
  let cookie = session_cookie(&board_resp);
  let board_id = body_json(board_resp).await["id"].as_str().unwrap().to_string();

  let col_resp = test::call_service(
    app,
    TestRequest::post()
      .uri(&format!("/boards/{board_id}/columns"))
      .cookie(cookie.clone())
      .set_json(json!({"name": "Col"}))
      .to_request(),
  )
  .await;
  let col_id = body_json(col_resp).await["id"].as_str().unwrap().to_string();

  (board_id, col_id, cookie)
}
