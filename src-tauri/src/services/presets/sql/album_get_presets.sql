SELECT p.id, p.title, p.user_nickname, p.character_name,
       p.downloads, p.views, p.likes, p.image_1, p.image_2, p.pab_file, p.created_at, p.updated_at
FROM presets p
JOIN classes c ON c.id_garmoth = p.class_id
WHERE c.display = ?
  AND p.is_ok IN (0, 1)
  AND p.is_popular = 0
  AND (? = '' OR p.title LIKE ? OR p.user_nickname LIKE ?)
ORDER BY p.{order_col} DESC
LIMIT ? OFFSET ?
