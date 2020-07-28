pub type BoxError = Box<dyn std::error::Error + Sync + Send + 'static>;
use tonic::{
    metadata::MetadataValue,
    transport::{Channel, ClientTlsConfig},
    Request,
};
use std::env;

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

const URL: &'static str = "https://firestore.googleapis.com";
const DOMAIN: &'static str = "firestore.googleapis.com";

pub async fn get_client() -> Result<FirestoreV1Client, BoxError> {
    let tls = ClientTlsConfig::new()
        .domain_name(DOMAIN);

    let channel = Channel::from_static(URL)
        .tls_config(tls)?
        .connect()
        .await?;

    let token = match env::var("FIRESTORE_TOKEN") {
      Err(_) => panic!("No firestore token provided"),
      Ok(s) => s
    };

    let bearer_token = format!("Bearer {}", token);
    let header_value = MetadataValue::from_str(&bearer_token)?;
    let client = FirestoreV1Client::with_interceptor(
      channel,
      move |mut req: Request<()>| {
        req.metadata_mut()
            .insert("authorization", header_value.clone());
        Ok(req)
      }
    );
    Ok(client)
}

#[macro_export]
macro_rules! get_string_field {
  ($document:expr, $field:literal) => {
    match $document.fields.get($field).and_then(|field| field.value_type.as_ref()) {
      Some(crate::firestore::v1::value::ValueType::StringValue(s)) => Some(s.clone()),
      _ => None
    }
  };
}

#[macro_export]
macro_rules! get_boolean_field {
  ($document:expr, $field:literal) => {
    match $document.fields.get($field).and_then(|field| field.value_type.as_ref()) {
      Some(crate::firestore::v1::value::ValueType::BooleanValue(b)) => Some(*b),
      _ => None
    }
  };
}
