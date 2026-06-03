use std::collections::HashSet;
use std::path::Path;

use chrono::{TimeZone, Utc};
use sea_orm::{ConnectionTrait, DatabaseConnection, DbBackend, DbErr, Statement, TransactionTrait, Value};
use serde_json;

pub struct PresetRepository;

impl PresetRepository {
    // ─── Album queries ───────────────────────────────────────────────────────

    /// Obtener presets paginados - unificado para album
    ///
    /// Llamado desde:
    /// - album: beauty/service.rs:21 (comando `get_album_presets`)
    /// - popular: beauty/service.rs:37 (comando `get_popular_presets`)
    ///
    /// is_popular: 0 = album (locales), 1 = popular (API Garmoth)
    /// Retorna: Vec de JSON con presets filtrados, paginados, con imágenes y metadatos
    pub async fn get_presets(
        db: &impl ConnectionTrait,
        presets_dir: &Path,
        popular_dir: &Path,
        class_name: &str,
        is_popular: i64,
        sort_by: &str,
        search: &str,
        offset: i64,
        limit: i64,
        region: Option<&str>,
        since_ts: i64,
    ) -> Result<Vec<serde_json::Value>, DbErr> {
        let order_col = match sort_by {
            "views" => "views",
            "favorites" => "likes",
            _ => "downloads",
        };
        let search_like = if search.is_empty() {
            String::new()
        } else {
            format!("%{}%", search)
        };

        let (select_cols, extra_where) = if is_popular == 0 {
            (
                "p.id, p.title, p.user_nickname, p.character_name, p.downloads, p.views, p.likes, p.image_1, p.image_2, p.pab_file, p.creation_at, p.updated_at",
                "AND p.is_popular = 0",
            )
        } else {
            (
                "p.id, p.title, p.user_nickname, p.downloads, p.views, p.likes, p.image_1, p.is_wanted, p.creation_at, p.updated_at, EXISTS(SELECT 1 FROM presets p2 WHERE p2.id = p.id AND p2.is_popular = 0) AS is_downloaded",
                "AND p.is_popular = 1 AND p.is_discarded = 0 AND (? = 0 OR p.creation_at >= ?) AND (? = '' OR p.region = ?)",
            )
        };

        let sql = format!(
            "SELECT {select_cols}
             FROM presets p
             JOIN classes c ON c.id_garmoth = p.class_id
             WHERE c.display = ?
               {extra_where}
               AND (? = '' OR p.title LIKE ? OR p.user_nickname LIKE ?)
             ORDER BY p.{order_col} DESC
             LIMIT ? OFFSET ?"
        );

        let mut params: Vec<sea_orm::Value> = vec![class_name.into()];
        if is_popular == 1 {
            params.push(since_ts.into());
            params.push(since_ts.into());
            params.push(search.into());
            params.push(search_like.as_str().into());
            params.push(search_like.as_str().into());
            params.push(region.unwrap_or("").into());
            params.push(region.unwrap_or("").into());
        } else {
            params.push(search.into());
            params.push(search_like.as_str().into());
            params.push(search_like.as_str().into());
        }
        params.push(limit.into());
        params.push(offset.into());

        let rows = db.query_all(Statement::from_sql_and_values(DbBackend::Sqlite, &sql, params)).await?;

        let mut result = Vec::new();
        for r in &rows {
            let id: i64 = r.try_get("", "id").unwrap_or(0);
            let id_str = id.to_string();
            let creation_at: Option<i64> = r.try_get("", "creation_at").unwrap_or(None);

            let mut preset_obj = serde_json::json!({
                "preset_id": id_str,
                "id": id,
                "title": r.try_get::<Option<String>>("", "title").unwrap_or(None),
                "creator": r.try_get::<Option<String>>("", "user_nickname").unwrap_or(None),
                "downloads": r.try_get::<i64>("", "downloads").unwrap_or(0),
                "views": r.try_get::<i64>("", "views").unwrap_or(0),
                "likes": r.try_get::<i64>("", "likes").unwrap_or(0),
                "favorites": r.try_get::<i64>("", "likes").unwrap_or(0),
                "creation_at": creation_at,
                "updated_at": r.try_get::<Option<i64>>("", "updated_at").unwrap_or(None),
                "date": format_date(creation_at),
            });

            if is_popular == 0 {
                let image_1: Option<String> = r.try_get("", "image_1").unwrap_or(None);
                let image_2: Option<String> = r.try_get("", "image_2").unwrap_or(None);
                let pab_file: Option<String> = r.try_get("", "pab_file").unwrap_or(None);
                let preset_dir = presets_dir.join(class_name).join(&id_str);

                let mut image_paths: Vec<String> = Vec::new();
                for img in [&image_1, &image_2].into_iter().flatten() {
                    let p = preset_dir.join(img);
                    if p.exists() {
                        image_paths.push(p.to_string_lossy().replace('\\', "/"));
                    }
                }
                let download_path: Option<String> = pab_file.as_deref().and_then(|pab| {
                    let p = preset_dir.join(pab);
                    p.exists().then(|| p.to_string_lossy().replace('\\', "/").to_string())
                });

                preset_obj["char_name"] = r.try_get::<Option<String>>("", "character_name").unwrap_or(None).into();
                preset_obj["image_paths"] = image_paths.into();
                preset_obj["download_path"] = download_path.into();
            } else {
                let image_1: Option<String> = r.try_get("", "image_1").unwrap_or(None);
                let popular_preset_dir = popular_dir.join(class_name).join(&id_str);
                let image_paths: Vec<String> = image_1
                    .as_deref()
                    .map(|img| {
                        let p = popular_preset_dir.join(img);
                        if p.exists() {
                            return vec![p.to_string_lossy().replace('\\', "/")];
                        }
                        let p2 = presets_dir.join(class_name).join(&id_str).join(img);
                        if p2.exists() {
                            vec![p2.to_string_lossy().replace('\\', "/")]
                        } else {
                            vec![]
                        }
                    })
                    .unwrap_or_default();

                preset_obj["image_paths"] = image_paths.into();
                preset_obj["download_path"] = serde_json::Value::Null;
                preset_obj["is_popular"] = true.into();
                preset_obj["is_downloaded"] = (r.try_get::<i64>("", "is_downloaded").unwrap_or(0) == 1).into();
                preset_obj["is_wanted"] = (r.try_get::<i64>("", "is_wanted").unwrap_or(0) == 1).into();
            }

            result.push(preset_obj);
        }
        Ok(result)
    }

    /// Obtener estadísticas de presets populares por rango temporal (20, 30, 60, 90, 180, 365 días)
    ///
    /// Llamado desde: beauty/service.rs:61 (comando `get_popular_stats`)
    /// Cuándo: Iniciar app o cambiar clase
    /// Retorna: JSON { "total": N, "d20": N, "d30": N, ..., "d365": N }
    pub async fn get_popular_stats(
        db: &impl ConnectionTrait,
        class_name: &str,
    ) -> Result<serde_json::Value, DbErr> {
        let row = db
            .query_one(Statement::from_sql_and_values(
                DbBackend::Sqlite,
                "SELECT
                   COUNT(*) AS total,
                   SUM(CASE WHEN p.creation_at >= strftime('%s', 'now') - 20*86400  THEN 1 ELSE 0 END) AS d20,
                   SUM(CASE WHEN p.creation_at >= strftime('%s', 'now') - 30*86400  THEN 1 ELSE 0 END) AS d30,
                   SUM(CASE WHEN p.creation_at >= strftime('%s', 'now') - 60*86400  THEN 1 ELSE 0 END) AS d60,
                   SUM(CASE WHEN p.creation_at >= strftime('%s', 'now') - 90*86400  THEN 1 ELSE 0 END) AS d90,
                   SUM(CASE WHEN p.creation_at >= strftime('%s', 'now') - 180*86400 THEN 1 ELSE 0 END) AS d180,
                   SUM(CASE WHEN p.creation_at >= strftime('%s', 'now') - 365*86400 THEN 1 ELSE 0 END) AS d365
                 FROM presets p
                 JOIN classes c ON c.id_garmoth = p.class_id
                 WHERE c.display = ? AND p.is_popular = 1 AND p.is_discarded = 0",
                [class_name.into()],
            ))
            .await?;

        Ok(serde_json::json!({
            "total": row.as_ref().and_then(|r| r.try_get::<i64>("", "total").ok()).unwrap_or(0),
            "d20":   row.as_ref().and_then(|r| r.try_get::<i64>("", "d20").ok()).unwrap_or(0),
            "d30":   row.as_ref().and_then(|r| r.try_get::<i64>("", "d30").ok()).unwrap_or(0),
            "d60":   row.as_ref().and_then(|r| r.try_get::<i64>("", "d60").ok()).unwrap_or(0),
            "d90":   row.as_ref().and_then(|r| r.try_get::<i64>("", "d90").ok()).unwrap_or(0),
            "d180":  row.as_ref().and_then(|r| r.try_get::<i64>("", "d180").ok()).unwrap_or(0),
            "d365":  row.as_ref().and_then(|r| r.try_get::<i64>("", "d365").ok()).unwrap_or(0),
        }))
    }

    /// Obtener detalles completos de un preset por ID (album o popular)
    ///
    /// Llamado desde: beauty/commands.rs (comando `get_preset_by_id`)
    /// Cuándo: User hace click en un preset para ver detalles
    /// Retorna: Option<JSON> con todos los detalles del preset
    pub async fn get_preset_by_id(
        db: &impl ConnectionTrait,
        popular_dir: &Path,
        presets_dir: &Path,
        id: i64,
    ) -> Result<Option<serde_json::Value>, DbErr> {
        let row = db
            .query_one(Statement::from_sql_and_values(
                DbBackend::Sqlite,
                "SELECT p.id, p.title, p.user_nickname, p.character_name, p.downloads, p.views, p.likes,
                        p.image_1, p.image_2, p.pab_file, p.is_wanted, p.is_popular, p.creation_at, p.updated_at,
                        c.display AS class_display
                 FROM presets p
                 JOIN classes c ON c.id_garmoth = p.class_id
                 WHERE p.id = ?",
                [id.into()],
            ))
            .await?;

        let Some(r) = row else { return Ok(None) };

        let id_str = id.to_string();
        let is_popular: i64 = r.try_get("", "is_popular").unwrap_or(0);
        let class_display: String = r.try_get("", "class_display").unwrap_or_default();
        let creation_at: Option<i64> = r.try_get("", "creation_at").unwrap_or(None);

        let image_paths: Vec<String> = if is_popular == 0 {
            let image_1: Option<String> = r.try_get("", "image_1").unwrap_or(None);
            let image_2: Option<String> = r.try_get("", "image_2").unwrap_or(None);
            let preset_dir = presets_dir.join(&class_display).join(&id_str);
            let mut paths = Vec::new();
            for img in [&image_1, &image_2].into_iter().flatten() {
                let p = preset_dir.join(img);
                if p.exists() {
                    paths.push(p.to_string_lossy().replace('\\', "/"));
                }
            }
            paths
        } else {
            let image_1: Option<String> = r.try_get("", "image_1").unwrap_or(None);
            image_1
                .as_deref()
                .map(|img| {
                    let p = popular_dir.join(&class_display).join(&id_str).join(img);
                    if p.exists() {
                        return vec![p.to_string_lossy().replace('\\', "/")];
                    }
                    let p2 = presets_dir.join(&class_display).join(&id_str).join(img);
                    if p2.exists() {
                        vec![p2.to_string_lossy().replace('\\', "/")]
                    } else {
                        vec![]
                    }
                })
                .unwrap_or_default()
        };

        let download_path: Option<String> = if is_popular == 0 {
            r.try_get::<Option<String>>("", "pab_file")
                .unwrap_or(None)
                .and_then(|pab| {
                    let p = presets_dir.join(&class_display).join(&id_str).join(&pab);
                    p.exists().then(|| p.to_string_lossy().replace('\\', "/").to_string())
                })
        } else {
            None
        };

        Ok(Some(serde_json::json!({
            "preset_id":    id_str,
            "id":           id,
            "title":        r.try_get::<Option<String>>("", "title").unwrap_or(None),
            "creator":      r.try_get::<Option<String>>("", "user_nickname").unwrap_or(None),
            "char_name":    if is_popular == 0 { r.try_get::<Option<String>>("", "character_name").unwrap_or(None) } else { None },
            "downloads":    r.try_get::<i64>("", "downloads").unwrap_or(0),
            "views":        r.try_get::<i64>("", "views").unwrap_or(0),
            "likes":        r.try_get::<i64>("", "likes").unwrap_or(0),
            "favorites":    r.try_get::<i64>("", "likes").unwrap_or(0),
            "image_paths":  image_paths,
            "download_path": download_path,
            "date":         format_date(creation_at),
            "creation_at":   creation_at,
            "updated_at":   r.try_get::<Option<i64>>("", "updated_at").unwrap_or(None),
            "is_popular":   is_popular == 1,
            "is_downloaded": false,
            "is_wanted":    r.try_get::<i64>("", "is_wanted").unwrap_or(0) == 1,
            "class_display": class_display,
        })))
    }

    // ─── Scraper operations ──────────────────────────────────────────────────

    pub async fn ok_exists(db: &impl ConnectionTrait, id: u64) -> bool {
        db.query_one(Statement::from_sql_and_values(
            DbBackend::Sqlite,
            "SELECT EXISTS(SELECT 1 FROM presets WHERE id=? AND is_ok=1 AND is_popular=0) AS e",
            [(id as i64).into()],
        ))
        .await
        .ok()
        .flatten()
        .and_then(|r| r.try_get::<i64>("", "e").ok())
        .map(|v| v == 1)
        .unwrap_or(false)
    }

    pub async fn mark_ok(db: &impl ConnectionTrait, id: u64) -> Result<(), DbErr> {
        db.execute(Statement::from_sql_and_values(
            DbBackend::Sqlite,
            "UPDATE presets SET is_ok=1 WHERE id=? AND is_popular=0",
            [(id as i64).into()],
        ))
        .await?;
        Ok(())
    }

    /// Convertir preset de popular a album cuando se encuentra el archivo .pab
    ///
    /// Llamado desde: scraping/service.rs:240 (en run() - album scraper)
    /// Cuándo: Cuando ok_exists() = true (preset ya existe como album)
    /// Para qué: Cambiar is_popular=1 → 0 y registrar archivo .pab asociado
    /// Efecto: DB: is_popular=0, pab_file=nombre_archivo
    /// Insertar nuevo preset de album con metadatos base
    ///
    /// Llamado desde: scraping/service.rs:270 (en run() - album scraper)
    /// Cuándo: Al procesar archivo .pab nuevo (si ok_exists() = false)
    /// Para qué: Crear registro inicial con metadatos del preset
    /// Efecto: INSERT OR REPLACE con is_ok=0, is_popular=0
    pub async fn insert_preset(
        db: &impl ConnectionTrait,
        id: i64,
        class_id: i64,
        title: Option<&str>,
        user_nickname: Option<&str>,
        character_name: Option<&str>,
        downloads: i64,
        views: i64,
        likes: i64,
        image_2: Option<&str>,
        pab_file: &str,
        creation_at: Option<i64>,
        updated_at: i64,
        raw_json: &str,
    ) -> Result<(), DbErr> {
        db.execute(Statement::from_sql_and_values(
            DbBackend::Sqlite,
            "INSERT OR REPLACE INTO presets
                (id, class_id, title, user_nickname, character_name, downloads, views, likes,
                 image_2, pab_file, creation_at, updated_at, is_ok, is_popular, raw_json)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, 0, 0, ?)",
            [
                id.into(),
                class_id.into(),
                title.map(Value::from).unwrap_or(Value::String(None)),
                user_nickname.map(Value::from).unwrap_or(Value::String(None)),
                character_name.map(Value::from).unwrap_or(Value::String(None)),
                downloads.into(),
                views.into(),
                likes.into(),
                image_2.map(Value::from).unwrap_or(Value::String(None)),
                pab_file.into(),
                creation_at.map(Value::from).unwrap_or(Value::BigInt(None)),
                updated_at.into(),
                raw_json.into(),
            ],
        ))
        .await?;
        Ok(())
    }

    /// Registrar ambas imágenes después de descargarlas exitosamente
    ///
    /// Llamado desde: scraping/service.rs:134 (en process_image() - album scraper)
    /// Cuándo: Después de descargar exitosamente image_1 Y image_2
    /// Para qué: Guardar nombres de archivo en BD
    /// Efecto: DB: image_1=nombre, image_2=nombre, updated_at=ahora
    pub async fn update_image_both(
        db: &impl ConnectionTrait,
        id: u64,
        img1: &str,
        img2: &str,
        now: i64,
    ) -> Result<(), DbErr> {
        db.execute(Statement::from_sql_and_values(
            DbBackend::Sqlite,
            "UPDATE presets SET image_1=?, image_2=?, updated_at=? WHERE id=?",
            [img1.into(), img2.into(), now.into(), (id as i64).into()],
        ))
        .await?;
        Ok(())
    }

    /// Registrar solo image_1 después de descargarla
    ///
    /// Llamado desde: scraping/service.rs:142 (en process_image() - album scraper)
    /// Cuándo: Cuando solo image_1 disponible (image_2 es None)
    /// Para qué: Guardar nombre de archivo en BD
    /// Efecto: DB: image_1=nombre, updated_at=ahora
    pub async fn update_image_one(
        db: &impl ConnectionTrait,
        id: u64,
        img1: &str,
        now: i64,
    ) -> Result<(), DbErr> {
        db.execute(Statement::from_sql_and_values(
            DbBackend::Sqlite,
            "UPDATE presets SET image_1=?, updated_at=? WHERE id=?",
            [img1.into(), now.into(), (id as i64).into()],
        ))
        .await?;
        Ok(())
    }

    /// Actualizar timestamp de preset (legacy - no usado actualmente)
    ///
    /// Llamado desde: ningún lado (dead code)
    /// Nota: Originalmente marcaba is_ok=1, ahora solo actualiza updated_at
    /// Efecto: DB: updated_at=now
    // ─── Popular sync operations ─────────────────────────────────────────────

    /// Obtener IDs de presets populares que ya tienen imagen descargada
    ///
    /// Llamado desde: scrapping/service.rs (en sync_popular())
    /// Cuándo: Al inicio de cada sync popular
    /// Para qué: DEDUPLICACIÓN - filtrar presets ya procesados
    /// Retorna: HashSet<i64> de presets donde image_1 IS NOT NULL
    /// Lógica: Preset "completo" = tiene image_1 descargada
    pub async fn popular_get_synced_ids(
        db: &impl ConnectionTrait,
    ) -> Result<HashSet<i64>, DbErr> {
        let rows = db
            .query_all(Statement::from_string(
                DbBackend::Sqlite,
                "SELECT id FROM presets WHERE is_popular = 1 AND image_1 IS NOT NULL".to_string(),
            ))
            .await?;
        Ok(rows
            .iter()
            .filter_map(|r| r.try_get::<i64>("", "id").ok())
            .collect())
    }

    /// Insertar preset popular fetcheado de API Garmoth
    ///
    /// Llamado desde: scrapping/service.rs (en insert_pending())
    /// Cuándo: Para cada preset único traído de la API
    /// Para qué: Crear registro con metadatos API
    /// Efecto: INSERT OR IGNORE si no existe, UPDATE para completar NULL fields
    /// Nota: raw_json se guarda completo para reintentos posteriores
    pub async fn popular_insert(
        db: &impl ConnectionTrait,
        id: i64,
        class_id: i64,
        title: Option<&str>,
        user_nickname: Option<&str>,
        character_name: Option<&str>,
        downloads: i64,
        views: i64,
        likes: i64,
        creation_at: Option<i64>,
        customizing_id: Option<i64>,
        region: Option<&str>,
        score: Option<i64>,
        now: i64,
        raw_json: &str,
    ) -> Result<(), DbErr> {
        db.execute(Statement::from_sql_and_values(
            DbBackend::Sqlite,
            "INSERT OR IGNORE INTO presets
                (id, class_id, title, user_nickname, character_name, downloads, views, likes,
                 creation_at, customizing_id, region, score, is_ok, is_popular, updated_at, raw_json)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, 0, 1, ?, ?)",
            [
                id.into(),
                class_id.into(),
                title.map(Value::from).unwrap_or(Value::String(None)),
                user_nickname.map(Value::from).unwrap_or(Value::String(None)),
                character_name.map(Value::from).unwrap_or(Value::String(None)),
                downloads.into(),
                views.into(),
                likes.into(),
                creation_at.map(Value::from).unwrap_or(Value::BigInt(None)),
                customizing_id.map(Value::from).unwrap_or(Value::BigInt(None)),
                region.map(Value::from).unwrap_or(Value::String(None)),
                score.map(Value::from).unwrap_or(Value::BigInt(None)),
                now.into(),
                raw_json.into(),
            ],
        ))
        .await?;
        Ok(())
    }

    /// Registrar imagen descargada exitosamente para preset popular
    ///
    /// Llamado desde: scrapping/service.rs (en download_all())
    /// Cuándo: Después de descargar exitosamente una imagen
    /// Para qué: Guardar nombre de archivo descargado
    /// Efecto: Preset ahora en popular_get_synced_ids() (tiene image_1)
    pub async fn popular_update_image_ok(
        db: &impl ConnectionTrait,
        id: i64,
        img1: &str,
        now: i64,
    ) -> Result<(), DbErr> {
        db.execute(Statement::from_sql_and_values(
            DbBackend::Sqlite,
            "UPDATE presets SET image_1 = ?, updated_at = ? WHERE id = ?",
            [img1.into(), now.into(), id.into()],
        ))
        .await?;
        Ok(())
    }

    /// Marcar preset popular sin imagen (no reintentar)
    ///
    /// Llamado desde: scrapping/service.rs (en download_all())
    /// Cuándo: Cuando image_1 y image_2 NULL (no hay imagen disponible)
    /// Para qué: Evitar reintentar descargar imagen inexistente
    /// Efecto: updated_at=now, preset saltado en próximos syncs
    pub async fn popular_update_no_image_ok(
        db: &impl ConnectionTrait,
        id: i64,
        now: i64,
    ) -> Result<(), DbErr> {
        db.execute(Statement::from_sql_and_values(
            DbBackend::Sqlite,
            "UPDATE presets SET updated_at = ? WHERE id = ?",
            [now.into(), id.into()],
        ))
        .await?;
        Ok(())
    }

    /// Obtener presets populares en cola (fallaron en descargas anteriores)
    ///
    /// Llamado desde: scrapping/service.rs (en sync_popular())
    /// Cuándo: Después de calcular nuevos (unique)
    /// Para qué: Reintentar presets que fallaron antes
    /// Retorna: Vec<raw_json> - metadatos guardados para reintentos
    /// Lógica: is_popular=1 AND is_ok=0 = intentados pero fallidos
    pub async fn popular_get_pending(
        db: &impl ConnectionTrait,
    ) -> Result<Vec<serde_json::Value>, DbErr> {
        let rows = db
            .query_all(Statement::from_string(
                DbBackend::Sqlite,
                "SELECT raw_json FROM presets WHERE is_popular = 1 AND is_ok = 0 AND raw_json IS NOT NULL"
                    .to_string(),
            ))
            .await?;
        Ok(rows
            .iter()
            .filter_map(|r| {
                r.try_get::<String>("", "raw_json")
                    .ok()
                    .and_then(|s| serde_json::from_str(&s).ok())
            })
            .collect())
    }

    // ─── Wanted / discard ────────────────────────────────────────────────────

    /// Obtener lista de IDs de presets populares marcados como favoritos
    ///
    /// Llamado desde: commands.rs:209 (comando `get_wanted_ids`)
    /// Cuándo: Sincronizar lista de favoritos con frontend
    /// Retorna: Vec<i64> de preset IDs con is_wanted=1
    pub async fn wanted_get_ids(db: &impl ConnectionTrait) -> Result<Vec<i64>, DbErr> {
        let rows = db
            .query_all(Statement::from_string(
                DbBackend::Sqlite,
                "SELECT id FROM presets WHERE is_wanted = 1 AND is_popular = 1".to_string(),
            ))
            .await?;
        Ok(rows
            .iter()
            .filter_map(|r| r.try_get::<i64>("", "id").ok())
            .collect())
    }

    /// Actualizar en bulk la lista completa de favoritos (reemplazar todos)
    ///
    /// Llamado desde: commands.rs:219 (comando `set_wanted_presets`)
    /// Cuándo: Frontend envía nueva lista de favoritos
    /// Para qué: Sincronizar lista local con servidor/preferencias
    /// Efecto: Pone todos en is_wanted=0, luego los IDs dados en is_wanted=1
    pub async fn wanted_set_bulk(
        db: &DatabaseConnection,
        ids: &[i64],
    ) -> Result<(), DbErr> {
        let txn = db.begin().await?;
        txn.execute(Statement::from_string(
            DbBackend::Sqlite,
            "UPDATE presets SET is_wanted = 0 WHERE is_popular = 1".to_string(),
        ))
        .await?;
        for &id in ids {
            txn.execute(Statement::from_sql_and_values(
                DbBackend::Sqlite,
                "UPDATE presets SET is_wanted = 1 WHERE id = ? AND is_popular = 1",
                [id.into()],
            ))
            .await?;
        }
        txn.commit().await?;
        Ok(())
    }

    /// Obtener estado de favorito (0 o 1) para un preset
    ///
    /// Llamado desde: commands.rs:226 (en toggle_wanted())
    /// Cuándo: Leer estado actual antes de togglear
    /// Retorna: i64 - 0 (no favorito) o 1 (favorito)
    pub async fn wanted_get(db: &impl ConnectionTrait, id: i64) -> Result<i64, DbErr> {
        let row = db
            .query_one(Statement::from_sql_and_values(
                DbBackend::Sqlite,
                "SELECT is_wanted FROM presets WHERE id = ? AND is_popular = 1",
                [id.into()],
            ))
            .await?;
        Ok(row
            .and_then(|r| r.try_get::<i64>("", "is_wanted").ok())
            .unwrap_or(0))
    }

    /// Cambiar estado de favorito de un preset (0 → 1 o 1 → 0)
    ///
    /// Llamado desde: commands.rs:228 (comando `toggle_wanted`)
    /// Cuándo: User hace click en corazón (favorite) en la UI
    /// Para qué: Agregar/quitar de favoritos
    pub async fn wanted_toggle(
        db: &impl ConnectionTrait,
        id: i64,
        new_val: i64,
    ) -> Result<(), DbErr> {
        db.execute(Statement::from_sql_and_values(
            DbBackend::Sqlite,
            "UPDATE presets SET is_wanted = ? WHERE id = ? AND is_popular = 1",
            [new_val.into(), id.into()],
        ))
        .await?;
        Ok(())
    }

    /// Marcar un preset popular como descartado (no mostrar más en listados)
    ///
    /// Llamado desde: commands.rs:236 (comando `discard_preset`)
    /// Cuándo: User hace click en X (discard) en la UI
    /// Para qué: Ocultar presets que no interesan
    /// Efecto: is_discarded=1 - filtrado en WHERE clauses de get_popular_presets
    pub async fn discard(db: &impl ConnectionTrait, id: i64) -> Result<(), DbErr> {
        db.execute(Statement::from_sql_and_values(
            DbBackend::Sqlite,
            "UPDATE presets SET is_discarded = 1 WHERE id = ? AND is_popular = 1",
            [id.into()],
        ))
        .await?;
        Ok(())
    }

    /// Obtener lista de regiones disponibles en BD para popular presets
    ///
    /// Llamado desde: commands.rs:203 (comando `get_regions`)
    /// Cuándo: Llenar dropdown de filtro de región en UI
    /// Retorna: Vec<String> - lista única de regiones ordenadas alfabéticamente
    ///
    /// Ejemplo: ["eu", "jp", "kr", "na", "ru", "sa", "sea", "tw"]
    pub async fn get_regions(db: &impl ConnectionTrait) -> Result<Vec<String>, DbErr> {
        let rows = db
            .query_all(Statement::from_string(
                DbBackend::Sqlite,
                "SELECT DISTINCT region FROM presets WHERE is_popular = 1 AND region IS NOT NULL ORDER BY region"
                    .to_string(),
            ))
            .await?;
        Ok(rows
            .iter()
            .filter_map(|r| r.try_get::<String>("", "region").ok())
            .collect())
    }

    /// Obtener IDs de presets con is_ok=1 (ya descargados)
    pub async fn get_ok_preset_ids(db: &impl ConnectionTrait) -> Result<HashSet<i64>, DbErr> {
        let rows = db
            .query_all(Statement::from_string(DbBackend::Sqlite, "SELECT id FROM presets WHERE is_ok = 1".to_string()))
            .await?;
        Ok(rows
            .iter()
            .filter_map(|r| r.try_get::<i64>("", "id").ok())
            .collect())
    }

    /// Obtener presets populares que necesitan descarga (sin image_1 o sin image_2)
    pub async fn popular_get_missing_images(db: &impl ConnectionTrait) -> Result<Vec<serde_json::Value>, DbErr> {
        let rows = db
            .query_all(Statement::from_string(
                DbBackend::Sqlite,
                "SELECT raw_json FROM presets WHERE is_popular = 1 AND is_discarded = 0 AND (image_1 IS NULL OR image_2 IS NULL) AND raw_json IS NOT NULL".to_string(),
            ))
            .await?;
        Ok(rows
            .iter()
            .filter_map(|r| {
                r.try_get::<String>("", "raw_json")
                    .ok()
                    .and_then(|s| serde_json::from_str(&s).ok())
            })
            .collect())
    }

    /// Actualizar detalles faltantes de preset popular (campos NULL)
    pub async fn popular_update_details(
        db: &impl ConnectionTrait,
        id: i64,
        title: Option<&str>,
        user_nickname: Option<&str>,
        character_name: Option<&str>,
        downloads: i64,
        views: i64,
        likes: i64,
        creation_at: Option<i64>,
        customizing_id: Option<i64>,
        region: Option<&str>,
        score: Option<i64>,
        raw_json: &str,
        now: i64,
    ) -> Result<(), DbErr> {
        db.execute(Statement::from_sql_and_values(
            DbBackend::Sqlite,
            "UPDATE presets SET
                title          = COALESCE(title, ?),
                user_nickname  = COALESCE(user_nickname, ?),
                character_name = COALESCE(character_name, ?),
                downloads      = ?,
                views          = ?,
                likes          = ?,
                creation_at    = COALESCE(creation_at, ?),
                customizing_id = COALESCE(customizing_id, ?),
                region         = COALESCE(region, ?),
                score          = COALESCE(score, ?),
                raw_json       = COALESCE(raw_json, ?),
                updated_at     = ?
             WHERE id = ? AND is_popular = 1",
            [
                title.map(Value::from).unwrap_or(Value::String(None)),
                user_nickname.map(Value::from).unwrap_or(Value::String(None)),
                character_name.map(Value::from).unwrap_or(Value::String(None)),
                downloads.into(),
                views.into(),
                likes.into(),
                creation_at.map(Value::from).unwrap_or(Value::BigInt(None)),
                customizing_id.map(Value::from).unwrap_or(Value::BigInt(None)),
                region.map(Value::from).unwrap_or(Value::String(None)),
                score.map(Value::from).unwrap_or(Value::BigInt(None)),
                raw_json.into(),
                now.into(),
                id.into(),
            ],
        ))
        .await?;
        Ok(())
    }
}

fn format_date(ts: Option<i64>) -> Option<String> {
    ts.and_then(|t| {
        if let chrono::LocalResult::Single(dt) = Utc.timestamp_opt(t, 0) {
            Some(dt.format("%Y-%m-%d").to_string())
        } else {
            None
        }
    })
}
