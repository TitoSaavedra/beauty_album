SELECT id, ts, tag, source, msg FROM logs
WHERE (? IS NULL OR tag = ?)
  AND (? IS NULL OR source = ?)
ORDER BY ts DESC
LIMIT ? OFFSET ?
