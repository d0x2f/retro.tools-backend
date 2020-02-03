use rocket_prometheus::{
  prometheus::IntCounter
};

lazy_static! {
  pub static ref PARTICIPANT_COUNT: IntCounter = IntCounter::new(
    "retrograde_participant_count",
    "The number of participants."
  )
  .unwrap();
  pub static ref BOARDS_COUNT: IntCounter = IntCounter::new(
    "retrograde_board_count",
    "The number of board creations."
  )
  .unwrap();
  pub static ref BOARD_PARTICIPANT_COUNT: IntCounter = IntCounter::new(
    "retrograde_board_participant_count",
    "The number of participants who joined a board."
  )
  .unwrap();
  pub static ref RANK_COUNT: IntCounter = IntCounter::new(
    "retrograde_rank_count",
    "The number of ranks."
  )
  .unwrap();
  pub static ref CARD_COUNT: IntCounter = IntCounter::new(
    "retrograde_card_count",
    "The number of cards."
  )
  .unwrap();
}
