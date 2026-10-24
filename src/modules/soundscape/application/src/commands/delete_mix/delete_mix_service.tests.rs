use super::*;
use crate::ports::NewMix;
use crate::test_support::{sample_epoch, InMemoryMixes};
use kernel::UserId;
use soundscape_domain::MixItem;

async fn mixes_with_one_owned_by(owner: i64) -> Arc<InMemoryMixes> {
    let mixes = Arc::new(InMemoryMixes::default());
    mixes
        .create(NewMix {
            user_id: UserId::new(owner),
            name: "Night".into(),
            items: vec![MixItem {
                sound_id: 1,
                volume: 50,
                position: 0,
            }],
            now: sample_epoch(),
        })
        .await
        .unwrap();
    mixes
}

#[tokio::test]
async fn owner_can_delete_the_mix() {
    let mixes = mixes_with_one_owned_by(7).await;
    DeleteMixService::new(mixes.clone())
        .handle(DeleteMix {
            id: 1,
            user_id: UserId::new(7),
        })
        .await
        .unwrap();
    assert!(mixes.snapshot().is_empty());
}

#[tokio::test]
async fn another_users_mix_is_not_found_and_kept() {
    let mixes = mixes_with_one_owned_by(7).await;
    let result = DeleteMixService::new(mixes.clone())
        .handle(DeleteMix {
            id: 1,
            user_id: UserId::new(8),
        })
        .await;
    assert_eq!(result, Err(ApplicationError::NotFound));
    assert_eq!(mixes.snapshot().len(), 1);
}

#[tokio::test]
async fn repository_failure_is_reported() {
    let result = DeleteMixService::new(Arc::new(InMemoryMixes::failing("boom")))
        .handle(DeleteMix {
            id: 1,
            user_id: UserId::new(7),
        })
        .await;
    assert_eq!(result, Err(ApplicationError::Repository("boom".into())));
}
