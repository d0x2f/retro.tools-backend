#![feature(proc_macro_hygiene, decl_macro)]

pub mod models;
pub mod persistence;
pub mod schema;

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

use self::models::*;
use self::persistence::*;
use diesel::PgConnection;
use log::error;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use rocket::http::Cookie;
use rocket::http::Status;
use rocket::request::FromRequest;
use rocket::*;
use rocket_contrib::json::{Json, JsonValue};

#[database("postgres")]
struct DatabaseConnection(PgConnection);

struct ParticipantId(String);

#[derive(Debug)]
enum CookieError {
    Error,
}

impl<'a, 'r> FromRequest<'a, 'r> for ParticipantId {
    type Error = CookieError;

    // TODO: session fixation
    fn from_request(request: &'a Request<'r>) -> request::Outcome<Self, Self::Error> {
        let cookie = request.cookies().get_private("id");
        if let Some(cookie) = cookie {
            return Outcome::Success(ParticipantId {
                0: String::from(cookie.value()),
            });
        }
        let participant_id: String = thread_rng().sample_iter(&Alphanumeric).take(16).collect();
        request
            .cookies()
            .add_private(Cookie::new("id", participant_id.clone()));
        Outcome::Success(ParticipantId { 0: participant_id })
    }
}

#[post("/boards", data = "<new_board>")]
fn post_board(
    participant_id: ParticipantId,
    postgres: DatabaseConnection,
    new_board: Json<NewBoard>,
) -> Result<JsonValue, Status> {
    put_board(&postgres, new_board.into_inner(), &participant_id.0)
        .map(|board| json!(board))
        .map_err(|error| {
            error!("{}", error.to_string());
            Status::InternalServerError
        })
}

#[get("/boards")]
fn get_boards(
    participant_id: ParticipantId,
    postgres: DatabaseConnection,
) -> Result<JsonValue, Status> {
    persistence::get_boards(&postgres, &participant_id.0)
        .map(|boards| json!(boards))
        .map_err(|error| {
            error!("{}", error.to_string());
            Status::InternalServerError
        })
}

#[get("/boards/<id>")]
fn get_board(postgres: DatabaseConnection, id: String) -> Result<JsonValue, Status> {
    let boards = persistence::get_board(&postgres, &id).map_err(|error| {
        error!("{}", error.to_string());
        Status::InternalServerError
    })?;
    if !boards.is_empty() {
        return Ok(json!(boards[0]));
    }
    Err(Status::NotFound)
}

#[patch("/boards/<id>", data = "<update_board>")]
fn patch_board(
    _participant_id: ParticipantId,
    postgres: DatabaseConnection,
    id: String,
    update_board: Json<UpdateBoard>,
) -> Result<JsonValue, Status> {
    // TODO:
    //   - check that the caller is the board's owner
    persistence::patch_board(&postgres, &id, &update_board)
        .map(|board| json!(board))
        .map_err(|error| {
            error!("{}", error.to_string());
            Status::InternalServerError
        })
}

#[delete("/boards/<id>")]
fn delete_board(
    _participant_id: ParticipantId,
    postgres: DatabaseConnection,
    id: String,
) -> Result<&'static str, Status> {
    // TODO:
    //   - check that the caller is the board's owner
    persistence::delete_board(&postgres, &id)
        .map(|_| "")
        .map_err(|error| {
            error!("{}", error.to_string());
            Status::InternalServerError
        })
}

#[catch(500)]
fn internal_error() -> &'static str {
    ""
}

#[catch(404)]
fn not_found() -> &'static str {
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
            routes![post_board, get_boards, get_board, patch_board, delete_board],
        )
        .register(catchers![internal_error, not_found, bad_request])
        .attach(DatabaseConnection::fairing())
        .launch();
}
