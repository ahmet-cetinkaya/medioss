use chrono::{TimeZone, Utc};
use meditation_domain::{Meditation, STORED_CONTENT_TYPE};

use super::{meditation_response, ProgramSummary};

fn sample() -> Meditation {
    let created = Utc.with_ymd_and_hms(2026, 1, 2, 3, 4, 5).unwrap();
    Meditation::new(
        9,
        "Morning",
        STORED_CONTENT_TYPE,
        1234,
        "key/9",
        60_000,
        created,
    )
    .unwrap()
}

#[test]
fn maps_metadata_and_defaults_progress_to_untouched() {
    let response = meditation_response(&sample());
    assert_eq!(response.id, 9);
    assert_eq!(response.name, "Morning");
    assert_eq!(response.content_type, STORED_CONTENT_TYPE);
    assert_eq!(response.size_bytes, 1234);
    assert_eq!(response.duration_ms, 60_000);
    assert_eq!(response.created_at, "2026-01-02T03:04:05+00:00");
    assert_eq!(response.description, None);
    assert!(!response.completed);
    assert_eq!(response.progress_percent, 0);
    assert_eq!(response.program_id, None);
    assert!(!response.program_has_cover);
    assert!(response.program.is_none());
}

#[test]
fn optional_program_fields_are_omitted_from_json() {
    let json = serde_json::to_value(meditation_response(&sample())).unwrap();
    let object = json.as_object().unwrap();
    assert!(!object.contains_key("program_id"));
    assert!(!object.contains_key("program_has_cover"));
    assert!(!object.contains_key("program"));
    assert_eq!(object["completed"], serde_json::json!(false));
    assert_eq!(object["progress_percent"], serde_json::json!(0));
}

#[test]
fn program_fields_serialize_when_present() {
    let mut response = meditation_response(&sample());
    response.program_id = Some(4);
    response.program_has_cover = true;
    response.program = Some(ProgramSummary {
        id: 4,
        title: "Calm".to_owned(),
        has_cover: true,
    });
    let json = serde_json::to_value(response).unwrap();
    assert_eq!(json["program_id"], serde_json::json!(4));
    assert_eq!(json["program_has_cover"], serde_json::json!(true));
    assert_eq!(
        json["program"],
        serde_json::json!({"id": 4, "title": "Calm", "has_cover": true})
    );
}
