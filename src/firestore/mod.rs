#[macro_use]
pub mod macros;

use settimeout::set_timeout;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tonic::{
  metadata::MetadataValue,
  transport::{Channel, ClientTlsConfig},
  Request,
};

use crate::error::Error;

pub mod google {
  pub mod firestore {
    pub mod v1 {
      tonic::include_proto!("google.firestore.v1");
    }
    pub mod v1beta1 {
      tonic::include_proto!("google.firestore.v1beta1");
    }
  }
  pub mod rpc {
    tonic::include_proto!("google.rpc");
  }
  pub mod r#type {
    tonic::include_proto!("google.r#type");
  }
}

pub use google::firestore::*;
pub type FirestoreV1Client = google::firestore::v1::firestore_client::FirestoreClient<Channel>;

const URL: &str = "https://firestore.googleapis.com";
const DOMAIN: &str = "firestore.googleapis.com";
const SCOPE: &str = "https://www.googleapis.com/auth/datastore";

// TODO: This is a mess
pub async fn get_client() -> Result<FirestoreV1Client, Error> {
  let tls = ClientTlsConfig::new().domain_name(DOMAIN);

  let channel = Channel::from_static(URL).tls_config(tls)?.connect().await?;

  let authentication_manager = Arc::new(Mutex::new(gcp_auth::init().await?));
  let token = Arc::new(Mutex::new(
    authentication_manager
      .lock()
      .expect("mutex lock")
      .get_token(&[SCOPE])
      .await?,
  ));
  let authentication_manager_ref = authentication_manager.clone();
  let token_ref = token.clone();

  actix_rt::spawn(async move {
    loop {
      set_timeout(Duration::from_secs(10)).await;
      if let Ok(t) = authentication_manager_ref
        .lock()
        .expect("mutex lock")
        .get_token(&[SCOPE])
        .await
      {
        let mut contents = token_ref.lock().expect("mutex lock");
        *contents = t;
      }
    }
  });

  let client = FirestoreV1Client::with_interceptor(channel, move |mut req: Request<()>| {
    let header_string = format!("Bearer {}", token.lock().expect("mutex lock").as_str());
    let header_value = MetadataValue::from_str(&header_string).expect("parsed metadata string");
    req.metadata_mut().insert("authorization", header_value);
    Ok(req)
  });
  Ok(client)
}
