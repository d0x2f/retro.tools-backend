CREATE TABLE participant (
  id CHAR(16) PRIMARY KEY DEFAULT random_string(16),
  owner BOOLEAN NOT NULL DEFAULT 'f',
  board_id CHAR(16) REFERENCES board NOT NULL
)