#![allow(clippy::option_option)]

use std::time::SystemTime;

use serde::{Deserialize, Deserializer};

use super::schema::board;
use super::schema::card;
use super::schema::participant;
use super::schema::participant_board;
use super::schema::rank;
use super::schema::vote;

// Custom deserializer that distinguishes between missing, null or some value.
fn deserialize_some<'de, T, D>(deserializer: D) -> Result<Option<T>, D::Error>
    where T: Deserialize<'de>,
          D: Deserializer<'de>
{
    Deserialize::deserialize(deserializer).map(Some)
}


#[derive(Queryable, Identifiable, Serialize, Deserialize)]
#[table_name = "board"]
pub struct Board {
  pub id: String,              // char(16)
  pub name: String,            // varchar
  pub max_votes: i16,          // tinyint
  pub voting_open: bool,       // bool
  pub cards_open: bool,        // bool
  pub created_at: SystemTime,  // timestamp
  pub owner: bool,             // bool
  pub data: serde_json::Value, // jsonb
}

#[derive(AsChangeset, Serialize, Deserialize)]
#[table_name = "board"]
pub struct UpdateBoard {
  pub name: Option<String>,
  pub max_votes: Option<i16>,
  pub voting_open: Option<bool>,
  pub cards_open: Option<bool>,
  pub data: Option<serde_json::Value>,
}

#[derive(Insertable, Serialize, Deserialize, Default)]
#[table_name = "board"]
pub struct NewBoard<'a> {
  pub id: Option<&'a str>,
  pub name: &'a str,
  pub max_votes: Option<i16>,
  pub voting_open: Option<bool>,
  pub cards_open: Option<bool>,
  pub data: Option<serde_json::Value>,
}

#[derive(Queryable, Identifiable, Serialize)]
#[table_name = "participant"]
pub struct Participant {
  pub id: String, // char(16)
}

#[derive(Queryable, Serialize)]
pub struct ParticipantBoard {
  pub participant_id: String, // char(16)
  pub board_id: String,       // char(16)
  pub owner: bool,            // bool
}

#[derive(Insertable, Deserialize)]
#[table_name = "participant_board"]
pub struct NewParticipantBoard<'a> {
  pub participant_id: &'a str,
  pub board_id: &'a str,
  pub owner: bool,
}

#[derive(Queryable, Identifiable, Serialize, Deserialize)]
#[table_name = "rank"]
pub struct Rank {
  pub id: String,              // char(16)
  pub board_id: String,        // char(16)
  pub name: String,            // varchar
  pub data: serde_json::Value, // jsonb
}

#[derive(AsChangeset, Serialize, Deserialize)]
#[table_name = "rank"]
pub struct UpdateRank {
  pub name: Option<String>,
  pub data: Option<serde_json::Value>,
}

#[derive(Deserialize)]
pub struct PostRank<'a> {
  pub id: Option<&'a str>,
  pub name: String,
  pub data: Option<serde_json::Value>,
}

#[derive(Insertable, Serialize, Deserialize)]
#[table_name = "rank"]
pub struct NewRank<'a> {
  pub id: Option<&'a str>,
  pub board_id: &'a str,
  pub name: &'a str,
  pub data: Option<serde_json::Value>,
}

#[derive(Queryable, Identifiable, Serialize, Deserialize)]
#[table_name = "card"]
pub struct Card {
  pub id: String,             // char(16)
  pub rank_id: String,        // char(16)
  pub name: String,           // varchar
  pub description: String,    // varchar
  pub votes: i64,             // count(*)
  pub voted: bool,            // bool
  pub owner: bool,            // bool
  pub created_at: SystemTime, // timestamp
  pub author: Option<String>, // varchar
}

#[derive(AsChangeset, Serialize, Deserialize)]
#[table_name = "card"]
pub struct UpdateCard {
  pub name: Option<String>,
  pub description: Option<String>,
  pub rank_id: Option<String>,
  #[serde(default, deserialize_with = "deserialize_some")]
  pub author: Option<Option<String>>,
}

#[derive(Deserialize)]
pub struct PostCard {
  pub name: String,
  pub description: String,
  pub author: Option<String>
}

#[derive(Insertable, Serialize, Deserialize)]
#[table_name = "card"]
pub struct NewCard<'a> {
  pub id: Option<&'a str>,
  pub rank_id: &'a str,
  pub name: &'a str,
  pub description: &'a str,
  pub participant_id: &'a str,
  pub author: Option<&'a str>,
}

#[derive(AsChangeset, Queryable, Serialize, Deserialize)]
#[table_name = "vote"]
pub struct Vote {
  pub participant_id: String, // char(16)
  pub card_id: String,        // char(16)
  pub count: i16,             // smallint
}

#[derive(AsChangeset, Serialize)]
#[table_name = "vote"]
pub struct UpdateVote<'a> {
  pub participant_id: &'a str,
  pub card_id: &'a str,
  pub count: i16,
}

#[derive(Insertable, Deserialize)]
#[table_name = "vote"]
pub struct NewVote<'a> {
  pub participant_id: &'a str,
  pub card_id: &'a str,
  pub count: Option<i16>,
}
