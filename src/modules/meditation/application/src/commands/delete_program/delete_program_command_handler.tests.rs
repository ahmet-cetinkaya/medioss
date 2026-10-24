use crate::ports::CommandHandler;
use std::sync::Arc;

use crate::commands::create_program::{CreateProgramCommand, CreateProgramCommandHandler};
use crate::commands::delete_program::{DeleteProgramCommand, DeleteProgramCommandHandler};
use crate::commands::upload_meditation::{UploadMeditationCommand, UploadMeditationCommandHandler};
use crate::test_support::{FakeConverter, FakeCoverValidator, FakeValidator, TestDeps};

#[tokio::test]
async fn deleting_a_program_keeps_meditations_and_cleans_cover_and_members() {
    let deps = TestDeps::new();
    let png = vec![0x89, b'P', b'N', b'G', 0, 0, 0, 0, 0, 0, 0, 0];
    let program = CreateProgramCommandHandler::new(
        deps.programs.clone(),
        Arc::new(FakeCoverValidator),
        deps.store.clone(),
        deps.clock.clone(),
    )
    .handle(CreateProgramCommand {
        title: "Covered".into(),
        description: Some("desc".into()),
        cover_bytes: Some(png),
    })
    .await
    .expect("program with cover");
    // Uploaded directly into the program being deleted, so its file actually
    // lives in that program's folder — exercising the relocation branch,
    // not the "stayed in a different program's folder" no-op path.
    let meditation = UploadMeditationCommandHandler::new(
        deps.meditations.clone(),
        deps.memberships.clone(),
        deps.programs.clone(),
        Arc::new(FakeValidator),
        Arc::new(FakeConverter { fail: false }),
        deps.store.clone(),
        deps.clock.clone(),
    )
    .handle(UploadMeditationCommand {
        name: "Home".into(),
        program_id: program.id,
        bytes: b"OggS....OpusHead".to_vec(),
        description: None,
    })
    .await
    .expect("home upload");
    let original_key = meditation.storage_key().to_owned();
    assert!(original_key.starts_with(&format!("{}/", program.folder)));

    DeleteProgramCommandHandler::new(
        deps.programs.clone(),
        deps.memberships.clone(),
        deps.meditations.clone(),
        deps.store.clone(),
    )
    .handle(DeleteProgramCommand {
        program_id: program.id,
    })
    .await
    .expect("delete program");

    let relocated_key = {
        let survivors = deps.meditations.0.meditations.lock().unwrap();
        let relocated = survivors
            .iter()
            .find(|row| row.id() == meditation.id())
            .expect("meditation row survives");
        assert_ne!(
            relocated.storage_key(),
            original_key,
            "file must be relocated"
        );
        assert!(relocated.storage_key().starts_with("_unfiled/"));
        relocated.storage_key().to_owned()
    };
    let memberships = deps.memberships.0.memberships.lock().unwrap();
    assert!(
        memberships.iter().all(|m| m.program_id != program.id),
        "no membership may reference the deleted program"
    );
    drop(memberships);
    // The cover blob is gone; the relocated audio blob remains (moved, not
    // duplicated).
    assert_eq!(deps.store.len(), 1);
    assert!(deps.store.contains_key(&relocated_key));
}
