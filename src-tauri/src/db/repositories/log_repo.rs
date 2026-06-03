use sea_orm::{ConnectionTrait, DbBackend, DbErr, Statement};

pub struct LogRepository;

impl LogRepository {
    pub async fn insert(
        db: &impl ConnectionTrait,
        ts: i64,
        tag: &str,
        source: &str,
        msg: &str,
    ) -> Result<i64, DbErr> {
        let result = db
            .execute(Statement::from_sql_and_values(
                DbBackend::Sqlite,
                "INSERT INTO logs (ts, tag, source, msg) VALUES (?, ?, ?, ?)",
                [ts.into(), tag.into(), source.into(), msg.into()],
            ))
            .await?;
        Ok(result.last_insert_id() as i64)
    }
}
