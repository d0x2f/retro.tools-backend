use crate::error;
use actix_http::httpmessage::HttpMessage;
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
  type Config = ();

  fn from_request(req: &HttpRequest, payload: &mut Payload) -> Self::Future {
    let legacy_id: Option<String> = match req.cookie("__session") {
      Some(cookie) => Some(cookie.value().into()),
      None => None,
    };
    let config = req.app_data::<Data<Config>>().expect("config");
    let config = &(*Arc::clone(&config.clone().into_inner()));
    Box::pin(super::new(
      config.clone(),
      Identity::from_request(req, payload),
      legacy_id,
    ))
  }
}
