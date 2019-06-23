#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;
#[macro_use]
extern crate rocket_contrib;
#[macro_use]
extern crate serde_derive;

extern crate base64;
extern crate google_firestore1_beta1 as firestore1_beta1;
extern crate hyper;
extern crate hyper_rustls;
extern crate yup_oauth2 as oauth2;

pub mod objects;
pub mod persistence;

use objects::Board;
use objects::IntoDocument;
use rocket_contrib::json::{Json, JsonValue};

const DATABASE_NAMESPACE: &str = "projects/retrograde/databases/(default)/documents";

#[post("/boards", data = "<board>")]
fn post_boards(board: Json<Board>) -> JsonValue {
    match persistence::put(
        DATABASE_NAMESPACE,
        "boards",
        board.into_inner().into_document(),
    ) {
        Ok(_) => json!({ "status": "ok" }),
        Err(err) => json!({ "status": "error", "message": err }),
    }
}

#[get("/boards")]
fn get_boards() -> JsonValue {
    match persistence::get_list(DATABASE_NAMESPACE, "boards") {
        Ok(list) => {
            let boards: Vec<Board> = list
                .into_iter()
                .map(|d| Board::from_document(d).unwrap())
                .collect();
            json!(boards)
        }
        Err(err) => json!({ "status": "error", "message": err }),
    }
}

#[get("/boards/<id>")]
fn get_board(id: String) -> JsonValue {
    match persistence::get(DATABASE_NAMESPACE, "boards", id) {
        Ok(board) => json!(Board::from_document(board)),
        Err(err) => json!({ "status": "error", "message": err }),
    }
}

#[patch("/boards/<id>", data = "<board>")]
fn patch_board(id: String, board: Json<Board>) -> JsonValue {
    match persistence::patch(DATABASE_NAMESPACE, "boards", id, board.into_document()) {
        Ok(_) => json!({ "status": "ok" }),
        Err(err) => json!({ "status": "error", "message": err }),
    }
}

#[catch(500)]
fn internal_error() -> JsonValue {
    json!({ "status": "error" })
}

#[catch(404)]
fn not_found() -> JsonValue {
    json!({ "status": "error" })
}

fn main() {
    rocket::ignite()
        .mount(
            "/",
            routes![post_boards, get_boards, get_board, patch_board],
        )
        .register(catchers![internal_error, not_found])
        .launch();
}
