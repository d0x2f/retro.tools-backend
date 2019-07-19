use super::guards::BoardOwner;
use super::guards::CardInRank;
use super::guards::DatabaseConnection;
use super::guards::ParticipantId;
use super::guards::RankInBoard;
use super::models::*;
use super::persistence;
use diesel::result::Error;
use log::error;
use rocket::http::Status;
use rocket_contrib::json::JsonValue;

#[post("/boards/<board_id>/ranks/<_rank_id>/cards/<card_id>/vote")]
pub fn post_vote(
    participant_id: ParticipantId,
    _board_owner: BoardOwner,
    _rank_in_board: RankInBoard,
    _card_in_rank: CardInRank,
    postgres: DatabaseConnection,
    board_id: String,
    _rank_id: String,
    card_id: String,
) -> Result<JsonValue, Status> {
    // Check that voting is open for the board
    let voting_open = match persistence::voting_open(&postgres, &board_id) {
        Ok(b) => Ok(b),
        Err(Error::NotFound) => Err(Status::NotFound),
        Err(_) => Err(Status::InternalServerError),
    }?;

    if !voting_open {
        return Err(Status::Forbidden);
    }

    // TODO: check vote limit on board

    let new_vote = NewVote {
        card_id: &card_id,
        participant_id: &participant_id.0,
    };

    persistence::put_vote(&postgres, &board_id, new_vote)
        .map(|vote| json!(vote))
        .map_err(|error| {
            error!("{}", error.to_string());
            Status::InternalServerError
        })
}
