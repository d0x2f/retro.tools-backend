use super::super::models::{Board, NewBoard, UpdateBoard};
use super::super::persistence::get_boards;
use super::super::testing::{create_board, run_test};
use diesel::pg::PgConnection;
use rocket::http::ContentType;
use rocket::http::Status;
use rocket::local::Client;

#[test]
fn test_post_board() {
  run_test(|client: Client, db: &PgConnection| {
    // Create a board
    let mut response = client
      .post("/boards")
      .header(ContentType::JSON)
      .body(
        serde_json::to_string(&NewBoard {
          id: None,
          name: "test board",
          max_votes: Some(37),
          voting_open: Some(false),
          cards_open: Some(true),
        })
        .unwrap(),
      )
      .dispatch();

    // Ensure response contains the right board
    let response_board: Board =
      serde_json::from_str(response.body_string().unwrap().as_str()).unwrap();

    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response_board.name, "test board");
    assert_eq!(response_board.max_votes, 37);
    assert_eq!(response_board.voting_open, false);
    assert_eq!(response_board.cards_open, true);

    // Ensure the database contains only the new board
    let cookies = response.cookies();
    let id_cookie = cookies[0].value();
    let db_boards = get_boards(db, id_cookie).unwrap();

    assert_eq!(db_boards.len(), 1);
    assert_eq!(db_boards[0].id, response_board.id);
    assert_eq!(db_boards[0].name, "test board");
    assert_eq!(db_boards[0].max_votes, 37);
    assert_eq!(db_boards[0].voting_open, false);
    assert_eq!(db_boards[0].cards_open, true);
  });
}

#[test]
fn test_get_boards_empty() {
  run_test(|client: Client, db: &PgConnection| {
    let request = client.get("/boards");
    let mut response = request.dispatch();

    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response.body_string(), Some("[]".into()));

    // Ensure the database contains no boards
    let cookies = response.cookies();
    let id_cookie = cookies[0].value();
    let db_boards = get_boards(db, id_cookie).unwrap();
    assert_eq!(db_boards.len(), 0);
  });
}

#[test]
fn test_get_boards() {
  run_test(|client: Client, db: &PgConnection| {
    // Create a new board
    let (board, participant_id) = create_board(
      &client,
      &NewBoard {
        id: None,
        name: "test",
        max_votes: Some(47),
        voting_open: Some(true),
        cards_open: Some(false),
      },
    );

    // Get the boards
    let request = client.get("/boards");
    let mut response = request.dispatch();
    let response_boards: Vec<Board> =
      serde_json::from_str(response.body_string().unwrap().as_str()).unwrap();

    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response_boards.len(), 1);
    assert_eq!(response_boards[0].id, board.id);
    assert_eq!(response_boards[0].name, "test");
    assert_eq!(response_boards[0].max_votes, 47);
    assert_eq!(response_boards[0].voting_open, true);
    assert_eq!(response_boards[0].cards_open, false);

    // Ensure the database contains only the one board
    let db_boards = get_boards(db, &participant_id).unwrap();

    assert_eq!(db_boards.len(), 1);
    assert_eq!(db_boards[0].id, response_boards[0].id);
    assert_eq!(db_boards[0].name, "test");
    assert_eq!(db_boards[0].max_votes, 47);
    assert_eq!(db_boards[0].voting_open, true);
    assert_eq!(db_boards[0].cards_open, false);
  });
}

#[test]
fn test_get_board_missing() {
  run_test(|client: Client, _db: &PgConnection| {
    // Get some board
    let request = client.get("/boards/Xighaeb9Oqua3Yei");
    let mut response = request.dispatch();

    assert_eq!(response.status(), Status::NotFound);
    assert_eq!(response.body_string().unwrap().as_str(), "");
  });
}

#[test]
fn test_get_board() {
  run_test(|client: Client, db: &PgConnection| {
    // Create a new board
    let (created_board, participant_id) = create_board(
      &client,
      &NewBoard {
        id: None,
        name: "test",
        max_votes: Some(2),
        voting_open: Some(true),
        cards_open: Some(true),
      },
    );

    // Get the board
    let mut response = client
      .get(format!("/boards/{}", created_board.id))
      .dispatch();
    let response_board: Board =
      serde_json::from_str(response.body_string().unwrap().as_str()).unwrap();

    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response_board.id, created_board.id);
    assert_eq!(response_board.name, "test");
    assert_eq!(response_board.max_votes, 2);
    assert_eq!(response_board.voting_open, true);
    assert_eq!(response_board.cards_open, true);

    // Ensure the database contains the same board
    let db_boards = get_boards(db, &participant_id).unwrap();

    assert_eq!(db_boards.len(), 1);
    assert_eq!(db_boards[0].id, response_board.id);
    assert_eq!(db_boards[0].name, "test");
    assert_eq!(db_boards[0].max_votes, 2);
    assert_eq!(db_boards[0].voting_open, true);
    assert_eq!(db_boards[0].cards_open, true);
  });
}

#[test]
fn test_patch_board() {
  run_test(|client: Client, db: &PgConnection| {
    // Create a new board
    let (created_board, participant_id) = create_board(
      &client,
      &NewBoard {
        id: None,
        name: "test",
        max_votes: Some(2),
        voting_open: Some(true),
        cards_open: Some(true),
      },
    );

    // Modify the board
    let mut modify_response = client
      .patch(format!("/boards/{}", created_board.id))
      .header(ContentType::JSON)
      .body(
        serde_json::to_string(&UpdateBoard {
          name: Some("tset".into()),
          max_votes: Some(3),
          voting_open: Some(false),
          cards_open: Some(false),
        })
        .unwrap(),
      )
      .dispatch();

    // Ensure the patch response contains the modified board
    let response_board: Board =
      serde_json::from_str(modify_response.body_string().unwrap().as_str()).unwrap();

    assert_eq!(modify_response.status(), Status::Ok);
    assert_eq!(response_board.id, created_board.id);
    assert_eq!(response_board.name, "tset");
    assert_eq!(response_board.max_votes, 3);
    assert_eq!(response_board.voting_open, false);
    assert_eq!(response_board.cards_open, false);

    // Ensure the database contains the modified board
    let db_boards = get_boards(db, &participant_id).unwrap();

    assert_eq!(db_boards.len(), 1);
    assert_eq!(db_boards[0].id, created_board.id);
    assert_eq!(db_boards[0].name, "tset");
    assert_eq!(db_boards[0].max_votes, 3);
    assert_eq!(db_boards[0].voting_open, false);
    assert_eq!(db_boards[0].cards_open, false);
  });
}

#[test]
fn test_delete_board() {
  run_test(|client: Client, db: &PgConnection| {
    // Create a new board
    let (created_board, participant_id) = create_board(
      &client,
      &NewBoard {
        name: "test board",
        ..Default::default()
      },
    );

    // Delete the board
    let response = client
      .delete(format!("/boards/{}", created_board.id))
      .dispatch();

    assert_eq!(response.status(), Status::Ok);

    // Ensure the database doesn't contain the board
    let db_boards = get_boards(db, &participant_id).unwrap();

    assert_eq!(db_boards.len(), 0);
  });
}
