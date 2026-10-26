//! Startup failures that happen before logging exists must still reach stderr,
//! otherwise a container restarts in a loop with no clue why.

use std::process::Command;

#[test]
fn an_unusable_data_dir_is_reported_on_stderr() {
    let workdir = tempfile::tempdir().unwrap();
    let blocking_file = workdir.path().join("not-a-directory");
    std::fs::write(&blocking_file, b"").unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_medioss"))
        .current_dir(workdir.path())
        .env_clear()
        .env("MEDIOSS_DATA_DIR", blocking_file.join("data"))
        .env("MEDIOSS_ORIGIN", "http://localhost:3000")
        .env("SESSION_SECRET", "startup-failure-test-secret")
        .output()
        .unwrap();

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(output.status.code(), Some(1), "{stderr}");
    assert!(stderr.contains("scratch"), "{stderr}");
}
