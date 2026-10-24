use async_trait::async_trait;
use chrono::{DateTime, Utc};
use kernel::UserId;
use sea_orm::{ConnectionTrait, DatabaseBackend, DatabaseConnection, Statement, TransactionTrait};
use soundscape_application::ports::{MixRepository, NewMix};
use soundscape_domain::{Mix, MixItem};

pub struct SeaOrmMixRepository {
    db: DatabaseConnection,
}
impl SeaOrmMixRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}
async fn fetch(db: &impl ConnectionTrait, id: i64, user_id: UserId) -> Result<Option<Mix>, String> {
    let Some(row)=db.query_one(Statement::from_sql_and_values(DatabaseBackend::Sqlite,"SELECT id,user_id,name,created_at,updated_at FROM soundscape_mixes WHERE id=? AND user_id=?",[id.into(),user_id.value().into()])).await.map_err(|e|e.to_string())? else {return Ok(None)};
    let created = row
        .try_get::<String>("", "created_at")
        .map_err(|e| e.to_string())?
        .parse::<DateTime<Utc>>()
        .map_err(|e| e.to_string())?;
    let updated = row
        .try_get::<String>("", "updated_at")
        .map_err(|e| e.to_string())?
        .parse::<DateTime<Utc>>()
        .map_err(|e| e.to_string())?;
    let items=db.query_all(Statement::from_sql_and_values(DatabaseBackend::Sqlite,"SELECT sound_id,volume,position FROM soundscape_mix_items WHERE mix_id=? ORDER BY position",[id.into()])).await.map_err(|e|e.to_string())?.into_iter().map(|x|Ok(MixItem{sound_id:x.try_get("","sound_id").map_err(|e|e.to_string())?,volume:x.try_get("","volume").map_err(|e|e.to_string())?,position:x.try_get("","position").map_err(|e|e.to_string())?})).collect::<Result<Vec<_>,String>>()?;
    Mix::new(
        row.try_get("", "id").map_err(|e| e.to_string())?,
        user_id.value(),
        &row.try_get::<String>("", "name")
            .map_err(|e| e.to_string())?,
        items,
        created,
        updated,
    )
    .map(Some)
    .map_err(|e| e.to_string())
}
#[async_trait]
impl MixRepository for SeaOrmMixRepository {
    async fn list_for(&self, user_id: UserId) -> Result<Vec<Mix>, String> {
        let rows = self
            .db
            .query_all(Statement::from_sql_and_values(
                DatabaseBackend::Sqlite,
                "SELECT id FROM soundscape_mixes WHERE user_id=? ORDER BY updated_at DESC,id DESC",
                [user_id.value().into()],
            ))
            .await
            .map_err(|e| e.to_string())?;
        let mut result = Vec::new();
        for row in rows {
            if let Some(mix) = fetch(
                &self.db,
                row.try_get("", "id").map_err(|e| e.to_string())?,
                user_id,
            )
            .await?
            {
                result.push(mix)
            }
        }
        Ok(result)
    }
    async fn find_for(&self, id: i64, user_id: UserId) -> Result<Option<Mix>, String> {
        fetch(&self.db, id, user_id).await
    }
    async fn create(&self, mix: NewMix) -> Result<Mix, String> {
        let tx = self.db.begin().await.map_err(|e| e.to_string())?;
        tx.execute(Statement::from_sql_and_values(
            DatabaseBackend::Sqlite,
            "INSERT INTO soundscape_mixes(user_id,name,created_at,updated_at) VALUES(?,?,?,?)",
            [
                mix.user_id.value().into(),
                mix.name.into(),
                mix.now.to_rfc3339().into(),
                mix.now.to_rfc3339().into(),
            ],
        ))
        .await
        .map_err(|e| e.to_string())?;
        let row = tx
            .query_one(Statement::from_string(
                DatabaseBackend::Sqlite,
                "SELECT last_insert_rowid() AS id".to_owned(),
            ))
            .await
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "mix insert missing".to_owned())?;
        let id: i64 = row.try_get("", "id").map_err(|e| e.to_string())?;
        for item in &mix.items {
            tx.execute(Statement::from_sql_and_values(
                DatabaseBackend::Sqlite,
                "INSERT INTO soundscape_mix_items(mix_id,sound_id,volume,position) VALUES(?,?,?,?)",
                [
                    id.into(),
                    item.sound_id.into(),
                    item.volume.into(),
                    item.position.into(),
                ],
            ))
            .await
            .map_err(|e| e.to_string())?;
        }
        let result = fetch(&tx, id, mix.user_id)
            .await?
            .ok_or_else(|| "mix insert missing".to_owned())?;
        tx.commit().await.map_err(|e| e.to_string())?;
        Ok(result)
    }
    async fn replace(&self, id: i64, mix: NewMix) -> Result<Option<Mix>, String> {
        let tx = self.db.begin().await.map_err(|e| e.to_string())?;
        let found = fetch(&tx, id, mix.user_id).await?;
        if found.is_none() {
            tx.rollback().await.map_err(|e| e.to_string())?;
            return Ok(None);
        };
        tx.execute(Statement::from_sql_and_values(
            DatabaseBackend::Sqlite,
            "UPDATE soundscape_mixes SET name=?,updated_at=? WHERE id=?",
            [mix.name.into(), mix.now.to_rfc3339().into(), id.into()],
        ))
        .await
        .map_err(|e| e.to_string())?;
        tx.execute(Statement::from_sql_and_values(
            DatabaseBackend::Sqlite,
            "DELETE FROM soundscape_mix_items WHERE mix_id=?",
            [id.into()],
        ))
        .await
        .map_err(|e| e.to_string())?;
        for item in &mix.items {
            tx.execute(Statement::from_sql_and_values(
                DatabaseBackend::Sqlite,
                "INSERT INTO soundscape_mix_items(mix_id,sound_id,volume,position) VALUES(?,?,?,?)",
                [
                    id.into(),
                    item.sound_id.into(),
                    item.volume.into(),
                    item.position.into(),
                ],
            ))
            .await
            .map_err(|e| e.to_string())?;
        }
        let result = fetch(&tx, id, mix.user_id).await?;
        tx.commit().await.map_err(|e| e.to_string())?;
        Ok(result)
    }
    async fn delete_for(&self, id: i64, user_id: UserId) -> Result<bool, String> {
        let tx = self.db.begin().await.map_err(|e| e.to_string())?;
        let r = tx
            .execute(Statement::from_sql_and_values(
                DatabaseBackend::Sqlite,
                "DELETE FROM soundscape_mixes WHERE id=? AND user_id=?",
                [id.into(), user_id.value().into()],
            ))
            .await
            .map_err(|e| e.to_string())?;
        if r.rows_affected() > 0 {
            tx.execute(Statement::from_sql_and_values(
                DatabaseBackend::Sqlite,
                "DELETE FROM soundscape_mix_items WHERE mix_id=?",
                [id.into()],
            ))
            .await
            .map_err(|e| e.to_string())?;
        }
        tx.commit().await.map_err(|e| e.to_string())?;
        Ok(r.rows_affected() > 0)
    }

    async fn delete_all_for_user(&self, user_id: UserId) -> Result<u64, String> {
        let tx = self.db.begin().await.map_err(|e| e.to_string())?;
        tx.execute(Statement::from_sql_and_values(
            DatabaseBackend::Sqlite,
            "DELETE FROM soundscape_mix_items WHERE mix_id IN (SELECT id FROM soundscape_mixes WHERE user_id=?)",
            [user_id.value().into()],
        ))
        .await
        .map_err(|e| e.to_string())?;
        let r = tx
            .execute(Statement::from_sql_and_values(
                DatabaseBackend::Sqlite,
                "DELETE FROM soundscape_mixes WHERE user_id=?",
                [user_id.value().into()],
            ))
            .await
            .map_err(|e| e.to_string())?;
        tx.commit().await.map_err(|e| e.to_string())?;
        Ok(r.rows_affected())
    }
}

#[cfg(test)]
#[path = "mix_repository.tests.rs"]
mod tests;
