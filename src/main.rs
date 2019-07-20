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
mod cards;
mod catchers;
mod guards;
mod models;
mod persistence;
mod ranks;
mod schema;
mod votes;

use rocket::*;

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
                ranks::delete_rank,
                cards::post_card,
                cards::get_cards,
                cards::get_card,
                cards::patch_card,
                cards::delete_card,
                votes::post_vote,
                votes::delete_vote
            ],
        )
        .register(catchers![
            catchers::internal_error,
            catchers::unprocessible_entity,
            catchers::not_found,
            catchers::forbidden,
            catchers::unauthorised,
            catchers::bad_request
        ])
        .attach(guards::DatabaseConnection::fairing())
        .launch();
}
