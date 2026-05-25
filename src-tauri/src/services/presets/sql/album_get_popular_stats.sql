SELECT
  COUNT(*) AS total,
  SUM(CASE WHEN p.created_at >= strftime('%s', 'now') - 20*86400  THEN 1 ELSE 0 END) AS d20,
  SUM(CASE WHEN p.created_at >= strftime('%s', 'now') - 30*86400  THEN 1 ELSE 0 END) AS d30,
  SUM(CASE WHEN p.created_at >= strftime('%s', 'now') - 60*86400  THEN 1 ELSE 0 END) AS d60,
  SUM(CASE WHEN p.created_at >= strftime('%s', 'now') - 90*86400  THEN 1 ELSE 0 END) AS d90,
  SUM(CASE WHEN p.created_at >= strftime('%s', 'now') - 180*86400 THEN 1 ELSE 0 END) AS d180,
  SUM(CASE WHEN p.created_at >= strftime('%s', 'now') - 365*86400 THEN 1 ELSE 0 END) AS d365
FROM presets p
JOIN classes c ON c.id_garmoth = p.class_id
WHERE c.display = ? AND p.is_popular = 1 AND p.is_discarded = 0 AND p.is_ok = 1
