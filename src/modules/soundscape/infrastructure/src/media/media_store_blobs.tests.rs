use super::{content_type_for, MediaStoreBlobs};
use soundscape_application::ports::media::BlobStore;
use tokio::io::AsyncReadExt;
use tokio_util::compat::FuturesAsyncReadCompatExt;

fn blobs() -> (tempfile::TempDir, MediaStoreBlobs) {
    let directory = tempfile::tempdir().unwrap();
    let store = media_store::MediaStore::new(directory.path().to_path_buf());
    (directory, MediaStoreBlobs(store))
}

#[test]
fn content_type_follows_the_key_extension() {
    assert_eq!(content_type_for("a/b.opus"), "audio/ogg");
    assert_eq!(content_type_for("a/b.flac"), "audio/flac");
    assert_eq!(content_type_for("a/b.mp3"), "audio/mpeg");
    assert_eq!(content_type_for("a/b.wav"), "audio/wav");
    assert_eq!(content_type_for("a/b.png"), "image/png");
    assert_eq!(content_type_for("a/b.jpg"), "image/jpeg");
    assert_eq!(content_type_for("a/b.webp"), "image/webp");
    assert_eq!(content_type_for("a/b.xyz"), "application/octet-stream");
    assert_eq!(content_type_for("noextension"), "application/octet-stream");
}

#[tokio::test]
async fn put_then_open_range_returns_the_requested_slice() {
    let (_directory, blobs) = blobs();
    let key = blobs
        .put("soundscape", "rain", "mp3", b"0123456789")
        .await
        .unwrap();
    assert!(key.starts_with("soundscape/rain-"));
    assert!(key.ends_with(".mp3"));

    let (reader, content_type, total) = blobs.open_range(&key, 2, 4).await.unwrap();
    let mut slice = Vec::new();
    let mut stream = reader.0.compat();
    stream.read_to_end(&mut slice).await.unwrap();

    assert_eq!(slice, b"2345");
    assert_eq!(content_type, "audio/mpeg");
    assert_eq!(total, 10);
}

#[tokio::test]
async fn delete_removes_the_blob_so_it_can_no_longer_be_opened() {
    let (_directory, blobs) = blobs();
    let key = blobs
        .put("soundscape", "wind", "wav", b"abc")
        .await
        .unwrap();
    blobs.delete(&key).await.unwrap();
    assert!(blobs.open_range(&key, 0, 3).await.is_err());
}

#[tokio::test]
async fn opening_an_invalid_key_is_an_error() {
    let (_directory, blobs) = blobs();
    assert!(blobs.open_range("../escape.mp3", 0, 1).await.is_err());
}
