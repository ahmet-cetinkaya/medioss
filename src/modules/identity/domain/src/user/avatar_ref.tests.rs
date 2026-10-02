use super::AvatarRef;
use crate::IdentityError;

#[test]
fn round_trips_extensions() {
    assert_eq!(AvatarRef::from_extension("webp").unwrap(), AvatarRef::Webp);
    assert_eq!(AvatarRef::from_extension("png").unwrap(), AvatarRef::Png);
    assert_eq!(AvatarRef::Webp.extension(), "webp");
    assert_eq!(AvatarRef::Png.extension(), "png");
}

#[test]
fn rejects_unknown_extensions() {
    for value in ["svg", "jpg", "jpeg", "", "WEBP"] {
        assert_eq!(
            AvatarRef::from_extension(value).unwrap_err(),
            IdentityError::InvalidProfile
        );
    }
}
