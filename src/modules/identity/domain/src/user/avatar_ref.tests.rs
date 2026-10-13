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

#[test]
fn accepts_a_minted_key() {
    let avatar = AvatarRef::new("avatar-0123456789abcdef.webp".to_owned(), "webp").unwrap();
    assert_eq!(avatar.key(), "avatar-0123456789abcdef.webp");
    assert_eq!(avatar.extension(), "webp");
}

#[test]
fn rejects_keys_that_are_not_minted_shaped() {
    for key in [
        ".",
        "..",
        "a\0b",
        "",
        "avatar-XYZ.webp",
        "avatar-0123456789ABCDEF.webp",
        "avatar-0123456789abcde.webp",
        "avatar-0123456789abcdef0.webp",
        "avatar-0123456789abcdef.png",
        "avatar-0123456789abcdef.webp.png",
        "../avatar-0123456789abcdef.webp",
    ] {
        assert_eq!(
            AvatarRef::new(key.to_owned(), "webp").unwrap_err(),
            IdentityError::InvalidProfile,
            "{key:?}"
        );
    }
}
