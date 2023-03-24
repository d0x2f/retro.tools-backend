use actix_web::{get, web, HttpResponse};
use jwt_simple::prelude::{Claims, Duration, RS256KeyPair, RSAKeyPairLike};
use serde::{Deserialize, Serialize};

use crate::{config::Config, error::Error};

use super::models::Participant;

#[derive(Deserialize, Serialize)]
struct GoogleClaims {
  iss: String,
  sub: String,
  aud: String,
  uid: String,
}

const AUD: &str =
  "https://identitytoolkit.googleapis.com/google.identity.identitytoolkit.v1.IdentityToolkit";

#[get("auth")]
pub async fn auth(
  config: web::Data<Config>,
  participant: Participant,
) -> Result<HttpResponse, Error> {
  let key_pair = RS256KeyPair::from_pem(&config.firebase_credentials.private_key)?;

  let google_claims = GoogleClaims {
    iss: config.firebase_credentials.client_email.clone(),
    sub: config.firebase_credentials.client_email.clone(),
    aud: AUD.into(),
    uid: participant.id,
  };
  let claims = Claims::with_custom_claims(google_claims, Duration::from_hours(1));
  let token = key_pair.sign(claims)?;

  Ok(HttpResponse::Ok().body(token))
}
