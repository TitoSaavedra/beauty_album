CREATE TABLE IF NOT EXISTS classes (
    id_garmoth  INTEGER PRIMARY KEY,
    id_pa       INTEGER NOT NULL,
    display     TEXT    NOT NULL UNIQUE,
    icon_svg    TEXT,
    is_favorite INTEGER NOT NULL DEFAULT 0,
    updated_at  INTEGER
);

CREATE TABLE IF NOT EXISTS presets (
    id              INTEGER PRIMARY KEY,
    class_id        INTEGER NOT NULL REFERENCES classes(id_garmoth),
    title           TEXT,
    user_nickname   TEXT,
    character_name  TEXT,
    downloads       INTEGER NOT NULL DEFAULT 0,
    views           INTEGER NOT NULL DEFAULT 0,
    likes           INTEGER NOT NULL DEFAULT 0,
    image_1         TEXT,
    image_2         TEXT,
    creation_at     INTEGER,
    customizing_id  INTEGER,
    region          TEXT,
    score           INTEGER,
    pab_file        TEXT,
    is_ok           INTEGER NOT NULL DEFAULT 0,
    is_popular      INTEGER NOT NULL DEFAULT 0,
    is_discarded    INTEGER NOT NULL DEFAULT 0,
    is_wanted       INTEGER NOT NULL DEFAULT 0,
    updated_at      INTEGER,
    raw_json        TEXT
);

CREATE TABLE IF NOT EXISTS logs (
    id      INTEGER PRIMARY KEY,
    ts      INTEGER NOT NULL,
    tag     TEXT    NOT NULL,
    source  TEXT    NOT NULL,
    msg     TEXT    NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_presets_class    ON presets(class_id, downloads DESC);
CREATE INDEX IF NOT EXISTS idx_presets_popular  ON presets(class_id, is_popular, downloads DESC);
CREATE INDEX IF NOT EXISTS idx_presets_wanted   ON presets(is_wanted) WHERE is_wanted = 1;
CREATE INDEX IF NOT EXISTS idx_classes_favorite ON classes(is_favorite) WHERE is_favorite = 1;
CREATE INDEX IF NOT EXISTS idx_logs_ts          ON logs(ts DESC);
CREATE INDEX IF NOT EXISTS idx_logs_source      ON logs(source, ts DESC);
CREATE INDEX IF NOT EXISTS idx_logs_tag         ON logs(tag, ts DESC);
