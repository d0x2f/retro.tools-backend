use actix_web::http::StatusCode;
use actix_web::{web, ResponseError};
use serde::Serialize;
use serde_json::{json, to_string_pretty};
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::time::SystemTimeError;

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

impl From<jwt_simple::Error> for Error {
  fn from(error: jwt_simple::Error) -> Self {
    Error::Other(format!("{}", error))
  }
}

trait InternalError {}

impl<T> From<T> for Error
where
  T: std::error::Error + InternalError,
{
  fn from(error: T) -> Self {
    Error::Other(format!("{}", error))
  }
}

impl InternalError for actix_http::error::Error {}
impl InternalError for actix_http::client::SendRequestError {}
impl InternalError for actix_http::error::PayloadError {}
impl InternalError for serde_json::error::Error {}
impl InternalError for std::string::FromUtf8Error {}
impl InternalError for tonic::metadata::errors::InvalidMetadataValue {}
impl InternalError for reqwest::Error {}
impl InternalError for tonic::transport::Error {}
impl InternalError for gcp_auth::Error {}
impl InternalError for csv::Error {}
impl<W> InternalError for csv::IntoInnerError<W> {}
impl InternalError for SystemTimeError {}
