use actix_web::http::StatusCode;
use actix_web::{web, ResponseError};
use serde::Serialize;
use serde_json::{json, to_string_pretty};
use std::fmt::{Display, Formatter, Result as FmtResult};

#[derive(Debug, Serialize)]
pub enum Error {
  NotFound,
  Forbidden,
  BadRequest(String),
  Other(String),
}

impl Display for Error {
  fn fmt(&self, f: &mut Formatter) -> FmtResult {
    write!(f, "{}", to_string_pretty(self).unwrap())
  }
}

impl ResponseError for Error {
  fn error_response(&self) -> web::HttpResponse {
    let (status, message, log_message) = match self {
      Error::NotFound => (StatusCode::NOT_FOUND, "Not Found", None),
      Error::Forbidden => (StatusCode::FORBIDDEN, "Forbidden", None),
      Error::BadRequest(s) => (StatusCode::BAD_REQUEST, s.as_str(), None),
      Error::Other(s) => (
        StatusCode::INTERNAL_SERVER_ERROR,
        "Something went wrong",
        Some(s),
      ),
    };
    if let Some(error) = log_message {
      error!("{}", error);
    }
    web::HttpResponse::build(status).json(json!({ "error": message }))
  }
}

impl From<tonic::Status> for Error {
  fn from(tonic_error: tonic::Status) -> Self {
    match tonic_error.code() {
      tonic::Code::NotFound => Error::NotFound,
      _ => Error::Other(format!("{}", tonic_error)),
    }
  }
}

impl From<actix_http::error::Error> for Error {
  fn from(actix_error: actix_http::error::Error) -> Self {
    Error::Other(format!("{}", actix_error))
  }
}
