#[cfg(test)]
mod tests;

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
  map_err!(
    persistence::put_board(&postgres, new_board.into_inner(), &participant_id.0)
      .map(|board| json!(board))
  )
}

#[get("/boards")]
pub fn get_boards(
  participant_id: ParticipantId,
  postgres: DatabaseConnection,
) -> Result<JsonValue, Status> {
  map_err!(
    persistence::get_boards(&postgres, &participant_id.0)
      .map(|boards| json!(boards))
  )
}

#[get("/boards/<board_id>")]
pub fn get_board(
  participant_id: ParticipantId,
  postgres: DatabaseConnection,
  board_id: String,
) -> Result<JsonValue, Status> {
  let new_participant = NewParticipantBoard {
    participant_id: Some(&participant_id.0),
    owner: false,
    board_id: &board_id,
  };

  let participant_result = map_err!(
    persistence::put_participant_board(&postgres, &new_participant)
  );

  if participant_result.is_err() {
    return Err(Status::NotFound);
  }

  let result = map_err!(persistence::get_board(&postgres, &board_id, &participant_id.0))?;

  match result {
    Some(board) => Ok(json!(board)),
    _ => Err(Status::NotFound),
  }
}

#[patch("/boards/<id>", data = "<update_board>")]
pub fn patch_board(
  participant_id: ParticipantId,
  _board_owner: BoardOwner,
  postgres: DatabaseConnection,
  id: String,
  update_board: Json<UpdateBoard>,
) -> Result<JsonValue, Status> {
  map_err!(
    persistence::patch_board(&postgres, &id, &participant_id.0, &update_board)
      .map(|board| json!(board))
  )
}

#[delete("/boards/<id>")]
pub fn delete_board(
  _participant_id: ParticipantId,
  _board_owner: BoardOwner,
  postgres: DatabaseConnection,
  id: String,
) -> Result<(), Status> {
  map_err!(
    persistence::delete_board(&postgres, &id)
      .map(|_| ())
  )
}

use std::io::Cursor;
use rocket::request::Request;
use rocket::response::{self, Response, Responder};
use rocket::http::ContentType;

pub struct CSVResponder {
  filename: String,
  csv: String
}

impl<'r> Responder<'r> for CSVResponder {
  fn respond_to(self, _: &Request) -> response::Result<'r> {
      Response::build()
          .sized_body(Cursor::new(self.csv))
          .raw_header("Content-Disposition", format!("attachment; filename=\"{}\"", self.filename))
          .header(ContentType::new("text", "csv"))
          .ok()
  }
}

#[get("/boards/<board_id>/csv")]
pub fn export_csv(
  participant_id: ParticipantId,
  postgres: DatabaseConnection,
  board_id: String,
) -> Result<CSVResponder, Status> {
  let new_participant = NewParticipantBoard {
    participant_id: Some(&participant_id.0),
    owner: false,
    board_id: &board_id,
  };

  let participant_result = map_err!(
    persistence::put_participant_board(&postgres, &new_participant)
  );

  if participant_result.is_err() {
    return Err(Status::NotFound);
  }

  let board = match map_err!(persistence::get_board(&postgres, &board_id, &participant_id.0))? {
    Some(b) => b,
    _ => return Err(Status::NotFound),
  };

  let cards = map_err!(persistence::get_board_cards(&postgres, &board_id, &participant_id.0))?;

  let mut writer = csv::Writer::from_writer(vec![]);
  map_err!(writer.write_record(&["text", "votes"]))?;
  for card in cards {
    map_err!(writer.write_record(&[card.description, card.votes.to_string()]))?;
  }
  let data = map_err!(String::from_utf8(map_err!(writer.into_inner())?))?;

  Ok(
    CSVResponder {
      filename: format!("{}.csv", board.name),
      csv: data
    }
  )
}
