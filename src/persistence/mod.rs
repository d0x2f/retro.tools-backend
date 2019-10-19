use super::models::*;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel::result::Error;
use diesel::dsl::count;

pub fn participant_owns_board(
  postgres: &PgConnection,
  participant_id: &str,
  board_id: &str,
) -> Result<bool, Error> {
  use super::schema::participant_board;

  let participant: ParticipantBoard = participant_board::dsl::participant_board
    .find((participant_id, board_id))
    .first(postgres)?;

  Ok(participant.owner)
}

pub fn rank_in_board(
  postgres: &PgConnection,
  rank_id: &str,
  board_id: &str,
) -> Result<bool, Error> {
  Ok(
    get_rank(&postgres, &rank_id)?
      .ok_or(Error::NotFound)?
      .board_id
      == board_id,
  )
}

pub fn card_in_rank(postgres: &PgConnection, card_id: &str, rank_id: &str) -> Result<bool, Error> {
  Ok(
    get_card(&postgres, &card_id)?
      .ok_or(Error::NotFound)?
      .rank_id
      == rank_id,
  )
}

pub fn cards_open(postgres: &PgConnection, board_id: &str) -> Result<bool, Error> {
  Ok(
    get_board(&postgres, &board_id)?
      .ok_or(Error::NotFound)?
      .cards_open,
  )
}

pub fn voting_open(postgres: &PgConnection, board_id: &str) -> Result<bool, Error> {
  Ok(
    get_board(&postgres, &board_id)?
      .ok_or(Error::NotFound)?
      .voting_open,
  )
}

pub fn put_board(
  postgres: &PgConnection,
  new_board: NewBoard,
  participant_id: &str,
) -> Result<Board, Error> {
  use super::schema::board;

  let board: Board = diesel::insert_into(board::table)
    .values(new_board)
    .get_result(postgres)?;

  let new_participant = NewParticipantBoard {
    participant_id: Some(participant_id),
    owner: true,
    board_id: &board.id,
  };

  put_participant_board(postgres, &new_participant)?;

  Ok(board)
}

pub fn get_boards(postgres: &PgConnection, participant_id: &str) -> Result<Vec<Board>, Error> {
  use super::schema::board::dsl::*;

  super::schema::participant_board::dsl::participant_board
    .inner_join(board)
    .filter(super::schema::participant_board::dsl::participant_id.eq(participant_id))
    .select((id, name, max_votes, voting_open, cards_open))
    .load(postgres)
}

pub fn get_board(postgres: &PgConnection, board_id: &str) -> Result<Option<Board>, Error> {
  use super::schema::board::dsl::*;
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

pub fn create_participant(postgres: &PgConnection) -> Result<Participant, Error> {
  use super::schema::participant::dsl::*;
  diesel::insert_into(participant)
    .default_values()
    .get_result(postgres)
}

pub fn put_participant_board(
  postgres: &PgConnection,
  new_participant: &NewParticipantBoard,
) -> Result<usize, Error> {
  use super::schema::participant_board::dsl::*;
  diesel::insert_into(participant_board)
    .values(new_participant)
    .on_conflict((participant_id, board_id))
    .do_nothing()
    .execute(postgres)
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

pub fn put_card(postgres: &PgConnection, new_card: NewCard) -> Result<Card, Error> {
  use super::schema::card;

  diesel::insert_into(card::table)
    .values(new_card)
    .get_result(postgres)
}

pub fn get_board_cards(postgres: &PgConnection, board_id: &str) -> Result<Vec<Card>, Error> {
  use super::schema::card;
  use super::schema::board;

  super::schema::rank::dsl::rank
    .inner_join(card::dsl::card)
    .inner_join(board::dsl::board)
    .filter(board::dsl::id.eq(board_id))
    .select((card::dsl::id, card::dsl::rank_id, card::dsl::name, card::dsl::description))
    .load(postgres)
}

pub fn get_rank_cards(postgres: &PgConnection, rank_id: &str) -> Result<Vec<Card>, Error> {
  use super::schema::card::dsl;

  super::schema::rank::dsl::rank
    .inner_join(dsl::card)
    .filter(super::schema::rank::dsl::id.eq(rank_id))
    .select((dsl::id, dsl::rank_id, dsl::name, dsl::description))
    .load(postgres)
}

pub fn get_card(postgres: &PgConnection, card_id: &str) -> Result<Option<Card>, Error> {
  use super::schema::card::dsl::*;

  let result = card.find(card_id).first(postgres);
  match result {
    Ok(r) => Ok(Some(r)),
    Err(Error::NotFound) => Ok(None),
    Err(e) => Err(e),
  }
}

pub fn patch_card(
  postgres: &PgConnection,
  card_id: &str,
  update_card: &UpdateCard,
) -> Result<Card, Error> {
  use super::schema::card::dsl::*;

  diesel::update(card.find(card_id))
    .set(update_card)
    .get_result(postgres)
}

pub fn delete_card(postgres: &PgConnection, card_id: &str) -> Result<usize, Error> {
  use super::schema::card::dsl::*;

  diesel::delete(card.find(card_id)).execute(postgres)
}

pub fn put_vote(
  postgres: &PgConnection,
  new_vote: NewVote,
) -> Result<Vote, Error> {
  use super::schema::vote::dsl::*;

  diesel::insert_into(vote)
    .values(new_vote)
    .on_conflict((card_id, participant_id))
    .do_update()
    .set(count.eq(count)) // Hack to get the vote back in the result
    .get_result(postgres)
}

pub fn get_vote(
  postgres: &PgConnection,
  card_id: &str,
  participant_id: &str,
) -> Result<Option<Vote>, Error> {
  use super::schema::vote::dsl;

  let result = dsl::vote.find((card_id, participant_id)).first(postgres);
  match result {
    Ok(r) => Ok(Some(r)),
    Err(Error::NotFound) => Ok(None),
    Err(e) => Err(e),
  }
}

pub fn patch_vote(postgres: &PgConnection, update_vote: &UpdateVote) -> Result<Vote, Error> {
  use super::schema::vote::dsl::*;

  diesel::update(vote.find((update_vote.card_id, update_vote.participant_id)))
    .set(count.eq(update_vote.count))
    .get_result(postgres)
}

pub fn get_votes(
  postgres: &PgConnection,
  card_id: &str
) -> Result<i64, Error> {
  use super::schema::vote::dsl;

  let result = dsl::vote.select(count(dsl::participant_id)).filter(dsl::card_id.eq(card_id)).first(postgres);
  match result {
    Ok(r) => Ok(r),
    Err(Error::NotFound) => Ok(0),
    Err(e) => Err(e),
  }
}