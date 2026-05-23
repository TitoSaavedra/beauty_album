UPDATE popular_presets
SET class_name = (
    SELECT c.display
    FROM classes c
    WHERE c.id_garmoth = CAST(json_extract(popular_presets.raw_json, '$.class') AS INTEGER)
)
WHERE json_extract(raw_json, '$.class') IS NOT NULL;
