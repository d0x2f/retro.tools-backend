mod db;
pub mod models;
pub mod routes;

use std::collections::HashMap;

use firestore::FirestoreDb;

use crate::error::Error;

pub async fn get_columns(
  firestore: &FirestoreDb,
  board_id: String,
) -> Result<HashMap<String, models::Column>, Error> {
  let columns = db::list(firestore, board_id.to_string()).await?;
  let mut map = HashMap::new();
  for column in columns {
    map.insert(column.id.clone(), column);
  }
  Ok(map)
}
