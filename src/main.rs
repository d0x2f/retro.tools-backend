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
mod schema;

use diesel::PgConnection;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use rocket::http::Cookie;
use rocket::request::FromRequest;
use rocket::*;

#[database("postgres")]
pub struct DatabaseConnection(PgConnection);

pub struct ParticipantId(String);

#[derive(Debug)]
pub enum CookieError {
    Error,
}

impl<'a, 'r> FromRequest<'a, 'r> for ParticipantId {
    type Error = CookieError;

    // TODO: session fixation
    fn from_request(request: &'a Request<'r>) -> request::Outcome<Self, Self::Error> {
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

#[catch(500)]
fn internal_error() -> &'static str {
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
                boards::delete_board
            ],
        )
        .register(catchers![
            internal_error,
            not_found,
            unauthorised,
            bad_request
        ])
        .attach(DatabaseConnection::fairing())
        .launch();
}
