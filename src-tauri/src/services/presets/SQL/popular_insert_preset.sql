INSERT OR IGNORE INTO presets
    (id, class_id, title, user_nickname, character_name, downloads, views, likes,
     created_at, customizing_id, region, score, is_ok, is_popular, updated_at, raw_json)
VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, 0, 1, ?, ?)
