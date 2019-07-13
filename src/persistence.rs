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
) -> Result<Vec<Board>, Error> {
    use super::schema::board::dsl::*;

    let new_participant = NewParticipant {
        id: Some(participant_id),
        owner: false,
        board_id: board_id,
    };

    put_participant(postgres, &new_participant)?;
    board.find(board_id).load(postgres)
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

pub fn does_participant_own_board(
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
