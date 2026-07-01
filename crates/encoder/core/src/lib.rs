//! Morph DA zstd encoder (compression).
//!
//! Mirror of [`morph_da_decoder_core`](../../decoder/core): pure-Rust API with
//! no C ABI, so it can be reused directly by other Rust crates. Compression is
//! delegated to the C `libzstd` via the [`zstd`] crate.
//!
//! The frame is produced with magic bytes omitted, matching the decoder which
//! restores them before decoding.

use std::{fmt, io::Write};

use zstd::{
    stream::Encoder,
    zstd_safe::{CParameter, ParamSwitch},
};

// we use offset window no more than = 17
pub const CL_WINDOW_LIMIT: u32 = 17;

/// zstd block size target.
pub const N_BLOCK_SIZE_TARGET: u32 = 124 * 1024;

/// Errors that can occur while encoding a Morph DA zstd payload.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    EmptyInput,
    OutputBufferTooSmall,
    InternalEncoderError,
}

impl Error {
    /// Returns a static, human-readable description of the error.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::EmptyInput => "empty input",
            Self::OutputBufferTooSmall => "output buffer too small",
            Self::InternalEncoderError => "internal encoder error",
        }
    }
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

pub type Result<T> = std::result::Result<T, Error>;

/// Builds the zstd encoder configured for Morph DA frames.
///
/// All parameter calls use compile-time-constant values, so they cannot fail in
/// practice; failures here indicate a broken zstd build and are treated as
/// `expect`-level invariants.
pub(crate) fn init_zstd_encoder(target_block_size: u32) -> Encoder<'static, Vec<u8>> {
    let mut encoder = Encoder::new(Vec::new(), 0).expect("infallible");

    // disable compression of literals, i.e. literals will be raw bytes.
    encoder
        .set_parameter(CParameter::LiteralCompressionMode(ParamSwitch::Disable))
        .expect("infallible");
    // Set window log = 17
    encoder
        .set_parameter(CParameter::WindowLog(CL_WINDOW_LIMIT))
        .expect("infallible");
    // set target block size to fit within a single block.
    encoder
        .set_parameter(CParameter::TargetCBlockSize(target_block_size))
        .expect("infallible");
    // do not include the checksum at the end of the encoded data.
    encoder.include_checksum(false).expect("infallible");
    // do not include magic bytes at the start of the frame since we will have a
    // single frame.
    encoder.include_magicbytes(false).expect("infallible");
    // do not include dictionary id so we have more simple content
    encoder.include_dictid(false).expect("infallible");
    // include the content size to know at decode time the expected size of
    // decoded data.
    encoder.include_contentsize(true).expect("infallible");

    encoder
}

/// Upper bound on the compressed size of `src_len` input bytes.
///
/// `ZSTD_compressBound` accounts for worst-case expansion; our magic-less frame
/// is never larger, so this is a safe output-buffer size.
pub fn compressed_size_bound(src_len: usize) -> usize {
    zstd::zstd_safe::compress_bound(src_len)
}

/// Compresses a Morph DA batch into a single magic-less zstd frame.
pub fn compress_morph_da_zstd(batch: &[u8]) -> Result<Vec<u8>> {
    let mut encoder = init_zstd_encoder(N_BLOCK_SIZE_TARGET);
    encoder
        .set_pledged_src_size(Some(batch.len() as u64))
        .map_err(|_| Error::InternalEncoderError)?;
    encoder
        .write_all(batch)
        .map_err(|_| Error::InternalEncoderError)?;

    encoder.finish().map_err(|_| Error::InternalEncoderError)
}

#[cfg(test)]
mod tests {
    use super::*;
    use morph_da_decoder_core::decompress_morph_da_zstd;

    #[test]
    fn round_trips_through_decoder() {
        let original = b"morph da batch payload \x00\x01\x02 repeated repeated repeated".repeat(50);

        let encoded = compress_morph_da_zstd(&original).expect("compress");
        // Magicless frame: must not start with the zstd magic bytes.
        assert_ne!(&encoded[..4], &morph_da_decoder_core::ZSTD_MAGIC_BYTES);

        let decoded = decompress_morph_da_zstd(&encoded).expect("decompress");
        assert_eq!(decoded, original);
    }

    #[test]
    fn compresses_empty_input() {
        // zstd produces a valid (small) frame for empty input; round-trip of an
        // empty payload is handled by the FFI/Go layer, not core.
        let encoded = compress_morph_da_zstd(b"").expect("compress empty");
        assert!(!encoded.is_empty());
    }
}
