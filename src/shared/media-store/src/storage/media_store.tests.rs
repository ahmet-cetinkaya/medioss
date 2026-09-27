use super::*;

async fn read_all(stream: MediaStream) -> Vec<u8> {
    use tokio::io::AsyncReadExt;
    let mut reader = stream.reader();
    let mut buffer = Vec::new();
    reader.read_to_end(&mut buffer).await.unwrap();
    buffer
}

#[tokio::test]
async fn put_writes_under_folder_with_slug_and_suffix() {
    let directory = tempfile::TempDir::new().unwrap();
    let store = MediaStore::new(directory.path().to_path_buf());

    let key = store
        .put("morning-3f9a2c", "Şafak Meditasyonu", "opus", b"bytes")
        .await
        .unwrap();
    let parts: Vec<&str> = key.as_str().split('/').collect();
    assert_eq!(parts.len(), 2);
    assert_eq!(parts[0], "morning-3f9a2c");
    assert!(parts[1].starts_with("safak-meditasyonu-"));
    assert!(parts[1].ends_with(".opus"));

    let read_back = read_all(store.open(&key).await.unwrap()).await;
    assert_eq!(read_back, b"bytes");
}

#[tokio::test]
async fn hostile_input_never_reaches_the_path() {
    let directory = tempfile::TempDir::new().unwrap();
    let store = MediaStore::new(directory.path().to_path_buf());

    let key = store
        .put("../../etc", ".. Pass/../Wörd ..", "opus", b"x")
        .await
        .unwrap();
    // Everything dangerous collapses into allowlisted slug segments.
    assert!(key.as_str().starts_with("pass-word") || key.as_str().split('/').count() == 2);
    assert!(!key.as_str().contains(".."));
    // The file materialized exactly at the sanitized relative path.
    assert!(
        directory.path().join("media").join(key.as_str()).exists()
            || directory.path().join(key.as_str()).exists()
    );
}

#[tokio::test]
async fn identical_puts_get_distinct_suffixes() {
    let directory = tempfile::TempDir::new().unwrap();
    let store = MediaStore::new(directory.path().to_path_buf());

    let first = store.put("p", "same", "opus", b"x").await.unwrap();
    let second = store.put("p", "same", "opus", b"x").await.unwrap();
    assert_ne!(first, second);
}

#[tokio::test]
async fn rename_moves_between_folders_and_delete_folder_sweeps() {
    let directory = tempfile::TempDir::new().unwrap();
    let store = MediaStore::new(directory.path().to_path_buf());

    let key = store.put("old-folder", "clip", "opus", b"x").await.unwrap();
    let moved = store
        .rename(&key, "_unfiled", "clip", "opus")
        .await
        .unwrap();
    assert!(moved.as_str().starts_with("_unfiled/clip-"));
    match store.open(&key).await {
        Err(MediaStoreError::NotFound { .. }) => {}
        _ => panic!("old location must be gone"),
    }
    let read_back = read_all(store.open(&moved).await.unwrap()).await;
    assert_eq!(read_back, b"x");

    store.delete_folder("old-folder").await.unwrap();
    store.delete_folder("never-existed").await.unwrap();
}

#[tokio::test]
async fn delete_twice_reports_not_found() {
    let directory = tempfile::TempDir::new().unwrap();
    let store = MediaStore::new(directory.path().to_path_buf());
    let key = store.put("f", "x", "opus", b"x").await.unwrap();
    store.delete(&key).await.unwrap();
    match store.delete(&key).await {
        Err(MediaStoreError::NotFound { .. }) => {}
        _ => panic!("expected NotFound on double delete"),
    }
}
