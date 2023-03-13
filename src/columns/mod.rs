mod db;
pub mod models;
pub mod routes;

use std::collections::HashMap;

use crate::config::Config;
use crate::error::Error;
use crate::firestore::FirestoreV1Client;

pub async fn get_columns(
  firestore: &mut FirestoreV1Client,
  config: &Config,
  board_id: String,
) -> Result<HashMap<String, models::Column>, Error> {
  let columns = db::list(firestore, config, board_id.to_string()).await?;
  let mut map = HashMap::new();
  for column in columns {
    map.insert(column.id.clone(), column);
  }
  Ok(map)
}
