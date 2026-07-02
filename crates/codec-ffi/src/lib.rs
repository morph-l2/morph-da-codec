//! Single staticlib aggregating the Morph DA decoder and encoder C ABIs.
//!
//! This crate holds no logic of its own. It exists solely to produce one `.a`
//! for Go to link, bundling exactly one copy of the Rust runtime. The
//! decoder/encoder FFI crates are plain rlibs; re-exporting their items here
//! keeps their `#[unsafe(no_mangle)]` symbols reachable from the staticlib root
//! so the linker does not strip them as dead code.

pub use morph_da_decoder_ffi::*;
pub use morph_da_encoder_ffi::*;
