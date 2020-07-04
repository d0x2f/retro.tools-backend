ALTER TABLE board
  ADD COLUMN data jsonb DEFAULT '{}'::jsonb NOT NULL;