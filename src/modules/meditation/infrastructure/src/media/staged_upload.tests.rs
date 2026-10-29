use meditation_application::ports::{StagingError, UploadStaging};

use super::{resolve_staged_path, TempFileStaging};

async fn stage(
    staging: &TempFileStaging,
    bytes: &[u8],
) -> meditation_application::ports::StagedMedia {
    let mut sink = staging.open().await.unwrap();
    sink.append(bytes).await.unwrap();
    sink.keep().await.unwrap()
}

#[tokio::test]
async fn kept_file_reports_its_size_and_reads_back_a_prefix() {
    let root = tempfile::tempdir().unwrap();
    let staging = TempFileStaging::new(root.path().to_path_buf());

    let media = stage(&staging, b"0123456789").await;

    assert_eq!(media.size_bytes, 10);
    assert_eq!(staging.read_prefix(&media, 4).await.unwrap(), b"0123");
    assert_eq!(
        staging.read_prefix(&media, 99).await.unwrap(),
        b"0123456789"
    );
}

#[tokio::test]
async fn dropping_an_unkept_sink_deletes_its_file() {
    let root = tempfile::tempdir().unwrap();
    let staging = TempFileStaging::new(root.path().to_path_buf());

    let mut sink = staging.open().await.unwrap();
    sink.append(b"partial").await.unwrap();
    assert_eq!(std::fs::read_dir(root.path()).unwrap().count(), 1);
    drop(sink);

    assert_eq!(std::fs::read_dir(root.path()).unwrap().count(), 0);
}

#[tokio::test]
async fn discard_removes_a_kept_file_and_tolerates_an_absent_one() {
    let root = tempfile::tempdir().unwrap();
    let staging = TempFileStaging::new(root.path().to_path_buf());
    let media = stage(&staging, b"x").await;

    staging.discard(&media).await.unwrap();
    staging.discard(&media).await.unwrap();

    assert_eq!(std::fs::read_dir(root.path()).unwrap().count(), 0);
}

#[tokio::test]
async fn concurrent_stagers_get_distinct_files_that_all_survive_until_discarded() {
    let root = tempfile::tempdir().unwrap();
    let staging = std::sync::Arc::new(TempFileStaging::new(root.path().to_path_buf()));

    let tasks: Vec<_> = (0..4u8)
        .map(|index| {
            let staging = staging.clone();
            tokio::spawn(async move { stage(&staging, &[index; 1024]).await })
        })
        .collect();
    let mut media = Vec::new();
    for task in tasks {
        media.push(task.await.unwrap());
    }

    let addresses: std::collections::HashSet<_> = media.iter().map(|m| m.address.clone()).collect();
    assert_eq!(addresses.len(), 4, "addresses are distinct");
    assert_eq!(std::fs::read_dir(root.path()).unwrap().count(), 4);
    for (index, item) in media.iter().enumerate() {
        let prefix = staging.read_prefix(item, 8).await.unwrap();
        assert_eq!(prefix, vec![index as u8; 8], "no cross-talk between files");
    }
    for item in &media {
        staging.discard(item).await.unwrap();
    }
    assert_eq!(std::fs::read_dir(root.path()).unwrap().count(), 0);
}

#[cfg(unix)]
#[tokio::test]
async fn free_bytes_reports_the_staging_volume() {
    let root = tempfile::tempdir().unwrap();
    let staging = TempFileStaging::new(root.path().join("nested"));

    assert!(staging.free_bytes().await.unwrap() > 0);
}

#[test]
fn staged_addresses_with_path_components_are_refused() {
    let root = std::path::Path::new("/staging");

    assert!(resolve_staged_path(root, "upload-abc").is_ok());
    for bad in ["../escape", "a/b", "/abs", "", ".."] {
        assert!(
            matches!(resolve_staged_path(root, bad), Err(StagingError::Io(_))),
            "{bad}"
        );
    }
}
