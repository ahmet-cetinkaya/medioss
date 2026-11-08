//! Soundscape domain entities and invariants.

// Each topic folder holds its type in a file of the same name
// (`mix/mix.rs`); that nesting is the layout convention, not inception.
#![allow(clippy::module_inception)]

pub mod error;
pub mod mix;
pub mod name;
pub mod sound;
pub mod sound_category;
pub mod sound_icon;

pub use error::DomainError;
pub use mix::{Mix, MixItem, MAX_MIX_SOUNDS};
pub use name::{valid_name, MAX_NAME_LENGTH};
pub use sound::{valid_content_type, Sound, STORED_CONTENT_TYPE};
pub use sound_category::SoundCategory;
pub use sound_icon::{sniff_icon, valid_icon_key, SoundIcon, MAX_ICON_BYTES, MAX_ICON_KEY_LENGTH};
