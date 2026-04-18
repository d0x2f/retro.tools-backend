use chrono::Utc;
use firestore::{FirestoreReference, FirestoreTimestamp};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::convert::TryFrom;

use crate::columns::models::Column;
use crate::error::Error;

#[derive(Deserialize, Serialize)]
pub struct CardMessage {
  pub author: Option<String>,
  pub text: Option<String>,
  pub column: Option<String>,
}

#[derive(Deserialize, Serialize)]
pub struct CardChangeSet {
  #[serde(skip_serializing_if = "Option::is_none")]
  pub author: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub text: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub column: Option<FirestoreReference>,
}

#[derive(Deserialize, Serialize)]
pub struct ReactMessage {
  pub emoji: String,
}

#[derive(Deserialize, Serialize)]
pub struct Card {
  pub id: String,
  pub column: FirestoreReference,
  pub owner: FirestoreReference,
  pub author: String,
  pub text: String,
  pub created_at: i64,
  pub votes: Vec<String>,
  pub reactions: HashMap<String, Vec<String>>,
}

#[derive(Deserialize, Serialize)]
pub struct CardResponse {
  pub id: String,
  pub column: String,
  pub owner: bool,
  pub author: String,
  pub text: String,
  pub created_at: i64,
  pub votes: usize,
  pub voted: bool,
  pub reactions: HashMap<String, usize>,
  pub reacted: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct NewCard {
  pub created_at: FirestoreTimestamp,
  pub column: FirestoreReference,
  pub owner: Option<FirestoreReference>,
  pub author: String,
  pub text: String,
}

#[derive(Deserialize, Serialize)]
pub struct CardInFirestore {
  pub _firestore_id: String,
  pub _firestore_created: FirestoreTimestamp,
  pub created_at: Option<FirestoreTimestamp>,
  pub author: String,
  pub text: String,
  pub owner: FirestoreReference,
  pub column: FirestoreReference,
  pub votes: Option<Vec<String>>,
  pub reactions: Option<HashMap<String, Vec<String>>>,
}

#[derive(Serialize)]
pub struct CardCSVRow {
  pub column: String,
  pub author: String,
  pub text: String,
  pub created_at: i64,
  pub votes: usize,
}

impl TryFrom<CardMessage> for NewCard {
  type Error = Error;

  fn try_from(card: CardMessage) -> Result<Self, Self::Error> {
    Ok(NewCard {
      author: card.author.unwrap_or("".into()),
      text: card.text.unwrap_or("".into()),
      created_at: FirestoreTimestamp(Utc::now()),
      owner: None,
      column: FirestoreReference(
        card
          .column
          .ok_or(Error::BadRequest("column is required".into()))?,
      ),
    })
  }
}

impl From<CardInFirestore> for Card {
  fn from(card: CardInFirestore) -> Self {
    Card {
      id: card._firestore_id,
      created_at: card
        .created_at
        .unwrap_or(card._firestore_created)
        .0
        .timestamp(),
      owner: card.owner,
      column: card.column,
      author: card.author,
      text: card.text,
      votes: card.votes.unwrap_or_default(),
      reactions: card.reactions.unwrap_or_default(),
    }
  }
}

impl CardCSVRow {
  pub fn from_card(card: Card, columns: &HashMap<String, Column>) -> CardCSVRow {
    CardCSVRow {
      column: match columns.get(&card.column.0.split('/').next_back().unwrap().to_string()) {
        Some(column) => column
          .name
          .clone()
          .split('.')
          .next_back()
          .unwrap_or(&column.name)
          .into(),
        _ => "Unknown Column".into(),
      },
      author: card.author,
      text: card.text,
      created_at: card.created_at,
      votes: card.votes.len(),
    }
  }
}

impl CardResponse {
  pub fn from_card(card: Card, participant_id: &FirestoreReference) -> CardResponse {
    CardResponse {
      id: card.id,
      column: card.column.0.split('/').next_back().unwrap().to_string(),
      owner: &card.owner == participant_id,
      author: card.author,
      text: card.text,
      created_at: card.created_at,
      votes: card.votes.len(),
      voted: card.votes.contains(&participant_id.0),
      reactions: card
        .reactions
        .clone()
        .into_iter()
        .map(|(k, v)| (k, v.len()))
        .collect(),
      reacted: {
        let mut reaction: Option<String> = None;
        for (emoji, participants) in &card.reactions {
          if participants.contains(&participant_id.0) {
            reaction = Some(emoji.to_string());
            break;
          }
        }
        match reaction {
          Some(emoji) => emoji,
          None => "".into(),
        }
      },
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use chrono::Utc;
  use firestore::{FirestoreReference, FirestoreTimestamp};

  use crate::columns::models::Column;

  fn ref_(path: &str) -> FirestoreReference {
    FirestoreReference(path.to_string())
  }

  fn make_card(id: &str, owner: &str, column: &str) -> Card {
    Card {
      id: id.to_string(),
      column: ref_(column),
      owner: ref_(owner),
      author: "Alice".to_string(),
      text: "Test card".to_string(),
      created_at: 1_000_000,
      votes: vec![],
      reactions: HashMap::new(),
    }
  }

  fn make_column(id: &str, name: &str) -> Column {
    Column {
      id: id.to_string(),
      name: name.to_string(),
      created_at: 1_000_000,
      data: serde_json::Value::Object(serde_json::Map::new()),
      position: 0,
    }
  }

  // --- TryFrom<CardMessage> ---

  #[test]
  fn try_from_card_message_missing_column_is_bad_request() {
    let msg = CardMessage { author: Some("Alice".into()), text: Some("text".into()), column: None };
    let err = NewCard::try_from(msg).unwrap_err();
    match err {
      crate::error::Error::BadRequest(s) => assert_eq!(s, "column is required"),
      other => panic!("expected BadRequest, got {other:?}"),
    }
  }

  #[test]
  fn try_from_card_message_with_column_succeeds() {
    let msg = CardMessage {
      author: Some("Alice".into()),
      text: Some("My text".into()),
      column: Some("boards/b1/columns/col1".into()),
    };
    let new_card = NewCard::try_from(msg).unwrap();
    assert_eq!(new_card.author, "Alice");
    assert_eq!(new_card.text, "My text");
    assert_eq!(new_card.column.0, "boards/b1/columns/col1");
  }

  #[test]
  fn try_from_card_message_none_author_and_text_default_to_empty() {
    let msg = CardMessage { author: None, text: None, column: Some("boards/b1/columns/col1".into()) };
    let new_card = NewCard::try_from(msg).unwrap();
    assert_eq!(new_card.author, "");
    assert_eq!(new_card.text, "");
  }

  // --- From<CardInFirestore> ---

  #[test]
  fn card_in_firestore_votes_and_reactions_default_to_empty() {
    let raw = CardInFirestore {
      _firestore_id: "c1".into(),
      _firestore_created: FirestoreTimestamp(Utc::now()),
      created_at: None,
      author: "Bob".into(),
      text: "Hello".into(),
      owner: ref_("participants/user1"),
      column: ref_("boards/b1/columns/col1"),
      votes: None,
      reactions: None,
    };
    let card: Card = raw.into();
    assert!(card.votes.is_empty());
    assert!(card.reactions.is_empty());
  }

  // --- CardResponse ---

  #[test]
  fn card_response_owner_true_when_refs_match() {
    let participant = ref_("participants/user1");
    let card = make_card("c1", "participants/user1", "boards/b1/columns/col1");
    assert!(CardResponse::from_card(card, &participant).owner);
  }

  #[test]
  fn card_response_owner_false_when_refs_differ() {
    let participant = ref_("participants/user2");
    let card = make_card("c1", "participants/user1", "boards/b1/columns/col1");
    assert!(!CardResponse::from_card(card, &participant).owner);
  }

  #[test]
  fn card_response_voted_true_when_participant_in_votes() {
    let participant = ref_("participants/user1");
    let mut card = make_card("c1", "participants/user1", "boards/b1/columns/col1");
    card.votes = vec!["participants/user1".into(), "participants/user2".into()];
    let resp = CardResponse::from_card(card, &participant);
    assert!(resp.voted);
    assert_eq!(resp.votes, 2);
  }

  #[test]
  fn card_response_voted_false_when_participant_not_in_votes() {
    let participant = ref_("participants/user3");
    let mut card = make_card("c1", "participants/user1", "boards/b1/columns/col1");
    card.votes = vec!["participants/user1".into()];
    let resp = CardResponse::from_card(card, &participant);
    assert!(!resp.voted);
    assert_eq!(resp.votes, 1);
  }

  #[test]
  fn card_response_reacted_returns_emoji_when_participant_reacted() {
    let participant = ref_("participants/user1");
    let mut card = make_card("c1", "participants/user1", "boards/b1/columns/col1");
    card.reactions.insert("👍".into(), vec!["participants/user1".into()]);
    let resp = CardResponse::from_card(card, &participant);
    assert_eq!(resp.reacted, "👍");
  }

  #[test]
  fn card_response_reacted_empty_when_participant_has_not_reacted() {
    let participant = ref_("participants/user2");
    let mut card = make_card("c1", "participants/user1", "boards/b1/columns/col1");
    card.reactions.insert("👍".into(), vec!["participants/user1".into()]);
    let resp = CardResponse::from_card(card, &participant);
    assert_eq!(resp.reacted, "");
  }

  #[test]
  fn card_response_column_extracts_last_path_segment() {
    let participant = ref_("participants/user1");
    let card = make_card(
      "c1",
      "participants/user1",
      "projects/p/databases/d/documents/boards/b1/columns/col99",
    );
    assert_eq!(CardResponse::from_card(card, &participant).column, "col99");
  }

  #[test]
  fn card_response_reactions_counts_participants_per_emoji() {
    let participant = ref_("participants/user1");
    let mut card = make_card("c1", "participants/user1", "boards/b1/columns/col1");
    card.reactions.insert("👍".into(), vec!["u1".into(), "u2".into(), "u3".into()]);
    card.reactions.insert("❤️".into(), vec!["u1".into()]);
    let resp = CardResponse::from_card(card, &participant);
    assert_eq!(resp.reactions["👍"], 3);
    assert_eq!(resp.reactions["❤️"], 1);
  }

  // --- CardCSVRow ---

  #[test]
  fn card_csv_row_known_column_uses_column_name() {
    let card = make_card("c1", "participants/user1", "boards/b1/columns/col1");
    let mut columns = HashMap::new();
    columns.insert("col1".into(), make_column("col1", "What went well"));
    assert_eq!(CardCSVRow::from_card(card, &columns).column, "What went well");
  }

  #[test]
  fn card_csv_row_unknown_column_falls_back_to_unknown() {
    let card = make_card("c1", "participants/user1", "boards/b1/columns/col_missing");
    let columns: HashMap<String, Column> = HashMap::new();
    assert_eq!(CardCSVRow::from_card(card, &columns).column, "Unknown Column");
  }

  #[test]
  fn card_csv_row_dotted_column_name_uses_last_segment() {
    let card = make_card("c1", "participants/user1", "boards/b1/columns/col1");
    let mut columns = HashMap::new();
    columns.insert("col1".into(), make_column("col1", "category.went_well"));
    assert_eq!(CardCSVRow::from_card(card, &columns).column, "went_well");
  }

  #[test]
  fn card_csv_row_votes_count_matches_votes_vec_length() {
    let mut card = make_card("c1", "participants/user1", "boards/b1/columns/col1");
    card.votes = vec!["u1".into(), "u2".into(), "u3".into()];
    let columns: HashMap<String, Column> = HashMap::new();
    assert_eq!(CardCSVRow::from_card(card, &columns).votes, 3);
  }

  // --- Column path edge cases ---

  #[test]
  fn card_response_column_plain_id_with_no_slashes() {
    let participant = ref_("participants/user1");
    let card = make_card("c1", "participants/user1", "col1");
    assert_eq!(CardResponse::from_card(card, &participant).column, "col1");
  }

  #[test]
  fn card_csv_row_column_plain_id_with_no_slashes() {
    let card = make_card("c1", "participants/user1", "col1");
    let mut columns = HashMap::new();
    columns.insert("col1".into(), make_column("col1", "Went Well"));
    assert_eq!(CardCSVRow::from_card(card, &columns).column, "Went Well");
  }

  // --- reacted: put_reaction enforces one reaction per participant, so a participant
  //     appears in at most one emoji bucket. The break ensures we stop at the first match
  //     rather than iterating the whole map unnecessarily.

  #[test]
  fn card_response_reacted_stops_at_first_match() {
    let participant = ref_("participants/user1");
    let mut card = make_card("c1", "participants/user1", "boards/b1/columns/col1");
    // Under normal operation put_reaction removes any prior reaction before adding a new
    // one, so a participant appears in exactly one bucket. Seed a single reaction to
    // confirm the fast-path returns it correctly.
    card.reactions.insert("🎉".into(), vec!["participants/user1".into()]);
    let resp = CardResponse::from_card(card, &participant);
    assert_eq!(resp.reacted, "🎉");
  }
}
