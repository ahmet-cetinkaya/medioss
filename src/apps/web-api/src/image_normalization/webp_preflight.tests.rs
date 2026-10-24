use super::*;

fn riff(chunks: &[([u8; 4], Vec<u8>)]) -> Vec<u8> {
    let mut bytes = b"RIFF\0\0\0\0WEBP".to_vec();
    for (kind, payload) in chunks {
        bytes.extend_from_slice(kind);
        bytes.extend_from_slice(&(payload.len() as u32).to_le_bytes());
        bytes.extend_from_slice(payload);
        if payload.len() % 2 == 1 {
            bytes.push(0);
        }
    }
    let declared_size = (bytes.len() - 8) as u32;
    bytes[4..8].copy_from_slice(&declared_size.to_le_bytes());
    bytes
}

fn vp8x(width: u32, height: u32) -> Vec<u8> {
    let mut payload = vec![0, 0, 0, 0];
    for dimension in [width, height] {
        let dimension = dimension - 1;
        payload.extend_from_slice(&dimension.to_le_bytes()[..3]);
    }
    payload
}

fn vp8(width: u16, height: u16) -> Vec<u8> {
    let mut payload = vec![0, 0, 0, 0x9d, 0x01, 0x2a];
    payload.extend_from_slice(&width.to_le_bytes());
    payload.extend_from_slice(&height.to_le_bytes());
    payload
}

#[test]
fn rejects_bytes_that_are_not_a_riff_webp_container() {
    assert_eq!(
        preflight_webp(b"not a webp at all"),
        Err(ImageNormalizationError::InvalidPayload)
    );
}

#[test]
fn rejects_a_container_whose_declared_size_disagrees() {
    let mut bytes = riff(&[(*b"VP8 ", vp8(2, 2))]);
    bytes[4..8].copy_from_slice(&99_u32.to_le_bytes());

    assert_eq!(
        preflight_webp(&bytes),
        Err(ImageNormalizationError::InvalidPayload)
    );
}

#[test]
fn rejects_animated_chunks() {
    for animated in [
        riff(&[(*b"VP8X", vp8x(4, 4)), (*b"ANIM", vec![0])]),
        riff(&[(*b"VP8X", vp8x(4, 4)), (*b"ANMF", vec![0, 1, 2])]),
    ] {
        assert_eq!(
            preflight_webp(&animated),
            Err(ImageNormalizationError::InvalidPayload)
        );
    }
}

#[test]
fn rejects_a_second_canvas_chunk() {
    let bytes = riff(&[
        (*b"VP8X", vp8x(4, 4)),
        (*b"VP8X", vp8x(4, 4)),
        (*b"VP8 ", vp8(4, 4)),
    ]);

    assert_eq!(
        preflight_webp(&bytes),
        Err(ImageNormalizationError::InvalidPayload)
    );
}

#[test]
fn canvas_dimensions_must_match_the_image() {
    let bytes = riff(&[(*b"VP8X", vp8x(4, 4)), (*b"VP8 ", vp8(8, 4))]);

    assert_eq!(
        preflight_webp(&bytes),
        Err(ImageNormalizationError::InvalidPayload)
    );
}

#[test]
fn accepts_a_simple_lossy_webp_with_a_matching_canvas() {
    let bytes = riff(&[(*b"VP8X", vp8x(4, 4)), (*b"VP8 ", vp8(4, 4))]);

    assert_eq!(preflight_webp(&bytes), Ok(()));
}

#[test]
fn rejects_a_truncated_chunk_length() {
    let mut bytes = riff(&[(*b"VP8 ", vp8(4, 4))]);
    bytes.truncate(bytes.len() - 2);

    assert_eq!(
        preflight_webp(&bytes),
        Err(ImageNormalizationError::InvalidPayload)
    );
}
