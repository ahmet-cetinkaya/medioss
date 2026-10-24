use chrono::{DateTime, Utc};
use soundscape_domain::Sound;

pub fn sample_epoch() -> DateTime<Utc> {
    DateTime::from_timestamp(0, 0).expect("epoch is a valid timestamp")
}

pub fn sample_sound(id: i64) -> Sound {
    Sound::new(
        id,
        "Rain",
        1,
        "audio/ogg",
        10,
        &format!("soundscape/sound-{id}.ogg"),
        1000,
        sample_epoch(),
    )
    .expect("sample sound is valid")
}
