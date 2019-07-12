CREATE TABLE participant (
  id CHAR(16) DEFAULT random_string(16),
  board_id CHAR(16) REFERENCES board NOT NULL,
  owner BOOLEAN NOT NULL DEFAULT 'f',
  PRIMARY KEY (id, board_id)
)