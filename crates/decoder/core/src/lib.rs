//! Morph DA zstd first-frame decoder.
//!
//! Morph DA zstd payloads omit the zstd magic bytes. This crate restores the
//! magic bytes, decodes exactly the first zstd frame with `ruzstd`, and ignores
//! all bytes after that frame.

use std::{fmt, io::Read};

use ruzstd::decoding::{FrameDecoder, StreamingDecoder};

/// The zstd magic number, stored as a little-endian `u32`.
pub const ZSTD_MAGIC: u32 = 0xFD2F_B528;
/// The zstd magic number as a little-endian byte array, prepended to payloads.
pub const ZSTD_MAGIC_BYTES: [u8; 4] = ZSTD_MAGIC.to_le_bytes();
/// Upper bound on the decompressed size we are willing to allocate for.
pub const MAX_DECOMPRESSED_SIZE: u64 = 1_000_000_000;

/// Errors that can occur while decoding a Morph DA zstd payload.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    EmptyInput,
    InvalidFrame,
    ContentSizeMismatch,
    OutputBufferTooSmall,
    DecompressedSizeTooLarge,
}

impl Error {
    /// Returns a static, human-readable description of the error.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::EmptyInput => "empty input",
            Self::InvalidFrame => "invalid frame",
            Self::ContentSizeMismatch => "content size mismatch",
            Self::OutputBufferTooSmall => "output buffer too small",
            Self::DecompressedSizeTooLarge => "decompressed size too large",
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

/// Prepends the zstd magic bytes to a raw Morph DA payload.
fn frame_with_magic(payload: &[u8]) -> Result<Vec<u8>> {
    let framed_len = ZSTD_MAGIC_BYTES
        .len()
        .checked_add(payload.len())
        .ok_or(Error::InvalidFrame)?;

    let mut framed = Vec::with_capacity(framed_len);
    framed.extend_from_slice(&ZSTD_MAGIC_BYTES);
    framed.extend_from_slice(payload);

    Ok(framed)
}

/// Decompresses the first zstd frame of a Morph DA payload.
pub fn decompress_morph_da_zstd(payload: &[u8]) -> Result<Vec<u8>> {
    // Read the declared content size up front so we can size buffers and limits.
    let expected_len = decompressed_size_bound(payload)?;
    if expected_len == 0 {
        return Err(Error::InvalidFrame);
    }

    // Pre-allocate the output buffer and cap reads one byte past the declared
    // size, so a frame that decodes to more than it claims is detected.
    let mut output = Vec::with_capacity(
        usize::try_from(expected_len).map_err(|_| Error::DecompressedSizeTooLarge)?,
    );
    let read_limit = expected_len
        .checked_add(1)
        .ok_or(Error::DecompressedSizeTooLarge)?;

    // Restore the magic bytes and decode exactly the first frame.
    let framed = frame_with_magic(payload)?;
    let mut source = framed.as_slice();
    let decoder = StreamingDecoder::new(&mut source).map_err(|_| Error::InvalidFrame)?;
    decoder
        .take(read_limit)
        .read_to_end(&mut output)
        .map_err(|_| Error::InvalidFrame)?;

    // The decoded output must match the declared content size exactly.
    if output.len() as u64 != expected_len {
        return Err(Error::ContentSizeMismatch);
    }

    Ok(output)
}

/// Returns the declared decompressed size of the first frame without decoding it.
pub fn decompressed_size_bound(payload: &[u8]) -> Result<u64> {
    if payload.is_empty() {
        return Err(Error::EmptyInput);
    }

    let framed = frame_with_magic(payload)?;

    // Initialize the frame decoder only to read the frame header metadata.
    let mut source = framed.as_slice();
    let mut decoder = FrameDecoder::new();
    decoder.init(&mut source).map_err(|_| Error::InvalidFrame)?;

    let content_size = decoder.content_size();
    if content_size > MAX_DECOMPRESSED_SIZE {
        return Err(Error::DecompressedSizeTooLarge);
    }

    Ok(content_size)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_empty_input() {
        assert_eq!(decompress_morph_da_zstd(b""), Err(Error::EmptyInput));
    }
}
