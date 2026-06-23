use std::{ptr, slice};

use morph_da_decoder_core::{Error, decompress_morph_da_zstd as decompress, decompressed_size_bound};

const OK: *const i8 = ptr::null();

const EMPTY_INPUT: &[u8] = b"empty input\0";
const INVALID_FRAME: &[u8] = b"invalid frame\0";
const CONTENT_SIZE_MISMATCH: &[u8] = b"content size mismatch\0";
const OUTPUT_BUFFER_TOO_SMALL: &[u8] = b"output buffer too small\0";
const DECOMPRESSED_SIZE_TOO_LARGE: &[u8] = b"decompressed size too large\0";

#[unsafe(no_mangle)]
pub extern "C" fn morph_da_zstd_decompress_bound(
    src: *const u8,
    src_size: u64,
    output_size: *mut u64,
) -> *const i8 {
    if output_size.is_null() {
        return err_ptr(Error::InvalidFrame);
    }

    let src = match input_slice(src, src_size) {
        Ok(src) => src,
        Err(err) => return err_ptr(err),
    };
    match decompressed_size_bound(src) {
        Ok(size) => {
            unsafe {
                *output_size = size;
            }
            OK
        }
        Err(err) => err_ptr(err),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn morph_da_zstd_decompress(
    src: *const u8,
    src_size: u64,
    output_buf: *mut u8,
    output_buf_size: *mut u64,
) -> *const i8 {
    if output_buf_size.is_null() {
        return err_ptr(Error::InvalidFrame);
    }

    let src = match input_slice(src, src_size) {
        Ok(src) => src,
        Err(err) => return err_ptr(err),
    };
    let capacity = unsafe { *output_buf_size };

    match decompress(src) {
        Ok(decoded) => {
            unsafe {
                *output_buf_size = decoded.len() as u64;
            }

            if decoded.len() as u64 > capacity {
                return err_ptr(Error::OutputBufferTooSmall);
            }
            if decoded.is_empty() {
                return OK;
            }
            if output_buf.is_null() {
                return err_ptr(Error::InvalidFrame);
            }

            unsafe {
                ptr::copy_nonoverlapping(decoded.as_ptr(), output_buf, decoded.len());
            }
            OK
        }
        Err(err) => err_ptr(err),
    }
}

fn input_slice<'a>(src: *const u8, src_size: u64) -> morph_da_decoder_core::Result<&'a [u8]> {
    if src_size == 0 {
        return Ok(&[]);
    }
    if src.is_null() {
        return Err(Error::InvalidFrame);
    }

    let len = usize::try_from(src_size).map_err(|_| Error::InvalidFrame)?;
    Ok(unsafe { slice::from_raw_parts(src, len) })
}

fn err_ptr(err: Error) -> *const i8 {
    match err {
        Error::EmptyInput => EMPTY_INPUT.as_ptr().cast(),
        Error::InvalidFrame => INVALID_FRAME.as_ptr().cast(),
        Error::ContentSizeMismatch => CONTENT_SIZE_MISMATCH.as_ptr().cast(),
        Error::OutputBufferTooSmall => OUTPUT_BUFFER_TOO_SMALL.as_ptr().cast(),
        Error::DecompressedSizeTooLarge => DECOMPRESSED_SIZE_TOO_LARGE.as_ptr().cast(),
    }
}
