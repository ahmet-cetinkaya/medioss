use sea_orm::ConnectionTrait;
use sea_orm::DatabaseConnection;
use sea_orm::Statement;
use serde::Deserialize;
use std::path::Path;

#[derive(Deserialize)]
struct ManifestEntry {
    category: String,
    category_name: String,
    name: String,
    file: String,
    duration_ms: i64,
    size: i64,
    content_type: String,
}

#[derive(thiserror::Error, Debug)]
pub enum SeedError {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("manifest: {0}")]
    Manifest(String),
    #[error("db: {0}")]
    Db(String),
    #[error("media store: {0}")]
    Media(String),
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
        let exists: Option<i64> = db
            .query_one(Statement::from_sql_and_values(
                sea_orm::DatabaseBackend::Sqlite,
                "SELECT id FROM soundscape_sounds WHERE builtin_id = ?",
                vec![builtin_id.clone().into()],
            ))
            .await
            .map_err(|e| SeedError::Db(e.to_string()))?
            .and_then(|row| row.try_get::<i64>("", "id").ok());
        if exists.is_some() {
            continue;
        }
        let bytes = tokio::fs::read(dir.join(&entry.file)).await?;
        let extension = Path::new(&entry.file)
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("bin");
        let key = store
            .put(
                "soundscape",
                &format!("builtin-{slug}-{}", entry.name),
                extension,
                &bytes,
            )
            .await
            .map_err(|e| SeedError::Media(e.to_string()))?;
        db.execute(Statement::from_sql_and_values(
            sea_orm::DatabaseBackend::Sqlite,
            "INSERT INTO soundscape_sounds (name, category_id, content_type, size_bytes, storage_key, duration_ms, created_at, builtin_id) VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
            vec![
                display_name(&entry.name).into(),
                category_id.into(),
                entry.content_type.into(),
                entry.size.into(),
                key.as_str().into(),
                entry.duration_ms.into(),
                now.clone().into(),
                builtin_id.into(),
            ],
        ))
        .await
        .map_err(|e| SeedError::Db(e.to_string()))?;
    }
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
