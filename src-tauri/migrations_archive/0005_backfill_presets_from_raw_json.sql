UPDATE presets
SET
    class_id    = COALESCE(class_id,
                    json_extract(raw_json, '$._backup.class'),
                    json_extract(raw_json, '$.class')),
    image_2     = COALESCE(image_2,
                    json_extract(raw_json, '$._backup.image_2'),
                    json_extract(raw_json, '$.image_2')),
    creation_at = COALESCE(creation_at,
                    json_extract(raw_json, '$._backup.creation_at'),
                    json_extract(raw_json, '$.creation_at'))
WHERE raw_json IS NOT NULL;
