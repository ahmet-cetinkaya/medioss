use std::{
    io::Cursor,
    sync::{Arc, LazyLock},
};

const MAX_DECODE_DIMENSION: u32 = 8_192;
const MAX_INPUT_PIXELS: u64 = 4_000_000;
const MAX_DECODE_ALLOC: u64 = 64 * 1024 * 1024;
// VP8L prefix-code alphabets (WebP lossless spec): green = 256 literals + 24 length codes
// (+ color-cache entries), the other channels are 256 wide, distance has 40 codes.
const GREEN_TREE_ALPHABET: u32 = 256 + 24;
const GREEN_LITERAL_LIMIT: u16 = 256;
const CHANNEL_TREE_ALPHABET: u32 = 256;
const DISTANCE_TREE_ALPHABET: u32 = 40;
const MAX_COLOR_CACHE_BITS: u8 = 11;
// Brief contention queues; sustained flooding still gets `Busy`.
#[cfg(not(test))]
const NORMALIZATION_ADMISSION_WAIT: std::time::Duration = std::time::Duration::from_secs(2);
#[cfg(test)]
const NORMALIZATION_ADMISSION_WAIT: std::time::Duration = std::time::Duration::from_millis(200);
static NORMALIZATION_WORKER: LazyLock<Arc<tokio::sync::Semaphore>> =
    LazyLock::new(|| Arc::new(tokio::sync::Semaphore::new(1)));

#[cfg(test)]
pub(crate) static NORMALIZATION_TEST_LOCK: LazyLock<tokio::sync::Mutex<()>> =
    LazyLock::new(|| tokio::sync::Mutex::new(()));

#[derive(Debug, Clone, Copy)]
pub struct ImageSpec {
    pub maximum_dimension: u32,
    pub maximum_output_bytes: usize,
    pub square: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageNormalizationError {
    InvalidPayload,
    OutputTooLarge,
    WorkerFailed,
    Busy,
}

pub async fn normalize(
    bytes: Vec<u8>,
    specification: ImageSpec,
) -> Result<Vec<u8>, ImageNormalizationError> {
    let permit = tokio::time::timeout(
        NORMALIZATION_ADMISSION_WAIT,
        NORMALIZATION_WORKER.clone().acquire_owned(),
    )
    .await
    .map_err(|_| ImageNormalizationError::Busy)?
    .map_err(|_| ImageNormalizationError::WorkerFailed)?;

    tokio::task::spawn_blocking(move || {
        let _permit = permit;
        normalize_blocking(&bytes, specification)
    })
    .await
    .map_err(|_| ImageNormalizationError::WorkerFailed)?
}

fn normalize_blocking(
    bytes: &[u8],
    specification: ImageSpec,
) -> Result<Vec<u8>, ImageNormalizationError> {
    validate_output_dimensions(specification)?;

    let format = image::guess_format(bytes).map_err(|_| ImageNormalizationError::InvalidPayload)?;
    if !matches!(
        format,
        image::ImageFormat::Jpeg | image::ImageFormat::Png | image::ImageFormat::WebP
    ) {
        return Err(ImageNormalizationError::InvalidPayload);
    }
    if format == image::ImageFormat::WebP {
        preflight_webp(bytes)?;
    }

    let reader = image_reader(bytes)?;
    let (width, height) = reader
        .into_dimensions()
        .map_err(|_| ImageNormalizationError::InvalidPayload)?;
    validate_input_dimensions(width, height)?;

    let image = image_reader(bytes)?
        .decode()
        .map_err(|_| ImageNormalizationError::InvalidPayload)?;
    let image = if specification.square {
        let side = image.width().min(image.height());
        image
            .crop_imm(
                (image.width() - side) / 2,
                (image.height() - side) / 2,
                side,
                side,
            )
            .resize_to_fill(
                specification.maximum_dimension,
                specification.maximum_dimension,
                image::imageops::FilterType::Lanczos3,
            )
    } else {
        image.resize(
            specification.maximum_dimension,
            specification.maximum_dimension,
            image::imageops::FilterType::Lanczos3,
        )
    };

    let mut output = Vec::new();
    image
        .write_to(&mut Cursor::new(&mut output), image::ImageFormat::WebP)
        .map_err(|_| ImageNormalizationError::WorkerFailed)?;
    if output.is_empty() || output.len() > specification.maximum_output_bytes {
        return Err(ImageNormalizationError::OutputTooLarge);
    }
    Ok(output)
}

fn image_reader(
    bytes: &[u8],
) -> Result<image::ImageReader<Cursor<&[u8]>>, ImageNormalizationError> {
    let mut reader = image::ImageReader::new(Cursor::new(bytes))
        .with_guessed_format()
        .map_err(|_| ImageNormalizationError::InvalidPayload)?;
    let mut limits = image::Limits::default();
    limits.max_image_width = Some(MAX_DECODE_DIMENSION);
    limits.max_image_height = Some(MAX_DECODE_DIMENSION);
    limits.max_alloc = Some(MAX_DECODE_ALLOC);
    reader.limits(limits);
    Ok(reader)
}

fn validate_input_dimensions(width: u32, height: u32) -> Result<(), ImageNormalizationError> {
    if width == 0
        || height == 0
        || width > MAX_DECODE_DIMENSION
        || height > MAX_DECODE_DIMENSION
        || pixel_count(width, height).is_none_or(|pixels| pixels > MAX_INPUT_PIXELS)
    {
        return Err(ImageNormalizationError::InvalidPayload);
    }
    Ok(())
}

fn validate_output_dimensions(specification: ImageSpec) -> Result<(), ImageNormalizationError> {
    if specification.maximum_dimension == 0
        || pixel_count(
            specification.maximum_dimension,
            specification.maximum_dimension,
        )
        .is_none_or(|pixels| pixels > MAX_INPUT_PIXELS)
    {
        return Err(ImageNormalizationError::OutputTooLarge);
    }
    Ok(())
}

fn pixel_count(width: u32, height: u32) -> Option<u64> {
    u64::from(width).checked_mul(u64::from(height))
}

fn preflight_webp(bytes: &[u8]) -> Result<(), ImageNormalizationError> {
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

/// VP8L pre-decode allocation preflight.
///
/// image-webp 0.2.4 builds the Huffman groups of the top-level ARGB stream outside any
/// `image::Limits` check (the image 0.25.10 WebP wrapper wires no limits into image-webp):
/// `hufftree_groups` (lossless.rs:328-344) is sized from `num_huff_groups`, which is the largest
/// entropy-image pixel plus one (lossless.rs:316-325). Those pixels are entropy-decoded, so the
/// group count is bounded only by the 16-bit pixel field: 65,536 groups of five trees, each tree
/// worth up to ~77 KiB (huffman.rs:82-83: a 1,024-entry u32 lookup table plus 2 * 2,328
/// sixteen-byte tree nodes for the 2,328-symbol green alphabet reachable with an 11-bit color
/// cache), total several GiB from a file far smaller than any upload cap. The worst case is not
/// decidable before decode, but the meta-Huffman flag that enables the path is, so any stream
/// that sets it is rejected. Without it the decoder builds exactly one group (lossless.rs:300,
/// 307), bounded by ~118 KiB of tables plus an 8 KiB color cache and canvas-bounded pixel
/// buffers, comfortably inside MAX_DECODE_ALLOC for legitimate <=4MP images.
fn preflight_vp8l(payload: &[u8]) -> Result<(u32, u32), ImageNormalizationError> {
    let mut reader = Vp8lBitReader::new(payload);
    if reader.read(8)? != 0x2f {
        return Err(ImageNormalizationError::InvalidPayload);
    }
    let dimensions = (reader.read(14)? + 1, reader.read(14)? + 1);
    reader.read(1)?; // alpha-used hint: informational, position-only here
    if reader.read(3)? != 0 {
        return Err(ImageNormalizationError::InvalidPayload); // reserved version number
    }
    preflight_vp8l_stream(&mut reader)?;
    Ok(dimensions)
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

fn preflight_vp8l_stream(reader: &mut Vp8lBitReader<'_>) -> Result<(), ImageNormalizationError> {
    skip_vp8l_transforms(reader)?;
    preflight_vp8l_entropy_header(reader)
}

/// Transform data and the entropy image are entropy-coded sub-streams with `read_meta` false, so
/// they hold exactly one Huffman group (lossless.rs:307) and stay bounded by ~118 KiB of tables
/// plus canvas-sized pixel buffers. Walking past them only needs their bit length, which is
/// decidable without decoding pixels when the decoder's fast path applies: with single-node
/// green/red/blue/alpha trees and a green symbol below 256, the pixel loop fills the sub-image
/// without consuming a bit (lossless.rs:503-532). That is exactly what the vendored encoder
/// writes for transform data (encoder.rs:414-421), so image-crate lossless files stay accepted;
/// any other tree shape would require a full entropy decode to locate the following fields and
/// is rejected instead.
fn skip_vp8l_transforms(reader: &mut Vp8lBitReader<'_>) -> Result<(), ImageNormalizationError> {
    let mut seen_types = 0u8;
    while reader.read(1)? == 1 {
        let transform_type = reader.read(2)? as u8;
        let type_flag = 1 << transform_type;
        if seen_types & type_flag != 0 {
            return Err(ImageNormalizationError::InvalidPayload); // decoder: TransformError
        }
        seen_types |= type_flag;

        match transform_type {
            0 | 1 => {
                reader.read(3)?; // predictor/color block size bits
                skip_vp8l_entropy_coded_image(reader)?;
            }
            2 => {}
            3 => {
                reader.read(8)?; // color table size minus one
                skip_vp8l_entropy_coded_image(reader)?;
            }
            _ => unreachable!(),
        }
    }
    Ok(())
}

fn skip_vp8l_entropy_coded_image(
    reader: &mut Vp8lBitReader<'_>,
) -> Result<(), ImageNormalizationError> {
    let color_cache_bits = read_vp8l_color_cache_bits(reader)?;
    let green_alphabet = match color_cache_bits {
        0 => GREEN_TREE_ALPHABET,
        bits => GREEN_TREE_ALPHABET + (1 << bits),
    };

    let green = read_vp8l_simple_tree(reader, green_alphabet)?;
    let red = read_vp8l_simple_tree(reader, CHANNEL_TREE_ALPHABET)?;
    let blue = read_vp8l_simple_tree(reader, CHANNEL_TREE_ALPHABET)?;
    let alpha = read_vp8l_simple_tree(reader, CHANNEL_TREE_ALPHABET)?;
    let _distance = read_vp8l_simple_tree(reader, DISTANCE_TREE_ALPHABET)?;

    let fast_path_symbols = [green, red, blue, alpha];
    if fast_path_symbols.iter().any(Option::is_none)
        || green.is_some_and(|symbol| symbol >= GREEN_LITERAL_LIMIT)
    {
        return Err(ImageNormalizationError::InvalidPayload);
    }
    Ok(())
}

fn preflight_vp8l_entropy_header(
    reader: &mut Vp8lBitReader<'_>,
) -> Result<(), ImageNormalizationError> {
    read_vp8l_color_cache_bits(reader)?;
    if reader.read(1)? == 1 {
        // Meta-Huffman groups: the only pre-decode-decidable signal of the unbounded allocation
        // documented on `preflight_vp8l`, so the stream is rejected outright.
        return Err(ImageNormalizationError::InvalidPayload);
    }
    Ok(())
}

/// Reads the color-cache field and returns its size in bits, 0 when absent. Mirrors the decoder's
/// 1..=11 validation (lossless.rs:626-636) so the cache allocation stays at 2^11 * 4 bytes
/// (lossless.rs:191).
fn read_vp8l_color_cache_bits(
    reader: &mut Vp8lBitReader<'_>,
) -> Result<u8, ImageNormalizationError> {
    if reader.read(1)? == 0 {
        return Ok(0);
    }
    let bits = reader.read(4)? as u8;
    if !(1..=MAX_COLOR_CACHE_BITS).contains(&bits) {
        return Err(ImageNormalizationError::InvalidPayload);
    }
    Ok(bits)
}

/// Reads one simple Huffman tree, returning its symbol when the tree is single-node or `None`
/// when it holds two symbols. Normal trees are rejected: they cannot be skipped cheaply.
fn read_vp8l_simple_tree(
    reader: &mut Vp8lBitReader<'_>,
    alphabet: u32,
) -> Result<Option<u16>, ImageNormalizationError> {
    if reader.read(1)? == 0 {
        return Err(ImageNormalizationError::InvalidPayload); // normal tree, not skippable
    }
    let symbol_count = reader.read(1)? + 1;
    let symbol_width = if reader.read(1)? == 1 { 8 } else { 1 };
    let first_symbol = reader.read(symbol_width)?;
    if first_symbol >= alphabet {
        return Err(ImageNormalizationError::InvalidPayload);
    }
    if symbol_count == 1 {
        return Ok(Some(first_symbol as u16));
    }
    let second_symbol = reader.read(8)?;
    if second_symbol >= alphabet {
        return Err(ImageNormalizationError::InvalidPayload);
    }
    Ok(None)
}

struct Vp8lBitReader<'a> {
    bytes: &'a [u8],
    bit_offset: usize,
}

impl<'a> Vp8lBitReader<'a> {
    const fn new(bytes: &'a [u8]) -> Self {
        Self {
            bytes,
            bit_offset: 0,
        }
    }

    /// Reads `bit_count` bits least-significant-bit first, matching the decoder's bit order.
    fn read(&mut self, bit_count: u8) -> Result<u32, ImageNormalizationError> {
        let end = self
            .bit_offset
            .checked_add(usize::from(bit_count))
            .ok_or(ImageNormalizationError::InvalidPayload)?;
        if end > self.bytes.len() * 8 {
            return Err(ImageNormalizationError::InvalidPayload);
        }
        let mut value = 0;
        for bit in 0..usize::from(bit_count) {
            let position = self.bit_offset + bit;
            value |= u32::from((self.bytes[position / 8] >> (position % 8)) & 1) << bit;
        }
        self.bit_offset = end;
        Ok(value)
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

#[cfg(test)]
#[path = "image_normalization.tests.rs"]
mod tests;
