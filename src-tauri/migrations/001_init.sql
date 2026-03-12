CREATE TABLE IF NOT EXISTS players (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    avatar_id INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS scores (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    player_id INTEGER NOT NULL REFERENCES players(id) ON DELETE CASCADE,
    game_id TEXT NOT NULL,
    score INTEGER NOT NULL,
    achieved_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS keybindings (
    player_id INTEGER NOT NULL REFERENCES players(id) ON DELETE CASCADE,
    action TEXT NOT NULL,
    device_type TEXT NOT NULL DEFAULT 'keyboard',
    binding TEXT NOT NULL,
    PRIMARY KEY (player_id, action, device_type)
);

CREATE TABLE IF NOT EXISTS plugin_storage (
    plugin_id TEXT NOT NULL,
    key TEXT NOT NULL,
    value TEXT NOT NULL,
    PRIMARY KEY (plugin_id, key)
);
