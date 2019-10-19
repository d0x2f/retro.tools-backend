use super::super::models::{NewBoard, NewRank, Rank, UpdateRank};
use super::super::schema::rank::dsl::rank as rank_table;
use super::super::testing::{create_board, create_rank, run_test};
use diesel::pg::PgConnection;
use diesel::prelude::*;
use rocket::http::ContentType;
use rocket::http::Status;
use rocket::local::Client;

#[test]
fn test_post_rank() {
  run_test(|client: Client, db: &PgConnection| {
    // Create a board
    let board = create_board(
      &client,
      &NewBoard {
        name: "test board",
        ..Default::default()
      },
    );

    // Create a Rank
    let mut response = client
      .post(format!("/boards/{}/ranks", board.id))
      .header(ContentType::JSON)
      .body(
        serde_json::to_string(&NewRank {
          id: None,
          name: "test rank",
          board_id: &board.id,
        })
        .unwrap(),
      )
      .dispatch();

    // Ensure response contains the right rank
    let response_rank: Rank =
      serde_json::from_str(response.body_string().unwrap().as_str()).unwrap();

    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response_rank.name, "test rank");

    // Ensure the database contains only the new rank
    let db_ranks = rank_table.load::<Rank>(db).unwrap();

    assert_eq!(db_ranks.len(), 1);
    assert_eq!(db_ranks[0].id, response_rank.id);
    assert_eq!(db_ranks[0].board_id, board.id);
    assert_eq!(db_ranks[0].name, "test rank");
  });
}

#[test]
fn test_get_ranks_empty() {
  run_test(|client: Client, db: &PgConnection| {
    // Create a board
    let board = create_board(
      &client,
      &NewBoard {
        name: "test board",
        ..Default::default()
      },
    );

    let request = client.get(format!("/boards/{}/ranks", board.id));
    let mut response = request.dispatch();

    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response.body_string(), Some("[]".into()));

    // Ensure the database contains no ranks
    let db_ranks = rank_table.load::<Rank>(db).unwrap();
    assert_eq!(db_ranks.len(), 0);
  });
}

#[test]
fn test_get_ranks() {
  run_test(|client: Client, db: &PgConnection| {
    // Create a board & rank
    let board = create_board(
      &client,
      &NewBoard {
        name: "test board",
        ..Default::default()
      },
    );
    let rank = create_rank(
      db,
      NewRank {
        id: None,
        board_id: &board.id,
        name: "test rank",
      },
    );

    // Get the ranks
    let request = client.get(format!("/boards/{}/ranks", board.id));
    let mut response = request.dispatch();
    let response_ranks: Vec<Rank> =
      serde_json::from_str(response.body_string().unwrap().as_str()).unwrap();

    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response_ranks.len(), 1);
    assert_eq!(response_ranks[0].id, rank.id);
    assert_eq!(response_ranks[0].name, "test rank");
    assert_eq!(response_ranks[0].board_id, board.id);

    // Ensure the database contains the same rank
    let db_ranks = rank_table.load::<Rank>(db).unwrap();

    assert_eq!(db_ranks.len(), 1);
    assert_eq!(db_ranks[0].id, response_ranks[0].id);
    assert_eq!(db_ranks[0].board_id, board.id);
    assert_eq!(db_ranks[0].name, "test rank");
  });
}

#[test]
fn test_get_rank() {
  run_test(|client: Client, db: &PgConnection| {
    // Create a board & rank
    let board = create_board(
      &client,
      &NewBoard {
        name: "test board",
        ..Default::default()
      },
    );
    let rank = create_rank(
      db,
      NewRank {
        id: None,
        board_id: &board.id,
        name: "test rank",
      },
    );

    // Get the rank
    let request = client.get(format!("/boards/{}/ranks/{}", board.id, rank.id));
    let mut response = request.dispatch();
    let response_rank: Rank =
      serde_json::from_str(response.body_string().unwrap().as_str()).unwrap();

    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response_rank.id, rank.id);
    assert_eq!(response_rank.name, "test rank");
    assert_eq!(response_rank.board_id, board.id);

    // Ensure the database contains the same rank
    let db_ranks = rank_table.load::<Rank>(db).unwrap();

    assert_eq!(db_ranks.len(), 1);
    assert_eq!(db_ranks[0].id, response_rank.id);
    assert_eq!(db_ranks[0].board_id, board.id);
    assert_eq!(db_ranks[0].name, "test rank");
  });
}

#[test]
fn test_patch_rank() {
  run_test(|client: Client, db: &PgConnection| {
    // Create a board & rank
    let board = create_board(
      &client,
      &NewBoard {
        name: "test board",
        ..Default::default()
      },
    );
    let rank = create_rank(
      db,
      NewRank {
        id: None,
        board_id: &board.id,
        name: "test rank",
      },
    );

    // Modify the rank
    let mut response = client
      .patch(format!("/boards/{}/ranks/{}", board.id, rank.id))
      .header(ContentType::JSON)
      .body(
        serde_json::to_string(&UpdateRank {
          name: "rank test".into(),
        })
        .unwrap(),
      )
      .dispatch();
    let response_rank: Rank =
      serde_json::from_str(response.body_string().unwrap().as_str()).unwrap();

    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response_rank.name, "rank test");
    assert_eq!(response_rank.board_id, board.id);

    // Ensure the database contains the same rank
    let db_ranks = rank_table.load::<Rank>(db).unwrap();

    assert_eq!(db_ranks.len(), 1);
    assert_eq!(db_ranks[0].id, response_rank.id);
    assert_eq!(db_ranks[0].board_id, board.id);
    assert_eq!(db_ranks[0].name, "rank test");
  });
}

#[test]
fn test_delete_rank() {
  run_test(|client: Client, db: &PgConnection| {
    // Create a board & rank
    let board = create_board(
      &client,
      &NewBoard {
        name: "test board",
        ..Default::default()
      },
    );
    let rank = create_rank(
      db,
      NewRank {
        id: None,
        board_id: &board.id,
        name: "test rank",
      },
    );

    // Delete the rank
    let response = client
      .delete(format!("/boards/{}/ranks/{}", board.id, rank.id))
      .dispatch();

    assert_eq!(response.status(), Status::Ok);

    // Ensure the database doesn't contain any ranks
    let db_ranks = rank_table.load::<Rank>(db).unwrap();

    assert_eq!(db_ranks.len(), 0);
  });
}
