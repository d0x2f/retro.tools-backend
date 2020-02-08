#[cfg(test)]
mod tests;

use super::guards::CardInRank;
use super::guards::DatabaseConnection;
use super::guards::ParticipantId;
use super::guards::RankInBoard;
use super::models::*;
use super::persistence;
use super::persistence::Error;
use log::error;
use rocket::http::Status;
use rocket_contrib::json::JsonValue;
use std::cmp::{max, min};

#[post("/boards/<board_id>/ranks/<_rank_id>/cards/<card_id>/vote")]
#[allow(clippy::too_many_arguments)]
pub fn post_vote(
  participant_id: ParticipantId,
  _rank_in_board: RankInBoard,
  _card_in_rank: CardInRank,
  postgres: DatabaseConnection,
  board_id: String,
  _rank_id: String,
  card_id: String,
) -> Result<JsonValue, Status> {
  let board = match persistence::boards::get_board(&postgres, &board_id, &participant_id.0) {
    Ok(Some(b)) => Ok(b),
    Ok(None) => Err(Status::NotFound),
    Err(Error::NotFound) => Err(Status::NotFound),
    Err(error) => {
      error!("Post vote: {}", error.to_string());
      Err(Status::InternalServerError)
    }
  }?;

  // Check that voting is open
  if !board.voting_open {
    return Err(Status::Forbidden);
  }

  // Get or create a new vote
  let new_vote = NewVote {
    card_id: &card_id,
    participant_id: &participant_id.0,
    count: Some(0),
  };

  let vote = match map_err!(persistence::votes::put_vote(&postgres, new_vote))? {
    None => return Err(Status::NotFound),
    Some(v) => v,
  };

  // If max votes is not yet exceeded, increment the vote.
  // If the vote is greater than the max votes, it was probably
  // made when the limit was previously higher and so should stay.
  if vote.count < board.max_votes {
    // Increment the vote
    let update_vote = UpdateVote {
      participant_id: &vote.participant_id,
      card_id: &vote.card_id,
      count: min(board.max_votes, vote.count + 1),
    };

    map_err!(persistence::votes::patch_vote(&postgres, &update_vote))?;
  }

  map_err!(
    persistence::cards::get_card(&postgres, &vote.card_id, &participant_id.0).map(|v| json!(v))
  )
}

#[delete("/boards/<board_id>/ranks/<_rank_id>/cards/<card_id>/vote")]
#[allow(clippy::too_many_arguments)]
pub fn delete_vote(
  participant_id: ParticipantId,
  _rank_in_board: RankInBoard,
  _card_in_rank: CardInRank,
  postgres: DatabaseConnection,
  board_id: String,
  _rank_id: String,
  card_id: String,
) -> Result<JsonValue, Status> {
  // Check that voting is open for the board
  let voting_open = match persistence::boards::voting_open(&postgres, &board_id, &participant_id.0)
  {
    Ok(b) => Ok(b),
    Err(Error::NotFound) => Err(Status::NotFound),
    Err(error) => {
      error!("{}", error.to_string());
      Err(Status::InternalServerError)
    }
  }?;

  if !voting_open {
    return Err(Status::Forbidden);
  }

  let vote = match persistence::votes::get_vote(&postgres, &card_id, &participant_id.0) {
    Ok(Some(v)) => Ok(v),
    Ok(None) => Err(Status::NotFound),
    Err(Error::NotFound) => Err(Status::NotFound),
    Err(error) => {
      error!("{}", error.to_string());
      Err(Status::InternalServerError)
    }
  }?;

  let update_vote = UpdateVote {
    participant_id: &vote.participant_id,
    card_id: &vote.card_id,
    count: max(0, vote.count - 1),
  };

  map_err!(persistence::votes::patch_vote(&postgres, &update_vote))?;

  map_err!(
    persistence::cards::get_card(&postgres, &vote.card_id, &participant_id.0).map(|v| json!(v))
  )
}
