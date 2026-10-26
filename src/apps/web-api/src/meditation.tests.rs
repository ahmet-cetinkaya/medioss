//! Composition-root meditation adapters: the blob store port over a real
//! media-store root, the ffmpeg probe against the actual PATH, and the
//! cover normalizer's error mapping.

use super::{ffmpeg_available, CoverNormalizer, MediaStoreBlobs};
use crate::image_normalization::{normalize, ImageSpec, NORMALIZATION_TEST_LOCK};
use meditation_application::ports::BlobStore as _;
use meditation_application::ports::CoverValidator as _;

fn store(root: &std::path::Path) -> MediaStoreBlobs {
    MediaStoreBlobs::new(media_store::MediaStore::new(root.to_path_buf()))
}

async fn png_bytes(width: u32, height: u32) -> Vec<u8> {
    let image = image::DynamicImage::new_rgba8(width, height);
    let mut bytes = Vec::new();
    image
        .write_to(
            &mut std::io::Cursor::new(&mut bytes),
            image::ImageFormat::Png,
        )
        .unwrap();
    bytes
}

#[tokio::test]
async fn put_read_and_delete_round_trip_a_blob() {
    let directory = tempfile::tempdir().unwrap();
    let blobs = store(directory.path());

    let stored = blobs
        .put("covers", "evening calm", "webp", b"blob-bytes")
        .await
        .unwrap();
    assert!(stored.storage_key.starts_with("covers/"));
    assert!(stored.storage_key.ends_with(".webp"));
    assert_eq!(stored.size_bytes, b"blob-bytes".len() as i64);

    let read_back = blobs.read(&stored.storage_key).await.unwrap();
    assert_eq!(read_back, b"blob-bytes".to_vec());

    blobs.delete(&stored.storage_key).await.unwrap();
    let error = blobs.read(&stored.storage_key).await.unwrap_err();
    assert!(error.contains("no blob at"), "{error}");
}

#[tokio::test]
async fn relocate_moves_a_blob_to_the_new_folder_and_stem() {
    let directory = tempfile::tempdir().unwrap();
    let blobs = store(directory.path());

    let stored = blobs
        .put("covers", "cover", "webp", b"cover-bytes")
        .await
        .unwrap();
    let moved = blobs
        .relocate(&stored.storage_key, "programs", "evening", "webp")
        .await
        .unwrap();

    assert!(moved.starts_with("programs/evening-"), "{moved}");
    assert_eq!(blobs.read(&moved).await.unwrap(), b"cover-bytes".to_vec());
    let error = blobs.read(&stored.storage_key).await.unwrap_err();
    assert!(error.contains("no blob at"), "{error}");
}

#[tokio::test]
async fn delete_folder_removes_every_blob_in_it_and_tolerates_an_absent_folder() {
    let directory = tempfile::tempdir().unwrap();
    let blobs = store(directory.path());

    let first = blobs.put("folder-a", "one", "bin", b"one").await.unwrap();
    let second = blobs.put("folder-a", "two", "bin", b"two").await.unwrap();
    let other = blobs
        .put("folder-b", "three", "bin", b"three")
        .await
        .unwrap();

    blobs.delete_folder("folder-a").await.unwrap();

    for key in [first.storage_key, second.storage_key] {
        assert!(blobs.read(&key).await.is_err(), "{key} must be gone");
    }
    assert!(blobs.read(&other.storage_key).await.is_ok());

    // Absent folders are already gone: the delete stays idempotent.
    blobs.delete_folder("folder-a").await.unwrap();
}

#[tokio::test]
async fn delete_folder_refuses_an_empty_or_parent_folder_and_deletes_nothing() {
    let directory = tempfile::tempdir().unwrap();
    let blobs = store(directory.path());
    let kept = blobs.put("folder-a", "one", "bin", b"one").await.unwrap();

    for folder in ["", ".."] {
        let outcome = blobs.delete_folder(folder).await;

        assert!(outcome.is_err(), "{folder:?} must be refused");
    }
    assert!(blobs.read(&kept.storage_key).await.is_ok());
}

#[tokio::test]
async fn a_key_that_fails_validation_is_an_err_string() {
    let directory = tempfile::tempdir().unwrap();
    let blobs = store(directory.path());

    for key in ["", "..", "folder/sub/file.bin", "no/slash/../traversal"] {
        assert!(blobs.read(key).await.is_err(), "read must reject {key:?}");
        assert!(
            blobs.delete(key).await.is_err(),
            "delete must reject {key:?}"
        );
        let moved = blobs.relocate(key, "programs", "x", "bin").await;
        assert!(moved.is_err(), "relocate must reject {key:?}");
    }
}

#[test]
fn ffmpeg_available_matches_whether_ffmpeg_answers_on_path() {
    let command_runs = std::process::Command::new("ffmpeg")
        .arg("-version")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .is_ok_and(|status| status.success());

    assert_eq!(ffmpeg_available(), command_runs);
}

const COVER_LIMIT_BYTES: usize = 5 * 1024 * 1024;

fn normalizer() -> CoverNormalizer {
    CoverNormalizer {
        maximum_output_bytes: COVER_LIMIT_BYTES,
    }
}

#[tokio::test(flavor = "current_thread")]
async fn cover_normalizer_converts_a_valid_png_and_maps_garbage_to_invalid_payload() {
    let _lock = NORMALIZATION_TEST_LOCK.lock().await;

    let normalized = normalizer()
        .normalize(png_bytes(2_000, 1_000).await)
        .await
        .unwrap();
    let format = image::guess_format(&normalized).unwrap();
    assert_eq!(format, image::ImageFormat::WebP);
    let decoded = image::load_from_memory(&normalized).unwrap();
    assert!(decoded.width() <= 1_024 && decoded.height() <= 1_024);

    let error = normalizer()
        .normalize(b"definitely not an image".to_vec())
        .await
        .unwrap_err();
    assert_eq!(
        error,
        meditation_application::ports::CoverNormalizationError::InvalidPayload
    );
}

#[tokio::test(flavor = "current_thread")]
async fn cover_normalizer_maps_an_oversized_input_to_invalid_payload() {
    let _lock = NORMALIZATION_TEST_LOCK.lock().await;

    // The normalizer refuses dimensions over its decode limit before any
    // resize; the port's only dimensional variant is InvalidPayload.
    let error = normalizer()
        .normalize(png_bytes(9_000, 9_000).await)
        .await
        .unwrap_err();
    assert_eq!(
        error,
        meditation_application::ports::CoverNormalizationError::InvalidPayload
    );
}

/// Pins the shared normalizer contract the cover adapter relies on: the
/// adapter maps every `ImageNormalizationError` onto the port, so the
/// conversion stays total.
#[tokio::test(flavor = "current_thread")]
async fn cover_normalizer_uses_the_shared_normalize_pipeline() {
    let _lock = NORMALIZATION_TEST_LOCK.lock().await;

    let specification = ImageSpec {
        maximum_dimension: 1_024,
        maximum_output_bytes: COVER_LIMIT_BYTES,
        square: false,
    };
    let normalized = normalize(png_bytes(500, 250).await, specification)
        .await
        .unwrap();
    assert_eq!(
        image::guess_format(&normalized).unwrap(),
        image::ImageFormat::WebP
    );
}
