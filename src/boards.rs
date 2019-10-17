use super::guards::BoardOwner;
use super::guards::DatabaseConnection;
use super::guards::ParticipantId;
use super::models::*;
use super::persistence;
use log::error;
use rocket::http::Status;
use rocket_contrib::json::{Json, JsonValue};

#[post("/boards", data = "<new_board>")]
pub fn post_board(
  participant_id: ParticipantId,
  postgres: DatabaseConnection,
  new_board: Json<NewBoard>,
) -> Result<JsonValue, Status> {
  persistence::put_board(&postgres, new_board.into_inner(), &participant_id.0)
    .map(|board| json!(board))
    .map_err(|error| {
      error!("{}", error.to_string());
      Status::InternalServerError
    })
}

#[get("/boards")]
pub fn get_boards(
  participant_id: ParticipantId,
  postgres: DatabaseConnection,
) -> Result<JsonValue, Status> {
  persistence::get_boards(&postgres, &participant_id.0)
    .map(|boards| json!(boards))
    .map_err(|error| {
      error!("{}", error.to_string());
      Status::InternalServerError
    })
}

#[get("/boards/<board_id>")]
pub fn get_board(
  participant_id: ParticipantId,
  postgres: DatabaseConnection,
  board_id: String,
) -> Result<JsonValue, Status> {
  let result = persistence::get_board(&postgres, &board_id).map_err(|error| {
    error!("{}", error.to_string());
    Status::InternalServerError
  })?;
  if let Some(board) = result {
    let new_participant = NewParticipantBoard {
      participant_id: Some(&participant_id.0),
      owner: false,
      board_id: &board_id,
    };

    persistence::put_participant_board(&postgres, &new_participant).map_err(|error| {
      error!("{}", error.to_string());
      Status::InternalServerError
    })?;
    return Ok(json!(board));
  }
  Err(Status::NotFound)
}

#[patch("/boards/<id>", data = "<update_board>")]
pub fn patch_board(
  _participant_id: ParticipantId,
  _board_owner: BoardOwner,
  postgres: DatabaseConnection,
  id: String,
  update_board: Json<UpdateBoard>,
) -> Result<JsonValue, Status> {
  persistence::patch_board(&postgres, &id, &update_board)
    .map(|board| json!(board))
    .map_err(|error| {
      error!("{}", error.to_string());
      Status::InternalServerError
    })
}

#[delete("/boards/<id>")]
pub fn delete_board(
  _participant_id: ParticipantId,
  _board_owner: BoardOwner,
  postgres: DatabaseConnection,
  id: String,
) -> Result<(), Status> {
  persistence::delete_board(&postgres, &id)
    .map(|_| ())
    .map_err(|error| {
      error!("{}", error.to_string());
      Status::InternalServerError
    })
}

#[cfg(test)]
mod tests {

  use super::super::models::Board;
  use super::super::run_test;
  use rocket::http::ContentType;
  use rocket::http::Status;
  use rocket::local::Client;

  #[test]
  fn test_post_board() {
    run_test(|client: Client| {
      let mut response = client
        .post("/boards")
        .header(ContentType::JSON)
        .body(
          r#"{ "name": "test board", "max_votes": 37, "voting_open": false, "cards_open": true }"#,
        )
        .dispatch();

      let response_board: Board =
        serde_json::from_str(response.body_string().unwrap().as_str()).unwrap();

      assert_eq!(response.status(), Status::Ok);
      assert_eq!(response_board.name, "test board");
      assert_eq!(response_board.max_votes, 37);
      assert_eq!(response_board.voting_open, false);
      assert_eq!(response_board.cards_open, true);
    });
  }

  #[test]
  fn test_get_boards_empty() {
    run_test(|client: Client| {
      let request = client.get("/boards");
      let mut response = request.dispatch();

      assert_eq!(response.status(), Status::Ok);
      assert_eq!(response.body_string(), Some("[]".into()));
    });
  }

  #[test]
  fn test_get_boards() {
    run_test(|client: Client| {
      let create_response = client
        .post("/boards")
        .header(ContentType::JSON)
        .body(r#"{ "name": "test", "max_votes": 47, "voting_open": true, "cards_open": false }"#)
        .dispatch();

      assert_eq!(create_response.status(), Status::Ok);

      // Get the boards
      let request = client.get("/boards");
      let mut response = request.dispatch();
      let response_boards: Vec<Board> =
        serde_json::from_str(response.body_string().unwrap().as_str()).unwrap();

      assert_eq!(response.status(), Status::Ok);
      assert_eq!(response_boards.len(), 1);
      assert_eq!(response_boards[0].name, "test");
      assert_eq!(response_boards[0].max_votes, 47);
      assert_eq!(response_boards[0].voting_open, true);
      assert_eq!(response_boards[0].cards_open, false);
    });
  }

}
