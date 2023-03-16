use actix_http::cookie::SameSite;
use serde::{Deserialize, Serialize};
use std::env;
use std::fs::File;
use std::io::BufReader;

use crate::cloudrun;

#[derive(Copy, Clone, PartialEq)]
pub enum Environment {
  Production,
  Development,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct GoogleAccountKey {
  pub private_key: String,
  pub client_email: String,
}

pub struct Config {
  pub port: u16,
  pub secret_key: Vec<u8>,
  pub environment: Environment,
  pub allowed_origins: Vec<String>,
  pub firestore_project: String,
  pub firebase_credentials: GoogleAccountKey,
  pub secure_cookie: bool,
  pub same_site: SameSite,
}

impl Config {
  pub async fn from_env() -> Config {
    let environment = match env::var("ENVIRONMENT") {
      Ok(env_string) => match env_string.to_lowercase().as_str() {
        "development" => Environment::Development,
        _ => Environment::Production,
      },
      _ => Environment::Production,
    };

    let secret_key = match env::var("SECRET_KEY") {
      Err(_) => match environment {
        Environment::Production => {
          panic!("No secret key provided despite being in production mode!")
        }
        _ => vec![0_u8; 32],
      },
      Ok(s) => s.as_bytes().to_owned(),
    };

    let port = match env::var("PORT") {
      Ok(port) => port.parse().expect("PORT to be an integer"),
      _ => 8000,
    };

    let firestore_project = match env::var("FIRESTORE_PROJECT") {
      Ok(s) => s,
      Err(_) => cloudrun::get_project_id().expect("FIRESTORE_PROJECT environment variable"),
    };

    let google_credentials_file_path = match env::var("FIREBASE_SERVICE_ACCOUNT_CREDENTIALS") {
      Ok(s) => s,
      Err(_) => {
        panic!(
          "No Google service account credentials given via 'FIREBASE_SERVICE_ACCOUNT_CREDENTIALS'."
        )
      }
    };

    let secure_cookie = match env::var("SECURE_COOKIE") {
      Ok(s) => s == "true",
      Err(_) => false,
    };

    let file = File::open(google_credentials_file_path)
      .expect("Unable to open file referenced by 'FIREBASE_SERVICE_ACCOUNT_CREDENTIALS'.");
    let reader = BufReader::new(file);
    let firebase_credentials: GoogleAccountKey = serde_json::from_reader(reader)
      .expect("Unable to read file referenced by 'FIREBASE_SERVICE_ACCOUNT_CREDENTIALS'.");

    let allowed_origins: Vec<String> = env::var("ALLOWED_ORIGINS")
      .expect("ALLOWED_ORIGINS environment variable")
      .split(',')
      .map(|s| s.to_string())
      .collect();

    let same_site = match env::var("SAME_SITE")
      .unwrap_or("strict".into())
      .to_lowercase()
      .as_ref()
    {
      "strict" => Ok(SameSite::Strict),
      "lax" => Ok(SameSite::Lax),
      "none" => Ok(SameSite::None),
      _ => Err(()),
    }
    .expect("invalid value for SAME_SITE.");

    Config {
      port,
      secret_key,
      environment,
      allowed_origins,
      firestore_project,
      firebase_credentials,
      secure_cookie,
      same_site,
    }
  }
}

impl Clone for Config {
  fn clone(&self) -> Config {
    Config {
      port: self.port,
      secret_key: self.secret_key.clone(),
      environment: self.environment,
      allowed_origins: self.allowed_origins.clone(),
      firestore_project: self.firestore_project.clone(),
      firebase_credentials: self.firebase_credentials.clone(),
      secure_cookie: self.secure_cookie,
      same_site: self.same_site,
    }
  }
}
