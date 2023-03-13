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
  pub allowed_origin: String,
  pub firestore_project: String,
  pub firebase_credentials: GoogleAccountKey,
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
      Ok(port) => port.parse().expect("integer port"),
      _ => 8000,
    };

    let firestore_project = match env::var("FIRESTORE_PROJECT") {
      Ok(s) => s,
      Err(_) => cloudrun::get_project_id().expect("cloudrun project"),
    };

    let google_credentials_file_path = match env::var("GOOGLE_APPLICATION_CREDENTIALS") {
      Ok(s) => s,
      Err(_) => {
        panic!("No Google service account credentials given via 'GOOGLE_APPLICATION_CREDENTIALS'.")
      }
    };

    let file = File::open(google_credentials_file_path)
      .expect("Unable to open file referenced by 'GOOGLE_APPLICATION_CREDENTIALS'.");
    let reader = BufReader::new(file);
    let firebase_credentials: GoogleAccountKey = serde_json::from_reader(reader)
      .expect("Unable to read file referenced by 'GOOGLE_APPLICATION_CREDENTIALS'.");

    Config {
      port,
      secret_key,
      environment,
      allowed_origin: env::var("ALLOWED_ORIGIN").expect("allowed origin"),
      firestore_project,
      firebase_credentials,
    }
  }
}

impl Clone for Config {
  fn clone(&self) -> Config {
    Config {
      port: self.port,
      secret_key: self.secret_key.clone(),
      environment: self.environment,
      allowed_origin: self.allowed_origin.clone(),
      firestore_project: self.firestore_project.clone(),
      firebase_credentials: self.firebase_credentials.clone(),
    }
  }
}
