#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;
#[macro_use]
extern crate rocket_contrib;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate diesel;
extern crate env_logger;
extern crate log;
extern crate rand;

mod boards;
mod models;
mod persistence;
mod ranks;
mod schema;

use diesel::PgConnection;
use log::info;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use rocket::http::Cookie;
use rocket::http::Status;
use rocket::request::FromRequest;
use rocket::*;

#[database("postgres")]
pub struct DatabaseConnection(PgConnection);

pub struct ParticipantId(String);
pub struct BoardOwner();
pub struct RankInBoard();

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
        let participant_id: String = thread_rng().sample_iter(&Alphanumeric).take(16).collect();
        cookies.add(Cookie::new("id", participant_id.clone()));
        Outcome::Success(ParticipantId { 0: participant_id })
    }
}

impl<'a, 'r> FromRequest<'a, 'r> for BoardOwner {
    type Error = ();

    fn from_request(request: &'a Request<'r>) -> request::Outcome<Self, ()> {
        let participant_id = request.guard::<ParticipantId>()?;
        if let Some(board_id) = request.get_param::<String>(1).and_then(|r| r.ok()) {
            let postgres = request.guard::<DatabaseConnection>()?;
            info!("{}", board_id);
            let participant_owns_board = match persistence::participant_owns_board(
                &postgres,
                &participant_id.0,
                &board_id,
            ) {
                Ok(r) => r,
                Err(_) => return Outcome::Failure((Status::InternalServerError, ())),
            };
            if participant_owns_board {
                return Outcome::Success(BoardOwner {});
            }
            return Outcome::Failure((Status::Unauthorized, ()));
        }
        return Outcome::Failure((Status::InternalServerError, ()));
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

                let rank_in_board = match persistence::rank_in_board(
                    &postgres,
                    &rank_id,
                    &board_id,
                ) {
                    Ok(r) => r,
                    Err(_) => return Outcome::Failure((Status::InternalServerError, ())),
                };
                if rank_in_board {
                    return Outcome::Success(RankInBoard {});
                }
                return Outcome::Failure((Status::NotFound, ()));
            }
        }
        return Outcome::Failure((Status::InternalServerError, ()));
    }
}

#[catch(500)]
fn internal_error() -> &'static str {
    ""
}

#[catch(422)]
fn unprocessible_entity() -> &'static str {
    ""
}

#[catch(404)]
fn not_found() -> &'static str {
    ""
}

#[catch(401)]
fn unauthorised() -> &'static str {
    ""
}

#[catch(400)]
fn bad_request() -> &'static str {
    ""
}

fn main() {
    rocket::ignite()
        .mount(
            "/",
            routes![
                boards::post_board,
                boards::get_boards,
                boards::get_board,
                boards::patch_board,
                boards::delete_board,
                ranks::post_rank,
                ranks::get_ranks,
                ranks::get_rank,
                ranks::patch_rank,
                ranks::delete_rank
            ],
        )
        .register(catchers![
            internal_error,
            unprocessible_entity,
            not_found,
            unauthorised,
            bad_request
        ])
        .attach(DatabaseConnection::fairing())
        .launch();
}
