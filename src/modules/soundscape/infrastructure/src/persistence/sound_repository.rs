use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sea_orm::{
    ConnectionTrait, DatabaseBackend, DatabaseConnection, QueryResult, Statement, TransactionTrait,
};
use soundscape_application::ports::{NewSound, SoundRepository};
use soundscape_domain::{Sound, SoundIcon};

pub struct SeaOrmSoundRepository {
    db: DatabaseConnection,
}
impl SeaOrmSoundRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}
fn to_sound(row: &QueryResult) -> Result<Sound, String> {
    let created_at = row
        .try_get::<String>("", "created_at")
        .map_err(|e| e.to_string())?
        .parse::<DateTime<Utc>>()
        .map_err(|e| e.to_string())?;
    let mut sound = Sound::new(
        row.try_get("", "id").map_err(|e| e.to_string())?,
        &row.try_get::<String>("", "name")
            .map_err(|e| e.to_string())?,
        row.try_get("", "category_id").map_err(|e| e.to_string())?,
        &row.try_get::<String>("", "content_type")
            .map_err(|e| e.to_string())?,
        row.try_get("", "size_bytes").map_err(|e| e.to_string())?,
        &row.try_get::<String>("", "storage_key")
            .map_err(|e| e.to_string())?,
        row.try_get("", "duration_ms").map_err(|e| e.to_string())?,
        created_at,
    )
    .map_err(|e| e.to_string())?;
    sound.builtin = row
        .try_get::<Option<String>>("", "builtin_id")
        .map_err(|e| e.to_string())?
        .is_some();
    sound.icon_key = row
        .try_get::<Option<String>>("", "icon_key")
        .map_err(|e| e.to_string())?;
    let storage_key = row
        .try_get::<Option<String>>("", "icon_storage_key")
        .map_err(|e| e.to_string())?;
    let content_type = row
        .try_get::<Option<String>>("", "icon_content_type")
        .map_err(|e| e.to_string())?;
    sound.icon = storage_key
        .zip(content_type)
        .map(|(storage_key, content_type)| SoundIcon {
            storage_key,
            content_type,
        });
    Ok(sound)
}
#[async_trait]
impl SoundRepository for SeaOrmSoundRepository {
    async fn list(&self) -> Result<Vec<Sound>, String> {
        self.db.query_all(Statement::from_string(DatabaseBackend::Sqlite,"SELECT id,name,category_id,content_type,size_bytes,storage_key,duration_ms,created_at,builtin_id,icon_key,icon_storage_key,icon_content_type FROM soundscape_sounds ORDER BY category_id,name COLLATE NOCASE,id".to_owned())).await.map_err(|e|e.to_string())?.iter().map(to_sound).collect()
    }
    async fn find(&self, id: i64) -> Result<Option<Sound>, String> {
        let row=self.db.query_one(Statement::from_sql_and_values(DatabaseBackend::Sqlite,"SELECT id,name,category_id,content_type,size_bytes,storage_key,duration_ms,created_at,builtin_id,icon_key,icon_storage_key,icon_content_type FROM soundscape_sounds WHERE id=?",[id.into()])).await.map_err(|e|e.to_string())?;
        row.as_ref().map(to_sound).transpose()
    }
    async fn create(&self, s: NewSound) -> Result<Sound, String> {
        // RETURNING binds the returned row to this INSERT: a concurrent create
        // can never be picked up (and then mutated or deleted) by mistake.
        let row = self
            .db
            .query_one(Statement::from_sql_and_values(
                DatabaseBackend::Sqlite,
                "INSERT INTO soundscape_sounds(name,category_id,content_type,size_bytes,storage_key,duration_ms,created_at,icon_key) VALUES(?,?,?,?,?,?,?,?) RETURNING id,name,category_id,content_type,size_bytes,storage_key,duration_ms,created_at,builtin_id,icon_key,icon_storage_key,icon_content_type",
                [s.name.into(), s.category_id.into(), s.content_type.into(), s.size_bytes.into(), s.storage_key.into(), s.duration_ms.into(), s.created_at.to_rfc3339().into(), s.icon_key.into()],
            ))
            .await
            .map_err(|e| e.to_string())?;
        row.as_ref()
            .map(to_sound)
            .transpose()?
            .ok_or_else(|| "sound insert missing".into())
    }
    async fn update(
        &self,
        id: i64,
        name: &str,
        category_id: i64,
        icon_key: Option<&str>,
    ) -> Result<Option<Sound>, String> {
        let r = self
            .db
            .execute(Statement::from_sql_and_values(
                DatabaseBackend::Sqlite,
                "UPDATE soundscape_sounds SET name=?,category_id=?,icon_key=? WHERE id=?",
                [
                    name.trim().into(),
                    category_id.into(),
                    icon_key.map(str::to_owned).into(),
                    id.into(),
                ],
            ))
            .await
            .map_err(|e| e.to_string())?;
        if r.rows_affected() == 0 {
            Ok(None)
        } else {
            self.find(id).await
        }
    }
    async fn set_icon(&self, id: i64, icon: Option<SoundIcon>) -> Result<Option<Sound>, String> {
        let (key, content_type) = match icon {
            Some(icon) => (Some(icon.storage_key), Some(icon.content_type)),
            None => (None, None),
        };
        let r = self
            .db
            .execute(Statement::from_sql_and_values(
                DatabaseBackend::Sqlite,
                "UPDATE soundscape_sounds SET icon_storage_key=?,icon_content_type=? WHERE id=?",
                [key.into(), content_type.into(), id.into()],
            ))
            .await
            .map_err(|e| e.to_string())?;
        if r.rows_affected() == 0 {
            Ok(None)
        } else {
            self.find(id).await
        }
    }
    async fn delete_with_mix_items(&self, id: i64) -> Result<Option<Sound>, String> {
        let sound = self.find(id).await?;
        if sound.is_none() {
            return Ok(None);
        };
        let tx = self.db.begin().await.map_err(|e| e.to_string())?;
        let builtin_id = tx
            .query_one(Statement::from_sql_and_values(
                DatabaseBackend::Sqlite,
                "SELECT builtin_id FROM soundscape_sounds WHERE id=?",
                [id.into()],
            ))
            .await
            .map_err(|e| e.to_string())?
            .and_then(|row| row.try_get::<String>("", "builtin_id").ok());
        tx.execute(Statement::from_sql_and_values(
            DatabaseBackend::Sqlite,
            "DELETE FROM soundscape_mix_items WHERE sound_id=?",
            [id.into()],
        ))
        .await
        .map_err(|e| e.to_string())?;
        // Remaining items must keep contiguous positions or every later read of the
        // affected mixes fails domain validation, so compact the gaps per mix.
        let affected = tx
            .query_all(Statement::from_string(
                DatabaseBackend::Sqlite,
                "SELECT DISTINCT mix_id FROM soundscape_mix_items".to_owned(),
            ))
            .await
            .map_err(|e| e.to_string())?;
        for row in &affected {
            let mix_id = row
                .try_get::<i64>("", "mix_id")
                .map_err(|e| e.to_string())?;
            let items = tx
                .query_all(Statement::from_sql_and_values(
                    DatabaseBackend::Sqlite,
                    "SELECT sound_id FROM soundscape_mix_items WHERE mix_id=? ORDER BY position",
                    [mix_id.into()],
                ))
                .await
                .map_err(|e| e.to_string())?;
            for (index, item) in items.iter().enumerate() {
                let sound_id = item
                    .try_get::<i64>("", "sound_id")
                    .map_err(|e| e.to_string())?;
                tx.execute(Statement::from_sql_and_values(
                    DatabaseBackend::Sqlite,
                    "UPDATE soundscape_mix_items SET position=? WHERE mix_id=? AND sound_id=?",
                    [(index as i64).into(), mix_id.into(), sound_id.into()],
                ))
                .await
                .map_err(|e| e.to_string())?;
            }
        }
        tx.execute(Statement::from_sql_and_values(
            DatabaseBackend::Sqlite,
            "DELETE FROM soundscape_sounds WHERE id=?",
            [id.into()],
        ))
        .await
        .map_err(|e| e.to_string())?;
        // A removed built-in must stay removed: record its builtin_id so the
        // startup seeder skips it instead of resurrecting the row (FR-029).
        if let Some(builtin_id) = builtin_id {
            tx.execute(Statement::from_sql_and_values(
                DatabaseBackend::Sqlite,
                "INSERT OR IGNORE INTO soundscape_builtin_tombstones(builtin_id, deleted_at) VALUES(?, ?)",
                [builtin_id.into(), chrono::Utc::now().to_rfc3339().into()],
            ))
            .await
            .map_err(|e| e.to_string())?;
        }
        tx.commit().await.map_err(|e| e.to_string())?;
        Ok(sound)
    }
}
