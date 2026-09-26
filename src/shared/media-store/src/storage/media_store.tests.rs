use super::*;

async fn read_all(stream: MediaStream) -> Vec<u8> {
    use tokio::io::AsyncReadExt;
    let mut reader = stream.reader();
    let mut buffer = Vec::new();
    reader.read_to_end(&mut buffer).await.unwrap();
    buffer
}

#[tokio::test]
async fn put_open_delete_round_trip_under_a_namespace() {
    let directory = tempfile::TempDir::new().unwrap();
    let store = MediaStore::new(directory.path().to_path_buf());

    let key = store
        .put(StorageNamespace::Meditation, b"opaque bytes")
        .await
        .unwrap();
    let read_back = read_all(
        store
            .open(StorageNamespace::Meditation, &key)
            .await
            .unwrap(),
    )
    .await;
    assert_eq!(read_back, b"opaque bytes");

    store
        .delete(StorageNamespace::Meditation, &key)
        .await
        .unwrap();
    match store.open(StorageNamespace::Meditation, &key).await {
        Err(MediaStoreError::NotFound { .. }) => {}
        _ => panic!("expected NotFound after delete"),
    }
}

#[tokio::test]
async fn every_put_generates_a_fresh_key() {
    let directory = tempfile::TempDir::new().unwrap();
    let store = MediaStore::new(directory.path().to_path_buf());

    let first = store
        .put(StorageNamespace::Meditation, b"same bytes")
        .await
        .unwrap();
    let second = store
        .put(StorageNamespace::Meditation, b"same bytes")
        .await
        .unwrap();
    assert_ne!(first, second, "identical inputs get distinct blobs");
}

#[tokio::test]
async fn namespaces_isolate_the_same_key() {
    let directory = tempfile::TempDir::new().unwrap();
    let store = MediaStore::new(directory.path().to_path_buf());

    let key = store
        .put(StorageNamespace::ProgramCover, b"cover")
        .await
        .unwrap();
    let result = store.open(StorageNamespace::Meditation, &key).await;
    match result {
        Err(MediaStoreError::NotFound { .. }) => {}
        _ => panic!("expected NotFound across namespaces"),
    }
}

#[tokio::test]
async fn delete_twice_reports_not_found() {
    let directory = tempfile::TempDir::new().unwrap();
    let store = MediaStore::new(directory.path().to_path_buf());
    let key = store.put(StorageNamespace::Meditation, b"x").await.unwrap();
    store
        .delete(StorageNamespace::Meditation, &key)
        .await
        .unwrap();
    match store.delete(StorageNamespace::Meditation, &key).await {
        Err(MediaStoreError::NotFound { .. }) => {}
        _ => panic!("expected NotFound on double delete"),
    }
}
