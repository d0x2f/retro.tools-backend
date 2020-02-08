use super::super::models::*;
use super::super::schema;
use super::Error;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel::result::Error as DieselError;

pub fn put_vote(postgres: &PgConnection, new_vote: NewVote) -> Result<Option<Vote>, Error> {
  use schema::vote::dsl::*;

  // Ensure the card exists
  if !card_exists(postgres, new_vote.card_id).map_err(Into::<Error>::into)? {
    return Ok(None);
  }

  diesel::insert_into(vote)
    .values(new_vote)
    .on_conflict((card_id, participant_id))
    .do_update()
    .set(count.eq(count)) // Hack to get the vote back in the result
    .get_result(postgres)
    .optional()
    .map_err(Into::into)
}

pub fn get_vote(
  postgres: &PgConnection,
  card_id: &str,
  participant_id: &str,
) -> Result<Option<Vote>, Error> {
  use schema::vote::dsl;

  dsl::vote
    .find((card_id, participant_id))
    .first(postgres)
    .optional()
    .map_err(Into::into)
}

pub fn patch_vote(postgres: &PgConnection, update_vote: &UpdateVote) -> Result<Vote, Error> {
  use schema::vote::dsl::*;

  diesel::update(vote.find((update_vote.card_id, update_vote.participant_id)))
    .set(count.eq(update_vote.count))
    .get_result(postgres)
    .map_err(Into::into)
}

fn card_exists(postgres: &PgConnection, card_id: &str) -> Result<bool, DieselError> {
  use diesel::expression::dsl::exists;
  use diesel::select;
  use schema::card::dsl::*;

  select(exists(card.filter(id.eq(card_id)))).get_result(postgres)
}
