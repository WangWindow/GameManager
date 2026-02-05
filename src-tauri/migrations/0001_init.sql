-- Initial schema
CREATE TABLE IF NOT EXISTS games (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    engine_type TEXT NOT NULL,
    path TEXT NOT NULL,
    runtime_version TEXT,
    cover_path TEXT,
    created_at INTEGER NOT NULL,
    last_played_at INTEGER
);

CREATE TABLE IF NOT EXISTS engines (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    version TEXT NOT NULL,
    engine_type TEXT NOT NULL,
    path TEXT NOT NULL,
    installed_at INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_games_path ON games(path);
