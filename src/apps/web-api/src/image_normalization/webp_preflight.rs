use super::vp8l_preflight::{preflight_vp8l, preflight_vp8l_stream, Vp8lBitReader};
use super::{validate_input_dimensions, ImageNormalizationError};

pub(super) fn preflight_webp(bytes: &[u8]) -> Result<(), ImageNormalizationError> {
    if bytes.len() < 12 || &bytes[..4] != b"RIFF" || &bytes[8..12] != b"WEBP" {
        return Err(ImageNormalizationError::InvalidPayload);
    }
    let declared_size = u32::from_le_bytes(bytes[4..8].try_into().unwrap()) as usize;
    if declared_size.checked_add(8) != Some(bytes.len()) {
        return Err(ImageNormalizationError::InvalidPayload);
    }

    let mut offset = 12;
    let mut canvas = None;
    let mut image_dimensions = None;
    while offset < bytes.len() {
        let header_end = offset
            .checked_add(8)
            .filter(|end| *end <= bytes.len())
            .ok_or(ImageNormalizationError::InvalidPayload)?;
        let chunk = &bytes[offset..offset + 4];
        let length = u32::from_le_bytes(bytes[offset + 4..header_end].try_into().unwrap()) as usize;
        let data_start = header_end;
        let data_end = data_start
            .checked_add(length)
            .filter(|end| *end <= bytes.len())
            .ok_or(ImageNormalizationError::InvalidPayload)?;
        let padded_end = data_end
            .checked_add(length & 1)
            .filter(|end| *end <= bytes.len())
            .ok_or(ImageNormalizationError::InvalidPayload)?;

        match chunk {
            b"ANIM" | b"ANMF" => return Err(ImageNormalizationError::InvalidPayload),
            b"VP8X" => {
                if canvas.is_some()
                    || length != 10
                    || bytes[data_start] & 0xc3 != 0
                    || bytes[data_start + 1..data_start + 4] != [0; 3]
                {
                    return Err(ImageNormalizationError::InvalidPayload);
                }
                let dimensions = (
                    1 + u32::from_le_bytes([
                        bytes[data_start + 4],
                        bytes[data_start + 5],
                        bytes[data_start + 6],
                        0,
                    ]),
                    1 + u32::from_le_bytes([
                        bytes[data_start + 7],
                        bytes[data_start + 8],
                        bytes[data_start + 9],
                        0,
                    ]),
                );
                validate_input_dimensions(dimensions.0, dimensions.1)?;
                canvas = Some(dimensions);
            }
            b"VP8 " => {
                if length < 10 || &bytes[data_start + 3..data_start + 6] != b"\x9d\x01\x2a" {
                    return Err(ImageNormalizationError::InvalidPayload);
                }
                let dimensions = (
                    u32::from(u16::from_le_bytes([
                        bytes[data_start + 6],
                        bytes[data_start + 7],
                    ])) & 0x3fff,
                    u32::from(u16::from_le_bytes([
                        bytes[data_start + 8],
                        bytes[data_start + 9],
                    ])) & 0x3fff,
                );
                validate_webp_image_dimensions(dimensions, &mut image_dimensions)?;
            }
            b"VP8L" => {
                let payload = &bytes[data_start..data_end];
                let dimensions = preflight_vp8l(payload)?;
                validate_webp_image_dimensions(dimensions, &mut image_dimensions)?;
            }
            b"ALPH" => preflight_alpha(&bytes[data_start..data_end])?,
            _ => {}
        }
        offset = padded_end;
    }

    let dimensions = image_dimensions.ok_or(ImageNormalizationError::InvalidPayload)?;
    if canvas.is_some_and(|canvas_dimensions| canvas_dimensions != dimensions) {
        return Err(ImageNormalizationError::InvalidPayload);
    }
    Ok(())
}

/// A lossless ALPH chunk carries a header-less VP8L stream (dimensions come from the image) that
/// reaches the same image-webp lossless decoder with meta-Huffman enabled, so it needs the same
/// allocation preflight. Only compression 0 (raw) and 1 (lossless) exist in the spec.
fn preflight_alpha(payload: &[u8]) -> Result<(), ImageNormalizationError> {
    const COMPRESSION_MASK: u8 = 0b11;
    const COMPRESSION_NONE: u8 = 0;
    const COMPRESSION_LOSSLESS: u8 = 1;

    let (header, stream) = payload
        .split_first()
        .ok_or(ImageNormalizationError::InvalidPayload)?;
    match header & COMPRESSION_MASK {
        COMPRESSION_NONE => Ok(()),
        COMPRESSION_LOSSLESS => preflight_vp8l_stream(&mut Vp8lBitReader::new(stream)),
        _ => Err(ImageNormalizationError::InvalidPayload),
    }
}

fn validate_webp_image_dimensions(
    dimensions: (u32, u32),
    image_dimensions: &mut Option<(u32, u32)>,
) -> Result<(), ImageNormalizationError> {
    if image_dimensions.is_some() {
        return Err(ImageNormalizationError::InvalidPayload);
    }
    validate_input_dimensions(dimensions.0, dimensions.1)?;
    *image_dimensions = Some(dimensions);
    Ok(())
}
