
ALTER TABLE card
  ADD COLUMN participant_id CHAR(16) REFERENCES participant ON DELETE CASCADE NULL,
  ADD COLUMN created_at TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP;

UPDATE card SET participant_id = (
  SELECT
    participant_board.participant_id
  FROM
    rank
    JOIN participant_board ON participant_board.board_id = rank.board_id
  WHERE
    rank.id = card.rank_id AND
    participant_board.owner = true
);

ALTER TABLE card ALTER COLUMN
  participant_id SET NOT NULL;