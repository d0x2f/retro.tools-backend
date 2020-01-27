#[cfg(test)]
mod tests;

use super::guards::BoardOwner;
use super::guards::DatabaseConnection;
use super::guards::ParticipantId;
use super::guards::RankInBoard;
use super::models::*;
use super::persistence;
use log::error;
use rocket::http::Status;
use rocket_contrib::json::{Json, JsonValue};

#[post("/boards/<board_id>/ranks", data = "<post_rank>")]
pub fn post_rank(
  _participant_id: ParticipantId,
  _board_owner: BoardOwner,
  postgres: DatabaseConnection,
  board_id: String,
  post_rank: Json<PostRank>,
) -> Result<JsonValue, Status> {
  let new_rank = NewRank {
    id: None,
    name: post_rank.name,
    board_id: &board_id,
    data: post_rank.data.clone(),
  };

  map_err!(
    persistence::put_rank(&postgres, new_rank)
      .map(|rank| json!(rank))
  )
}

#[get("/boards/<board_id>/ranks")]
pub fn get_ranks(
  _participant_id: ParticipantId,
  postgres: DatabaseConnection,
  board_id: String,
) -> Result<JsonValue, Status> {
  map_err!(
    persistence::get_ranks(&postgres, &board_id)
      .map(|ranks| json!(ranks))
  )
}

#[get("/boards/<_board_id>/ranks/<rank_id>")]
pub fn get_rank(
  _participant_id: ParticipantId,
  _rank_in_board: RankInBoard,
  postgres: DatabaseConnection,
  _board_id: String,
  rank_id: String,
) -> Result<JsonValue, Status> {
  let rank = map_err!(persistence::get_rank(&postgres, &rank_id))?;
  Ok(json!(rank))
}

#[patch("/boards/<_board_id>/ranks/<rank_id>", data = "<update_rank>")]
pub fn patch_rank(
  _participant_id: ParticipantId,
  _board_owner: BoardOwner,
  _rank_in_board: RankInBoard,
  postgres: DatabaseConnection,
  _board_id: String,
  rank_id: String,
  update_rank: Json<UpdateRank>,
) -> Result<JsonValue, Status> {
  map_err!(
    persistence::patch_rank(&postgres, &rank_id, &update_rank)
      .map(|board| json!(board))
  )
}

#[delete("/boards/<_board_id>/ranks/<rank_id>")]
pub fn delete_rank(
  _participant_id: ParticipantId,
  _board_owner: BoardOwner,
  _rank_in_board: RankInBoard,
  postgres: DatabaseConnection,
  _board_id: String,
  rank_id: String,
) -> Result<(), Status> {
  map_err!(
    persistence::delete_rank(&postgres, &rank_id)
      .map(|_| ())
  )
}
