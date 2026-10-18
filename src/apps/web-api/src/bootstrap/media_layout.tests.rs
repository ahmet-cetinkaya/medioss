use super::migrate_media_layout;
use std::fs;

fn touch(path: &std::path::Path) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, b"x").unwrap();
}

#[test]
fn moves_legacy_user_and_program_folders_into_module_roots() {
    let dir = tempfile::tempdir().unwrap();
    touch(&dir.path().join("media/42/avatar-abc.webp"));
    touch(&dir.path().join("media/Mindful-Walks-3a1b/song-1.opus"));
    touch(&dir.path().join("media/_unfiled/song-2.opus"));

    migrate_media_layout(dir.path());

    assert!(dir
        .path()
        .join("media/identity/42/avatar-abc.webp")
        .exists());
    assert!(dir
        .path()
        .join("media/meditation/Mindful-Walks-3a1b/song-1.opus")
        .exists());
    assert!(dir
        .path()
        .join("media/meditation/_unfiled/song-2.opus")
        .exists());
    assert!(!dir.path().join("media/42").exists());
}

#[test]
fn leaves_module_roots_and_unknown_children_alone() {
    let dir = tempfile::tempdir().unwrap();
    touch(&dir.path().join("media/soundscape/builtin/rain.opus"));
    touch(&dir.path().join("media/meditation/P/cover.webp"));
    touch(&dir.path().join("media/identity/7/avatar.webp"));
    touch(&dir.path().join("media/notes.txt"));
    touch(&dir.path().join("media/something.else/file.bin"));

    migrate_media_layout(dir.path());

    assert!(dir
        .path()
        .join("media/soundscape/builtin/rain.opus")
        .exists());
    assert!(dir.path().join("media/meditation/P/cover.webp").exists());
    assert!(dir.path().join("media/identity/7/avatar.webp").exists());
    assert!(dir.path().join("media/notes.txt").exists());
    assert!(dir.path().join("media/something.else").exists());
}

#[test]
fn is_idempotent_and_missing_media_is_fine() {
    let dir = tempfile::tempdir().unwrap();
    migrate_media_layout(dir.path()); // no media dir at all

    touch(&dir.path().join("media/9/avatar.webp"));
    migrate_media_layout(dir.path());
    migrate_media_layout(dir.path()); // second run: nothing left

    assert!(dir.path().join("media/identity/9/avatar.webp").exists());
    assert!(!dir.path().join("media/9").exists());
}
