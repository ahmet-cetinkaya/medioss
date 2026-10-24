use super::*;
use kernel::UserId;
use soundscape_domain::MixItem;

fn item(sound_id: i64, position: i64) -> MixItem {
    MixItem {
        sound_id,
        volume: 50,
        position,
    }
}

fn command(name: &str, items: Vec<MixItem>) -> UpdateMixCommand {
    UpdateMixCommand {
        id: 1,
        user_id: UserId::new(7),
        name: name.into(),
        items,
    }
}

#[test]
fn accepts_a_valid_mix() {
    let command = command("Night", vec![item(1, 0)]);
    assert_eq!(UpdateMixCommandValidator.validate(&command), Ok(()));
}

#[test]
fn rejects_a_blank_name() {
    let command = command("   ", vec![item(1, 0)]);
    assert_eq!(
        UpdateMixCommandValidator.validate(&command),
        Err(ApplicationError::InvalidInput("mix name"))
    );
}

#[test]
fn rejects_a_name_over_the_length_limit() {
    let command = command(&"a".repeat(81), vec![item(1, 0)]);
    assert_eq!(
        UpdateMixCommandValidator.validate(&command),
        Err(ApplicationError::InvalidInput("mix name"))
    );
}

#[test]
fn rejects_empty_items() {
    let command = command("Night", Vec::new());
    assert_eq!(
        UpdateMixCommandValidator.validate(&command),
        Err(ApplicationError::InvalidInput("mix items"))
    );
}

#[test]
fn rejects_misnumbered_items() {
    let command = command("Night", vec![item(1, 3)]);
    assert_eq!(
        UpdateMixCommandValidator.validate(&command),
        Err(ApplicationError::InvalidInput("mix items"))
    );
}

#[test]
fn checks_the_name_before_the_items() {
    let command = command("   ", Vec::new());
    assert_eq!(
        UpdateMixCommandValidator.validate(&command),
        Err(ApplicationError::InvalidInput("mix name"))
    );
}

#[test]
fn normalizes_the_name_by_trimming() {
    let command = command("  Night  ", vec![item(1, 0)]);
    assert_eq!(UpdateMixCommandValidator.normalized_name(&command), "Night");
}
