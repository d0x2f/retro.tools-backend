use actix_identity::error::LoginError;
use actix_web::http::StatusCode;
use actix_web::{HttpResponse, ResponseError};
use firestore::errors::FirestoreError;
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
  fn error_response(&self) -> HttpResponse {
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
    HttpResponse::build(status).json(json!({ "error": message }))
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
impl InternalError for actix_http::error::PayloadError {}
impl InternalError for serde_json::error::Error {}
impl InternalError for std::string::FromUtf8Error {}
impl InternalError for reqwest::Error {}
impl InternalError for csv::Error {}
impl<W> InternalError for csv::IntoInnerError<W> {}
impl InternalError for SystemTimeError {}
impl InternalError for FirestoreError {}
impl InternalError for LoginError {}

#[cfg(test)]
mod tests {
  use super::*;
  use actix_web::body::to_bytes;
  use actix_web::http::StatusCode;
  use actix_web::ResponseError;

  #[test]
  fn not_found_status_is_404() {
    assert_eq!(Error::NotFound.error_response().status(), StatusCode::NOT_FOUND);
  }

  #[test]
  fn forbidden_status_is_403() {
    assert_eq!(Error::Forbidden.error_response().status(), StatusCode::FORBIDDEN);
  }

  #[test]
  fn bad_request_status_is_400() {
    assert_eq!(
      Error::BadRequest("oops".into()).error_response().status(),
      StatusCode::BAD_REQUEST
    );
  }

  #[test]
  fn other_status_is_500() {
    assert_eq!(
      Error::Other("db down".into()).error_response().status(),
      StatusCode::INTERNAL_SERVER_ERROR
    );
  }

  #[tokio::test]
  async fn not_found_body_has_error_key() {
    let body = to_bytes(Error::NotFound.error_response().into_body()).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["error"], "Not Found");
  }

  #[tokio::test]
  async fn bad_request_body_contains_caller_message() {
    let body = to_bytes(
      Error::BadRequest("column is required".into()).error_response().into_body(),
    )
    .await
    .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["error"], "column is required");
  }

  #[tokio::test]
  async fn other_error_body_does_not_leak_internal_detail() {
    let body = to_bytes(
      Error::Other("secret db password".into()).error_response().into_body(),
    )
    .await
    .unwrap();
    let raw = std::str::from_utf8(&body).unwrap();
    // Client receives a safe generic message
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["error"], "Something went wrong");
    // Internal detail must not be forwarded to the client
    assert!(!raw.contains("secret db password"), "internal error leaked: {raw}");
  }
}
