use super::*;

fn lossless_header(width: u32, height: u32) -> Vec<u8> {
    // Minimal single-node-tree VP8L stream bytes are exercised through
    // preflight_vp8l; these helpers build only the signed 14-bit dimensions.
    let pack = |dimension: u32| dimension - 1;
    let mut bytes = vec![0x2f];
    let w = pack(width);
    let h = pack(height);
    // 14-bit widths packed LSB-first, then alpha hint + version nibble (0).
    bytes.extend_from_slice(&(w as u16).to_le_bytes()[..1]);
    bytes.push(((w >> 8) & 0x3f) as u8 | ((h & 0x3f) << 6) as u8);
    bytes.push((h >> 6) as u8);
    bytes
}

#[test]
fn rejects_a_payload_without_the_lossless_signature() {
    let mut bytes = lossless_header(2, 2);
    bytes[0] = 0x00;

    assert_eq!(
        preflight_vp8l(&bytes),
        Err(ImageNormalizationError::InvalidPayload)
    );
}

#[test]
fn rejects_a_nonzero_version_field() {
    let mut bytes = lossless_header(2, 2);
    // version bits sit after the 8-bit signature and 2x14-bit dimensions plus
    // 1 alpha-hint bit: bit 51..54 of the stream; set the lowest version bit.
    let bit = 8 + 14 + 14 + 1 + 3;
    bytes.resize(bit / 8 + 1, 0);
    bytes[bit / 8] |= 1 << (bit % 8);

    assert_eq!(
        preflight_vp8l(&bytes),
        Err(ImageNormalizationError::InvalidPayload)
    );
}

#[test]
fn bit_reader_reads_bits_lsb_first_across_bytes() {
    let mut reader = Vp8lBitReader::new(&[0b1011_0100, 0b0000_0011]);

    assert_eq!(reader.read(1), Ok(0));
    assert_eq!(reader.read(3), Ok(0b010));
    assert_eq!(reader.read(4), Ok(0b1011));
}

#[test]
fn bit_reader_fails_when_the_payload_runs_out() {
    let mut reader = Vp8lBitReader::new(&[0xff]);

    assert_eq!(reader.read(1), Ok(1));
    assert_eq!(
        reader.read(16),
        Err(ImageNormalizationError::InvalidPayload)
    );
}
