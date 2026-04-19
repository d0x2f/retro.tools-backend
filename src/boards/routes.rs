use actix_web::{delete, get, patch, post, web, HttpResponse};
use firestore::FirestoreDb;
use firestore::FirestoreReference;
use futures::future::join;

use super::db;
use super::models::*;
use crate::error::Error;
use crate::participants::db::*;
use crate::participants::models::Participant;

fn check_delete_permission(board: &Board, participant: &FirestoreReference) -> Result<(), Error> {
  if board.owner != *participant {
    return Err(Error::Forbidden);
  }
  Ok(())
}

fn check_update_permission(
  board: &Board,
  participant: &FirestoreReference,
  message: &BoardMessage,
) -> Result<(), Error> {
  let is_owner = board.owner == *participant;
  if !is_owner && !board.anyone_is_owner {
    return Err(Error::Forbidden);
  }
  if !is_owner && message.anyone_is_owner.is_some() {
    return Err(Error::Forbidden);
  }
  Ok(())
}

#[post("boards")]
pub async fn new(
  firestore: web::Data<FirestoreDb>,
  participant: Participant,
  board_message: web::Json<BoardMessage>,
) -> Result<HttpResponse, Error> {
  let mut board_message = board_message.into_inner();
  board_message.voting_open.get_or_insert(true);
  board_message.cards_open.get_or_insert(true);
  let board = db::new(&firestore, &participant, board_message).await?;
  add_participant_board(&firestore, &participant, &board.id).await?;
  Ok(
    HttpResponse::Ok().json(BoardResponse::from_board(
      board,
      &FirestoreReference(
        firestore
          .parent_path("participants", &participant.id)
          .unwrap()
          .into(),
      ),
    )),
  )
}

#[get("boards")]
pub async fn list(
  firestore: web::Data<FirestoreDb>,
  participant: Participant,
) -> Result<HttpResponse, Error> {
  let boards = db::list(&firestore, &participant).await?;
  Ok(
    HttpResponse::Ok().json(
      boards
        .into_iter()
        .map(|board| {
          BoardResponse::from_board(
            board,
            &FirestoreReference(
              firestore
                .parent_path("participants", &participant.id)
                .unwrap()
                .into(),
            ),
          )
        })
        .collect::<Vec<BoardResponse>>(),
    ),
  )
}

#[get("boards/{board_id}")]
pub async fn get(
  firestore: web::Data<FirestoreDb>,
  participant: Participant,
  board_id: web::Path<String>,
) -> Result<HttpResponse, Error> {
  let (register, board) = join(
    add_participant_board(&firestore, &participant, &board_id),
    db::get(&firestore, &board_id),
  )
  .await;
  register?;
  Ok(
    HttpResponse::Ok().json(BoardResponse::from_board(
      board?,
      &FirestoreReference(
        firestore
          .parent_path("participants", &participant.id)
          .unwrap()
          .into(),
      ),
    )),
  )
}

#[patch("boards/{board_id}")]
pub async fn update(
  firestore: web::Data<FirestoreDb>,
  participant: Participant,
  board_id: web::Path<String>,
  board_message: web::Json<BoardMessage>,
) -> Result<HttpResponse, Error> {
  let board = db::get(&firestore, &board_id).await?;
  let participant_reference = FirestoreReference(
    firestore
      .parent_path("participants", &participant.id)
      .unwrap()
      .into(),
  );
  let board_message = board_message.into_inner();
  check_update_permission(&board, &participant_reference, &board_message)?;
  let board = db::update(&firestore, &board_id, board_message).await?;
  Ok(HttpResponse::Ok().json(BoardResponse::from_board(board, &participant_reference)))
}

#[delete("boards/{board_id}")]
pub async fn delete(
  firestore: web::Data<FirestoreDb>,
  participant: Participant,
  board_id: web::Path<String>,
) -> Result<HttpResponse, Error> {
  let board = db::get(&firestore, &board_id).await?;
  let participant_reference = FirestoreReference(
    firestore
      .parent_path("participants", &participant.id)
      .unwrap()
      .into(),
  );
  check_delete_permission(&board, &participant_reference)?;
  db::delete(&firestore, &board_id).await?;
  Ok(HttpResponse::Ok().finish())
}

#[cfg(test)]
mod tests {
  use super::*;
  use chrono::Utc;
  use firestore::FirestoreReference;
  use serde_json::Map;

  fn ref_(s: &str) -> FirestoreReference {
    FirestoreReference(s.to_string())
  }

  fn make_board(owner: &str, anyone_is_owner: bool) -> Board {
    Board {
      id: "board1".to_string(),
      name: "Test".to_string(),
      cards_open: true,
      voting_open: true,
      ice_breaking: "".to_string(),
      created_at: Utc::now().timestamp(),
      owner: ref_(owner),
      anyone_is_owner,
      data: serde_json::Value::Object(Map::new()),
    }
  }

  fn msg(anyone_is_owner: Option<bool>) -> BoardMessage {
    BoardMessage {
      name: None,
      cards_open: None,
      voting_open: None,
      ice_breaking: None,
      data: None,
      anyone_is_owner,
    }
  }

  #[test]
  fn owner_can_delete() {
    let board = make_board("participants/owner", false);
    assert!(check_delete_permission(&board, &ref_("participants/owner")).is_ok());
  }

  #[test]
  fn non_owner_cannot_delete() {
    let board = make_board("participants/owner", false);
    assert!(check_delete_permission(&board, &ref_("participants/other")).is_err());
  }

  #[test]
  fn non_owner_cannot_delete_even_when_anyone_is_owner() {
    let board = make_board("participants/owner", true);
    assert!(check_delete_permission(&board, &ref_("participants/other")).is_err());
  }

  #[test]
  fn owner_can_update_normally() {
    let board = make_board("participants/owner", false);
    assert!(check_update_permission(&board, &ref_("participants/owner"), &msg(None)).is_ok());
  }

  #[test]
  fn non_owner_blocked_when_anyone_is_owner_false() {
    let board = make_board("participants/owner", false);
    assert!(check_update_permission(&board, &ref_("participants/other"), &msg(None)).is_err());
  }

  #[test]
  fn non_owner_allowed_when_anyone_is_owner_true() {
    let board = make_board("participants/owner", true);
    assert!(check_update_permission(&board, &ref_("participants/other"), &msg(None)).is_ok());
  }

  #[test]
  fn non_owner_cannot_toggle_anyone_is_owner_off() {
    let board = make_board("participants/owner", true);
    assert!(
      check_update_permission(&board, &ref_("participants/other"), &msg(Some(false))).is_err()
    );
  }

  #[test]
  fn non_owner_cannot_toggle_anyone_is_owner_on() {
    let board = make_board("participants/owner", false);
    assert!(
      check_update_permission(&board, &ref_("participants/other"), &msg(Some(true))).is_err()
    );
  }

  #[test]
  fn owner_can_toggle_anyone_is_owner_on() {
    let board = make_board("participants/owner", false);
    assert!(
      check_update_permission(&board, &ref_("participants/owner"), &msg(Some(true))).is_ok()
    );
  }

  #[test]
  fn owner_can_toggle_anyone_is_owner_off() {
    let board = make_board("participants/owner", true);
    assert!(
      check_update_permission(&board, &ref_("participants/owner"), &msg(Some(false))).is_ok()
    );
  }
}
