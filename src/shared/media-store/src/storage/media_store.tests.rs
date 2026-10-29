use super::{MediaStore, StorageKey};
use tokio::io::AsyncReadExt;

#[tokio::test]
async fn open_range_reads_only_the_requested_window() {
    let root = tempfile::tempdir().expect("media root");
    let store = MediaStore::new(root.path().to_path_buf());
    let key = store
        .put("soundscape", "tone", "opus", b"0123456789")
        .await
        .expect("store blob");

    let mut stream = store.open_range(&key, 3, 4).await.expect("open range");
    let mut bytes = Vec::new();
    stream.read_to_end(&mut bytes).await.expect("read range");

    assert_eq!(bytes, b"3456");
}

#[tokio::test]
async fn size_reports_the_stored_blob_length() {
    let root = tempfile::tempdir().expect("media root");
    let store = MediaStore::new(root.path().to_path_buf());
    let key = store
        .put("soundscape", "tone", "opus", b"12345")
        .await
        .expect("store blob");

    assert_eq!(store.size(&key).await.expect("blob size"), 5);
    assert!(matches!(
        store
            .size(&StorageKey::parse_with_separator("soundscape/missing.opus").expect("valid key"))
            .await,
        Err(super::MediaStoreError::NotFound { .. })
    ));
}

#[tokio::test]
async fn move_in_consumes_the_source_and_stores_it_under_a_slugged_key() {
    let root = tempfile::tempdir().expect("media root");
    let scratch = tempfile::tempdir().expect("scratch");
    let source = scratch.path().join("upload-123");
    tokio::fs::write(&source, b"staged-bytes")
        .await
        .expect("source");
    let store = MediaStore::new(root.path().to_path_buf());

    let key = store
        .move_in(&source, "Evening Calm", "Deep Rest", "opus")
        .await
        .expect("move in");

    assert!(key.as_str().starts_with("evening-calm/deep-rest-"));
    assert!(key.as_str().ends_with(".opus"));
    assert!(!source.exists(), "the staged file is consumed");
    assert_eq!(store.size(&key).await.expect("size"), 12);
}

#[tokio::test]
async fn move_in_leaves_no_blob_when_the_source_is_missing() {
    let root = tempfile::tempdir().expect("media root");
    let store = MediaStore::new(root.path().to_path_buf());

    let result = store
        .move_in(&root.path().join("absent"), "folder", "stem", "opus")
        .await;

    assert!(result.is_err());
    let leftovers = std::fs::read_dir(root.path().join("folder"))
        .expect("folder")
        .count();
    assert_eq!(leftovers, 0, "the reserved key is released");
}
