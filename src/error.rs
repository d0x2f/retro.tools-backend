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
  fn from(error: tonic::Status) -> Self {
    match error.code() {
      tonic::Code::NotFound => Error::NotFound,
      _ => Error::Other(format!("{}", error)),
    }
  }
}

impl From<actix_http::error::Error> for Error {
  fn from(error: actix_http::error::Error) -> Self {
    Error::Other(format!("{}", error))
  }
}

impl From<serde_json::error::Error> for Error {
  fn from(error: serde_json::error::Error) -> Self {
    Error::Other(format!("{}", error))
  }
}

impl From<actix_http::client::SendRequestError> for Error {
  fn from(error: actix_http::client::SendRequestError) -> Self {
    Error::Other(format!("{}", error))
  }
}

impl From<actix_http::error::PayloadError> for Error {
  fn from(error: actix_http::error::PayloadError) -> Self {
    Error::Other(format!("{}", error))
  }
}

impl From<std::string::FromUtf8Error> for Error {
  fn from(error: std::string::FromUtf8Error) -> Self {
    Error::Other(format!("{}", error))
  }
}

impl From<tonic::metadata::errors::InvalidMetadataValue> for Error {
  fn from(error: tonic::metadata::errors::InvalidMetadataValue) -> Self {
    Error::Other(format!("{}", error))
  }
}

impl From<tonic::transport::Error> for Error {
  fn from(error: tonic::transport::Error) -> Self {
    Error::Other(format!("{}", error))
  }
}

impl From<reqwest::Error> for Error {
  fn from(error: reqwest::Error) -> Self {
    Error::Other(format!("{}", error))
  }
}

// impl<T: Display> From<T> for Error {
//   fn from(error: T) -> Self {
//     Error::Other(format!("{}", error))
//   }
// }
