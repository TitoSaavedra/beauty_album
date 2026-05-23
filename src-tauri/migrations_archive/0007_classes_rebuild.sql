CREATE TEMP TABLE classes_backup AS
    SELECT id, is_favorite FROM classes WHERE is_favorite = 1;

DROP TABLE IF EXISTS classes;

CREATE TABLE classes (
    id_garmoth  INTEGER PRIMARY KEY,
    id_pa       INTEGER NOT NULL,
    display     TEXT    NOT NULL UNIQUE,
    icon_svg    TEXT,
    is_favorite INTEGER NOT NULL DEFAULT 0,
    updated_at  INTEGER
);

INSERT INTO classes (id_garmoth, id_pa, display) VALUES
    ( 0, 12, 'Berserker'),
    ( 1,  4, 'Ranger'),
    ( 2,  8, 'Sorceress'),
    ( 3, 16, 'Tamer'),
    ( 4, 24, 'Valkyrie'),
    ( 5,  0, 'Warrior'),
    ( 6, 31, 'Witch'),
    ( 7, 28, 'Wizard'),
    ( 8, 20, 'Musa'),
    ( 9, 21, 'Maehwa'),
    (10, 26, 'Ninja'),
    (11, 25, 'Kunoichi'),
    (12, 27, 'Dark Knight'),
    (13, 19, 'Striker'),
    (14, 23, 'Mystic'),
    (15, 11, 'Lahn'),
    (16, 29, 'Archer'),
    (17, 17, 'Shai'),
    (18,  5, 'Guardian'),
    (19,  1, 'Hashashin'),
    (20,  9, 'Nova'),
    (21,  2, 'Sage'),
    (22, 10, 'Corsair'),
    (23,  7, 'Drakania'),
    (24, 30, 'Woosa'),
    (25, 15, 'Maegu'),
    (26,  6, 'Scholar'),
    (27, 33, 'Dosa'),
    (28, 34, 'Deadeye'),
    (29,  3, 'Wukong'),
    (30, 32, 'Seraph');

UPDATE classes SET is_favorite = 1
WHERE id_garmoth IN (SELECT id FROM classes_backup);

DROP TABLE classes_backup;

CREATE INDEX IF NOT EXISTS idx_classes_favorite ON classes(is_favorite) WHERE is_favorite = 1;
