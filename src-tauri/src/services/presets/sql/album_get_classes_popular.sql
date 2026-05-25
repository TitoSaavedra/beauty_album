SELECT c.id_garmoth, c.display, c.icon_svg, c.is_favorite,
       COUNT(p.id) AS preset_count
FROM classes c
LEFT JOIN presets p ON p.class_id = c.id_garmoth
  AND p.is_ok = 1
  AND p.is_popular = 1
  AND p.is_discarded = 0
  AND (? = 0 OR p.created_at >= ?)
GROUP BY c.id_garmoth
ORDER BY preset_count DESC
