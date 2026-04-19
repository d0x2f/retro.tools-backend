use actix_web::http::StatusCode;
use actix_web::test::TestRequest;
use serde_json::json;

use crate::boards;
use crate::integration_tests::{body_json, emulator_db, make_app, session_cookie};

#[tokio::test]
#[ignore = "requires Firestore emulator: FIRESTORE_EMULATOR_HOST=localhost:8080"]
async fn create_returns_200_with_correct_fields() {
  let db = emulator_db().await;
  let app = make_app!(db.clone());

  let resp = actix_web::test::call_service(
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
async fn create_defaults_cards_and_voting_open_to_true() {
  let db = emulator_db().await;
  let app = make_app!(db.clone());

  let resp = actix_web::test::call_service(
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
async fn list_returns_200_with_array() {
  let app = make_app!(emulator_db().await);

  let resp =
    actix_web::test::call_service(&app, TestRequest::get().uri("/boards").to_request()).await;

  assert_eq!(resp.status(), StatusCode::OK);
  assert!(body_json(resp).await.is_array());
}

#[tokio::test]
#[ignore = "requires Firestore emulator: FIRESTORE_EMULATOR_HOST=localhost:8080"]
async fn get_returns_200() {
  let db = emulator_db().await;
  let app = make_app!(db.clone());

  let create_resp = actix_web::test::call_service(
    &app,
    TestRequest::post().uri("/boards").set_json(json!({"name": "Fetch Me"})).to_request(),
  )
  .await;
  let cookie = session_cookie(&create_resp);
  let board_id = body_json(create_resp).await["id"].as_str().unwrap().to_string();

  let resp = actix_web::test::call_service(
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
async fn get_nonexistent_returns_404() {
  let app = make_app!(emulator_db().await);

  let resp = actix_web::test::call_service(
    &app,
    TestRequest::get().uri("/boards/does-not-exist").to_request(),
  )
  .await;

  assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
#[ignore = "requires Firestore emulator: FIRESTORE_EMULATOR_HOST=localhost:8080"]
async fn update_as_owner_returns_200() {
  let db = emulator_db().await;
  let app = make_app!(db.clone());

  let create_resp = actix_web::test::call_service(
    &app,
    TestRequest::post().uri("/boards").set_json(json!({"name": "Before"})).to_request(),
  )
  .await;
  let cookie = session_cookie(&create_resp);
  let board_id = body_json(create_resp).await["id"].as_str().unwrap().to_string();

  let resp = actix_web::test::call_service(
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
async fn update_as_non_owner_returns_403() {
  let db = emulator_db().await;
  let app = make_app!(db.clone());

  let create_resp = actix_web::test::call_service(
    &app,
    TestRequest::post().uri("/boards").set_json(json!({"name": "Owner's Board"})).to_request(),
  )
  .await;
  let board_id = body_json(create_resp).await["id"].as_str().unwrap().to_string();

  // Participant B gets a session by listing boards (no cookie → new participant)
  let list_resp =
    actix_web::test::call_service(&app, TestRequest::get().uri("/boards").to_request()).await;
  let cookie_b = session_cookie(&list_resp);

  let resp = actix_web::test::call_service(
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
async fn update_as_non_owner_with_anyone_is_owner_returns_200() {
  let db = emulator_db().await;
  let app = make_app!(db.clone());

  let create_resp = actix_web::test::call_service(
    &app,
    TestRequest::post()
      .uri("/boards")
      .set_json(json!({"name": "Open Board", "anyone_is_owner": true}))
      .to_request(),
  )
  .await;
  let board_id = body_json(create_resp).await["id"].as_str().unwrap().to_string();

  let list_resp =
    actix_web::test::call_service(&app, TestRequest::get().uri("/boards").to_request()).await;
  let cookie_b = session_cookie(&list_resp);

  let resp = actix_web::test::call_service(
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
async fn delete_as_owner_returns_200() {
  let app = make_app!(emulator_db().await);

  let create_resp = actix_web::test::call_service(
    &app,
    TestRequest::post().uri("/boards").set_json(json!({})).to_request(),
  )
  .await;
  let cookie = session_cookie(&create_resp);
  let board_id = body_json(create_resp).await["id"].as_str().unwrap().to_string();

  let resp = actix_web::test::call_service(
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
async fn delete_as_non_owner_returns_403() {
  let db = emulator_db().await;
  let app = make_app!(db.clone());

  let create_resp = actix_web::test::call_service(
    &app,
    TestRequest::post().uri("/boards").set_json(json!({})).to_request(),
  )
  .await;
  let board_id = body_json(create_resp).await["id"].as_str().unwrap().to_string();

  let list_resp =
    actix_web::test::call_service(&app, TestRequest::get().uri("/boards").to_request()).await;
  let cookie_b = session_cookie(&list_resp);

  let resp = actix_web::test::call_service(
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
async fn delete_nonexistent_returns_404() {
  let app = make_app!(emulator_db().await);

  let resp = actix_web::test::call_service(
    &app,
    TestRequest::delete().uri("/boards/no-such-board").to_request(),
  )
  .await;

  assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}
