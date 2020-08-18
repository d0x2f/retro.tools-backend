#[macro_use]
pub mod macros;

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

pub async fn get_client() -> Result<FirestoreV1Client, Error> {
  let tls = ClientTlsConfig::new().domain_name(DOMAIN);

  let channel = Channel::from_static(URL).tls_config(tls)?.connect().await?;
  let token_manager = gcp_auth::init().await?;
  let token = token_manager.get_token(&[SCOPE]).await?;
  let token_str = token.as_str();
  let header_string = format!("Bearer {}", token_str);
  let header_value = MetadataValue::from_str(&header_string).expect("parsed metadata string");

  let client = FirestoreV1Client::with_interceptor(channel, move |mut req: Request<()>| {
    req
      .metadata_mut()
      .insert("authorization", header_value.clone());
    Ok(req)
  });
  Ok(client)
}
