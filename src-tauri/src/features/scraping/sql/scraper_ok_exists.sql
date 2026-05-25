SELECT EXISTS(SELECT 1 FROM presets WHERE id=? AND is_ok=1 AND is_popular=0)
