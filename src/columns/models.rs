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

#[cfg(test)]
mod tests {
  use super::*;
  use chrono::Utc;
  use firestore::FirestoreTimestamp;

  fn column_in_firestore(id: &str, position: Option<i64>) -> ColumnInFirestore {
    ColumnInFirestore {
      _firestore_id: id.to_string(),
      _firestore_created: FirestoreTimestamp(Utc::now()),
      name: "What went well".to_string(),
      created_at: None,
      data: serde_json::Value::Object(serde_json::Map::new()),
      position,
    }
  }

  #[test]
  fn column_message_all_none_uses_defaults() {
    let msg = ColumnMessage {
      name: None,
      data: None,
      position: None,
    };
    let c: NewColumn = msg.into();
    assert_eq!(c.name, "");
    assert_eq!(c.data, serde_json::Value::Object(serde_json::Map::new()));
    assert!(c.position.is_none());
  }

  #[test]
  fn column_message_explicit_values_preserved() {
    let msg = ColumnMessage {
      name: Some("Action Items".to_string()),
      data: Some(serde_json::json!({"color": "blue"})),
      position: Some(3),
    };
    let c: NewColumn = msg.into();
    assert_eq!(c.name, "Action Items");
    assert_eq!(c.data, serde_json::json!({"color": "blue"}));
    assert_eq!(c.position, Some(3));
  }

  #[test]
  fn column_in_firestore_preserves_position() {
    let c: Column = column_in_firestore("col1", Some(2)).into();
    assert_eq!(c.id, "col1");
    assert_eq!(c.name, "What went well");
    assert_eq!(c.position, 2);
  }

  #[test]
  fn column_in_firestore_position_none_defaults_to_zero() {
    let c: Column = column_in_firestore("col2", None).into();
    assert_eq!(c.position, 0);
  }
}
