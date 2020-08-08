use crate::error::Error;
use bytes::Bytes;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::thread;
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

#[derive(Serialize, Deserialize, Debug)]
struct TokenResponse {
  access_token: String,
  expires_in: u64,
  token_type: String,
}

pub struct Token {
  access_token: String,
  issued_at: Instant,
  expires_in: Duration,
  renew_thread: Option<JoinHandle<()>>,
}

impl Token {
  pub fn new(token: Option<String>) -> Result<Arc<Mutex<Token>>, Error> {
    if let Some(t) = token {
      return Ok(Arc::new(Mutex::new(Token {
        access_token: t,
        issued_at: Instant::now(),
        expires_in: Duration::new(u64::MAX, 0),
        renew_thread: None,
      })));
    }
    let mut token = Token {
      access_token: "".into(),
      issued_at: Instant::now(),
      expires_in: Duration::new(0, 0),
      renew_thread: None,
    };
    token.renew()?;
    Ok(Arc::new(Mutex::new(token)))
  }

  pub fn start_auto_renew(token: Arc<Mutex<Token>>) {
    let thread_token = Arc::clone(&token);
    let thread = thread::spawn(move || loop {
      // Wait until 50 seconds before expiry
      let sleep_time;
      {
        let token = thread_token.lock().unwrap();
        let time_since_issued = Instant::now().saturating_duration_since(token.issued_at);
        sleep_time = token
          .expires_in
          .checked_sub(time_since_issued)
          .unwrap_or_else(|| Duration::new(0, 0))
          .checked_sub(Duration::from_secs(30))
          .unwrap_or_else(|| Duration::new(10, 0));
      }
      thread::sleep(sleep_time);
      {
        let mut token = thread_token.lock().unwrap();
        match token.renew() {
          Ok(_) => info!("Firestore token renewed, expiry: {:?}", token.expires_in),
          Err(e) => error!("Firestore token renewal failed, error: {}", e),
        }
      }
    });
    token.lock().unwrap().renew_thread = Some(thread);
  }

  fn fetch_new() -> Result<TokenResponse, Error> {
    let bytes = get_metadata(
      "instance/service-accounts/default/token?scopes=https://www.googleapis.com/auth/datastore",
    )?;
    let response: TokenResponse = serde_json::from_slice(&bytes.to_vec())?;
    Ok(response)
  }

  pub fn renew(&mut self) -> Result<(), Error> {
    let response = Token::fetch_new()?;
    self.access_token = response.access_token.clone();
    self.issued_at = Instant::now();
    self.expires_in = Duration::new(response.expires_in, 0);
    Ok(())
  }

  pub fn get(&self) -> &str {
    &self.access_token
  }
}

pub fn get_metadata(entry: &'static str) -> std::result::Result<Bytes, Error> {
  let response = reqwest::blocking::Client::new()
    .get(
      format!(
        "http://metadata.google.internal/computeMetadata/v1/{}",
        entry
      )
      .as_str(),
    )
    .header("Metadata-Flavor", "Google")
    .send()?;

  Ok(response.bytes()?)
}

pub fn get_project_id() -> std::result::Result<String, Error> {
  let bytes = get_metadata("project/project-id")?;
  let project_id = String::from_utf8(bytes.to_vec())?;
  Ok(project_id)
}
