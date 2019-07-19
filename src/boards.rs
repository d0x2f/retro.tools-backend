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
