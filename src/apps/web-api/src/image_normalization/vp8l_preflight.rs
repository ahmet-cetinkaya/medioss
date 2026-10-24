use super::ImageNormalizationError;

// VP8L prefix-code alphabets (WebP lossless spec): green = 256 literals + 24 length codes
// (+ color-cache entries), the other channels are 256 wide, distance has 40 codes.
const GREEN_TREE_ALPHABET: u32 = 256 + 24;
const GREEN_LITERAL_LIMIT: u16 = 256;
const CHANNEL_TREE_ALPHABET: u32 = 256;
const DISTANCE_TREE_ALPHABET: u32 = 40;
const MAX_COLOR_CACHE_BITS: u8 = 11;

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
pub(super) fn preflight_vp8l(payload: &[u8]) -> Result<(u32, u32), ImageNormalizationError> {
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

pub(super) fn preflight_vp8l_stream(
    reader: &mut Vp8lBitReader<'_>,
) -> Result<(), ImageNormalizationError> {
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

pub(super) struct Vp8lBitReader<'a> {
    bytes: &'a [u8],
    bit_offset: usize,
}

impl<'a> Vp8lBitReader<'a> {
    pub(super) const fn new(bytes: &'a [u8]) -> Self {
        Self {
            bytes,
            bit_offset: 0,
        }
    }

    /// Reads `bit_count` bits least-significant-bit first, matching the decoder's bit order.
    pub(super) fn read(&mut self, bit_count: u8) -> Result<u32, ImageNormalizationError> {
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

#[cfg(test)]
#[path = "vp8l_preflight.tests.rs"]
mod tests;
