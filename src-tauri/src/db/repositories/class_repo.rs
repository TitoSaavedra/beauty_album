use std::collections::HashMap;

use sea_orm::{ConnectionTrait, DbBackend, DbErr, Statement};
use serde_json;

pub struct ClassRepository;

impl ClassRepository {
    pub async fn get_for_presets(db: &impl ConnectionTrait) -> Result<Vec<serde_json::Value>, DbErr> {
        let rows = db
            .query_all(Statement::from_string(
                DbBackend::Sqlite,
                "SELECT c.id_garmoth, c.display, c.icon_svg, c.is_favorite,
                        COUNT(p.id) AS preset_count
                 FROM classes c
                 LEFT JOIN presets p ON p.class_id = c.id_garmoth AND p.is_ok = 1 AND p.is_popular = 0
                 GROUP BY c.id_garmoth
                 ORDER BY preset_count DESC"
                    .to_string(),
            ))
            .await?;

        Ok(Self::map_class_rows(&rows))
    }

    pub async fn get_for_popular(
        db: &impl ConnectionTrait,
        since_ts: i64,
    ) -> Result<Vec<serde_json::Value>, DbErr> {
        let rows = db
            .query_all(Statement::from_sql_and_values(
                DbBackend::Sqlite,
                "SELECT c.id_garmoth, c.display, c.icon_svg, c.is_favorite,
                        COUNT(p.id) AS preset_count
                 FROM classes c
                 LEFT JOIN presets p ON p.class_id = c.id_garmoth
                   AND p.is_ok = 1
                   AND p.is_popular = 1
                   AND p.is_discarded = 0
                   AND (? = 0 OR p.created_at >= ?)
                 GROUP BY c.id_garmoth
                 ORDER BY preset_count DESC",
                [since_ts.into(), since_ts.into()],
            ))
            .await?;

        Ok(Self::map_class_rows(&rows))
    }

    pub async fn get_display_map(
        db: &impl ConnectionTrait,
    ) -> Result<HashMap<u32, String>, DbErr> {
        let rows = db
            .query_all(Statement::from_string(
                DbBackend::Sqlite,
                "SELECT id_garmoth, display FROM classes".to_string(),
            ))
            .await?;

        Ok(rows
            .iter()
            .filter_map(|r| {
                let id = r.try_get::<i64>("", "id_garmoth").ok()?;
                let display = r.try_get::<String>("", "display").ok()?;
                Some((id as u32, display))
            })
            .collect())
    }

    pub async fn get_favorites(db: &impl ConnectionTrait) -> Result<Vec<String>, DbErr> {
        let rows = db
            .query_all(Statement::from_string(
                DbBackend::Sqlite,
                "SELECT display FROM classes WHERE is_favorite = 1".to_string(),
            ))
            .await?;
        Ok(rows
            .iter()
            .filter_map(|r| r.try_get::<String>("", "display").ok())
            .collect())
    }

    pub async fn set_favorite(
        db: &impl ConnectionTrait,
        class_name: &str,
        is_favorite: bool,
    ) -> Result<(), DbErr> {
        db.execute(Statement::from_sql_and_values(
            DbBackend::Sqlite,
            "UPDATE classes SET is_favorite = ? WHERE display = ?",
            [((is_favorite as i64)).into(), class_name.into()],
        ))
        .await?;
        Ok(())
    }

    fn map_class_rows(rows: &[sea_orm::QueryResult]) -> Vec<serde_json::Value> {
        rows.iter()
            .map(|r| {
                serde_json::json!({
                    "class_id":     r.try_get::<i64>("", "id_garmoth").unwrap_or(0),
                    "name":         r.try_get::<String>("", "display").unwrap_or_default(),
                    "icon_svg":     r.try_get::<Option<String>>("", "icon_svg").unwrap_or(None),
                    "preset_count": r.try_get::<i64>("", "preset_count").unwrap_or(0),
                    "is_favorite":  r.try_get::<i32>("", "is_favorite").unwrap_or(0) == 1,
                })
            })
            .collect()
    }
}
