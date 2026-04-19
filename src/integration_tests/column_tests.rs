use actix_web::http::StatusCode;
use actix_web::test::TestRequest;
use serde_json::json;

use crate::boards;
use crate::integration_tests::{body_json, emulator_db, make_app, session_cookie, setup_board};

#[tokio::test]
#[ignore = "requires Firestore emulator: FIRESTORE_EMULATOR_HOST=localhost:8080"]
async fn create_returns_200_with_column_fields() {
  let db = emulator_db().await;
  let app = make_app!(db.clone());
  let (board_id, cookie) = setup_board(&app).await;

  let resp = actix_web::test::call_service(
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
async fn list_returns_200_with_array() {
  let db = emulator_db().await;
  let app = make_app!(db.clone());
  let (board_id, cookie) = setup_board(&app).await;

  actix_web::test::call_service(
    &app,
    TestRequest::post()
      .uri(&format!("/boards/{board_id}/columns"))
      .cookie(cookie.clone())
      .set_json(json!({"name": "Col"}))
      .to_request(),
  )
  .await;

  let resp = actix_web::test::call_service(
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
async fn get_returns_200() {
  let db = emulator_db().await;
  let app = make_app!(db.clone());
  let (board_id, cookie) = setup_board(&app).await;

  let col_resp = actix_web::test::call_service(
    &app,
    TestRequest::post()
      .uri(&format!("/boards/{board_id}/columns"))
      .cookie(cookie.clone())
      .set_json(json!({"name": "Col"}))
      .to_request(),
  )
  .await;
  let col_id = body_json(col_resp).await["id"].as_str().unwrap().to_string();

  let resp = actix_web::test::call_service(
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
async fn update_as_owner_returns_200() {
  let db = emulator_db().await;
  let app = make_app!(db.clone());
  let (board_id, cookie) = setup_board(&app).await;

  let col_resp = actix_web::test::call_service(
    &app,
    TestRequest::post()
      .uri(&format!("/boards/{board_id}/columns"))
      .cookie(cookie.clone())
      .set_json(json!({"name": "Before"}))
      .to_request(),
  )
  .await;
  let col_id = body_json(col_resp).await["id"].as_str().unwrap().to_string();

  let resp = actix_web::test::call_service(
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
async fn update_as_non_owner_returns_403() {
  let db = emulator_db().await;
  let app = make_app!(db.clone());
  let (board_id, cookie) = setup_board(&app).await;

  let col_resp = actix_web::test::call_service(
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
    actix_web::test::call_service(&app, TestRequest::get().uri("/boards").to_request()).await;
  let cookie_b = session_cookie(&list_resp);

  let resp = actix_web::test::call_service(
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
async fn update_as_non_owner_with_anyone_is_owner_still_returns_403() {
  let db = emulator_db().await;
  let app = make_app!(db.clone());

  let board_resp = actix_web::test::call_service(
    &app,
    TestRequest::post()
      .uri("/boards")
      .set_json(json!({"anyone_is_owner": true}))
      .to_request(),
  )
  .await;
  let cookie = session_cookie(&board_resp);
  let board_id = body_json(board_resp).await["id"].as_str().unwrap().to_string();

  let col_resp = actix_web::test::call_service(
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
    actix_web::test::call_service(&app, TestRequest::get().uri("/boards").to_request()).await;
  let cookie_b = session_cookie(&list_resp);

  let resp = actix_web::test::call_service(
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
async fn delete_as_owner_returns_200() {
  let db = emulator_db().await;
  let app = make_app!(db.clone());
  let (board_id, cookie) = setup_board(&app).await;

  let col_resp = actix_web::test::call_service(
    &app,
    TestRequest::post()
      .uri(&format!("/boards/{board_id}/columns"))
      .cookie(cookie.clone())
      .set_json(json!({"name": "Col"}))
      .to_request(),
  )
  .await;
  let col_id = body_json(col_resp).await["id"].as_str().unwrap().to_string();

  let resp = actix_web::test::call_service(
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
async fn delete_as_non_owner_returns_403() {
  let db = emulator_db().await;
  let app = make_app!(db.clone());
  let (board_id, cookie) = setup_board(&app).await;

  let col_resp = actix_web::test::call_service(
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
    actix_web::test::call_service(&app, TestRequest::get().uri("/boards").to_request()).await;
  let cookie_b = session_cookie(&list_resp);

  let resp = actix_web::test::call_service(
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
