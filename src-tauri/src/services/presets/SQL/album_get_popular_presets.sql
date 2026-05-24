SELECT p.id, p.title, p.user_nickname, p.downloads, p.views, p.likes,
       p.image_1, p.is_wanted, p.created_at, p.updated_at,
       EXISTS(SELECT 1 FROM presets p2 WHERE p2.id = p.id AND p2.is_popular = 0 AND p2.is_ok = 1) AS is_downloaded
FROM presets p
JOIN classes c ON c.id_garmoth = p.class_id
WHERE c.display = ?
  AND p.is_popular = 1
  AND p.is_discarded = 0
  AND p.is_ok = 1
  AND (? = 0 OR p.created_at >= ?)
  AND (? = '' OR p.title LIKE ? OR p.user_nickname LIKE ?)
ORDER BY p.{order_col} DESC
LIMIT ? OFFSET ?
