use chrono::Utc;
use firestore::FirestoreTimestamp;
use serde::{Deserialize, Serialize};
use serde_json::Map;

#[derive(Deserialize, Serialize)]
pub struct ColumnMessage {
  #[serde(skip_serializing_if = "Option::is_none")]
  pub name: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub data: Option<serde_json::Value>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub position: Option<i64>,
}

#[derive(Deserialize, Serialize)]
pub struct Column {
  pub id: String,
  pub name: String,
  pub created_at: i64,
  pub data: serde_json::Value,
  pub position: i64,
}

#[derive(Deserialize, Serialize)]
pub struct NewColumn {
  pub name: String,
  pub created_at: FirestoreTimestamp,
  pub data: serde_json::Value,
  pub position: Option<i64>,
}

#[derive(Deserialize, Serialize)]
pub struct ColumnInFirestore {
  pub _firestore_id: String,
  pub _firestore_created: FirestoreTimestamp,
  pub name: String,
  pub created_at: Option<FirestoreTimestamp>,
  pub data: serde_json::Value,
  pub position: Option<i64>,
}

impl From<ColumnMessage> for NewColumn {
  fn from(column: ColumnMessage) -> Self {
    NewColumn {
      name: column.name.unwrap_or_else(|| "".into()),
      created_at: FirestoreTimestamp(Utc::now()),
      position: column.position,
      data: column
        .data
        .unwrap_or_else(|| serde_json::Value::Object(Map::new())),
    }
  }
}

impl From<ColumnInFirestore> for Column {
  fn from(column: ColumnInFirestore) -> Self {
    Column {
      id: column._firestore_id,
      name: column.name,
      position: column.position.unwrap_or(0),
      created_at: column
        .created_at
        .unwrap_or(column._firestore_created)
        .0
        .timestamp(),
      data: column.data,
    }
  }
}
