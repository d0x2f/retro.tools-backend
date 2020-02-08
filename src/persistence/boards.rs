use super::super::metrics::*;
use super::super::models::*;
use super::super::schema;
use super::Error;
use diesel::pg::PgConnection;
use diesel::prelude::*;

pub fn put_board(
  postgres: &PgConnection,
  new_board: NewBoard,
  participant_id: &str,
) -> Result<Board, Error> {
  use schema::board;

  let inserted_id: String = diesel::insert_into(board::table)
    .values(new_board)
    .returning(board::dsl::id)
    .get_result(postgres)?;

  BOARDS_COUNT.inc();

  let new_participant = NewParticipantBoard {
    participant_id,
    owner: true,
    board_id: &inserted_id,
  };

  super::participants::put_participant_board(postgres, &new_participant)?;

  let board = get_board(postgres, &inserted_id, participant_id)?;
  match board {
    Some(b) => Ok(b),
    None => Err(Error::NotFound),
  }
}

pub fn get_boards(postgres: &PgConnection, participant_id: &str) -> Result<Vec<Board>, Error> {
  use schema::board::dsl::*;

  schema::participant_board::dsl::participant_board
    .inner_join(board)
    .filter(schema::participant_board::dsl::participant_id.eq(participant_id))
    .select((
      id,
      name,
      max_votes,
      voting_open,
      cards_open,
      created_at,
      schema::participant_board::dsl::owner,
    ))
    .load(postgres)
    .map_err(Into::into)
}

pub fn get_board(
  postgres: &PgConnection,
  board_id: &str,
  participant_id: &str,
) -> Result<Option<Board>, Error> {
  use schema::board::dsl::*;

  schema::participant_board::dsl::participant_board
    .inner_join(board)
    .filter(schema::participant_board::dsl::participant_id.eq(participant_id))
    .filter(id.eq(board_id))
    .select((
      id,
      name,
      max_votes,
      voting_open,
      cards_open,
      created_at,
      schema::participant_board::dsl::owner,
    ))
    .first(postgres)
    .optional()
    .map_err(Into::into)
}

pub fn patch_board(
  postgres: &PgConnection,
  board_id: &str,
  participant_id: &str,
  update_board: &UpdateBoard,
) -> Result<Board, Error> {
  use schema::board::dsl::*;

  let board_id: String = diesel::update(board.find(board_id))
    .set(update_board)
    .returning(id)
    .get_result(postgres)?;

  let result = get_board(postgres, &board_id, participant_id);
  match result {
    Ok(Some(b)) => Ok(b),
    Ok(None) => Err(Error::NotFound),
    Err(e) => Err(e),
  }
}

pub fn delete_board(postgres: &PgConnection, board_id: &str) -> Result<usize, Error> {
  use schema::board::dsl::*;

  diesel::delete(board.find(board_id))
    .execute(postgres)
    .map_err(Into::into)
}

pub fn cards_open(
  postgres: &PgConnection,
  board_id: &str,
  participant_id: &str,
) -> Result<bool, Error> {
  Ok(
    get_board(&postgres, &board_id, &participant_id)?
      .ok_or(Error::NotFound)?
      .cards_open,
  )
}

pub fn voting_open(
  postgres: &PgConnection,
  board_id: &str,
  participant_id: &str,
) -> Result<bool, Error> {
  Ok(
    get_board(&postgres, &board_id, participant_id)?
      .ok_or(Error::NotFound)?
      .voting_open,
  )
}
