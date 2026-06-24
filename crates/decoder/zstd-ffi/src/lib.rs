use std::{
    os::raw::{c_char, c_uchar},
    panic::{self, AssertUnwindSafe},
    ptr, slice,
};

use morph_da_decoder_core::{
    Error, decompress_morph_da_zstd as decompress, decompressed_size_bound,
};

const OK: *const c_char = ptr::null();

const EMPTY_INPUT: &[u8] = b"empty input\0";
const INVALID_FRAME: &[u8] = b"invalid frame\0";
const CONTENT_SIZE_MISMATCH: &[u8] = b"content size mismatch\0";
const OUTPUT_BUFFER_TOO_SMALL: &[u8] = b"output buffer too small\0";
const DECOMPRESSED_SIZE_TOO_LARGE: &[u8] = b"decompressed size too large\0";
const INTERNAL_DECODER_ERROR: &[u8] = b"internal decoder error\0";

/// Computes the decompressed size bound.
///
/// # Safety
///
/// `src` must be valid for `src_size` readable bytes unless `src_size == 0`.
/// `output_size` must be non-null and valid for writing one `u64`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn morph_da_zstd_decompress_bound(
    src: *const c_uchar,
    src_size: u64,
    output_size: *mut u64,
) -> *const c_char {
    match panic::catch_unwind(AssertUnwindSafe(|| unsafe {
        decompress_bound_impl(src, src_size, output_size)
    })) {
        Ok(Ok(())) => OK,
        Ok(Err(err)) => err_ptr(err),
        Err(_) => internal_decoder_error_ptr(),
    }
}

/// Decompresses into a caller-provided buffer.
///
/// # Safety
///
/// `src` must be valid for `src_size` readable bytes unless `src_size == 0`.
/// `output_buf_size` must be non-null and valid for one `u64`.
/// If output is produced, `output_buf` must be valid for the input capacity and
/// must not overlap `src`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn morph_da_zstd_decompress(
    src: *const c_uchar,
    src_size: u64,
    output_buf: *mut c_uchar,
    output_buf_size: *mut u64,
) -> *const c_char {
    match panic::catch_unwind(AssertUnwindSafe(|| unsafe {
        decompress_impl(src, src_size, output_buf, output_buf_size)
    })) {
        Ok(Ok(())) => OK,
        Ok(Err(err)) => err_ptr(err),
        Err(_) => internal_decoder_error_ptr(),
    }
}

unsafe fn decompress_bound_impl(
    src: *const c_uchar,
    src_size: u64,
    output_size: *mut u64,
) -> morph_da_decoder_core::Result<()> {
    if output_size.is_null() {
        return Err(Error::InvalidFrame);
    }

    let src = input_slice(src, src_size)?;
    let size = decompressed_size_bound(src)?;
    unsafe {
        *output_size = size;
    }
    Ok(())
}

unsafe fn decompress_impl(
    src: *const c_uchar,
    src_size: u64,
    output_buf: *mut c_uchar,
    output_buf_size: *mut u64,
) -> morph_da_decoder_core::Result<()> {
    if output_buf_size.is_null() {
        return Err(Error::InvalidFrame);
    }

    let src = input_slice(src, src_size)?;
    let capacity = unsafe { *output_buf_size };
    let decoded = decompress(src)?;

    unsafe {
        *output_buf_size = decoded.len() as u64;
    }

    if decoded.len() as u64 > capacity {
        return Err(Error::OutputBufferTooSmall);
    }
    if decoded.is_empty() {
        return Ok(());
    }
    if output_buf.is_null() {
        return Err(Error::InvalidFrame);
    }

    unsafe {
        ptr::copy_nonoverlapping(decoded.as_ptr(), output_buf, decoded.len());
    }
    Ok(())
}

fn input_slice<'a>(src: *const c_uchar, src_size: u64) -> morph_da_decoder_core::Result<&'a [u8]> {
    if src_size == 0 {
        return Ok(&[]);
    }
    if src.is_null() {
        return Err(Error::InvalidFrame);
    }

    let len = usize::try_from(src_size).map_err(|_| Error::InvalidFrame)?;
    Ok(unsafe { slice::from_raw_parts(src, len) })
}

fn err_ptr(err: Error) -> *const c_char {
    match err {
        Error::EmptyInput => EMPTY_INPUT.as_ptr().cast(),
        Error::InvalidFrame => INVALID_FRAME.as_ptr().cast(),
        Error::ContentSizeMismatch => CONTENT_SIZE_MISMATCH.as_ptr().cast(),
        Error::OutputBufferTooSmall => OUTPUT_BUFFER_TOO_SMALL.as_ptr().cast(),
        Error::DecompressedSizeTooLarge => DECOMPRESSED_SIZE_TOO_LARGE.as_ptr().cast(),
    }
}

fn internal_decoder_error_ptr() -> *const c_char {
    INTERNAL_DECODER_ERROR.as_ptr().cast()
}
