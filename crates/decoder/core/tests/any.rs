use morph_da_decoder_core::{Error, decompress_morph_da_zstd};

const HELLO_PAYLOAD: &[u8] = include_bytes!("../../../../testdata/any/text-1.bin");
const EMPTY_FRAME_PAYLOAD: &[u8] = &[0x20, 0x00, 0x01, 0x00, 0x00];

#[test]
fn decompresses_valid_single_frame_payload() {
    let decoded = decompress_morph_da_zstd(HELLO_PAYLOAD).unwrap();
    assert_eq!(decoded, b"hello world");
}

#[test]
fn ignores_single_trailing_byte() {
    let mut payload = HELLO_PAYLOAD.to_vec();
    payload.push(0x05);

    let decoded = decompress_morph_da_zstd(&payload).unwrap();
    assert_eq!(decoded, b"hello world");
}

#[test]
fn ignores_trailing_second_payload() {
    let mut payload = HELLO_PAYLOAD.to_vec();
    payload.extend_from_slice(HELLO_PAYLOAD);

    let decoded = decompress_morph_da_zstd(&payload).unwrap();
    assert_eq!(decoded, b"hello world");
}

#[test]
fn rejects_empty_input() {
    assert_eq!(decompress_morph_da_zstd(b""), Err(Error::EmptyInput));
}

#[test]
fn rejects_empty_frame_payload() {
    assert_eq!(
        decompress_morph_da_zstd(EMPTY_FRAME_PAYLOAD),
        Err(Error::InvalidFrame)
    );
}

#[test]
fn rejects_corrupted_first_frame() {
    let mut payload = HELLO_PAYLOAD.to_vec();
    payload.truncate(payload.len() - 1);

    assert_eq!(decompress_morph_da_zstd(&payload), Err(Error::InvalidFrame));
}

#[test]
fn rejects_content_size_mismatch() {
    let mut payload = HELLO_PAYLOAD.to_vec();
    payload[1] = 12;

    assert_eq!(
        decompress_morph_da_zstd(&payload),
        Err(Error::ContentSizeMismatch)
    );
}

#[test]
fn decompresses_complex_str() {
    let data = std::fs::read("../../../testdata/any/text-2.bin").unwrap();
    let decoded = decompress_morph_da_zstd(&data).unwrap();

    assert_eq!(decoded, b"TEXT: Jun 23 2026 22:26:07 PM (+08:00 UTC)");
}

#[test]
fn decompresses_blobs() {
    // 20-blobs.bin's preimage is repeat(65536) of "76f91869161dc4348230d5f60883dd17462035f4"
    let data = std::fs::read("../../../testdata/any/20-blobs.bin").unwrap();
    let decoded = decompress_morph_da_zstd(&data).unwrap();

    assert_eq!(
        decoded[0..40],
        b"76f91869161dc4348230d5f60883dd17462035f4".to_vec()
    );
    assert_eq!(decoded.len(), 2621440);
}
