use super::*;

fn png_bytes(width: u32, height: u32) -> Vec<u8> {
    let image = if width * height > 1_000_000 {
        image::DynamicImage::new_luma8(width, height)
    } else {
        image::DynamicImage::new_rgb8(width, height)
    };
    let mut bytes = Vec::new();
    image
        .write_to(
            &mut std::io::Cursor::new(&mut bytes),
            image::ImageFormat::Png,
        )
        .unwrap();
    bytes
}

fn store() -> FileAvatarStore {
    FileAvatarStore::new(
        std::env::temp_dir().join(format!("medioss-avatar-test-{}", std::process::id())),
    )
}

#[tokio::test]
async fn normalizes_to_bounded_square_webp() {
    let _lock = crate::image_normalization::NORMALIZATION_TEST_LOCK
        .lock()
        .await;
    let (encoded, ext) = store().normalize(png_bytes(400, 200)).await.unwrap();
    let reloaded = image::load_from_memory(&encoded).unwrap();
    assert_eq!(ext, "webp");
    assert_eq!(reloaded.width(), 256);
    assert_eq!(reloaded.height(), 256);
}

#[tokio::test]
async fn rejects_non_raster_payloads() {
    let _lock = crate::image_normalization::NORMALIZATION_TEST_LOCK
        .lock()
        .await;
    let store = store();
    assert_eq!(
        store.normalize(b"<svg/>".to_vec()).await.unwrap_err(),
        AvatarStoreError::InvalidPayload
    );
    assert_eq!(
        store.normalize(b"plain text".to_vec()).await.unwrap_err(),
        AvatarStoreError::InvalidPayload
    );
    assert_eq!(
        store
            .normalize(vec![0x89, 0x50, 0x4E, 0x47, 0x00, 0x00, 0x00])
            .await
            .unwrap_err(),
        AvatarStoreError::InvalidPayload
    );
}

#[tokio::test]
async fn rejects_dimension_bombs_before_allocating() {
    let _lock = crate::image_normalization::NORMALIZATION_TEST_LOCK
        .lock()
        .await;
    let bomb = png_bytes(9_000, 9_000);
    assert_eq!(
        store().normalize(bomb).await.unwrap_err(),
        AvatarStoreError::InvalidPayload
    );
}

#[tokio::test]
async fn write_read_remove_round_trip() {
    let store = store();
    let id = UserId::new(9_999_001);

    let avatar = store.write(id, "webp", b"bytes").await.unwrap();
    assert_eq!(store.read(id, &avatar).await.unwrap(), b"bytes");

    store.remove(id, &avatar).await.unwrap();
    store.remove(id, &avatar).await.unwrap();
    assert!(store.read(id, &avatar).await.is_err());
}

#[tokio::test]
async fn write_mints_a_fresh_key_and_never_overwrites_an_existing_avatar() {
    let store = store();
    let id = UserId::new(9_999_002);

    let first = store.write(id, "webp", b"first").await.unwrap();
    let second = store.write(id, "webp", b"second").await.unwrap();

    assert_ne!(first.key(), second.key());
    // The still-referenced first avatar keeps its bytes after a new write.
    assert_eq!(store.read(id, &first).await.unwrap(), b"first");
    assert_eq!(store.read(id, &second).await.unwrap(), b"second");
}

#[tokio::test]
async fn reads_still_resolve_the_legacy_avatar_png_reference() {
    let store = store();
    let id = UserId::new(9_999_003);
    let legacy = AvatarRef::from_extension("png").unwrap();

    let legacy_dir = store.media_root.join(id.value().to_string());
    tokio::fs::create_dir_all(&legacy_dir).await.unwrap();
    tokio::fs::write(legacy_dir.join("avatar.png"), b"legacy-bytes")
        .await
        .unwrap();

    // References persisted before per-avatar keys existed keep reading.
    assert_eq!(store.read(id, &legacy).await.unwrap(), b"legacy-bytes");
    store.remove(id, &legacy).await.unwrap();
    assert!(store.read(id, &legacy).await.is_err());
}
