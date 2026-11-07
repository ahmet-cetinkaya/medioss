use super::SeedError;
use sea_orm::ConnectionTrait;
use sea_orm::DatabaseConnection;
use sea_orm::Statement;
use serde::Deserialize;
use std::path::Path;

#[derive(Deserialize)]
struct ManifestEntry {
    category: String,
    category_name: String,
    /// Turkish category title; optional so an older manifest still seeds.
    #[serde(default)]
    category_name_tr: Option<String>,
    name: String,
    /// Turkish sound title; optional so an older manifest still seeds.
    #[serde(default)]
    name_tr: Option<String>,
    file: String,
    duration_ms: i64,
    size: i64,
    content_type: String,
}

fn default_dirs() -> Vec<std::path::PathBuf> {
    let mut dirs = vec![];
    if let Ok(dir) = std::env::var("MEDIOSS_BUILTIN_SOUNDS_DIR") {
        dirs.push(std::path::PathBuf::from(dir));
    }
    dirs.push(std::path::PathBuf::from("/srv/medioss/builtin-sounds"));
    dirs.push(std::path::PathBuf::from("builtin-sounds"));
    dirs
}

pub async fn seed_if_present(
    db: &DatabaseConnection,
    media_root: &Path,
) -> Result<bool, SeedError> {
    for dir in default_dirs() {
        let manifest_path = dir.join("manifest.json");
        if tokio::fs::try_exists(&manifest_path).await.unwrap_or(false) {
            seed(db, media_root, &dir, &manifest_path).await?;
            return Ok(true);
        }
    }
    Ok(false)
}

async fn seed(
    db: &DatabaseConnection,
    media_root: &Path,
    dir: &Path,
    manifest_path: &Path,
) -> Result<(), SeedError> {
    let raw = tokio::fs::read_to_string(manifest_path).await?;
    let entries: Vec<ManifestEntry> =
        serde_json::from_str(&raw).map_err(|e| SeedError::Manifest(e.to_string()))?;
    let store = media_store::MediaStore::new(media_root.to_path_buf());
    let now = chrono::Utc::now().to_rfc3339();
    for entry in entries {
        let slug = slug_for(&entry.category);
        let builtin_id = format!("{slug}:{entry_name}", entry_name = entry.name);
        let category_id: i64 = match lookup_category(db, &slug).await? {
            Some(id) => id,
            None => insert_category(db, &slug, &entry.category_name).await?,
        };
        fill_titles(db, &entry, &slug, &builtin_id).await?;
        if !needs_seeding(db, &builtin_id).await? {
            continue;
        }
        let bytes = tokio::fs::read(dir.join(&entry.file)).await?;
        let key = store_builtin_blob(&store, &slug, &entry, &bytes).await?;
        insert_builtin_sound(db, &slug, category_id, entry, builtin_id, &key, &now).await?;
    }
    Ok(())
}

/// False when the sound is already seeded or an admin removed it on an
/// earlier run; removal must stick (FR-029), so tombstoned sounds are not
/// resurrected.
async fn needs_seeding(db: &DatabaseConnection, builtin_id: &str) -> Result<bool, SeedError> {
    let exists = db
        .query_one(Statement::from_sql_and_values(
            sea_orm::DatabaseBackend::Sqlite,
            "SELECT id FROM soundscape_sounds WHERE builtin_id = ?",
            vec![builtin_id.into()],
        ))
        .await
        .map_err(|e| SeedError::Db(e.to_string()))?
        .and_then(|row| row.try_get::<i64>("", "id").ok())
        .is_some();
    if exists {
        return Ok(false);
    }
    let tombstoned = db
        .query_one(Statement::from_sql_and_values(
            sea_orm::DatabaseBackend::Sqlite,
            "SELECT builtin_id FROM soundscape_builtin_tombstones WHERE builtin_id = ?",
            vec![builtin_id.into()],
        ))
        .await
        .map_err(|e| SeedError::Db(e.to_string()))?
        .is_some();
    Ok(!tombstoned)
}

async fn store_builtin_blob(
    store: &media_store::MediaStore,
    slug: &str,
    entry: &ManifestEntry,
    bytes: &[u8],
) -> Result<String, SeedError> {
    let extension = Path::new(&entry.file)
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("bin");
    store
        .put(
            "soundscape",
            &format!("builtin-{slug}-{}", entry.name),
            extension,
            bytes,
        )
        .await
        .map(|key| key.as_str().to_owned())
        .map_err(|e| SeedError::Media(e.to_string()))
}

async fn insert_builtin_sound(
    db: &DatabaseConnection,
    slug: &str,
    category_id: i64,
    entry: ManifestEntry,
    builtin_id: String,
    key: &str,
    now: &str,
) -> Result<(), SeedError> {
    let name = display_name(&entry.name);
    let name_tr = entry.name_tr.clone();
    let icon_key = crate::icon_defaults::default_icon_key(slug, &name).map(str::to_owned);
    db.execute(Statement::from_sql_and_values(
        sea_orm::DatabaseBackend::Sqlite,
        "INSERT INTO soundscape_sounds (name, category_id, content_type, size_bytes, storage_key, duration_ms, created_at, builtin_id, icon_key, name_tr) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        vec![
            name.into(),
            category_id.into(),
            entry.content_type.into(),
            entry.size.into(),
            key.into(),
            entry.duration_ms.into(),
            now.into(),
            builtin_id.into(),
            icon_key.into(),
            name_tr.into(),
        ],
    ))
    .await
    .map_err(|e| SeedError::Db(e.to_string()))?;
    Ok(())
}

/// Writes the manifest's Turkish titles onto rows that already exist, so a
/// database seeded before the titles shipped gets them too. A row whose name
/// no longer matches the seeded English name was renamed by an admin and is
/// left alone: the user's name always wins.
async fn fill_titles(
    db: &DatabaseConnection,
    entry: &ManifestEntry,
    slug: &str,
    builtin_id: &str,
) -> Result<(), SeedError> {
    if let Some(title) = &entry.name_tr {
        execute(
            db,
            "UPDATE soundscape_sounds SET name_tr = ? WHERE builtin_id = ? AND name = ?",
            vec![
                title.clone().into(),
                builtin_id.into(),
                display_name(&entry.name).into(),
            ],
        )
        .await?;
    }
    if let Some(title) = &entry.category_name_tr {
        execute(
            db,
            "UPDATE soundscape_categories SET name_tr = ? WHERE slug = ? AND name = ?",
            vec![
                title.clone().into(),
                slug.into(),
                entry.category_name.clone().into(),
            ],
        )
        .await?;
    }
    Ok(())
}

async fn execute(
    db: &DatabaseConnection,
    sql: &str,
    values: Vec<sea_orm::Value>,
) -> Result<(), SeedError> {
    db.execute(Statement::from_sql_and_values(
        sea_orm::DatabaseBackend::Sqlite,
        sql,
        values,
    ))
    .await
    .map_err(|e| SeedError::Db(e.to_string()))?;
    Ok(())
}

fn slug_for(category: &str) -> String {
    category.trim().to_ascii_lowercase()
}

fn display_name(name: &str) -> String {
    name.split('-')
        .filter(|word| !word.is_empty())
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

async fn lookup_category(db: &DatabaseConnection, slug: &str) -> Result<Option<i64>, SeedError> {
    Ok(db
        .query_one(Statement::from_sql_and_values(
            sea_orm::DatabaseBackend::Sqlite,
            "SELECT id FROM soundscape_categories WHERE slug = ?",
            vec![slug.into()],
        ))
        .await
        .map_err(|e| SeedError::Db(e.to_string()))?
        .and_then(|row| row.try_get::<i64>("", "id").ok()))
}

async fn insert_category(
    db: &DatabaseConnection,
    slug: &str,
    name: &str,
) -> Result<i64, SeedError> {
    db.execute(Statement::from_sql_and_values(
        sea_orm::DatabaseBackend::Sqlite,
        "INSERT INTO soundscape_categories (slug, name) VALUES (?, ?)",
        vec![slug.into(), name.into()],
    ))
    .await
    .map_err(|e| SeedError::Db(e.to_string()))?;
    lookup_category(db, slug)
        .await?
        .ok_or_else(|| SeedError::Db("category row vanished after insert".to_owned()))
}

#[cfg(test)]
#[path = "seeder.tests.rs"]
mod tests;
