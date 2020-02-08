use super::super::metrics::*;
use super::super::models::*;
use super::super::schema;
use super::Error;
use diesel::dsl::sql;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel::result::Error as DieselError;
use diesel::sql_types::Text;

const VOTES_SQL: &str =
  "(select coalesce(sum(count), 0) from vote where vote.card_id = card.id) as votes";
const VOTED_SQL: &str =
  "select coalesce(sum(count), 0) > 0 from vote where vote.card_id = card.id and vote.participant_id =";

pub fn put_card(
  postgres: &PgConnection,
  new_card: NewCard,
  participant_id: &str,
) -> Result<Card, Error> {
  use schema::card::dsl;

  let inserted_id: String = map_diesel_err!(diesel::insert_into(dsl::card)
    .values(new_card)
    .returning(dsl::id)
    .get_result(postgres))?;

  CARD_COUNT.inc();

  let card = get_card(postgres, &inserted_id, participant_id);
  match card {
    Ok(Some(c)) => Ok(c),
    Ok(None) => Err(Error::NotFound),
    Err(e) => Err(e),
  }
}

pub fn get_board_cards(
  postgres: &PgConnection,
  board_id: &str,
  participant_id: &str,
) -> Result<Vec<Card>, Error> {
  use schema::board;
  use schema::card::dsl;

  map_diesel_err!(schema::rank::dsl::rank
    .inner_join(dsl::card)
    .inner_join(board::dsl::board)
    .filter(board::dsl::id.eq(board_id))
    .select((
      dsl::id,
      dsl::rank_id,
      dsl::name,
      dsl::description,
      sql(VOTES_SQL),
      sql("(")
        .sql(VOTED_SQL)
        .bind::<Text, _>(participant_id)
        .sql(") as voted"),
      dsl::participant_id.eq(participant_id),
      dsl::created_at,
    ))
    .order(dsl::created_at.asc())
    .load(postgres))
}

pub fn get_rank_cards(
  postgres: &PgConnection,
  rank_id: &str,
  participant_id: &str,
) -> Result<Vec<Card>, Error> {
  use schema::card::dsl;

  map_diesel_err!(schema::rank::dsl::rank
    .inner_join(dsl::card)
    .filter(schema::rank::dsl::id.eq(rank_id))
    .select((
      dsl::id,
      dsl::rank_id,
      dsl::name,
      dsl::description,
      sql(VOTES_SQL),
      sql("(")
        .sql(VOTED_SQL)
        .bind::<Text, _>(participant_id)
        .sql(") as voted"),
      dsl::participant_id.eq(participant_id),
      dsl::created_at,
    ))
    .order(dsl::created_at.asc())
    .load(postgres))
}

pub fn get_card(
  postgres: &PgConnection,
  card_id: &str,
  participant_id: &str,
) -> Result<Option<Card>, Error> {
  use schema::card::dsl;

  map_diesel_err!(dsl::card
    .select((
      dsl::id,
      dsl::rank_id,
      dsl::name,
      dsl::description,
      sql(VOTES_SQL),
      sql("(")
        .sql(VOTED_SQL)
        .bind::<Text, _>(participant_id)
        .sql(") as voted"),
      dsl::participant_id.eq(participant_id),
      dsl::created_at,
    ))
    .find(card_id)
    .first(postgres)
    .optional())
}

pub fn patch_card(
  postgres: &PgConnection,
  card_id: &str,
  update_card: &UpdateCard,
  participant_id: &str,
) -> Result<Card, Error> {
  use schema::card::dsl;

  let inserted_id: String = map_diesel_err!(diesel::update(dsl::card.find(card_id))
    .set(update_card)
    .returning(dsl::id)
    .get_result(postgres))?;

  match get_card(postgres, &inserted_id, participant_id) {
    Ok(Some(c)) => Ok(c),
    Ok(None) => Err(Error::NotFound),
    Err(e) => Err(e),
  }
}

pub fn delete_card(postgres: &PgConnection, card_id: &str) -> Result<usize, Error> {
  use schema::card::dsl::*;

  map_diesel_err!(diesel::delete(card.find(card_id)).execute(postgres))
}

pub fn card_in_rank(postgres: &PgConnection, card_id: &str, rank_id: &str) -> Result<bool, Error> {
  Ok(match get_card(&postgres, &card_id, "")? {
    Some(c) => c.rank_id == rank_id,
    None => false,
  })
}
