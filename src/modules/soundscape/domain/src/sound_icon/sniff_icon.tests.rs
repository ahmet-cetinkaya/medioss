use super::*;

#[test]
fn icon_sniffing_accepts_raster_and_rejects_svg() {
    assert_eq!(
        sniff_icon(b"\x89PNG\r\n\x1a\n..").map(|t| t.0),
        Some("image/png")
    );
    assert_eq!(
        sniff_icon(&[0xFF, 0xD8, 0xFF, 0xE0]).map(|t| t.0),
        Some("image/jpeg")
    );
    assert_eq!(
        sniff_icon(b"RIFF\0\0\0\0WEBPVP8 ").map(|t| t.0),
        Some("image/webp")
    );
    assert_eq!(sniff_icon(b"<svg xmlns=''></svg>"), None);
}
