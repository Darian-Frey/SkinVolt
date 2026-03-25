CREATE TABLE IF NOT EXISTS inventory (
    market_hash_name TEXT PRIMARY KEY,
    quantity INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS price_cache (
    market_hash_name TEXT PRIMARY KEY,
    price REAL NOT NULL,
    timestamp INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_price_cache_timestamp
    ON price_cache (timestamp);

CREATE TABLE IF NOT EXISTS price_history (
    market_hash_name TEXT PRIMARY KEY,
    history_json TEXT NOT NULL,
    last_updated INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);

INSERT OR IGNORE INTO settings (key, value) VALUES
    ('refresh_interval', '60'),
    ('currency', 'USD'),
    ('dark_mode', 'false');

CREATE TABLE IF NOT EXISTS logs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    message TEXT NOT NULL,
    context TEXT,
    timestamp INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_logs_timestamp
    ON logs (timestamp);
