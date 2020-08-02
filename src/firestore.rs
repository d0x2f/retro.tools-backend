pub type BoxError = Box<dyn std::error::Error + Sync + Send + 'static>;
use std::env;
use tonic::{
  metadata::MetadataValue,
  transport::{Channel, ClientTlsConfig},
  Request,
};

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

pub async fn get_client() -> Result<FirestoreV1Client, BoxError> {
  let tls = ClientTlsConfig::new().domain_name(DOMAIN);

  let channel = Channel::from_static(URL).tls_config(tls)?.connect().await?;

  let token = match env::var("FIRESTORE_TOKEN") {
    Err(_) => panic!("No firestore token provided"),
    Ok(s) => s,
  };

  let bearer_token = format!("Bearer {}", token);
  let header_value = MetadataValue::from_str(&bearer_token)?;
  let client = FirestoreV1Client::with_interceptor(channel, move |mut req: Request<()>| {
    req
      .metadata_mut()
      .insert("authorization", header_value.clone());
    Ok(req)
  });
  Ok(client)
}

#[macro_export]
macro_rules! to_participant_reference {
  ($project:expr, $id:expr) => {
    format!("projects/{}/databases/(default)/documents/participants/{}", $project, $id)
  };
}

#[macro_export]
macro_rules! to_board_reference {
  ($project:expr, $id:expr) => {
    format!("projects/{}/databases/(default)/documents/boards/{}", $project, $id)
  };
}

#[macro_export]
macro_rules! from_reference {
  ($reference:expr) => {
    $reference.rsplitn(2, '/').next().expect("document id")
  };
}

#[macro_export]
macro_rules! get_id {
  ($document:expr) => {
    from_reference!($document.name).into()
  };
}

#[macro_export]
macro_rules! get_create_time {
  ($document:expr) => {
    $document
      .create_time
      .ok_or_else(|| crate::error::Error::Other(
        "field `create_time` not set in document.".into(),
      ))?
      .seconds
  };
}

#[macro_export]
macro_rules! get_reference_field {
  ($document:expr, $field:literal) => {
    match $document
      .fields
      .get($field)
      .and_then(|field| field.value_type.as_ref())
    {
      Some(crate::firestore::v1::value::ValueType::ReferenceValue(s)) => Ok(s.to_string()),
      _ => Err(crate::error::Error::Other(format!(
        "field `{}` not set in document.",
        $field
      ))),
    }
  };
}

#[macro_export]
macro_rules! get_string_field {
  ($document:expr, $field:literal) => {
    match $document
      .fields
      .get($field)
      .and_then(|field| field.value_type.as_ref())
    {
      Some(crate::firestore::v1::value::ValueType::StringValue(s)) => Ok(s.clone()),
      _ => Err(crate::error::Error::Other(format!(
        "field `{}` not set in document.",
        $field
      ))),
    }
  };
}

#[macro_export]
macro_rules! get_boolean_field {
  ($document:expr, $field:literal) => {
    match $document
      .fields
      .get($field)
      .and_then(|field| field.value_type.as_ref())
    {
      Some(crate::firestore::v1::value::ValueType::BooleanValue(b)) => Ok(*b),
      _ => Err(crate::error::Error::Other(format!(
        "field `{}` not set in document.",
        $field
      ))),
    }
  };
}

#[macro_export]
macro_rules! reference_value {
  ($document_path:expr) => {
    Value {
      value_type: Some(crate::firestore::v1::value::ValueType::ReferenceValue($document_path))
    }
  };
}

#[macro_export]
macro_rules! string_value {
  ($string:expr) => {
    Value {
      value_type: Some(crate::firestore::v1::value::ValueType::StringValue(
        $string.into(),
      )),
    }
  };
}

#[macro_export]
macro_rules! boolean_value {
  ($bool:expr) => {
    Value {
      value_type: Some(crate::firestore::v1::value::ValueType::BooleanValue($bool)),
    }
  };
}
