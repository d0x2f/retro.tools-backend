CREATE TABLE participant_board (
  participant_id CHAR(16) REFERENCES participant ON DELETE CASCADE NOT NULL,
  board_id CHAR(16) REFERENCES board ON DELETE CASCADE NOT NULL,
  owner BOOLEAN NOT NULL DEFAULT 'f',
  PRIMARY KEY (participant_id, board_id)
)