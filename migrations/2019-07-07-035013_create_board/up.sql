CREATE TABLE board (
  id CHAR(16) PRIMARY KEY DEFAULT random_string(16),
  name VARCHAR NOT NULL DEFAULT 'Untitled',
  max_votes SMALLINT NOT NULL DEFAULT 1,
  voting_open BOOLEAN NOT NULL DEFAULT 'f',
  cards_open BOOLEAN NOT NULL DEFAULT 'f'
);