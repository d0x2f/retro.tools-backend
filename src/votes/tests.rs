use super::super::models::{Card, NewBoard, NewCard, NewRank, Vote};
use super::super::schema::vote::dsl::vote as vote_table;
use super::super::testing::{create_board, create_card, create_rank, run_test};
use diesel::pg::PgConnection;
use diesel::prelude::*;
use rocket::http::Status;
use rocket::local::Client;

#[test]
fn test_post_vote() {
  run_test(|client: Client, db: &PgConnection| {
    // Create a board, rank & card
    let board = create_board(
      &client,
      &NewBoard {
        name: "test board",
        voting_open: Some(true),
        cards_open: Some(true),
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
    let card = create_card(
      db,
      NewCard {
        id: None,
        rank_id: &rank.id,
        name: "test card",
        description: "card description",
      },
    );

    // Make a vote
    let request = client.post(format!(
      "/boards/{}/ranks/{}/cards/{}/vote",
      board.id, rank.id, card.id
    ));
    let mut response = request.dispatch();
    let response_card: Card =
      serde_json::from_str(response.body_string().unwrap().as_str()).unwrap();

    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response_card.id, card.id);
    assert_eq!(response_card.rank_id, rank.id);
    assert_eq!(response_card.name, "test card");
    assert_eq!(response_card.description, "card description");
    assert_eq!(response_card.votes, 1);

    // Ensure the database contains the same vote info
    let db_votes = vote_table.load::<Vote>(db).unwrap();

    assert_eq!(db_votes.len(), 1);
    assert_eq!(db_votes[0].card_id, card.id);
    assert_eq!(db_votes[0].count, 1);
  });
}

#[test]
fn test_post_vote_over_limit() {
  run_test(|client: Client, db: &PgConnection| {
    // Create a board, rank & card
    let board = create_board(
      &client,
      &NewBoard {
        name: "test board",
        max_votes: Some(1),
        voting_open: Some(true),
        cards_open: Some(true),
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
    let card = create_card(
      db,
      NewCard {
        id: None,
        rank_id: &rank.id,
        name: "test card",
        description: "card description",
      },
    );

    // Make two votes
    let path = format!(
      "/boards/{}/ranks/{}/cards/{}/vote",
      board.id, rank.id, card.id
    );
    client.post(&path).dispatch();
    let mut response = client.post(path).dispatch();
    let response_card: Card =
      serde_json::from_str(response.body_string().unwrap().as_str()).unwrap();

    // Vote count stays as 1 despite two votes made
    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response_card.id, card.id);
    assert_eq!(response_card.rank_id, rank.id);
    assert_eq!(response_card.name, "test card");
    assert_eq!(response_card.description, "card description");
    assert_eq!(response_card.votes, 1);
    assert_eq!(response_card.voted, true);

    // Ensure the database contains the same vote info
    let db_votes = vote_table.load::<Vote>(db).unwrap();

    assert_eq!(db_votes.len(), 1);
    assert_eq!(db_votes[0].card_id, card.id);
    assert_eq!(db_votes[0].count, 1);
  });
}

#[test]
fn test_post_vote_forbidden() {
  run_test(|client: Client, db: &PgConnection| {
    // Create a board, rank & card
    let board = create_board(
      &client,
      &NewBoard {
        name: "test board",
        max_votes: Some(1),
        voting_open: Some(false),
        cards_open: Some(true),
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
    let card = create_card(
      db,
      NewCard {
        id: None,
        rank_id: &rank.id,
        name: "test card",
        description: "card description",
      },
    );

    // Make a vote
    let path = format!(
      "/boards/{}/ranks/{}/cards/{}/vote",
      board.id, rank.id, card.id
    );
    let response = client.post(path).dispatch();

    // Vote count stays as 1 despite two votes made
    assert_eq!(response.status(), Status::Forbidden);

    // Ensure the database contains no vote info
    let db_votes = vote_table.load::<Vote>(db).unwrap();

    assert_eq!(db_votes.len(), 0);
  });
}

#[test]
fn test_delete_vote() {
  run_test(|client: Client, db: &PgConnection| {
    // Create a board, rank & card
    let board = create_board(
      &client,
      &NewBoard {
        name: "test board",
        voting_open: Some(true),
        cards_open: Some(true),
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
    let card = create_card(
      db,
      NewCard {
        id: None,
        rank_id: &rank.id,
        name: "test card",
        description: "card description",
      },
    );

    // Make a vote
    let mut response = client
      .post(format!(
        "/boards/{}/ranks/{}/cards/{}/vote",
        board.id, rank.id, card.id
      ))
      .dispatch();
    let response_card: Card =
      serde_json::from_str(response.body_string().unwrap().as_str()).unwrap();

    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response_card.id, card.id);
    assert_eq!(response_card.rank_id, rank.id);
    assert_eq!(response_card.name, "test card");
    assert_eq!(response_card.description, "card description");
    assert_eq!(response_card.votes, 1);
    assert_eq!(response_card.voted, true);

    // Ensure the database contains the same vote info
    let db_votes = vote_table.load::<Vote>(db).unwrap();

    assert_eq!(db_votes.len(), 1);
    assert_eq!(db_votes[0].card_id, card.id);
    assert_eq!(db_votes[0].count, 1);

    // Retract the vote
    let mut response = client
      .delete(format!(
        "/boards/{}/ranks/{}/cards/{}/vote",
        board.id, rank.id, card.id
      ))
      .dispatch();
    let response_card: Card =
      serde_json::from_str(response.body_string().unwrap().as_str()).unwrap();

    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response_card.id, card.id);
    assert_eq!(response_card.rank_id, rank.id);
    assert_eq!(response_card.name, "test card");
    assert_eq!(response_card.description, "card description");
    assert_eq!(response_card.votes, 0);
    assert_eq!(response_card.voted, false);

    // Ensure the database contains the same vote info
    let db_votes = vote_table.load::<Vote>(db).unwrap();

    assert_eq!(db_votes.len(), 1);
    assert_eq!(db_votes[0].card_id, card.id);
    assert_eq!(db_votes[0].count, 0);

    // Retract the vote again
    let mut response = client
      .delete(format!(
        "/boards/{}/ranks/{}/cards/{}/vote",
        board.id, rank.id, card.id
      ))
      .dispatch();
    let response_card: Card =
      serde_json::from_str(response.body_string().unwrap().as_str()).unwrap();

    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response_card.id, card.id);
    assert_eq!(response_card.rank_id, rank.id);
    assert_eq!(response_card.name, "test card");
    assert_eq!(response_card.description, "card description");
    assert_eq!(response_card.votes, 0);

    // Ensure the database contains the same vote info
    let db_votes = vote_table.load::<Vote>(db).unwrap();

    assert_eq!(db_votes.len(), 1);
    assert_eq!(db_votes[0].card_id, card.id);
    assert_eq!(db_votes[0].count, 0);
  });
}

#[test]
fn test_delete_vote_forbidden() {
  run_test(|client: Client, db: &PgConnection| {
    // Create a board, rank & card
    let board = create_board(
      &client,
      &NewBoard {
        name: "test board",
        max_votes: Some(1),
        voting_open: Some(false),
        cards_open: Some(true),
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
    let card = create_card(
      db,
      NewCard {
        id: None,
        rank_id: &rank.id,
        name: "test card",
        description: "card description",
      },
    );

    // Make a vote
    let path = format!(
      "/boards/{}/ranks/{}/cards/{}/vote",
      board.id, rank.id, card.id
    );
    let response = client.delete(path).dispatch();

    // Vote count stays as 1 despite two votes made
    assert_eq!(response.status(), Status::Forbidden);

    // Ensure the database contains no vote info
    let db_votes = vote_table.load::<Vote>(db).unwrap();

    assert_eq!(db_votes.len(), 0);
  });
}
