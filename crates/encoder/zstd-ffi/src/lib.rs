//! C ABI for the Morph DA zstd encoder.
//!
//! Mirror of [`morph_da_decoder_ffi`](../../decoder/zstd-ffi): thin
//! `#[unsafe(no_mangle)] extern "C"` wrappers over [`morph_da_encoder_core`].
//! These are compiled into the single `codec-ffi` staticlib, not a staticlib of
//! their own.

use std::{
    os::raw::{c_char, c_uchar},
    panic::{self, AssertUnwindSafe},
    ptr, slice,
};

use morph_da_encoder_core::{Error, compress_morph_da_zstd as compress, compressed_size_bound};

const OK: *const c_char = ptr::null();

const EMPTY_INPUT: &[u8] = b"empty input\0";
const OUTPUT_BUFFER_TOO_SMALL: &[u8] = b"output buffer too small\0";
const INTERNAL_ENCODER_ERROR: &[u8] = b"internal encoder error\0";

/// Computes an upper bound on the compressed size.
///
/// # Safety
///
/// `src` must be valid for `src_size` readable bytes unless `src_size == 0`.
/// `output_size` must be non-null and valid for writing one `u64`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn morph_da_zstd_compress_bound(
    src: *const c_uchar,
    src_size: u64,
    output_size: *mut u64,
) -> *const c_char {
    match panic::catch_unwind(AssertUnwindSafe(|| unsafe {
        compress_bound_impl(src, src_size, output_size)
    })) {
        Ok(Ok(())) => OK,
        Ok(Err(err)) => err_ptr(err),
        Err(_) => internal_encoder_error_ptr(),
    }
}

/// Compresses into a caller-provided buffer.
///
/// # Safety
///
/// `src` must be valid for `src_size` readable bytes unless `src_size == 0`.
/// `output_buf_size` must be non-null and valid for one `u64`.
/// If output is produced, `output_buf` must be valid for the input capacity and
/// must not overlap `src`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn morph_da_zstd_compress(
    src: *const c_uchar,
    src_size: u64,
    output_buf: *mut c_uchar,
    output_buf_size: *mut u64,
) -> *const c_char {
    match panic::catch_unwind(AssertUnwindSafe(|| unsafe {
        compress_impl(src, src_size, output_buf, output_buf_size)
    })) {
        Ok(Ok(())) => OK,
        Ok(Err(err)) => err_ptr(err),
        Err(_) => internal_encoder_error_ptr(),
    }
}

unsafe fn compress_bound_impl(
    src: *const c_uchar,
    src_size: u64,
    output_size: *mut u64,
) -> morph_da_encoder_core::Result<()> {
    if output_size.is_null() {
        return Err(Error::InternalEncoderError);
    }

    let src = input_slice(src, src_size)?;
    let bound = compressed_size_bound(src.len());
    unsafe {
        *output_size = bound as u64;
    }
    Ok(())
}

unsafe fn compress_impl(
    src: *const c_uchar,
    src_size: u64,
    output_buf: *mut c_uchar,
    output_buf_size: *mut u64,
) -> morph_da_encoder_core::Result<()> {
    if output_buf_size.is_null() {
        return Err(Error::InternalEncoderError);
    }

    let src = input_slice(src, src_size)?;
    let capacity = unsafe { *output_buf_size };
    let encoded = compress(src)?;

    unsafe {
        *output_buf_size = encoded.len() as u64;
    }

    if encoded.len() as u64 > capacity {
        return Err(Error::OutputBufferTooSmall);
    }
    if encoded.is_empty() {
        return Ok(());
    }
    if output_buf.is_null() {
        return Err(Error::InternalEncoderError);
    }

    unsafe {
        ptr::copy_nonoverlapping(encoded.as_ptr(), output_buf, encoded.len());
    }
    Ok(())
}

fn input_slice<'a>(src: *const c_uchar, src_size: u64) -> morph_da_encoder_core::Result<&'a [u8]> {
    if src_size == 0 {
        return Err(Error::EmptyInput);
    }
    if src.is_null() {
        return Err(Error::InternalEncoderError);
    }

    let len = usize::try_from(src_size).map_err(|_| Error::InternalEncoderError)?;
    Ok(unsafe { slice::from_raw_parts(src, len) })
}

fn err_ptr(err: Error) -> *const c_char {
    match err {
        Error::EmptyInput => EMPTY_INPUT.as_ptr().cast(),
        Error::OutputBufferTooSmall => OUTPUT_BUFFER_TOO_SMALL.as_ptr().cast(),
        Error::InternalEncoderError => INTERNAL_ENCODER_ERROR.as_ptr().cast(),
    }
}

fn internal_encoder_error_ptr() -> *const c_char {
    INTERNAL_ENCODER_ERROR.as_ptr().cast()
}
