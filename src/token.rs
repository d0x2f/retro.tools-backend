use gcp_auth::{AuthenticationManager, Token as InnerToken};
use settimeout::set_timeout;
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::Duration;
use tokio::runtime::Runtime;

use crate::error::Error;

const SCOPE: &str = "https://www.googleapis.com/auth/datastore";

pub struct Token {
  manager: Arc<Mutex<AuthenticationManager>>,
  inner: Arc<Mutex<InnerToken>>,
  thread: Option<JoinHandle<()>>,
}

impl Token {
  pub async fn new() -> Result<Token, Error> {
    let manager = Arc::new(Mutex::new(gcp_auth::init().await?));
    let inner = Arc::new(Mutex::new(
      manager
        .lock()
        .expect("mutex lock")
        .get_token(&[SCOPE])
        .await?,
    ));
    let mut token = Token {
      inner,
      manager,
      thread: None,
    };
    token.start_refresh_thread();
    Ok(token)
  }

  pub fn get(&self) -> String {
    self.inner.lock().expect("mutex lock").as_str().into()
  }

  fn start_refresh_thread(&mut self) -> () {
    let manager = self.manager.clone();
    let inner = self.inner.clone();
    let handle = std::thread::spawn(move || {
      let mut rt = Runtime::new().unwrap();
      rt.block_on(async {
        loop {
          set_timeout(Duration::from_secs(10)).await;
          if let Ok(t) = manager
            .lock()
            .expect("mutex lock")
            .get_token(&[SCOPE])
            .await
          {
            let mut contents = inner.lock().expect("mutex lock");
            *contents = t;
          }
        }
      })
    });
    self.thread = Some(handle);
  }
}
