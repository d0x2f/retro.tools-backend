use actix_web::http::StatusCode;
use actix_web::{web, ResponseError};
use serde::Serialize;
use serde_json::{json, to_string_pretty};
use std::fmt::{Display, Formatter, Result as FmtResult};

#[derive(Debug, Serialize)]
pub enum Error {
  // NotFound,
  // Forbidden,
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
      // Error::NotFound => (StatusCode::NOT_FOUND, "Not Found"),
      // Error::Forbidden => (StatusCode::FORBIDDEN, "Forbidden"),
      Error::Other => (StatusCode::INTERNAL_SERVER_ERROR, "Something went wrong"),
    };
    web::HttpResponse::build(status).json(json!({ "error": message }))
  }
}

impl From<tonic::Status> for Error {
  fn from(tonic_error: tonic::Status) -> Self {
    error!("{:?}", tonic_error);
    Error::Other
  }
}

impl From<actix_http::error::Error> for Error {
  fn from(actix_error: actix_http::error::Error) -> Self {
    error!("{:?}", actix_error);
    Error::Other
  }
}
