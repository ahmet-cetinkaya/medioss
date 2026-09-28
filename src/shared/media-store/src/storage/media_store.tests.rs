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
