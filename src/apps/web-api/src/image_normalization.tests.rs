use super::*;

fn raster(format: image::ImageFormat) -> Vec<u8> {
    let image = image::DynamicImage::new_rgba8(400, 200);
    let mut bytes = Vec::new();
    image
        .write_to(&mut Cursor::new(&mut bytes), format)
        .unwrap();
    bytes
}

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

fn vp8x(width: u32, height: u32, flags: u8) -> Vec<u8> {
    let mut payload = vec![flags, 0, 0, 0];
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

fn oversized_png() -> Vec<u8> {
    let mut bytes = raster(image::ImageFormat::Png);
    bytes[16..20].copy_from_slice(&2_001_u32.to_be_bytes());
    bytes[20..24].copy_from_slice(&2_000_u32.to_be_bytes());
    bytes
}

fn oversized_jpeg() -> Vec<u8> {
    let mut bytes = raster(image::ImageFormat::Jpeg);
    let sof = bytes
        .windows(2)
        .position(|marker| marker == [0xff, 0xc0])
        .unwrap();
    bytes[sof + 5..sof + 7].copy_from_slice(&2_000_u16.to_be_bytes());
    bytes[sof + 7..sof + 9].copy_from_slice(&2_001_u16.to_be_bytes());
    bytes
}

#[tokio::test(flavor = "current_thread")]
async fn converts_png_jpeg_and_webp_to_bounded_webp() {
    let _lock = NORMALIZATION_TEST_LOCK.lock().await;
    for format in [
        image::ImageFormat::Png,
        image::ImageFormat::Jpeg,
        image::ImageFormat::WebP,
    ] {
        let normalized = normalize(
            raster(format),
            ImageSpec {
                maximum_dimension: 128,
                maximum_output_bytes: 64 * 1024,
                square: false,
            },
        )
        .await
        .unwrap();
        assert_eq!(
            image::guess_format(&normalized).unwrap(),
            image::ImageFormat::WebP
        );
        let decoded = image::load_from_memory(&normalized).unwrap();
        assert_eq!((decoded.width(), decoded.height()), (128, 64));
    }
}

#[tokio::test(flavor = "current_thread")]
async fn crops_avatars_to_a_square() {
    let _lock = NORMALIZATION_TEST_LOCK.lock().await;
    let normalized = normalize(
        raster(image::ImageFormat::Png),
        ImageSpec {
            maximum_dimension: 64,
            maximum_output_bytes: 64 * 1024,
            square: true,
        },
    )
    .await
    .unwrap();
    let decoded = image::load_from_memory(&normalized).unwrap();
    assert_eq!((decoded.width(), decoded.height()), (64, 64));
}

#[tokio::test(flavor = "current_thread")]
async fn rejects_malformed_webp_before_decode() {
    let _lock = NORMALIZATION_TEST_LOCK.lock().await;
    let error = normalize(
        b"RIFF\x04\0\0\0WEBP".to_vec(),
        ImageSpec {
            maximum_dimension: 64,
            maximum_output_bytes: 64 * 1024,
            square: false,
        },
    )
    .await
    .unwrap_err();
    assert_eq!(error, ImageNormalizationError::InvalidPayload);
}

#[test]
fn rejects_oversized_embedded_image_after_small_canvas() {
    let bytes = riff(&[
        (*b"VP8X", vp8x(1, 1, 0)),
        (*b"VP8 ", vp8(MAX_DECODE_DIMENSION as u16 + 1, 1)),
    ]);

    assert_eq!(
        preflight_webp(&bytes),
        Err(ImageNormalizationError::InvalidPayload)
    );
}

#[test]
fn rejects_truncated_odd_chunk_padding() {
    let mut bytes = riff(&[(*b"EXIF", vec![1])]);
    bytes.pop();
    let declared_size = (bytes.len() - 8) as u32;
    bytes[4..8].copy_from_slice(&declared_size.to_le_bytes());

    assert_eq!(
        preflight_webp(&bytes),
        Err(ImageNormalizationError::InvalidPayload)
    );
}

#[test]
fn rejects_exif_length_beyond_riff_payload() {
    let mut bytes = riff(&[(*b"EXIF", Vec::new())]);
    bytes[16..20].copy_from_slice(&8_u32.to_le_bytes());

    assert_eq!(
        preflight_webp(&bytes),
        Err(ImageNormalizationError::InvalidPayload)
    );
}

#[test]
fn rejects_animated_webp_chunks_and_flags() {
    for chunks in [
        vec![(*b"VP8X", vp8x(1, 1, 0x02)), (*b"VP8 ", vp8(1, 1))],
        vec![(*b"VP8X", vp8x(1, 1, 0)), (*b"ANIM", vec![0; 6])],
        vec![(*b"VP8X", vp8x(1, 1, 0)), (*b"ANMF", Vec::new())],
    ] {
        assert_eq!(
            preflight_webp(&riff(&chunks)),
            Err(ImageNormalizationError::InvalidPayload)
        );
    }
}

#[tokio::test(flavor = "current_thread")]
async fn rejects_requests_while_the_normalizer_is_busy() {
    let _lock = NORMALIZATION_TEST_LOCK.lock().await;
    let permit = NORMALIZATION_WORKER.clone().try_acquire_owned().unwrap();

    assert_eq!(
        normalize(
            raster(image::ImageFormat::Png),
            ImageSpec {
                maximum_dimension: 64,
                maximum_output_bytes: 64 * 1024,
                square: false,
            },
        )
        .await,
        Err(ImageNormalizationError::Busy)
    );
    drop(permit);
}

#[tokio::test(flavor = "current_thread")]
async fn rejects_oversized_png_and_jpeg_before_decode() {
    let _lock = NORMALIZATION_TEST_LOCK.lock().await;
    for bytes in [oversized_png(), oversized_jpeg()] {
        assert_eq!(
            normalize(
                bytes,
                ImageSpec {
                    maximum_dimension: 64,
                    maximum_output_bytes: 64 * 1024,
                    square: false,
                },
            )
            .await,
            Err(ImageNormalizationError::InvalidPayload)
        );
    }
}

#[test]
fn rejects_input_and_output_exceeding_pixel_limit() {
    assert_eq!(
        validate_input_dimensions(2_001, 2_000),
        Err(ImageNormalizationError::InvalidPayload)
    );
    assert_eq!(
        validate_output_dimensions(ImageSpec {
            maximum_dimension: 2_001,
            maximum_output_bytes: 1,
            square: false,
        }),
        Err(ImageNormalizationError::OutputTooLarge)
    );
}

/// Packs `(value, bit_count)` fields least-significant-bit first, like the VP8L bitstream.
fn vp8l_bits(fields: &[(u32, u8)]) -> Vec<u8> {
    let mut bytes = Vec::new();
    let mut bit_offset = 0;
    for &(value, bit_count) in fields {
        for bit in 0..bit_count {
            if bit_offset % 8 == 0 {
                bytes.push(0);
            }
            *bytes.last_mut().unwrap() |= (((value >> bit) & 1) as u8) << (bit_offset % 8);
            bit_offset += 1;
        }
    }
    bytes
}

/// A VP8L header for a 1x1 image with no transforms, followed by `entropy_header` fields.
fn vp8l_webp(entropy_header: &[(u32, u8)]) -> Vec<u8> {
    let mut fields = vec![(0x2f, 8), (0, 14), (0, 14), (0, 1), (0, 3), (0, 1)];
    fields.extend_from_slice(entropy_header);
    riff(&[(*b"VP8L", vp8l_bits(&fields))])
}

#[test]
fn rejects_meta_huffman_streams_that_could_allocate_thousands_of_groups() {
    // No color cache, meta-Huffman flag set: the decoder would size its group table from
    // entropy-coded pixels, up to 65,536 groups from a tiny file.
    let bytes = vp8l_webp(&[(0, 1), (1, 1)]);

    assert_eq!(
        preflight_webp(&bytes),
        Err(ImageNormalizationError::InvalidPayload)
    );
}

#[test]
fn rejects_color_cache_bits_beyond_the_decoder_limit() {
    for bits in [0, 12, 15] {
        let bytes = vp8l_webp(&[(1, 1), (bits, 4), (0, 1)]);

        assert_eq!(
            preflight_webp(&bytes),
            Err(ImageNormalizationError::InvalidPayload),
            "cache bits {bits}"
        );
    }
}

#[test]
fn rejects_a_truncated_vp8l_entropy_header() {
    // Stops on a byte boundary right after the version bits, so no padding bits can stand in
    // for the transform, color-cache and meta-Huffman flags that must follow.
    let header = vp8l_bits(&[(0x2f, 8), (0, 14), (0, 14), (0, 1), (0, 3)]);
    let bytes = riff(&[(*b"VP8L", header)]);

    assert_eq!(
        preflight_webp(&bytes),
        Err(ImageNormalizationError::InvalidPayload)
    );
}

#[test]
fn accepts_a_genuine_lossless_webp_without_meta_huffman_groups() {
    assert_eq!(preflight_webp(&raster(image::ImageFormat::WebP)), Ok(()));
}

/// A lossy-with-alpha WebP whose ALPH chunk holds `alpha_stream` after the given header byte.
fn lossy_with_alpha(alpha_header: u8, alpha_stream_fields: &[(u32, u8)]) -> Vec<u8> {
    let mut alpha = vec![alpha_header];
    alpha.extend(vp8l_bits(alpha_stream_fields));
    riff(&[
        (*b"VP8X", vp8x(1, 1, 0x10)),
        (*b"ALPH", alpha),
        (*b"VP8 ", vp8(1, 1)),
    ])
}

#[test]
fn rejects_meta_huffman_in_a_lossless_alpha_chunk() {
    // compression 1 = lossless: no transform, no color cache, meta-Huffman flag set.
    let bytes = lossy_with_alpha(1, &[(0, 1), (0, 1), (1, 1)]);

    assert_eq!(
        preflight_webp(&bytes),
        Err(ImageNormalizationError::InvalidPayload)
    );
}

#[test]
fn rejects_unknown_alpha_compression() {
    let bytes = lossy_with_alpha(2, &[(0, 1)]);

    assert_eq!(
        preflight_webp(&bytes),
        Err(ImageNormalizationError::InvalidPayload)
    );
}

#[test]
fn accepts_lossy_webp_with_lossless_and_raw_alpha() {
    // No transform, no color cache, no meta-Huffman: the shape ordinary encoders produce.
    assert_eq!(
        preflight_webp(&lossy_with_alpha(1, &[(0, 1), (0, 1), (0, 1)])),
        Ok(())
    );
    assert_eq!(preflight_webp(&lossy_with_alpha(0, &[(0, 1)])), Ok(()));
}
