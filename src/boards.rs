use super::models::*;
use super::persistence;
use super::DatabaseConnection;
use super::ParticipantId;
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

#[get("/boards/<id>")]
pub fn get_board(
    participant_id: ParticipantId,
    postgres: DatabaseConnection,
    id: String,
) -> Result<JsonValue, Status> {
    let result = persistence::get_board(&postgres, &id, &participant_id.0).map_err(|error| {
        error!("{}", error.to_string());
        Status::InternalServerError
    })?;
    if let Some(board) = result {
        return Ok(json!(board));
    }
    Err(Status::NotFound)
}

#[patch("/boards/<id>", data = "<update_board>")]
pub fn patch_board(
    participant_id: ParticipantId,
    postgres: DatabaseConnection,
    id: String,
    update_board: Json<UpdateBoard>,
) -> Result<JsonValue, Status> {
    let owner = persistence::participant_owns_board(&postgres, &id, &participant_id.0).map_err(
        |error| {
            error!("{}", error.to_string());
            Status::InternalServerError
        },
    )?;

    if !owner {
        return Err(Status::Unauthorized);
    }

    persistence::patch_board(&postgres, &id, &update_board)
        .map(|board| json!(board))
        .map_err(|error| {
            error!("{}", error.to_string());
            Status::InternalServerError
        })
}

#[delete("/boards/<id>")]
pub fn delete_board(
    participant_id: ParticipantId,
    postgres: DatabaseConnection,
    id: String,
) -> Result<(), Status> {
    let owner = persistence::participant_owns_board(&postgres, &id, &participant_id.0).map_err(
        |error| {
            error!("{}", error.to_string());
            Status::InternalServerError
        },
    )?;

    if !owner {
        return Err(Status::Unauthorized);
    }

    persistence::delete_board(&postgres, &id)
        .map(|_| ())
        .map_err(|error| {
            error!("{}", error.to_string());
            Status::InternalServerError
        })
}
