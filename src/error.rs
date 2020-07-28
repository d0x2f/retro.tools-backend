use std::fmt::{Display, Formatter, Result as FmtResult};
use actix_web::http::StatusCode;
use actix_web::{web, ResponseError};
use serde_json::{json, to_string_pretty};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub enum Error {
  NotFound,
  Forbidden,
  Other,
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "{}", to_string_pretty(self).unwrap())
    }
}

impl ResponseError for Error {
  fn error_response(&self) -> web::HttpResponse {
    let (status, message) = match self {
      Error::NotFound => (StatusCode::NOT_FOUND, "Not Found"),
      Error::Forbidden => (StatusCode::FORBIDDEN, "Forbidden"),
      Error::Other => (StatusCode::INTERNAL_SERVER_ERROR, "Something went wrong"),
    };
    web::HttpResponse::build(status).json(json!({ "error": message }))
  }
}