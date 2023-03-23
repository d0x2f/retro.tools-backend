use firestore::paths;
use firestore::FirestoreDb;
use futures::stream::BoxStream;
use futures::StreamExt;

use super::models::*;
use crate::error::Error;

pub async fn new(
  firestore: &FirestoreDb,
  board_id: &String,
  column: ColumnMessage,
) -> Result<Column, Error> {
  let new_column: NewColumn = column.into();
  firestore
    .fluent()
    .insert()
    .into("columns")
    .generate_document_id()
    .parent(firestore.parent_path("boards", board_id)?)
    .object(&new_column)
    .execute::<ColumnInFirestore>()
    .await
    .map(|column| column.into())
    .map_err(|e| e.into())
}

pub async fn list(firestore: &FirestoreDb, board_id: &String) -> Result<Vec<Column>, Error> {
  let mut object_stream: BoxStream<Option<ColumnInFirestore>> = firestore
    .fluent()
    .list()
    .from("columns")
    .parent(firestore.parent_path("boards", board_id)?)
    .obj::<Option<ColumnInFirestore>>()
    .stream_all()
    .await?;

  let mut columns: Vec<Column> = vec![];
  while let Some(Some(column)) = object_stream.next().await {
    columns.push(column.into());
  }
  Ok(columns)
}

pub async fn get(
  firestore: &FirestoreDb,
  board_id: &String,
  column_id: &String,
) -> Result<Column, Error> {
  firestore
    .fluent()
    .select()
    .by_id_in("columns")
    .parent(firestore.parent_path("boards", board_id)?)
    .obj::<ColumnInFirestore>()
    .one(column_id)
    .await?
    .ok_or(Error::NotFound)
    .map(|column| column.into())
}

pub async fn update(
  firestore: &FirestoreDb,
  board_id: &String,
  column_id: &String,
  column: ColumnMessage,
) -> Result<Column, Error> {
  let serialised_column = serde_json::to_value(&column)?;
  firestore
    .fluent()
    .update()
    .fields(
      paths!(ColumnMessage::{name, position, data})
        .into_iter()
        .filter(|f| serialised_column.get(f).is_some()),
    )
    .in_col("columns")
    .document_id(column_id)
    .parent(firestore.parent_path("boards", board_id)?)
    .object(&column)
    .execute::<ColumnInFirestore>()
    .await
    .map(|column| column.into())
    .map_err(|e| e.into())
}

pub async fn delete(
  firestore: &FirestoreDb,
  board_id: &String,
  column_id: &String,
) -> Result<(), Error> {
  firestore
    .fluent()
    .delete()
    .from("columns")
    .document_id(column_id)
    .parent(firestore.parent_path("boards", board_id)?)
    .execute()
    .await
    .map_err(|e| e.into())
}
