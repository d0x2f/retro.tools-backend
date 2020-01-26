ALTER TABLE rank
  ADD COLUMN data jsonb DEFAULT '{}'::jsonb NOT NULL;