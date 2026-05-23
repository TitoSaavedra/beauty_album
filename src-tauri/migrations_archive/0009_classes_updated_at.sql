UPDATE classes SET updated_at = strftime('%s', 'now') WHERE updated_at IS NULL;
