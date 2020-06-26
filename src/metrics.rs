use rocket_prometheus::prometheus::{register_int_counter, IntCounter};

lazy_static! {
  pub static ref PARTICIPANT_COUNT: IntCounter = register_int_counter!(
    "retrograde_participants_total",
    "The number of participants."
  )
  .unwrap();
  pub static ref BOARDS_COUNT: IntCounter =
    register_int_counter!("retrograde_boards_total", "The number of board creations.").unwrap();
  pub static ref BOARD_PARTICIPANT_COUNT: IntCounter = register_int_counter!(
    "retrograde_board_participants_total",
    "The number of participants who joined a board."
  )
  .unwrap();
  pub static ref RANK_COUNT: IntCounter =
    register_int_counter!("retrograde_ranks_total", "The number of ranks.").unwrap();
  pub static ref CARD_COUNT: IntCounter =
    register_int_counter!("retrograde_cards_total", "The number of cards.").unwrap();
}
