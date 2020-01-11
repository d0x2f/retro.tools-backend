use super::super::models::{Card, NewBoard, NewCard, NewRank, UpdateCard};
use super::super::persistence::get_rank_cards;
use super::super::testing::{create_board, create_card, create_new_client, create_rank, run_test};
use diesel::pg::PgConnection;
use rocket::http::ContentType;
use rocket::http::Status;
use rocket::local::Client;

#[test]
fn test_post_card() {
  run_test(|client: Client, db: &PgConnection| {
    // Create a board & rank
    let (board, participant_id) = create_board(
      &client,
      &NewBoard {
        name: "test board",
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

    // Create a card
    let mut response = client
      .post(format!("/boards/{}/ranks/{}/cards", board.id, rank.id))
      .header(ContentType::JSON)
      .body(
        serde_json::to_string(&NewCard {
          id: None,
          rank_id: &rank.id,
          name: "test card",
          description: "card description",
          participant_id: &participant_id,
        })
        .unwrap(),
      )
      .dispatch();

    // Ensure response contains the right card
    let response_card: Card =
      serde_json::from_str(response.body_string().unwrap().as_str()).unwrap();

    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response_card.name, "test card");
    assert_eq!(response_card.description, "card description");
    assert_eq!(response_card.votes, 0);

    // Ensure the database contains only the new card
    let db_cards = get_rank_cards(db, &rank.id, "").unwrap();

    assert_eq!(db_cards.len(), 1);
    assert_eq!(db_cards[0].id, response_card.id);
    assert_eq!(db_cards[0].rank_id, rank.id);
    assert_eq!(db_cards[0].name, "test card");
    assert_eq!(db_cards[0].description, "card description");
    assert_eq!(db_cards[0].votes, 0);
  });
}

#[test]
fn test_post_card_forbidden() {
  run_test(|client: Client, db: &PgConnection| {
    // Create a board & rank (cards disallowed)
    let (board, participant_id) = create_board(
      &client,
      &NewBoard {
        name: "test board",
        cards_open: Some(false),
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

    // Create a card
    let response = client
      .post(format!("/boards/{}/ranks/{}/cards", board.id, rank.id))
      .header(ContentType::JSON)
      .body(
        serde_json::to_string(&NewCard {
          id: None,
          rank_id: &rank.id,
          name: "test card",
          description: "card description",
          participant_id: &participant_id,
        })
        .unwrap(),
      )
      .dispatch();

    assert_eq!(response.status(), Status::Forbidden);

    // Ensure the database contains no cards
    let db_cards = get_rank_cards(db, &rank.id, "").unwrap();

    assert_eq!(db_cards.len(), 0);
  });
}

#[test]
fn test_get_cards() {
  run_test(|client: Client, db: &PgConnection| {
    // Create a board, rank & card
    let (board, participant_id) = create_board(
      &client,
      &NewBoard {
        name: "test board",
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
        participant_id: &participant_id,
      },
    );

    // Get the cards
    let request = client.get(format!("/boards/{}/ranks/{}/cards", board.id, rank.id));
    let mut response = request.dispatch();
    let response_cards: Vec<Card> =
      serde_json::from_str(response.body_string().unwrap().as_str()).unwrap();

    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response_cards.len(), 1);
    assert_eq!(response_cards[0].id, card.id);
    assert_eq!(response_cards[0].rank_id, rank.id);
    assert_eq!(response_cards[0].name, "test card");
    assert_eq!(response_cards[0].description, "card description");
    assert_eq!(response_cards[0].votes, 0);

    // Ensure the database contains the same card
    let db_cards = get_rank_cards(db, &rank.id, "").unwrap();

    assert_eq!(db_cards.len(), 1);
    assert_eq!(db_cards[0].id, response_cards[0].id);
    assert_eq!(db_cards[0].rank_id, rank.id);
    assert_eq!(db_cards[0].name, "test card");
    assert_eq!(db_cards[0].description, "card description");
    assert_eq!(db_cards[0].votes, 0);
  });
}

#[test]
fn test_get_card() {
  run_test(|client: Client, db: &PgConnection| {
    // Create a board, rank & card
    let (board, participant_id) = create_board(
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
    let card = create_card(
      db,
      NewCard {
        id: None,
        rank_id: &rank.id,
        name: "test card",
        description: "card description",
        participant_id: &participant_id,
      },
    );

    // Get the card
    let request = client.get(format!(
      "/boards/{}/ranks/{}/cards/{}",
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
    assert_eq!(response_card.votes, 0);
    assert_eq!(response_card.owner, true);

    // Ensure the database contains the same card
    let db_cards = get_rank_cards(db, &rank.id, "").unwrap();

    assert_eq!(db_cards.len(), 1);
    assert_eq!(db_cards[0].id, response_card.id);
    assert_eq!(db_cards[0].rank_id, rank.id);
    assert_eq!(db_cards[0].name, "test card");
    assert_eq!(db_cards[0].description, "card description");
    assert_eq!(db_cards[0].votes, 0);
  });
}

#[test]
fn test_non_author_get_card() {
  run_test(|client: Client, db: &PgConnection| {
    // Create another client
    let other_client = create_new_client();

    // Create a board, rank & card
    let (board, participant_id) = create_board(
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
    let card = create_card(
      db,
      NewCard {
        id: None,
        rank_id: &rank.id,
        name: "test card",
        description: "card description",
        participant_id: &participant_id,
      },
    );

    // Get the card as another participant
    let request = other_client.get(format!(
      "/boards/{}/ranks/{}/cards/{}",
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
    assert_eq!(response_card.votes, 0);
    assert_eq!(response_card.owner, false);

    // Ensure the database contains the same card
    let db_cards = get_rank_cards(db, &rank.id, "").unwrap();

    assert_eq!(db_cards.len(), 1);
    assert_eq!(db_cards[0].id, response_card.id);
    assert_eq!(db_cards[0].rank_id, rank.id);
    assert_eq!(db_cards[0].name, "test card");
    assert_eq!(db_cards[0].description, "card description");
    assert_eq!(db_cards[0].votes, 0);
  });
}

#[test]
fn test_patch_card() {
  run_test(|client: Client, db: &PgConnection| {
    // Create a board, rank & card
    let (board, participant_id) = create_board(
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
    let rank_b = create_rank(
      db,
      NewRank {
        id: None,
        board_id: &board.id,
        name: "test rank b",
      },
    );
    let card = create_card(
      db,
      NewCard {
        id: None,
        rank_id: &rank.id,
        name: "test card",
        description: "card description",
        participant_id: &participant_id,
      },
    );

    // Modify the card
    let mut response = client
      .patch(format!(
        "/boards/{}/ranks/{}/cards/{}",
        board.id, rank.id, card.id
      ))
      .header(ContentType::JSON)
      .body(
        serde_json::to_string(&UpdateCard {
          name: Some("card test".into()),
          description: Some("description test".into()),
          rank_id: Some(rank_b.id.clone()),
        })
        .unwrap(),
      )
      .dispatch();
    let response_card: Card =
      serde_json::from_str(response.body_string().unwrap().as_str()).unwrap();

    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response_card.id, card.id);
    assert_eq!(response_card.rank_id, rank_b.id);
    assert_eq!(response_card.name, "card test");
    assert_eq!(response_card.description, "description test");
    assert_eq!(response_card.votes, 0);

    // Ensure the database contains the same card
    let db_cards = get_rank_cards(db, &rank_b.id, "").unwrap();

    assert_eq!(db_cards.len(), 1);
    assert_eq!(db_cards[0].id, response_card.id);
    assert_eq!(db_cards[0].rank_id, rank_b.id);
    assert_eq!(db_cards[0].name, "card test");
    assert_eq!(db_cards[0].description, "description test");
    assert_eq!(db_cards[0].votes, 0);
  });
}

#[test]
fn test_non_author_patch_card() {
  run_test(|client: Client, db: &PgConnection| {
    // Create another client
    let other_client = create_new_client();

    // Create a board, rank & card
    let (board, participant_id) = create_board(
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
    let rank_b = create_rank(
      db,
      NewRank {
        id: None,
        board_id: &board.id,
        name: "test rank b",
      },
    );
    let card = create_card(
      db,
      NewCard {
        id: None,
        rank_id: &rank.id,
        name: "test card",
        description: "card description",
        participant_id: &participant_id,
      },
    );

    // Modify the card
    let response = other_client
      .patch(format!(
        "/boards/{}/ranks/{}/cards/{}",
        board.id, rank.id, card.id
      ))
      .header(ContentType::JSON)
      .body(
        serde_json::to_string(&UpdateCard {
          name: Some("card test".into()),
          description: Some("description test".into()),
          rank_id: Some(rank_b.id.clone()),
        })
        .unwrap(),
      )
      .dispatch();

    assert_eq!(response.status(), Status::Unauthorized);
  });
}

#[test]
fn test_delete_card() {
  run_test(|client: Client, db: &PgConnection| {
    // Create a board, rank & card
    let (board, participant_id) = create_board(
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
    let card = create_card(
      db,
      NewCard {
        id: None,
        rank_id: &rank.id,
        name: "test card",
        description: "card description",
        participant_id: &participant_id,
      },
    );

    // Delete the card
    let response = client
      .delete(format!(
        "/boards/{}/ranks/{}/cards/{}",
        board.id, rank.id, card.id
      ))
      .dispatch();

    assert_eq!(response.status(), Status::Ok);

    // Ensure the database doesn't contain any cards
    let db_cards = get_rank_cards(db, &rank.id, "").unwrap();

    assert_eq!(db_cards.len(), 0);
  });
}
