INSERT OR REPLACE INTO presets
    (id, class_id, title, user_nickname, character_name, downloads, views, likes,
     image_2, pab_file, created_at, updated_at, is_ok, is_popular, raw_json)
VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, 0, 0, ?)
