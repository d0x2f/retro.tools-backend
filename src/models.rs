use super::schema::board;
use super::schema::participant;

#[derive(Queryable, Identifiable, Serialize)]
#[table_name = "board"]
pub struct Board {
    pub id: String,        // char(16)
    pub name: String,      // varchar
    pub max_votes: i16,    // tinyint
    pub voting_open: bool, // bool
    pub cards_open: bool,  // bool
}

#[derive(AsChangeset, Deserialize)]
#[table_name = "board"]
pub struct UpdateBoard {
    pub name: Option<String>,
    pub max_votes: Option<i16>,
    pub voting_open: Option<bool>,
    pub cards_open: Option<bool>,
}

#[derive(Insertable, Deserialize)]
#[table_name = "board"]
pub struct NewBoard<'a> {
    pub id: Option<&'a str>,
    pub name: &'a str,
    pub max_votes: Option<i16>,
    pub voting_open: Option<bool>,
    pub cards_open: Option<bool>,
}

#[derive(Queryable, Identifiable, Serialize)]
#[table_name = "participant"]
pub struct Participant {
    pub id: String,       // char(16)
    pub board_id: String, // char(16)
    pub owner: bool,      // bool
}

#[derive(Insertable, Deserialize)]
#[table_name = "participant"]
pub struct NewParticipant<'a> {
    pub id: Option<&'a str>,
    pub board_id: &'a str,
    pub owner: bool,
}
