use super::models::*;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel::result::Error;

pub fn put_board(
    postgres: &PgConnection,
    new_board: NewBoard,
    participant_id: &str,
) -> Result<Board, Error> {
    use super::schema::board;

    let board: Board = diesel::insert_into(board::table)
        .values(new_board)
        .get_result(postgres)?;

    let new_participant = NewParticipant {
        id: Some(participant_id),
        owner: true,
        board_id: &board.id,
    };

    put_participant(postgres, &new_participant)?;

    Ok(board)
}

pub fn get_boards(postgres: &PgConnection, participant_id: &str) -> Result<Vec<Board>, Error> {
    use super::schema::board::dsl::*;

    super::schema::participant::dsl::participant
        .inner_join(board)
        .filter(super::schema::participant::dsl::id.eq(participant_id))
        .select((id, name, max_votes, voting_open, cards_open))
        .load(postgres)
}

pub fn get_board(
    postgres: &PgConnection,
    board_id: &str,
    participant_id: &str,
) -> Result<Option<Board>, Error> {
    use super::schema::board::dsl::*;

    let new_participant = NewParticipant {
        id: Some(participant_id),
        owner: false,
        board_id: board_id,
    };

    put_participant(postgres, &new_participant)?;
    let result = board.find(board_id).first(postgres);
    match result {
        Ok(r) => Ok(Some(r)),
        Err(Error::NotFound) => Ok(None),
        Err(e) => Err(e),
    }
}

pub fn patch_board(
    postgres: &PgConnection,
    board_id: &str,
    update_board: &UpdateBoard,
) -> Result<Board, Error> {
    use super::schema::board::dsl::*;

    diesel::update(board.find(board_id))
        .set(update_board)
        .get_result(postgres)
}

pub fn delete_board(postgres: &PgConnection, board_id: &str) -> Result<usize, Error> {
    use super::schema::board::dsl::*;

    diesel::delete(board.find(board_id)).execute(postgres)
}

pub fn put_participant(
    postgres: &PgConnection,
    new_participant: &NewParticipant,
) -> Result<usize, Error> {
    use super::schema::participant::dsl::*;

    diesel::insert_into(participant)
        .values(new_participant)
        .on_conflict((id, board_id))
        .do_nothing()
        .execute(postgres)
}

pub fn participant_owns_board(
    postgres: &PgConnection,
    participant_id: &str,
    board_id: &str,
) -> Result<bool, Error> {
    use super::schema::participant;

    let found_participants: Vec<Participant> = participant::dsl::participant
        .find((board_id, participant_id)) // Not sure why this needs to be backwards
        .load(postgres)?;

    if found_participants.is_empty() {
        return Ok(false);
    }

    Ok(found_participants[0].owner)
}

pub fn rank_in_board(
    postgres: &PgConnection,
    rank_id: &str,
    board_id: &str,
) -> Result<bool, Error> {
    let result = get_rank(postgres, rank_id);
    match result {
        Ok(r) => {
            if let Some(rank) = r {
                return Ok(rank.board_id == board_id);
            }
            Ok(false)
        }
        Err(Error::NotFound) => Ok(false),
        Err(e) => Err(e),
    }
}

pub fn put_rank(postgres: &PgConnection, new_rank: NewRank) -> Result<Rank, Error> {
    use super::schema::rank;

    diesel::insert_into(rank::table)
        .values(new_rank)
        .get_result(postgres)
}

pub fn get_ranks(postgres: &PgConnection, board_id: &str) -> Result<Vec<Rank>, Error> {
    use super::schema::rank::dsl;

    super::schema::board::dsl::board
        .inner_join(dsl::rank)
        .filter(super::schema::board::dsl::id.eq(board_id))
        .select((dsl::id, dsl::board_id, dsl::name))
        .load(postgres)
}

pub fn get_rank(postgres: &PgConnection, rank_id: &str) -> Result<Option<Rank>, Error> {
    use super::schema::rank::dsl::*;

    let result = rank.find(rank_id).first(postgres);
    match result {
        Ok(r) => Ok(Some(r)),
        Err(Error::NotFound) => Ok(None),
        Err(e) => Err(e),
    }
}

pub fn patch_rank(
    postgres: &PgConnection,
    rank_id: &str,
    update_rank: &UpdateRank,
) -> Result<Rank, Error> {
    use super::schema::rank::dsl::*;

    diesel::update(rank.find(rank_id))
        .set(update_rank)
        .get_result(postgres)
}

pub fn delete_rank(postgres: &PgConnection, rank_id: &str) -> Result<usize, Error> {
    use super::schema::rank::dsl::*;

    diesel::delete(rank.find(rank_id)).execute(postgres)
}
