use super::seed_builtin_soundscapes;
use crate::{
    bootstrap::{database, migrations::migrate_all},
    test_support::test_config,
};

const SOUNDS_DIR_VAR: &str = "MEDIOSS_BUILTIN_SOUNDS_DIR";

struct EnvVarGuard(&'static str);

impl Drop for EnvVarGuard {
    fn drop(&mut self) {
        std::env::remove_var(self.0);
    }
}

// The only test in this crate that touches MEDIOSS_BUILTIN_SOUNDS_DIR, so it cannot race.
#[tokio::test]
async fn seeding_with_no_manifest_returns_without_creating_media() {
    let manifest_free_dir = tempfile::tempdir().unwrap();
    let _guard = EnvVarGuard(SOUNDS_DIR_VAR);
    std::env::set_var(SOUNDS_DIR_VAR, manifest_free_dir.path());
    let data = tempfile::tempdir().unwrap();
    let config = test_config(data.path(), false);
    let database = database::open(&data.path().join("medioss.db"))
        .await
        .unwrap();
    migrate_all(&database).await;

    seed_builtin_soundscapes(&database, &config).await;

    assert!(!data.path().join("media").exists());
}
