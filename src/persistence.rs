use super::models::*;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel::result::Error;

pub fn create_connection(database_url: String) -> Result<PgConnection, String> {
    PgConnection::establish(&database_url).map_err(|error| error.to_string())
}

pub fn put_board(postgres: &PgConnection, new_board: &NewBoard) -> Result<Board, Error> {
    use super::schema::board;

    diesel::insert_into(board::table)
        .values(new_board)
        .get_result(postgres)
}

pub fn get_boards(postgres: &PgConnection) -> Result<Vec<Board>, Error> {
    use super::schema::board::dsl::*;

    board.load::<Board>(postgres)
}

pub fn get_board(postgres: &PgConnection, board_id: String) -> Result<Vec<Board>, Error> {
    use super::schema::board::dsl::*;

    board
        .filter(id.eq(board_id))
        .limit(1)
        .load::<Board>(postgres)
}

pub fn patch_board(postgres: &PgConnection, board_id: String, update_board: &UpdateBoard) -> Result<Board, Error> {
    use super::schema::board::dsl::*;

    diesel::update(board.find(board_id)).set(update_board).get_result(postgres)
}

pub fn delete_board(postgres: &PgConnection, board_id: String) -> Result<usize, Error> {
    use super::schema::board::dsl::*;

    diesel::delete(board.find(board_id)).execute(postgres)
}
