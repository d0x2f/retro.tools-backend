// Run with: FIRESTORE_EMULATOR_HOST=localhost:8080 cargo test -- --ignored

use actix_identity::IdentityMiddleware;
use actix_session::{storage::CookieSessionStore, SessionMiddleware};
use actix_web::cookie::{Cookie, Key, SameSite};
use actix_web::http::StatusCode;
use actix_web::test::{self, TestRequest};
use actix_web::{web::Data, App};
use chrono::Utc;
use firestore::{FirestoreDb, FirestoreDbOptions};
use gcloud_sdk::{ExternalJwtFunctionSource, Token, TokenSourceType};
use serde_json::{json, Value};

use crate::boards;
use crate::cards;
use crate::columns;
use crate::config::{Config, Environment, GoogleAccountKey};

// ── test infrastructure ────────────────────────────────���─────────────────────

async fn emulator_db() -> FirestoreDb {
  let token_source = ExternalJwtFunctionSource::new(|| async {
    Ok(Token::new(
      "Bearer".to_string(),
      "eyJhbGciOiJub25lIiwidHlwIjoiSldUIn0.eyJzdWIiOiJ0ZXN0In0.".into(),
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

fn test_config() -> Config {
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
    let key = Key::derive_from(&[0_u8; 64]);
    test::init_service(
      App::new()
        .app_data(Data::new($db))
        .app_data(Data::new(test_config()))
        .wrap(IdentityMiddleware::default())
        .wrap(
          SessionMiddleware::builder(CookieSessionStore::default(), key)
            .cookie_secure(false)
            .cookie_name("__session".into())
            .build(),
        )
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
        .service(cards::routes::delete_reaction),
    )
    .await
  }};
}

// Extract the __session cookie from a response's Set-Cookie headers.
fn session_cookie(resp: &actix_web::dev::ServiceResponse<impl actix_web::body::MessageBody>) -> Cookie<'static> {
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

async fn body_json(resp: actix_web::dev::ServiceResponse<impl actix_web::body::MessageBody>) -> Value {
  let bytes = test::read_body(resp).await;
  serde_json::from_slice(&bytes).expect("response body should be valid JSON")
}

// ── boards ───────────────────────────────���───────────────────────────────��───

#[tokio::test]
#[ignore = "requires Firestore emulator: FIRESTORE_EMULATOR_HOST=localhost:8080"]
async fn board_create_returns_200_with_correct_fields() {
  let db = emulator_db().await;
  let app = make_app!(db.clone());

  let resp = test::call_service(
    &app,
    TestRequest::post().uri("/boards").set_json(json!({"name": "My Retro"})).to_request(),
  )
  .await;

  assert_eq!(resp.status(), StatusCode::OK);
  let json = body_json(resp).await;
  let board_id = json["id"].as_str().unwrap().to_string();
  assert_eq!(json["name"], "My Retro");
  assert_eq!(json["cards_open"], true);
  assert_eq!(json["voting_open"], true);
  assert_eq!(json["owner"], true);
  assert_eq!(json["anyone_is_owner"], false);

  boards::db::delete(&db, &board_id).await.unwrap();
}

#[tokio::test]
#[ignore = "requires Firestore emulator: FIRESTORE_EMULATOR_HOST=localhost:8080"]
async fn board_create_defaults_cards_and_voting_open_to_true() {
  let db = emulator_db().await;
  let app = make_app!(db.clone());

  let resp = test::call_service(
    &app,
    TestRequest::post().uri("/boards").set_json(json!({})).to_request(),
  )
  .await;

  assert_eq!(resp.status(), StatusCode::OK);
  let json = body_json(resp).await;
  let board_id = json["id"].as_str().unwrap().to_string();
  assert_eq!(json["cards_open"], true);
  assert_eq!(json["voting_open"], true);

  boards::db::delete(&db, &board_id).await.unwrap();
}

#[tokio::test]
#[ignore = "requires Firestore emulator: FIRESTORE_EMULATOR_HOST=localhost:8080"]
async fn board_list_returns_200_with_array() {
  let app = make_app!(emulator_db().await);

  let resp = test::call_service(&app, TestRequest::get().uri("/boards").to_request()).await;

  assert_eq!(resp.status(), StatusCode::OK);
  assert!(body_json(resp).await.is_array());
}

#[tokio::test]
#[ignore = "requires Firestore emulator: FIRESTORE_EMULATOR_HOST=localhost:8080"]
async fn board_get_returns_200() {
  let db = emulator_db().await;
  let app = make_app!(db.clone());

  let create_resp = test::call_service(
    &app,
    TestRequest::post().uri("/boards").set_json(json!({"name": "Fetch Me"})).to_request(),
  )
  .await;
  let cookie = session_cookie(&create_resp);
  let board_id = body_json(create_resp).await["id"].as_str().unwrap().to_string();

  let resp = test::call_service(
    &app,
    TestRequest::get().uri(&format!("/boards/{board_id}")).cookie(cookie).to_request(),
  )
  .await;

  assert_eq!(resp.status(), StatusCode::OK);
  let json = body_json(resp).await;
  assert_eq!(json["id"], board_id.as_str());
  assert_eq!(json["name"], "Fetch Me");

  boards::db::delete(&db, &board_id).await.unwrap();
}

#[tokio::test]
#[ignore = "requires Firestore emulator: FIRESTORE_EMULATOR_HOST=localhost:8080"]
async fn board_get_nonexistent_returns_404() {
  let app = make_app!(emulator_db().await);

  let resp = test::call_service(
    &app,
    TestRequest::get().uri("/boards/does-not-exist").to_request(),
  )
  .await;

  assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
#[ignore = "requires Firestore emulator: FIRESTORE_EMULATOR_HOST=localhost:8080"]
async fn board_update_as_owner_returns_200() {
  let db = emulator_db().await;
  let app = make_app!(db.clone());

  let create_resp = test::call_service(
    &app,
    TestRequest::post().uri("/boards").set_json(json!({"name": "Before"})).to_request(),
  )
  .await;
  let cookie = session_cookie(&create_resp);
  let board_id = body_json(create_resp).await["id"].as_str().unwrap().to_string();

  let resp = test::call_service(
    &app,
    TestRequest::patch()
      .uri(&format!("/boards/{board_id}"))
      .cookie(cookie)
      .set_json(json!({"name": "After"}))
      .to_request(),
  )
  .await;

  assert_eq!(resp.status(), StatusCode::OK);
  assert_eq!(body_json(resp).await["name"], "After");

  boards::db::delete(&db, &board_id).await.unwrap();
}

#[tokio::test]
#[ignore = "requires Firestore emulator: FIRESTORE_EMULATOR_HOST=localhost:8080"]
async fn board_update_as_non_owner_returns_403() {
  let db = emulator_db().await;
  let app = make_app!(db.clone());

  // Participant A creates the board
  let create_resp = test::call_service(
    &app,
    TestRequest::post().uri("/boards").set_json(json!({"name": "Owner's Board"})).to_request(),
  )
  .await;
  let board_id = body_json(create_resp).await["id"].as_str().unwrap().to_string();

  // Participant B gets a session by listing boards (no cookie → new participant)
  let list_resp =
    test::call_service(&app, TestRequest::get().uri("/boards").to_request()).await;
  let cookie_b = session_cookie(&list_resp);

  let resp = test::call_service(
    &app,
    TestRequest::patch()
      .uri(&format!("/boards/{board_id}"))
      .cookie(cookie_b)
      .set_json(json!({"name": "Hijacked"}))
      .to_request(),
  )
  .await;

  assert_eq!(resp.status(), StatusCode::FORBIDDEN);

  boards::db::delete(&db, &board_id).await.unwrap();
}

#[tokio::test]
#[ignore = "requires Firestore emulator: FIRESTORE_EMULATOR_HOST=localhost:8080"]
async fn board_update_as_non_owner_with_anyone_is_owner_returns_200() {
  let db = emulator_db().await;
  let app = make_app!(db.clone());

  // Participant A creates the board with anyone_is_owner=true
  let create_resp = test::call_service(
    &app,
    TestRequest::post()
      .uri("/boards")
      .set_json(json!({"name": "Open Board", "anyone_is_owner": true}))
      .to_request(),
  )
  .await;
  let board_id = body_json(create_resp).await["id"].as_str().unwrap().to_string();

  // Participant B
  let list_resp =
    test::call_service(&app, TestRequest::get().uri("/boards").to_request()).await;
  let cookie_b = session_cookie(&list_resp);

  let resp = test::call_service(
    &app,
    TestRequest::patch()
      .uri(&format!("/boards/{board_id}"))
      .cookie(cookie_b)
      .set_json(json!({"name": "Updated by B"}))
      .to_request(),
  )
  .await;

  assert_eq!(resp.status(), StatusCode::OK);

  boards::db::delete(&db, &board_id).await.unwrap();
}

#[tokio::test]
#[ignore = "requires Firestore emulator: FIRESTORE_EMULATOR_HOST=localhost:8080"]
async fn board_delete_as_owner_returns_200() {
  let app = make_app!(emulator_db().await);

  let create_resp = test::call_service(
    &app,
    TestRequest::post().uri("/boards").set_json(json!({})).to_request(),
  )
  .await;
  let cookie = session_cookie(&create_resp);
  let board_id = body_json(create_resp).await["id"].as_str().unwrap().to_string();

  let resp = test::call_service(
    &app,
    TestRequest::delete()
      .uri(&format!("/boards/{board_id}"))
      .cookie(cookie)
      .to_request(),
  )
  .await;

  assert_eq!(resp.status(), StatusCode::OK);
}

#[tokio::test]
#[ignore = "requires Firestore emulator: FIRESTORE_EMULATOR_HOST=localhost:8080"]
async fn board_delete_as_non_owner_returns_403() {
  let db = emulator_db().await;
  let app = make_app!(db.clone());

  let create_resp = test::call_service(
    &app,
    TestRequest::post().uri("/boards").set_json(json!({})).to_request(),
  )
  .await;
  let board_id = body_json(create_resp).await["id"].as_str().unwrap().to_string();

  let list_resp =
    test::call_service(&app, TestRequest::get().uri("/boards").to_request()).await;
  let cookie_b = session_cookie(&list_resp);

  let resp = test::call_service(
    &app,
    TestRequest::delete()
      .uri(&format!("/boards/{board_id}"))
      .cookie(cookie_b)
      .to_request(),
  )
  .await;

  assert_eq!(resp.status(), StatusCode::FORBIDDEN);

  boards::db::delete(&db, &board_id).await.unwrap();
}

#[tokio::test]
#[ignore = "requires Firestore emulator: FIRESTORE_EMULATOR_HOST=localhost:8080"]
async fn board_delete_nonexistent_returns_404() {
  let app = make_app!(emulator_db().await);

  let resp = test::call_service(
    &app,
    TestRequest::delete().uri("/boards/no-such-board").to_request(),
  )
  .await;

  assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

// ── columns ──────────────────────────────────────────────────────────────���───

async fn setup_board(app: &impl actix_web::dev::Service<actix_http::Request, Response = actix_web::dev::ServiceResponse<impl actix_web::body::MessageBody>, Error = actix_web::Error>) -> (String, Cookie<'static>) {
  let resp = test::call_service(
    app,
    TestRequest::post().uri("/boards").set_json(json!({})).to_request(),
  )
  .await;
  let cookie = session_cookie(&resp);
  let board_id = body_json(resp).await["id"].as_str().unwrap().to_string();
  (board_id, cookie)
}

#[tokio::test]
#[ignore = "requires Firestore emulator: FIRESTORE_EMULATOR_HOST=localhost:8080"]
async fn column_create_returns_200_with_column_fields() {
  let db = emulator_db().await;
  let app = make_app!(db.clone());
  let (board_id, cookie) = setup_board(&app).await;

  let resp = test::call_service(
    &app,
    TestRequest::post()
      .uri(&format!("/boards/{board_id}/columns"))
      .cookie(cookie)
      .set_json(json!({"name": "Went Well"}))
      .to_request(),
  )
  .await;

  assert_eq!(resp.status(), StatusCode::OK);
  assert_eq!(body_json(resp).await["name"], "Went Well");

  boards::db::delete(&db, &board_id).await.unwrap();
}

#[tokio::test]
#[ignore = "requires Firestore emulator: FIRESTORE_EMULATOR_HOST=localhost:8080"]
async fn column_list_returns_200_with_array() {
  let db = emulator_db().await;
  let app = make_app!(db.clone());
  let (board_id, cookie) = setup_board(&app).await;

  test::call_service(
    &app,
    TestRequest::post()
      .uri(&format!("/boards/{board_id}/columns"))
      .cookie(cookie.clone())
      .set_json(json!({"name": "Col"}))
      .to_request(),
  )
  .await;

  let resp = test::call_service(
    &app,
    TestRequest::get()
      .uri(&format!("/boards/{board_id}/columns"))
      .cookie(cookie)
      .to_request(),
  )
  .await;

  assert_eq!(resp.status(), StatusCode::OK);
  assert!(body_json(resp).await.is_array());

  boards::db::delete(&db, &board_id).await.unwrap();
}

#[tokio::test]
#[ignore = "requires Firestore emulator: FIRESTORE_EMULATOR_HOST=localhost:8080"]
async fn column_get_returns_200() {
  let db = emulator_db().await;
  let app = make_app!(db.clone());
  let (board_id, cookie) = setup_board(&app).await;

  let col_resp = test::call_service(
    &app,
    TestRequest::post()
      .uri(&format!("/boards/{board_id}/columns"))
      .cookie(cookie.clone())
      .set_json(json!({"name": "Col"}))
      .to_request(),
  )
  .await;
  let col_id = body_json(col_resp).await["id"].as_str().unwrap().to_string();

  let resp = test::call_service(
    &app,
    TestRequest::get()
      .uri(&format!("/boards/{board_id}/columns/{col_id}"))
      .cookie(cookie)
      .to_request(),
  )
  .await;

  assert_eq!(resp.status(), StatusCode::OK);
  assert_eq!(body_json(resp).await["id"], col_id.as_str());

  boards::db::delete(&db, &board_id).await.unwrap();
}

#[tokio::test]
#[ignore = "requires Firestore emulator: FIRESTORE_EMULATOR_HOST=localhost:8080"]
async fn column_update_as_owner_returns_200() {
  let db = emulator_db().await;
  let app = make_app!(db.clone());
  let (board_id, cookie) = setup_board(&app).await;

  let col_resp = test::call_service(
    &app,
    TestRequest::post()
      .uri(&format!("/boards/{board_id}/columns"))
      .cookie(cookie.clone())
      .set_json(json!({"name": "Before"}))
      .to_request(),
  )
  .await;
  let col_id = body_json(col_resp).await["id"].as_str().unwrap().to_string();

  let resp = test::call_service(
    &app,
    TestRequest::patch()
      .uri(&format!("/boards/{board_id}/columns/{col_id}"))
      .cookie(cookie)
      .set_json(json!({"name": "After"}))
      .to_request(),
  )
  .await;

  assert_eq!(resp.status(), StatusCode::OK);
  assert_eq!(body_json(resp).await["name"], "After");

  boards::db::delete(&db, &board_id).await.unwrap();
}

#[tokio::test]
#[ignore = "requires Firestore emulator: FIRESTORE_EMULATOR_HOST=localhost:8080"]
async fn column_update_as_non_owner_returns_403() {
  let db = emulator_db().await;
  let app = make_app!(db.clone());
  let (board_id, cookie) = setup_board(&app).await;

  let col_resp = test::call_service(
    &app,
    TestRequest::post()
      .uri(&format!("/boards/{board_id}/columns"))
      .cookie(cookie)
      .set_json(json!({"name": "Col"}))
      .to_request(),
  )
  .await;
  let col_id = body_json(col_resp).await["id"].as_str().unwrap().to_string();

  let list_resp =
    test::call_service(&app, TestRequest::get().uri("/boards").to_request()).await;
  let cookie_b = session_cookie(&list_resp);

  let resp = test::call_service(
    &app,
    TestRequest::patch()
      .uri(&format!("/boards/{board_id}/columns/{col_id}"))
      .cookie(cookie_b)
      .set_json(json!({"name": "Hijacked"}))
      .to_request(),
  )
  .await;

  assert_eq!(resp.status(), StatusCode::FORBIDDEN);

  boards::db::delete(&db, &board_id).await.unwrap();
}

#[tokio::test]
#[ignore = "requires Firestore emulator: FIRESTORE_EMULATOR_HOST=localhost:8080"]
async fn column_delete_as_owner_returns_200() {
  let db = emulator_db().await;
  let app = make_app!(db.clone());
  let (board_id, cookie) = setup_board(&app).await;

  let col_resp = test::call_service(
    &app,
    TestRequest::post()
      .uri(&format!("/boards/{board_id}/columns"))
      .cookie(cookie.clone())
      .set_json(json!({"name": "Col"}))
      .to_request(),
  )
  .await;
  let col_id = body_json(col_resp).await["id"].as_str().unwrap().to_string();

  let resp = test::call_service(
    &app,
    TestRequest::delete()
      .uri(&format!("/boards/{board_id}/columns/{col_id}"))
      .cookie(cookie)
      .to_request(),
  )
  .await;

  assert_eq!(resp.status(), StatusCode::OK);

  boards::db::delete(&db, &board_id).await.unwrap();
}

#[tokio::test]
#[ignore = "requires Firestore emulator: FIRESTORE_EMULATOR_HOST=localhost:8080"]
async fn column_delete_as_non_owner_returns_403() {
  let db = emulator_db().await;
  let app = make_app!(db.clone());
  let (board_id, cookie) = setup_board(&app).await;

  let col_resp = test::call_service(
    &app,
    TestRequest::post()
      .uri(&format!("/boards/{board_id}/columns"))
      .cookie(cookie)
      .set_json(json!({"name": "Col"}))
      .to_request(),
  )
  .await;
  let col_id = body_json(col_resp).await["id"].as_str().unwrap().to_string();

  let list_resp =
    test::call_service(&app, TestRequest::get().uri("/boards").to_request()).await;
  let cookie_b = session_cookie(&list_resp);

  let resp = test::call_service(
    &app,
    TestRequest::delete()
      .uri(&format!("/boards/{board_id}/columns/{col_id}"))
      .cookie(cookie_b)
      .to_request(),
  )
  .await;

  assert_eq!(resp.status(), StatusCode::FORBIDDEN);

  boards::db::delete(&db, &board_id).await.unwrap();
}

// ── cards ─────────────────────────────────────────────────────────────────────

async fn setup_board_and_column(app: &impl actix_web::dev::Service<actix_http::Request, Response = actix_web::dev::ServiceResponse<impl actix_web::body::MessageBody>, Error = actix_web::Error>) -> (String, String, Cookie<'static>) {
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

#[tokio::test]
#[ignore = "requires Firestore emulator: FIRESTORE_EMULATOR_HOST=localhost:8080"]
async fn card_create_returns_200_with_card_fields() {
  let db = emulator_db().await;
  let app = make_app!(db.clone());
  let (board_id, col_id, cookie) = setup_board_and_column(&app).await;

  let resp = test::call_service(
    &app,
    TestRequest::post()
      .uri(&format!("/boards/{board_id}/columns/{col_id}/cards"))
      .cookie(cookie)
      .set_json(json!({"text": "A good thing"}))
      .to_request(),
  )
  .await;

  assert_eq!(resp.status(), StatusCode::OK);
  let json = body_json(resp).await;
  assert_eq!(json["text"], "A good thing");
  assert_eq!(json["owner"], true);
  assert_eq!(json["column"], col_id.as_str());

  boards::db::delete(&db, &board_id).await.unwrap();
}

#[tokio::test]
#[ignore = "requires Firestore emulator: FIRESTORE_EMULATOR_HOST=localhost:8080"]
async fn card_create_empty_text_returns_400() {
  let db = emulator_db().await;
  let app = make_app!(db.clone());
  let (board_id, col_id, cookie) = setup_board_and_column(&app).await;

  let resp = test::call_service(
    &app,
    TestRequest::post()
      .uri(&format!("/boards/{board_id}/columns/{col_id}/cards"))
      .cookie(cookie)
      .set_json(json!({"text": ""}))
      .to_request(),
  )
  .await;

  assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

  boards::db::delete(&db, &board_id).await.unwrap();
}

#[tokio::test]
#[ignore = "requires Firestore emulator: FIRESTORE_EMULATOR_HOST=localhost:8080"]
async fn card_create_missing_text_returns_400() {
  let db = emulator_db().await;
  let app = make_app!(db.clone());
  let (board_id, col_id, cookie) = setup_board_and_column(&app).await;

  let resp = test::call_service(
    &app,
    TestRequest::post()
      .uri(&format!("/boards/{board_id}/columns/{col_id}/cards"))
      .cookie(cookie)
      .set_json(json!({"author": "Alice"}))
      .to_request(),
  )
  .await;

  assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

  boards::db::delete(&db, &board_id).await.unwrap();
}

#[tokio::test]
#[ignore = "requires Firestore emulator: FIRESTORE_EMULATOR_HOST=localhost:8080"]
async fn card_create_when_cards_closed_returns_403() {
  let db = emulator_db().await;
  let app = make_app!(db.clone());

  let board_resp = test::call_service(
    &app,
    TestRequest::post()
      .uri("/boards")
      .set_json(json!({"cards_open": false}))
      .to_request(),
  )
  .await;
  let cookie = session_cookie(&board_resp);
  let board_id = body_json(board_resp).await["id"].as_str().unwrap().to_string();

  let col_resp = test::call_service(
    &app,
    TestRequest::post()
      .uri(&format!("/boards/{board_id}/columns"))
      .cookie(cookie.clone())
      .set_json(json!({"name": "Col"}))
      .to_request(),
  )
  .await;
  let col_id = body_json(col_resp).await["id"].as_str().unwrap().to_string();

  let resp = test::call_service(
    &app,
    TestRequest::post()
      .uri(&format!("/boards/{board_id}/columns/{col_id}/cards"))
      .cookie(cookie)
      .set_json(json!({"text": "A card"}))
      .to_request(),
  )
  .await;

  assert_eq!(resp.status(), StatusCode::FORBIDDEN);

  boards::db::delete(&db, &board_id).await.unwrap();
}

#[tokio::test]
#[ignore = "requires Firestore emulator: FIRESTORE_EMULATOR_HOST=localhost:8080"]
async fn card_list_returns_200_with_array() {
  let db = emulator_db().await;
  let app = make_app!(db.clone());
  let (board_id, col_id, cookie) = setup_board_and_column(&app).await;

  test::call_service(
    &app,
    TestRequest::post()
      .uri(&format!("/boards/{board_id}/columns/{col_id}/cards"))
      .cookie(cookie.clone())
      .set_json(json!({"text": "A card"}))
      .to_request(),
  )
  .await;

  let resp = test::call_service(
    &app,
    TestRequest::get()
      .uri(&format!("/boards/{board_id}/cards"))
      .cookie(cookie)
      .to_request(),
  )
  .await;

  assert_eq!(resp.status(), StatusCode::OK);
  let json = body_json(resp).await;
  assert!(json.is_array());
  assert_eq!(json.as_array().unwrap().len(), 1);

  boards::db::delete(&db, &board_id).await.unwrap();
}

#[tokio::test]
#[ignore = "requires Firestore emulator: FIRESTORE_EMULATOR_HOST=localhost:8080"]
async fn card_get_returns_200() {
  let db = emulator_db().await;
  let app = make_app!(db.clone());
  let (board_id, col_id, cookie) = setup_board_and_column(&app).await;

  let card_resp = test::call_service(
    &app,
    TestRequest::post()
      .uri(&format!("/boards/{board_id}/columns/{col_id}/cards"))
      .cookie(cookie.clone())
      .set_json(json!({"text": "A card"}))
      .to_request(),
  )
  .await;
  let card_id = body_json(card_resp).await["id"].as_str().unwrap().to_string();

  let resp = test::call_service(
    &app,
    TestRequest::get()
      .uri(&format!("/boards/{board_id}/cards/{card_id}"))
      .cookie(cookie)
      .to_request(),
  )
  .await;

  assert_eq!(resp.status(), StatusCode::OK);
  assert_eq!(body_json(resp).await["id"], card_id.as_str());

  boards::db::delete(&db, &board_id).await.unwrap();
}

#[tokio::test]
#[ignore = "requires Firestore emulator: FIRESTORE_EMULATOR_HOST=localhost:8080"]
async fn card_update_as_card_owner_returns_200() {
  let db = emulator_db().await;
  let app = make_app!(db.clone());
  let (board_id, col_id, cookie) = setup_board_and_column(&app).await;

  let card_resp = test::call_service(
    &app,
    TestRequest::post()
      .uri(&format!("/boards/{board_id}/columns/{col_id}/cards"))
      .cookie(cookie.clone())
      .set_json(json!({"text": "Original"}))
      .to_request(),
  )
  .await;
  let card_id = body_json(card_resp).await["id"].as_str().unwrap().to_string();

  let resp = test::call_service(
    &app,
    TestRequest::patch()
      .uri(&format!("/boards/{board_id}/cards/{card_id}"))
      .cookie(cookie)
      .set_json(json!({"text": "Updated"}))
      .to_request(),
  )
  .await;

  assert_eq!(resp.status(), StatusCode::OK);
  assert_eq!(body_json(resp).await["text"], "Updated");

  boards::db::delete(&db, &board_id).await.unwrap();
}

#[tokio::test]
#[ignore = "requires Firestore emulator: FIRESTORE_EMULATOR_HOST=localhost:8080"]
async fn card_update_as_non_owner_returns_403() {
  let db = emulator_db().await;
  let app = make_app!(db.clone());
  let (board_id, col_id, cookie) = setup_board_and_column(&app).await;

  let card_resp = test::call_service(
    &app,
    TestRequest::post()
      .uri(&format!("/boards/{board_id}/columns/{col_id}/cards"))
      .cookie(cookie)
      .set_json(json!({"text": "Original"}))
      .to_request(),
  )
  .await;
  let card_id = body_json(card_resp).await["id"].as_str().unwrap().to_string();

  let list_resp =
    test::call_service(&app, TestRequest::get().uri("/boards").to_request()).await;
  let cookie_b = session_cookie(&list_resp);

  let resp = test::call_service(
    &app,
    TestRequest::patch()
      .uri(&format!("/boards/{board_id}/cards/{card_id}"))
      .cookie(cookie_b)
      .set_json(json!({"text": "Hijacked"}))
      .to_request(),
  )
  .await;

  assert_eq!(resp.status(), StatusCode::FORBIDDEN);

  boards::db::delete(&db, &board_id).await.unwrap();
}

#[tokio::test]
#[ignore = "requires Firestore emulator: FIRESTORE_EMULATOR_HOST=localhost:8080"]
async fn card_delete_as_card_owner_returns_200() {
  let db = emulator_db().await;
  let app = make_app!(db.clone());
  let (board_id, col_id, cookie) = setup_board_and_column(&app).await;

  let card_resp = test::call_service(
    &app,
    TestRequest::post()
      .uri(&format!("/boards/{board_id}/columns/{col_id}/cards"))
      .cookie(cookie.clone())
      .set_json(json!({"text": "Bye"}))
      .to_request(),
  )
  .await;
  let card_id = body_json(card_resp).await["id"].as_str().unwrap().to_string();

  let resp = test::call_service(
    &app,
    TestRequest::delete()
      .uri(&format!("/boards/{board_id}/cards/{card_id}"))
      .cookie(cookie)
      .to_request(),
  )
  .await;

  assert_eq!(resp.status(), StatusCode::OK);

  boards::db::delete(&db, &board_id).await.unwrap();
}

#[tokio::test]
#[ignore = "requires Firestore emulator: FIRESTORE_EMULATOR_HOST=localhost:8080"]
async fn card_delete_as_board_owner_returns_200() {
  let db = emulator_db().await;
  let app = make_app!(db.clone());
  let (board_id, col_id, owner_cookie) = setup_board_and_column(&app).await;

  // Participant B creates a card
  let list_resp =
    test::call_service(&app, TestRequest::get().uri("/boards").to_request()).await;
  let cookie_b = session_cookie(&list_resp);

  // Trigger participant B to join the board (GET board registers participant)
  test::call_service(
    &app,
    TestRequest::get()
      .uri(&format!("/boards/{board_id}"))
      .cookie(cookie_b.clone())
      .to_request(),
  )
  .await;

  let card_resp = test::call_service(
    &app,
    TestRequest::post()
      .uri(&format!("/boards/{board_id}/columns/{col_id}/cards"))
      .cookie(cookie_b)
      .set_json(json!({"text": "B's card"}))
      .to_request(),
  )
  .await;
  let card_id = body_json(card_resp).await["id"].as_str().unwrap().to_string();

  // Board owner (A) deletes B's card
  let resp = test::call_service(
    &app,
    TestRequest::delete()
      .uri(&format!("/boards/{board_id}/cards/{card_id}"))
      .cookie(owner_cookie)
      .to_request(),
  )
  .await;

  assert_eq!(resp.status(), StatusCode::OK);

  boards::db::delete(&db, &board_id).await.unwrap();
}

#[tokio::test]
#[ignore = "requires Firestore emulator: FIRESTORE_EMULATOR_HOST=localhost:8080"]
async fn card_vote_returns_201() {
  let db = emulator_db().await;
  let app = make_app!(db.clone());
  let (board_id, col_id, cookie) = setup_board_and_column(&app).await;

  let card_resp = test::call_service(
    &app,
    TestRequest::post()
      .uri(&format!("/boards/{board_id}/columns/{col_id}/cards"))
      .cookie(cookie.clone())
      .set_json(json!({"text": "Vote me"}))
      .to_request(),
  )
  .await;
  let card_id = body_json(card_resp).await["id"].as_str().unwrap().to_string();

  let resp = test::call_service(
    &app,
    TestRequest::put()
      .uri(&format!("/boards/{board_id}/cards/{card_id}/vote"))
      .cookie(cookie)
      .to_request(),
  )
  .await;

  assert_eq!(resp.status(), StatusCode::CREATED);

  boards::db::delete(&db, &board_id).await.unwrap();
}

#[tokio::test]
#[ignore = "requires Firestore emulator: FIRESTORE_EMULATOR_HOST=localhost:8080"]
async fn card_vote_when_voting_closed_returns_403() {
  let db = emulator_db().await;
  let app = make_app!(db.clone());

  let board_resp = test::call_service(
    &app,
    TestRequest::post()
      .uri("/boards")
      .set_json(json!({"voting_open": false}))
      .to_request(),
  )
  .await;
  let cookie = session_cookie(&board_resp);
  let board_id = body_json(board_resp).await["id"].as_str().unwrap().to_string();

  let col_resp = test::call_service(
    &app,
    TestRequest::post()
      .uri(&format!("/boards/{board_id}/columns"))
      .cookie(cookie.clone())
      .set_json(json!({"name": "Col"}))
      .to_request(),
  )
  .await;
  let col_id = body_json(col_resp).await["id"].as_str().unwrap().to_string();

  let card_resp = test::call_service(
    &app,
    TestRequest::post()
      .uri(&format!("/boards/{board_id}/columns/{col_id}/cards"))
      .cookie(cookie.clone())
      .set_json(json!({"text": "A card"}))
      .to_request(),
  )
  .await;
  let card_id = body_json(card_resp).await["id"].as_str().unwrap().to_string();

  let resp = test::call_service(
    &app,
    TestRequest::put()
      .uri(&format!("/boards/{board_id}/cards/{card_id}/vote"))
      .cookie(cookie)
      .to_request(),
  )
  .await;

  assert_eq!(resp.status(), StatusCode::FORBIDDEN);

  boards::db::delete(&db, &board_id).await.unwrap();
}

#[tokio::test]
#[ignore = "requires Firestore emulator: FIRESTORE_EMULATOR_HOST=localhost:8080"]
async fn card_delete_vote_returns_201() {
  let db = emulator_db().await;
  let app = make_app!(db.clone());
  let (board_id, col_id, cookie) = setup_board_and_column(&app).await;

  let card_resp = test::call_service(
    &app,
    TestRequest::post()
      .uri(&format!("/boards/{board_id}/columns/{col_id}/cards"))
      .cookie(cookie.clone())
      .set_json(json!({"text": "Vote me"}))
      .to_request(),
  )
  .await;
  let card_id = body_json(card_resp).await["id"].as_str().unwrap().to_string();

  test::call_service(
    &app,
    TestRequest::put()
      .uri(&format!("/boards/{board_id}/cards/{card_id}/vote"))
      .cookie(cookie.clone())
      .to_request(),
  )
  .await;

  let resp = test::call_service(
    &app,
    TestRequest::delete()
      .uri(&format!("/boards/{board_id}/cards/{card_id}/vote"))
      .cookie(cookie)
      .to_request(),
  )
  .await;

  assert_eq!(resp.status(), StatusCode::CREATED);

  boards::db::delete(&db, &board_id).await.unwrap();
}

#[tokio::test]
#[ignore = "requires Firestore emulator: FIRESTORE_EMULATOR_HOST=localhost:8080"]
async fn card_react_returns_201() {
  let db = emulator_db().await;
  let app = make_app!(db.clone());
  let (board_id, col_id, cookie) = setup_board_and_column(&app).await;

  let card_resp = test::call_service(
    &app,
    TestRequest::post()
      .uri(&format!("/boards/{board_id}/columns/{col_id}/cards"))
      .cookie(cookie.clone())
      .set_json(json!({"text": "React to me"}))
      .to_request(),
  )
  .await;
  let card_id = body_json(card_resp).await["id"].as_str().unwrap().to_string();

  let resp = test::call_service(
    &app,
    TestRequest::put()
      .uri(&format!("/boards/{board_id}/cards/{card_id}/react"))
      .cookie(cookie)
      .set_json(json!({"emoji": "👍"}))
      .to_request(),
  )
  .await;

  assert_eq!(resp.status(), StatusCode::CREATED);

  boards::db::delete(&db, &board_id).await.unwrap();
}

#[tokio::test]
#[ignore = "requires Firestore emulator: FIRESTORE_EMULATOR_HOST=localhost:8080"]
async fn card_delete_react_returns_201() {
  let db = emulator_db().await;
  let app = make_app!(db.clone());
  let (board_id, col_id, cookie) = setup_board_and_column(&app).await;

  let card_resp = test::call_service(
    &app,
    TestRequest::post()
      .uri(&format!("/boards/{board_id}/columns/{col_id}/cards"))
      .cookie(cookie.clone())
      .set_json(json!({"text": "React to me"}))
      .to_request(),
  )
  .await;
  let card_id = body_json(card_resp).await["id"].as_str().unwrap().to_string();

  test::call_service(
    &app,
    TestRequest::put()
      .uri(&format!("/boards/{board_id}/cards/{card_id}/react"))
      .cookie(cookie.clone())
      .set_json(json!({"emoji": "👍"}))
      .to_request(),
  )
  .await;

  let resp = test::call_service(
    &app,
    TestRequest::delete()
      .uri(&format!("/boards/{board_id}/cards/{card_id}/react"))
      .cookie(cookie)
      .to_request(),
  )
  .await;

  assert_eq!(resp.status(), StatusCode::CREATED);

  boards::db::delete(&db, &board_id).await.unwrap();
}

#[tokio::test]
#[ignore = "requires Firestore emulator: FIRESTORE_EMULATOR_HOST=localhost:8080"]
async fn board_csv_returns_200_with_attachment_header() {
  let db = emulator_db().await;
  let app = make_app!(db.clone());
  let (board_id, col_id, cookie) = setup_board_and_column(&app).await;

  test::call_service(
    &app,
    TestRequest::post()
      .uri(&format!("/boards/{board_id}/columns/{col_id}/cards"))
      .cookie(cookie.clone())
      .set_json(json!({"text": "A card", "author": "Alice"}))
      .to_request(),
  )
  .await;

  let resp = test::call_service(
    &app,
    TestRequest::get()
      .uri(&format!("/boards/{board_id}/csv"))
      .cookie(cookie)
      .to_request(),
  )
  .await;

  assert_eq!(resp.status(), StatusCode::OK);
  let content_disposition = resp
    .headers()
    .get("content-disposition")
    .unwrap()
    .to_str()
    .unwrap();
  assert!(content_disposition.contains("attachment"));

  boards::db::delete(&db, &board_id).await.unwrap();
}
