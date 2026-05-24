SELECT source, COUNT(*) AS n FROM logs GROUP BY source ORDER BY n DESC
