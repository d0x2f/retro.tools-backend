use super::super::metrics::*;
use super::super::models::*;
use super::super::schema;
use super::Error;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel::result::Error as DieselError;

pub fn put_rank(postgres: &PgConnection, new_rank: NewRank) -> Result<Rank, Error> {
  use schema::rank;

  let result = map_diesel_err!(diesel::insert_into(rank::table)
    .values(new_rank)
    .get_result(postgres));

  if result.is_ok() {
    RANK_COUNT.inc();
  }
  result
}

pub fn get_ranks(postgres: &PgConnection, board_id: &str) -> Result<Vec<Rank>, Error> {
  use schema::rank::dsl;

  map_diesel_err!(schema::board::dsl::board
    .inner_join(dsl::rank)
    .filter(schema::board::dsl::id.eq(board_id))
    .select((dsl::id, dsl::board_id, dsl::name, dsl::data))
    .load(postgres))
}

pub fn get_rank(postgres: &PgConnection, rank_id: &str) -> Result<Option<Rank>, Error> {
  use schema::rank::dsl::*;

  map_diesel_err!(rank.find(rank_id).first(postgres).optional())
}

pub fn patch_rank(
  postgres: &PgConnection,
  rank_id: &str,
  update_rank: &UpdateRank,
) -> Result<Rank, Error> {
  use schema::rank::dsl::*;

  map_diesel_err!(diesel::update(rank.find(rank_id))
    .set(update_rank)
    .get_result(postgres))
}

pub fn delete_rank(postgres: &PgConnection, rank_id: &str) -> Result<usize, Error> {
  use schema::rank::dsl::*;

  map_diesel_err!(diesel::delete(rank.find(rank_id)).execute(postgres))
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
