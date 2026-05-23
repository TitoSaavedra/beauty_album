UPDATE popular_presets
SET
    created_at     = COALESCE(created_at,
                        json_extract(raw_json, '$.created_at')),
    char_name      = COALESCE(char_name,
                        json_extract(raw_json, '$.character_name')),
    customizing_id = COALESCE(customizing_id,
                        json_extract(raw_json, '$.customizing_id')),
    region         = COALESCE(region,
                        json_extract(raw_json, '$.region')),
    score          = COALESCE(score,
                        json_extract(raw_json, '$.score'))
WHERE raw_json IS NOT NULL;
