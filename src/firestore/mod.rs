#[macro_use]
pub mod macros;

use crate::error::Error;

use gcp_auth::{AuthenticationManager, Token};
use tonic::{
  metadata::MetadataValue,
  service::{interceptor::InterceptedService, Interceptor},
  transport::{Channel, ClientTlsConfig},
  Status,
};

#[allow(warnings)]
pub mod google {
  pub mod firestore {
    pub mod v1 {
      tonic::include_proto!("google.firestore.v1");
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
pub type FirestoreV1Client = google::firestore::v1::firestore_client::FirestoreClient<
  InterceptedService<Channel, InsertAuthInterceptor>,
>;

const URL: &str = "https://firestore.googleapis.com";
const DOMAIN: &str = "firestore.googleapis.com";
const SCOPE: &str = "https://www.googleapis.com/auth/datastore";

#[derive(Clone)]
pub struct InsertAuthInterceptor {
  token: Token,
}

impl Interceptor for InsertAuthInterceptor {
  fn call(&mut self, mut request: tonic::Request<()>) -> Result<tonic::Request<()>, Status> {
    let token_str = self.token.as_str();
    let header_string = format!("Bearer {}", token_str);
    let header_value: MetadataValue<_> = header_string.try_into().expect("parsed metadata string");
    request.metadata_mut().insert("authorization", header_value);
    Ok(request)
  }
}

pub async fn get_client(gcp_auth: AuthenticationManager) -> Result<FirestoreV1Client, Error> {
  let tls = ClientTlsConfig::new().domain_name(DOMAIN);
  let channel = Channel::from_static(URL).tls_config(tls)?.connect().await;
  let client = google::firestore::v1::firestore_client::FirestoreClient::with_interceptor(
    channel?,
    InsertAuthInterceptor {
      token: gcp_auth
        .get_token(&[SCOPE])
        .await
        .map_err(|e| Status::internal(e.to_string()))?,
    },
  );
  Ok(client)
}
