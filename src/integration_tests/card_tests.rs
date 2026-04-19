use actix_web::http::StatusCode;
use actix_web::test::TestRequest;
use serde_json::json;

use crate::boards;
use crate::integration_tests::{
  body_json, emulator_db, make_app, session_cookie, setup_board_and_column,
};

#[tokio::test]
#[ignore = "requires Firestore emulator: FIRESTORE_EMULATOR_HOST=localhost:8080"]
async fn create_returns_200_with_card_fields() {
  let db = emulator_db().await;
  let app = make_app!(db.clone());
  let (board_id, col_id, cookie) = setup_board_and_column(&app).await;

  let resp = actix_web::test::call_service(
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
async fn create_empty_text_returns_400() {
  let db = emulator_db().await;
  let app = make_app!(db.clone());
  let (board_id, col_id, cookie) = setup_board_and_column(&app).await;

  let resp = actix_web::test::call_service(
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
async fn create_missing_text_returns_400() {
  let db = emulator_db().await;
  let app = make_app!(db.clone());
  let (board_id, col_id, cookie) = setup_board_and_column(&app).await;

  let resp = actix_web::test::call_service(
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
async fn create_when_cards_closed_returns_403() {
  let db = emulator_db().await;
  let app = make_app!(db.clone());

  let board_resp = actix_web::test::call_service(
    &app,
    TestRequest::post()
      .uri("/boards")
      .set_json(json!({"cards_open": false}))
      .to_request(),
  )
  .await;
  let cookie = session_cookie(&board_resp);
  let board_id = body_json(board_resp).await["id"].as_str().unwrap().to_string();

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
async fn list_returns_200_with_array() {
  let db = emulator_db().await;
  let app = make_app!(db.clone());
  let (board_id, col_id, cookie) = setup_board_and_column(&app).await;

  actix_web::test::call_service(
    &app,
    TestRequest::post()
      .uri(&format!("/boards/{board_id}/columns/{col_id}/cards"))
      .cookie(cookie.clone())
      .set_json(json!({"text": "A card"}))
      .to_request(),
  )
  .await;

  let resp = actix_web::test::call_service(
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
async fn get_returns_200() {
  let db = emulator_db().await;
  let app = make_app!(db.clone());
  let (board_id, col_id, cookie) = setup_board_and_column(&app).await;

  let card_resp = actix_web::test::call_service(
    &app,
    TestRequest::post()
      .uri(&format!("/boards/{board_id}/columns/{col_id}/cards"))
      .cookie(cookie.clone())
      .set_json(json!({"text": "A card"}))
      .to_request(),
  )
  .await;
  let card_id = body_json(card_resp).await["id"].as_str().unwrap().to_string();

  let resp = actix_web::test::call_service(
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
async fn update_as_card_owner_returns_200() {
  let db = emulator_db().await;
  let app = make_app!(db.clone());
  let (board_id, col_id, cookie) = setup_board_and_column(&app).await;

  let card_resp = actix_web::test::call_service(
    &app,
    TestRequest::post()
      .uri(&format!("/boards/{board_id}/columns/{col_id}/cards"))
      .cookie(cookie.clone())
      .set_json(json!({"text": "Original"}))
      .to_request(),
  )
  .await;
  let card_id = body_json(card_resp).await["id"].as_str().unwrap().to_string();

  let resp = actix_web::test::call_service(
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
async fn update_as_non_owner_returns_403() {
  let db = emulator_db().await;
  let app = make_app!(db.clone());
  let (board_id, col_id, cookie) = setup_board_and_column(&app).await;

  let card_resp = actix_web::test::call_service(
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
    actix_web::test::call_service(&app, TestRequest::get().uri("/boards").to_request()).await;
  let cookie_b = session_cookie(&list_resp);

  let resp = actix_web::test::call_service(
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
async fn delete_as_non_owner_returns_403() {
  let db = emulator_db().await;
  let app = make_app!(db.clone());
  let (board_id, col_id, cookie) = setup_board_and_column(&app).await;

  let card_resp = actix_web::test::call_service(
    &app,
    TestRequest::post()
      .uri(&format!("/boards/{board_id}/columns/{col_id}/cards"))
      .cookie(cookie)
      .set_json(json!({"text": "Mine"}))
      .to_request(),
  )
  .await;
  let card_id = body_json(card_resp).await["id"].as_str().unwrap().to_string();

  let list_resp =
    actix_web::test::call_service(&app, TestRequest::get().uri("/boards").to_request()).await;
  let cookie_b = session_cookie(&list_resp);

  let resp = actix_web::test::call_service(
    &app,
    TestRequest::delete()
      .uri(&format!("/boards/{board_id}/cards/{card_id}"))
      .cookie(cookie_b)
      .to_request(),
  )
  .await;

  assert_eq!(resp.status(), StatusCode::FORBIDDEN);

  boards::db::delete(&db, &board_id).await.unwrap();
}

#[tokio::test]
#[ignore = "requires Firestore emulator: FIRESTORE_EMULATOR_HOST=localhost:8080"]
async fn delete_as_card_owner_returns_200() {
  let db = emulator_db().await;
  let app = make_app!(db.clone());
  let (board_id, col_id, cookie) = setup_board_and_column(&app).await;

  let card_resp = actix_web::test::call_service(
    &app,
    TestRequest::post()
      .uri(&format!("/boards/{board_id}/columns/{col_id}/cards"))
      .cookie(cookie.clone())
      .set_json(json!({"text": "Bye"}))
      .to_request(),
  )
  .await;
  let card_id = body_json(card_resp).await["id"].as_str().unwrap().to_string();

  let resp = actix_web::test::call_service(
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
async fn delete_as_board_owner_returns_200() {
  let db = emulator_db().await;
  let app = make_app!(db.clone());
  let (board_id, col_id, owner_cookie) = setup_board_and_column(&app).await;

  // Participant B gets a session
  let list_resp =
    actix_web::test::call_service(&app, TestRequest::get().uri("/boards").to_request()).await;
  let cookie_b = session_cookie(&list_resp);

  // Trigger participant B to join the board (GET board registers participant)
  actix_web::test::call_service(
    &app,
    TestRequest::get()
      .uri(&format!("/boards/{board_id}"))
      .cookie(cookie_b.clone())
      .to_request(),
  )
  .await;

  let card_resp = actix_web::test::call_service(
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
  let resp = actix_web::test::call_service(
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
async fn update_as_non_owner_with_open_permission_returns_200() {
  let db = emulator_db().await;
  let app = make_app!(db.clone());

  let board_resp = actix_web::test::call_service(
    &app,
    TestRequest::post()
      .uri("/boards")
      .set_json(json!({"open_permission": true}))
      .to_request(),
  )
  .await;
  let owner_cookie = session_cookie(&board_resp);
  let board_id = body_json(board_resp).await["id"].as_str().unwrap().to_string();

  let col_resp = actix_web::test::call_service(
    &app,
    TestRequest::post()
      .uri(&format!("/boards/{board_id}/columns"))
      .cookie(owner_cookie.clone())
      .set_json(json!({"name": "Col"}))
      .to_request(),
  )
  .await;
  let col_id = body_json(col_resp).await["id"].as_str().unwrap().to_string();

  let card_resp = actix_web::test::call_service(
    &app,
    TestRequest::post()
      .uri(&format!("/boards/{board_id}/columns/{col_id}/cards"))
      .cookie(owner_cookie)
      .set_json(json!({"text": "Original"}))
      .to_request(),
  )
  .await;
  let card_id = body_json(card_resp).await["id"].as_str().unwrap().to_string();

  let list_resp =
    actix_web::test::call_service(&app, TestRequest::get().uri("/boards").to_request()).await;
  let cookie_b = session_cookie(&list_resp);

  let resp = actix_web::test::call_service(
    &app,
    TestRequest::patch()
      .uri(&format!("/boards/{board_id}/cards/{card_id}"))
      .cookie(cookie_b)
      .set_json(json!({"text": "Updated by non-owner"}))
      .to_request(),
  )
  .await;

  assert_eq!(resp.status(), StatusCode::OK);
  assert_eq!(body_json(resp).await["text"], "Updated by non-owner");

  boards::db::delete(&db, &board_id).await.unwrap();
}

#[tokio::test]
#[ignore = "requires Firestore emulator: FIRESTORE_EMULATOR_HOST=localhost:8080"]
async fn delete_as_non_owner_with_open_permission_returns_200() {
  let db = emulator_db().await;
  let app = make_app!(db.clone());

  let board_resp = actix_web::test::call_service(
    &app,
    TestRequest::post()
      .uri("/boards")
      .set_json(json!({"open_permission": true}))
      .to_request(),
  )
  .await;
  let owner_cookie = session_cookie(&board_resp);
  let board_id = body_json(board_resp).await["id"].as_str().unwrap().to_string();

  let col_resp = actix_web::test::call_service(
    &app,
    TestRequest::post()
      .uri(&format!("/boards/{board_id}/columns"))
      .cookie(owner_cookie.clone())
      .set_json(json!({"name": "Col"}))
      .to_request(),
  )
  .await;
  let col_id = body_json(col_resp).await["id"].as_str().unwrap().to_string();

  let card_resp = actix_web::test::call_service(
    &app,
    TestRequest::post()
      .uri(&format!("/boards/{board_id}/columns/{col_id}/cards"))
      .cookie(owner_cookie)
      .set_json(json!({"text": "To be deleted"}))
      .to_request(),
  )
  .await;
  let card_id = body_json(card_resp).await["id"].as_str().unwrap().to_string();

  let list_resp =
    actix_web::test::call_service(&app, TestRequest::get().uri("/boards").to_request()).await;
  let cookie_b = session_cookie(&list_resp);

  let resp = actix_web::test::call_service(
    &app,
    TestRequest::delete()
      .uri(&format!("/boards/{board_id}/cards/{card_id}"))
      .cookie(cookie_b)
      .to_request(),
  )
  .await;

  assert_eq!(resp.status(), StatusCode::OK);

  boards::db::delete(&db, &board_id).await.unwrap();
}

#[tokio::test]
#[ignore = "requires Firestore emulator: FIRESTORE_EMULATOR_HOST=localhost:8080"]
async fn vote_returns_201() {
  let db = emulator_db().await;
  let app = make_app!(db.clone());
  let (board_id, col_id, cookie) = setup_board_and_column(&app).await;

  let card_resp = actix_web::test::call_service(
    &app,
    TestRequest::post()
      .uri(&format!("/boards/{board_id}/columns/{col_id}/cards"))
      .cookie(cookie.clone())
      .set_json(json!({"text": "Vote me"}))
      .to_request(),
  )
  .await;
  let card_id = body_json(card_resp).await["id"].as_str().unwrap().to_string();

  let resp = actix_web::test::call_service(
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
async fn vote_when_voting_closed_returns_403() {
  let db = emulator_db().await;
  let app = make_app!(db.clone());

  let board_resp = actix_web::test::call_service(
    &app,
    TestRequest::post()
      .uri("/boards")
      .set_json(json!({"voting_open": false}))
      .to_request(),
  )
  .await;
  let cookie = session_cookie(&board_resp);
  let board_id = body_json(board_resp).await["id"].as_str().unwrap().to_string();

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

  let card_resp = actix_web::test::call_service(
    &app,
    TestRequest::post()
      .uri(&format!("/boards/{board_id}/columns/{col_id}/cards"))
      .cookie(cookie.clone())
      .set_json(json!({"text": "A card"}))
      .to_request(),
  )
  .await;
  let card_id = body_json(card_resp).await["id"].as_str().unwrap().to_string();

  let resp = actix_web::test::call_service(
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
async fn delete_vote_returns_201() {
  let db = emulator_db().await;
  let app = make_app!(db.clone());
  let (board_id, col_id, cookie) = setup_board_and_column(&app).await;

  let card_resp = actix_web::test::call_service(
    &app,
    TestRequest::post()
      .uri(&format!("/boards/{board_id}/columns/{col_id}/cards"))
      .cookie(cookie.clone())
      .set_json(json!({"text": "Vote me"}))
      .to_request(),
  )
  .await;
  let card_id = body_json(card_resp).await["id"].as_str().unwrap().to_string();

  actix_web::test::call_service(
    &app,
    TestRequest::put()
      .uri(&format!("/boards/{board_id}/cards/{card_id}/vote"))
      .cookie(cookie.clone())
      .to_request(),
  )
  .await;

  let resp = actix_web::test::call_service(
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
async fn react_returns_201() {
  let db = emulator_db().await;
  let app = make_app!(db.clone());
  let (board_id, col_id, cookie) = setup_board_and_column(&app).await;

  let card_resp = actix_web::test::call_service(
    &app,
    TestRequest::post()
      .uri(&format!("/boards/{board_id}/columns/{col_id}/cards"))
      .cookie(cookie.clone())
      .set_json(json!({"text": "React to me"}))
      .to_request(),
  )
  .await;
  let card_id = body_json(card_resp).await["id"].as_str().unwrap().to_string();

  let resp = actix_web::test::call_service(
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
async fn delete_react_returns_201() {
  let db = emulator_db().await;
  let app = make_app!(db.clone());
  let (board_id, col_id, cookie) = setup_board_and_column(&app).await;

  let card_resp = actix_web::test::call_service(
    &app,
    TestRequest::post()
      .uri(&format!("/boards/{board_id}/columns/{col_id}/cards"))
      .cookie(cookie.clone())
      .set_json(json!({"text": "React to me"}))
      .to_request(),
  )
  .await;
  let card_id = body_json(card_resp).await["id"].as_str().unwrap().to_string();

  actix_web::test::call_service(
    &app,
    TestRequest::put()
      .uri(&format!("/boards/{board_id}/cards/{card_id}/react"))
      .cookie(cookie.clone())
      .set_json(json!({"emoji": "👍"}))
      .to_request(),
  )
  .await;

  let resp = actix_web::test::call_service(
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
async fn csv_returns_200_with_attachment_header() {
  let db = emulator_db().await;
  let app = make_app!(db.clone());
  let (board_id, col_id, cookie) = setup_board_and_column(&app).await;

  actix_web::test::call_service(
    &app,
    TestRequest::post()
      .uri(&format!("/boards/{board_id}/columns/{col_id}/cards"))
      .cookie(cookie.clone())
      .set_json(json!({"text": "A card", "author": "Alice"}))
      .to_request(),
  )
  .await;

  let resp = actix_web::test::call_service(
    &app,
    TestRequest::get()
      .uri(&format!("/boards/{board_id}/csv"))
      .cookie(cookie)
      .to_request(),
  )
  .await;

  assert_eq!(resp.status(), StatusCode::OK);
  let content_disposition =
    resp.headers().get("content-disposition").unwrap().to_str().unwrap();
  assert!(content_disposition.contains("attachment"));

  boards::db::delete(&db, &board_id).await.unwrap();
}
