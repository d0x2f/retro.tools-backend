use diesel::PgConnection;
use rocket::http::Cookie;
use rocket::http::Status;
use rocket::request::FromRequest;
use rocket::*;
use time::Duration;

#[database("postgres")]
pub struct DatabaseConnection(PgConnection);

pub struct ParticipantId(pub String);
pub struct BoardOwner();
pub struct RankInBoard();
pub struct CardInRank();

impl<'a, 'r> FromRequest<'a, 'r> for ParticipantId {
  type Error = ();

  // TODO: session fixation
  fn from_request(request: &'a Request<'r>) -> request::Outcome<Self, ()> {
    let mut cookies = request.cookies();
    let cookie = cookies.get("id");
    if let Some(cookie) = cookie {
      return Outcome::Success(ParticipantId {
        0: String::from(cookie.value()),
      });
    }
    let postgres = request.guard::<DatabaseConnection>()?;
    let participant = match super::persistence::create_participant(&postgres) {
      Ok(p) => p,
      Err(_) => {
        error!("Database error during ParticipantId guard.");
        return Outcome::Failure((Status::InternalServerError, ()));
      }
    };
    cookies.add(
      Cookie::build("id", participant.id.clone())
        .http_only(true)
        .max_age(Duration::days(7))
        .finish(),
    );
    Outcome::Success(ParticipantId { 0: participant.id })
  }
}

impl<'a, 'r> FromRequest<'a, 'r> for BoardOwner {
  type Error = ();

  fn from_request(request: &'a Request<'r>) -> request::Outcome<Self, ()> {
    let participant_id = request.guard::<ParticipantId>()?;
    if let Some(board_id) = request.get_param::<String>(1).and_then(|r| r.ok()) {
      let postgres = request.guard::<DatabaseConnection>()?;
      let participant_owns_board =
        match super::persistence::participant_owns_board(&postgres, &participant_id.0, &board_id) {
          Ok(r) => r,
          Err(diesel::result::Error::NotFound) => return Outcome::Failure((Status::NotFound, ())),
          Err(_) => {
            error!("Database error during BoardOwner guard.");
            return Outcome::Failure((Status::InternalServerError, ()));
          }
        };
      if participant_owns_board {
        return Outcome::Success(BoardOwner {});
      }
      return Outcome::Failure((Status::Unauthorized, ()));
    }
    error!("Error in BoardOwner guard - board_id is not available.");
    Outcome::Failure((Status::InternalServerError, ()))
  }
}

impl<'a, 'r> FromRequest<'a, 'r> for RankInBoard {
  type Error = ();

  fn from_request(request: &'a Request<'r>) -> request::Outcome<Self, ()> {
    let board_id_result = request.get_param::<String>(1).and_then(|r| r.ok());
    let rank_id_result = request.get_param::<String>(3).and_then(|r| r.ok());

    if let Some(board_id) = board_id_result {
      if let Some(rank_id) = rank_id_result {
        let postgres = request.guard::<DatabaseConnection>()?;

        let rank_in_board = match super::persistence::rank_in_board(&postgres, &rank_id, &board_id)
        {
          Ok(r) => r,
          Err(diesel::result::Error::NotFound) => return Outcome::Failure((Status::NotFound, ())),
          Err(_) => {
            error!("Database error during RankInBoard guard.");
            return Outcome::Failure((Status::InternalServerError, ()));
          }
        };
        if rank_in_board {
          return Outcome::Success(RankInBoard {});
        }
        return Outcome::Failure((Status::NotFound, ()));
      }
    }
    error!("Error in RankInBoard guard - either board_id or rank_id is not available.");
    Outcome::Failure((Status::InternalServerError, ()))
  }
}

impl<'a, 'r> FromRequest<'a, 'r> for CardInRank {
  type Error = ();

  fn from_request(request: &'a Request<'r>) -> request::Outcome<Self, ()> {
    let rank_id_result = request.get_param::<String>(3).and_then(|r| r.ok());
    let card_id_result = request.get_param::<String>(5).and_then(|r| r.ok());

    if let Some(card_id) = card_id_result {
      if let Some(rank_id) = rank_id_result {
        let postgres = request.guard::<DatabaseConnection>()?;

        let card_in_rank = match super::persistence::card_in_rank(&postgres, &card_id, &rank_id) {
          Ok(r) => r,
          Err(diesel::result::Error::NotFound) => return Outcome::Failure((Status::NotFound, ())),
          Err(_) => {
            error!("Database error during CardInRank guard.");
            return Outcome::Failure((Status::InternalServerError, ()));
          }
        };
        if card_in_rank {
          return Outcome::Success(CardInRank {});
        }
        return Outcome::Failure((Status::NotFound, ()));
      }
    }
    error!("Error in RankInBoard guard - either card_id or rank_id is not available.");
    Outcome::Failure((Status::InternalServerError, ()))
  }
}
