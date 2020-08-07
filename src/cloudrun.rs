use actix_web::client::Client;
use bytes::Bytes;
use serde::{Deserialize, Serialize};

use crate::error::Error;

#[derive(Serialize, Deserialize, Debug)]
struct TokenResponse {
  access_token: String,
  expires_in: u32,
  token_type: String,
}

pub async fn get_metadata(entry: &'static str) -> std::result::Result<Bytes, Error> {
  let client = Client::default();

  let mut response = client
    .get(format!(
      "http://metadata.google.internal/computeMetadata/v1/{}",
      entry
    ))
    .header("Metadata-Flavor", "Google")
    .send()
    .await?;

  Ok(response.body().await?)
}

pub async fn get_token() -> std::result::Result<String, Error> {
  let bytes = get_metadata(
    "instance/service-accounts/default/token?scopes=https://www.googleapis.com/auth/datastore",
  )
  .await?;
  let body: TokenResponse = serde_json::from_slice(&bytes.to_vec())?;
  let token = body.access_token;
  Ok(token)
}

pub async fn get_project_id() -> std::result::Result<String, Error> {
  let bytes = get_metadata("project/project-id").await?;
  let project_id = String::from_utf8(bytes.to_vec())?;
  Ok(project_id)
}
