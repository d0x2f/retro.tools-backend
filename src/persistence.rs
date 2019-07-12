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
    use super::schema::participant;

    let board: Board = diesel::insert_into(board::table)
        .values(new_board)
        .get_result(postgres)?;

    let new_participant = NewParticipant {
        id: Some(participant_id),
        owner: true,
        board_id: &board.id,
    };

    diesel::insert_into(participant::table)
        .values(new_participant)
        .execute(postgres)?;

    Ok(board)
}

pub fn get_boards(
    postgres: &PgConnection,
    participant_id: &str,
) -> Result<Vec<Board>, Error> {
    use super::schema::board::dsl::*;

    println!("{}", participant_id);

    super::schema::participant::dsl::participant
        .inner_join(board)
        .filter(super::schema::participant::dsl::id.eq(participant_id))
        .select((id, name, max_votes, voting_open, cards_open))
        .load(postgres)
}

pub fn get_board(postgres: &PgConnection, board_id: &str) -> Result<Vec<Board>, Error> {
    use super::schema::board::dsl::*;

    board.filter(id.eq(board_id)).limit(1).load(postgres)
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
