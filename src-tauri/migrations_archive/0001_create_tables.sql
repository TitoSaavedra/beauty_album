PRAGMA journal_mode=WAL;

CREATE TABLE IF NOT EXISTS classes (
    id          INTEGER PRIMARY KEY,
    display     TEXT NOT NULL UNIQUE,
    icon        TEXT,
    is_favorite INTEGER NOT NULL DEFAULT 0,
    updated_at  INTEGER
);

CREATE TABLE IF NOT EXISTS presets (
    id          INTEGER PRIMARY KEY,
    class_name  TEXT NOT NULL,
    title       TEXT,
    user_nick   TEXT,
    char_name   TEXT,
    downloads   INTEGER NOT NULL DEFAULT 0,
    views       INTEGER NOT NULL DEFAULT 0,
    likes       INTEGER NOT NULL DEFAULT 0,
    image_file  TEXT,
    pab_file    TEXT,
    creation_at INTEGER,
    updated_at  INTEGER,
    is_ok       INTEGER NOT NULL DEFAULT 0,
    raw_json    TEXT
);

CREATE TABLE IF NOT EXISTS popular_presets (
    id           INTEGER PRIMARY KEY,
    class_name   TEXT NOT NULL,
    title        TEXT,
    user_nick    TEXT,
    downloads    INTEGER NOT NULL DEFAULT 0,
    views        INTEGER NOT NULL DEFAULT 0,
    likes        INTEGER NOT NULL DEFAULT 0,
    image_file   TEXT,
    is_discarded INTEGER NOT NULL DEFAULT 0,
    is_wanted    INTEGER NOT NULL DEFAULT 0,
    is_ok        INTEGER NOT NULL DEFAULT 0,
    updated_at   INTEGER,
    raw_json     TEXT
);

CREATE INDEX IF NOT EXISTS idx_presets_class        ON presets(class_name, downloads DESC);
CREATE INDEX IF NOT EXISTS idx_popular_class        ON popular_presets(class_name, downloads DESC);
CREATE INDEX IF NOT EXISTS idx_popular_class_views  ON popular_presets(class_name, views DESC);
CREATE INDEX IF NOT EXISTS idx_popular_wanted       ON popular_presets(is_wanted) WHERE is_wanted = 1;
CREATE INDEX IF NOT EXISTS idx_classes_favorite     ON classes(is_favorite) WHERE is_favorite = 1;
