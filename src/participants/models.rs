use crate::error;
use actix_identity::Identity;
use actix_web::dev::Payload;
use actix_web::web::Data;
use actix_web::{FromRequest, HttpRequest};
use futures::future::Future;
use serde::{Deserialize, Serialize};
use std::pin::Pin;
use std::sync::Arc;

use crate::config::Config;
use crate::firestore::v1::Document;

#[derive(Deserialize, Serialize, Clone)]
pub struct Participant {
  pub id: String,
}

#[derive(Deserialize)]
pub struct ParticipantBoardIds {
  pub boards: Vec<String>,
}

impl From<Document> for Participant {
  fn from(document: Document) -> Self {
    Participant {
      id: get_id!(document),
    }
  }
}

impl FromRequest for Participant {
  type Error = error::Error;
  type Future = Pin<Box<dyn Future<Output = Result<Self, error::Error>>>>;

  fn from_request(req: &HttpRequest, payload: &mut Payload) -> Self::Future {
    let config = req.app_data::<Data<Config>>().expect("config");
    let config = &(*Arc::clone(&config.clone().into_inner()));
    Box::pin(super::new(
      config.clone(),
      Identity::from_request(req, payload),
      req.clone(),
    ))
  }
}
