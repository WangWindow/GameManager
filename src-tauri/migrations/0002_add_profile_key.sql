-- Add readable profile directory key
ALTER TABLE games ADD COLUMN profile_key TEXT;

UPDATE games
SET profile_key = id
WHERE profile_key IS NULL OR profile_key = '';

CREATE INDEX IF NOT EXISTS idx_games_profile_key ON games(profile_key);
