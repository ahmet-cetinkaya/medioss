//! Test fakes for the portability application ports (meditation test_support
//! style: one fake per port, plain state behind `Arc<Mutex<..>>`).
#![allow(dead_code)]

pub mod fake_library;
pub mod fake_mood_store;
pub mod fake_practice_sink;
pub mod fake_practice_source;
pub mod fake_profile;
pub mod fixtures;

pub use fake_library::FakeLibrary;
pub use fake_mood_store::{FakeMoodStore, MoodSinkFailingOnSecondInsert};
pub use fake_practice_sink::FakePracticeSink;
pub use fake_practice_source::FakePracticeSource;
pub use fake_profile::FakeProfile;
