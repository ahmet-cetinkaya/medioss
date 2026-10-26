use super::prepare_scratch_dir;

#[test]
fn creates_a_writable_scratch_folder_under_the_data_dir() {
    let data = tempfile::tempdir().unwrap();

    let scratch = prepare_scratch_dir(data.path()).unwrap();

    assert_eq!(scratch, data.path().join("tmp"));
    std::fs::write(scratch.join("probe"), b"x").unwrap();
}

#[test]
fn clears_files_left_by_a_previous_run() {
    let data = tempfile::tempdir().unwrap();
    let stale = data.path().join("tmp/stale.flac");
    std::fs::create_dir_all(stale.parent().unwrap()).unwrap();
    std::fs::write(&stale, b"old").unwrap();

    let scratch = prepare_scratch_dir(data.path()).unwrap();

    assert!(!stale.exists());
    assert!(scratch.is_dir());
}

#[test]
fn an_unusable_data_dir_is_reported_with_the_scratch_path() {
    let data = tempfile::tempdir().unwrap();
    let file_in_the_way = data.path().join("not-a-dir");
    std::fs::write(&file_in_the_way, b"x").unwrap();

    let error = prepare_scratch_dir(&file_in_the_way).unwrap_err();

    assert!(error.to_string().contains("not-a-dir/tmp"));
}
