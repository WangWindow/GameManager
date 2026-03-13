-- Optimize games schema for classification, caching and future extension
ALTER TABLE games ADD COLUMN normalized_path TEXT;
ALTER TABLE games ADD COLUMN game_type TEXT NOT NULL DEFAULT 'unknown';
ALTER TABLE games ADD COLUMN detection_confidence INTEGER NOT NULL DEFAULT 0;
ALTER TABLE games ADD COLUMN play_count INTEGER NOT NULL DEFAULT 0;
ALTER TABLE games ADD COLUMN metadata_json TEXT;
ALTER TABLE games ADD COLUMN updated_at INTEGER NOT NULL DEFAULT 0;

UPDATE games
SET normalized_path = path
WHERE normalized_path IS NULL OR normalized_path = '';

UPDATE games
SET updated_at = CASE
	WHEN last_played_at IS NOT NULL THEN last_played_at
	ELSE created_at
END
WHERE updated_at = 0;

CREATE UNIQUE INDEX IF NOT EXISTS idx_games_normalized_path ON games(normalized_path);
CREATE INDEX IF NOT EXISTS idx_games_type ON games(game_type);
CREATE INDEX IF NOT EXISTS idx_games_updated_at ON games(updated_at);
