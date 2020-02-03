use diesel::PgConnection;
use rocket::http::Cookie;
use rocket::http::Status;
use rocket::request::FromRequest;
use rocket::*;
use time::Duration;
use super::models::NewParticipantBoard;
use super::persistence;

#[database("postgres")]
pub struct DatabaseConnection(PgConnection);

pub struct ParticipantId(pub String);
pub struct BoardParticipant();
pub struct BoardOwner();
pub struct CardOwner();
pub struct RankInBoard();
pub struct CardInRank();

impl<'a, 'r> FromRequest<'a, 'r> for ParticipantId {
  type Error = ();

  fn from_request(request: &'a Request<'r>) -> request::Outcome<Self, ()> {
    let mut cookies = request.cookies();
    let cookie = cookies.get("id");
    if let Some(cookie) = cookie {
      let participant_id = String::from(cookie.value());
      // Verify the session id is real
      let postgres = request.guard::<DatabaseConnection>()?;
      match persistence::get_participant(&postgres, &participant_id) {
        Ok(Some(_)) => return Outcome::Success(ParticipantId {
            0: participant_id,
          }),
        Ok(None) => (),
        Err(_) => {
          error!("Database error during ParticipantId guard.");
          return Outcome::Failure((Status::InternalServerError, ()));
        }
      }
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
        .path("/")
        .finish(),
    );
    Outcome::Success(ParticipantId { 0: participant.id })
  }
}

impl<'a, 'r> FromRequest<'a, 'r> for BoardParticipant {
  type Error = ();

  fn from_request(request: &'a Request<'r>) -> request::Outcome<Self, ()> {
    let board_id = match request.get_param::<String>(1) {
      Some(Ok(id)) => id,
      _ => {
        error!("Error in BoardParticipant guard - board_id is not available.");
        return Outcome::Failure((Status::InternalServerError, ()));
      }
    };

    let participant_id = request.guard::<ParticipantId>()?;
    let postgres = request.guard::<DatabaseConnection>()?;

    let new_participant = NewParticipantBoard {
      participant_id: Some(&participant_id.0),
      owner: false,
      board_id: &board_id,
    };

    match persistence::put_participant_board(&postgres, &new_participant) {
      Ok(_) => Outcome::Success(BoardParticipant {}),
      Err(_) => Outcome::Failure((Status::NotFound, ()))
    }
  }
}

impl<'a, 'r> FromRequest<'a, 'r> for BoardOwner {
  type Error = ();

  fn from_request(request: &'a Request<'r>) -> request::Outcome<Self, ()> {
    let board_id = match request.get_param::<String>(1) {
      Some(Ok(id)) => id,
      _ => {
        error!("Error in BoardOwner guard - board_id is not available.");
        return Outcome::Failure((Status::InternalServerError, ()));
      }
    };

    let participant_id = request.guard::<ParticipantId>()?;
    let postgres = request.guard::<DatabaseConnection>()?;

    match super::persistence::participant_owns_board(&postgres, &participant_id.0, &board_id) {
      Ok(true) => Outcome::Success(BoardOwner {}),
      Ok(false) => Outcome::Failure((Status::Unauthorized, ())),
      Err(_) => {
        error!("Database error during BoardOwner guard.");
        Outcome::Failure((Status::InternalServerError, ()))
      }
    }
  }
}

impl<'a, 'r> FromRequest<'a, 'r> for CardOwner {
  type Error = ();

  fn from_request(request: &'a Request<'r>) -> request::Outcome<Self, ()> {
    let board_id = match request.get_param::<String>(1) {
      Some(Ok(id)) => id,
      _ => {
        error!("Error in CardOwner guard - board_id is not available.");
        return Outcome::Failure((Status::InternalServerError, ()));
      }
    };

    let card_id = match request.get_param::<String>(5) {
      Some(Ok(id)) => id,
      _ => {
        error!("Error in CardOwner guard - card_id is not available.");
        return Outcome::Failure((Status::InternalServerError, ()));
      }
    };

    let participant_id = request.guard::<ParticipantId>()?;
    let postgres = request.guard::<DatabaseConnection>()?;
    match super::persistence::participant_owns_card(
      &postgres,
      &participant_id.0,
      &board_id,
      &card_id,
    ) {
      Ok(true) => Outcome::Success(CardOwner {}),
      Ok(false) => Outcome::Failure((Status::Unauthorized, ())),
      Err(_) => {
        error!("Database error during CardOwner guard.");
        Outcome::Failure((Status::InternalServerError, ()))
      }
    }
  }
}

impl<'a, 'r> FromRequest<'a, 'r> for RankInBoard {
  type Error = ();

  fn from_request(request: &'a Request<'r>) -> request::Outcome<Self, ()> {
    let board_id = match request.get_param::<String>(1) {
      Some(Ok(id)) => id,
      _ => {
        error!("Error in CardOwner guard - board_id is not available.");
        return Outcome::Failure((Status::InternalServerError, ()));
      }
    };

    let rank_id = match request.get_param::<String>(3) {
      Some(Ok(id)) => id,
      _ => {
        error!("Error in CardOwner guard - rank_id is not available.");
        return Outcome::Failure((Status::InternalServerError, ()));
      }
    };

    let postgres = request.guard::<DatabaseConnection>()?;
    match super::persistence::rank_in_board(&postgres, &rank_id, &board_id) {
      Ok(true) => Outcome::Success(RankInBoard {}),
      Ok(false) => Outcome::Failure((Status::NotFound, ())),
      Err(_) => {
        error!("Database error during RankInBoard guard.");
        Outcome::Failure((Status::InternalServerError, ()))
      }
    }
  }
}

impl<'a, 'r> FromRequest<'a, 'r> for CardInRank {
  type Error = ();

  fn from_request(request: &'a Request<'r>) -> request::Outcome<Self, ()> {
    let card_id = match request.get_param::<String>(5) {
      Some(Ok(id)) => id,
      _ => {
        error!("Error in CardInRank guard - card_id is not available.");
        return Outcome::Failure((Status::InternalServerError, ()));
      }
    };

    let rank_id = match request.get_param::<String>(3) {
      Some(Ok(id)) => id,
      _ => {
        error!("Error in CardInRank guard - rank_id is not available.");
        return Outcome::Failure((Status::InternalServerError, ()));
      }
    };

    let postgres = request.guard::<DatabaseConnection>()?;

    match super::persistence::card_in_rank(&postgres, &card_id, &rank_id) {
      Ok(true) => Outcome::Success(CardInRank {}),
      Ok(false) => Outcome::Failure((Status::NotFound, ())),
      Err(_) => {
        error!("Database error during CardInRank guard.");
        Outcome::Failure((Status::InternalServerError, ()))
      }
    }
  }
}
