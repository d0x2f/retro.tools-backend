use super::super::metrics::*;
use super::super::models::*;
use super::super::schema;
use super::Error;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel::result::Error as DieselError;

pub fn participant_owns_board(
  postgres: &PgConnection,
  participant_id: &str,
  board_id: &str,
) -> Result<bool, Error> {
  use schema::participant_board;

  let participant: Option<ParticipantBoard> =
    map_diesel_err!(participant_board::dsl::participant_board
      .find((participant_id, board_id))
      .first(postgres)
      .optional())?;

  Ok(match participant {
    Some(p) => p.owner,
    _ => false,
  })
}

pub fn participant_owns_card(
  postgres: &PgConnection,
  participant_id: &str,
  board_id: &str,
  card_id: &str,
) -> Result<bool, Error> {
  use schema::card::dsl;

  if participant_owns_board(postgres, participant_id, board_id)? {
    return Ok(true);
  }

  let card_owner: Option<String> = map_diesel_err!(dsl::card
    .filter(dsl::id.eq(card_id))
    .select(dsl::participant_id)
    .first(postgres)
    .optional())?;

  Ok(match card_owner {
    Some(owner) => owner == participant_id,
    _ => false,
  })
}

pub fn create_participant(postgres: &PgConnection) -> Result<Participant, Error> {
  use schema::participant::dsl::*;
  let result = map_diesel_err!(diesel::insert_into(participant)
    .default_values()
    .get_result(postgres));

  if result.is_ok() {
    PARTICIPANT_COUNT.inc();
  }
  result
}

pub fn get_participant(
  postgres: &PgConnection,
  participant_id: &str,
) -> Result<Option<Participant>, Error> {
  map_diesel_err!(schema::participant::dsl::participant
    .find(participant_id)
    .first(postgres)
    .optional())
}

pub fn put_participant_board(
  postgres: &PgConnection,
  new_participant: &NewParticipantBoard,
) -> Result<usize, Error> {
  use schema::participant_board::dsl::*;

  // Ensure the board exists
  if !map_diesel_err!(board_exists(postgres, new_participant.board_id))? {
    return Err(Error::NotFound);
  }

  let result = map_diesel_err!(diesel::insert_into(participant_board)
    .values(new_participant)
    .on_conflict((participant_id, board_id))
    .do_nothing()
    .execute(postgres));

  match result {
    Ok(0) => (),
    Ok(_) => BOARD_PARTICIPANT_COUNT.inc(),
    Err(_) => (),
  };
  result
}

fn board_exists(postgres: &PgConnection, board_id: &str) -> Result<bool, DieselError> {
  use diesel::expression::dsl::exists;
  use diesel::select;
  use schema::board::dsl::*;

  select(exists(board.filter(id.eq(board_id)))).get_result(postgres)
}
