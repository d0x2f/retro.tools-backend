use std::cell::RefCell;
use actix_web::web;
use crate::error::Error;
use super::db;
use crate::firestore::FirestoreV1Client;

pub async fn get_boards(firestore: web::Data<RefCell<FirestoreV1Client>>) -> Result<web::HttpResponse, Error> {
  let boards = db::get_boards(firestore.get_ref()).await?;
  Ok(web::HttpResponse::Ok().json(boards))
}