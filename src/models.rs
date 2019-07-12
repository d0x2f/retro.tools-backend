use super::schema::board;

#[derive(Queryable, Identifiable, Serialize)]
#[table_name = "board"]
pub struct Board {
    id: String,          // char(16)
    name: String,        // varchar
    max_votes: i16,      // tinyint
    voting_open: bool,   // bool
    cards_open: bool,    // bool
}

#[derive(AsChangeset, Deserialize)]
#[table_name = "board"]
pub struct UpdateBoard {
    name: Option<String>,
    max_votes: Option<i16>,
    voting_open: Option<bool>,
    cards_open: Option<bool>,
}

#[derive(Insertable, Deserialize)]
#[table_name = "board"]
pub struct NewBoard<'a> {
    pub name: &'a str,
    pub max_votes: Option<i16>,
    pub voting_open: Option<bool>,
    pub cards_open: Option<bool>,
}
