//! zkVM guest-facing Morph DA zstd decoder API.

pub use morph_da_decoder_core::{Error, Result};

pub fn decompress_morph_da_zstd(payload: &[u8]) -> Result<Vec<u8>> {
    morph_da_decoder_core::decompress_morph_da_zstd(payload)
}
