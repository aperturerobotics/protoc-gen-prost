//! WASI reactor entry point for protoc-gen-prost
//!
//! This module provides exported functions for running protoc-gen-prost as a WASI
//! reactor, enabling it to be embedded in WebAssembly hosts like wazero.
//!
//! # Exported Functions
//!
//! - `prost_malloc(size)` - Allocate memory
//! - `prost_free(ptr)` - Free allocated memory
//! - `prost_execute(input_ptr, input_len)` - Execute the plugin, returns output length
//! - `prost_get_output_ptr()` - Get pointer to output buffer
//! - `prost_get_output_len()` - Get output buffer length
//! - `prost_clear_output()` - Clear the output buffer

use std::alloc::{alloc, dealloc, Layout};
use std::cell::RefCell;

use prost::Message;

use crate::GeneratorResultExt;

// Thread-local storage for the output buffer.
// WASI is single-threaded, so this is safe.
thread_local! {
    static OUTPUT: RefCell<Vec<u8>> = const { RefCell::new(Vec::new()) };
}

/// Allocate memory with 8-byte alignment.
///
/// This is a wrapper around Rust's allocator for use by the host.
///
/// # Safety
///
/// The caller must ensure the returned pointer is eventually freed with `prost_free`.
#[no_mangle]
pub extern "C" fn prost_malloc(size: usize) -> *mut u8 {
    if size == 0 {
        return std::ptr::null_mut();
    }

    let align = 8;
    let layout = match Layout::from_size_align(size, align) {
        Ok(layout) => layout,
        Err(_) => return std::ptr::null_mut(),
    };

    // SAFETY: Layout is valid and non-zero sized
    unsafe { alloc(layout) }
}

/// Free memory allocated by `prost_malloc`.
///
/// # Safety
///
/// The caller must ensure:
/// - `ptr` was allocated by `prost_malloc`
/// - `ptr` has not already been freed
/// - `size` matches the size used in `prost_malloc`
#[no_mangle]
pub extern "C" fn prost_free(ptr: *mut u8, size: usize) {
    if ptr.is_null() || size == 0 {
        return;
    }

    let align = 8;
    let layout = match Layout::from_size_align(size, align) {
        Ok(layout) => layout,
        Err(_) => return,
    };

    // SAFETY: Caller guarantees ptr was allocated with this layout
    unsafe { dealloc(ptr, layout) }
}

/// Execute the protoc-gen-prost plugin.
///
/// Reads a `CodeGeneratorRequest` from the input buffer, processes it,
/// and stores the `CodeGeneratorResponse` in the output buffer.
///
/// # Arguments
///
/// * `input_ptr` - Pointer to the input buffer containing a serialized `CodeGeneratorRequest`
/// * `input_len` - Length of the input buffer in bytes
///
/// # Returns
///
/// The length of the output buffer containing the serialized `CodeGeneratorResponse`.
/// On error, the response will contain the error message.
///
/// # Safety
///
/// The caller must ensure:
/// - `input_ptr` points to a valid buffer of at least `input_len` bytes
/// - The buffer remains valid for the duration of this call
#[no_mangle]
pub extern "C" fn prost_execute(input_ptr: *const u8, input_len: usize) -> usize {
    // SAFETY: Caller guarantees the pointer and length are valid
    let input = unsafe { std::slice::from_raw_parts(input_ptr, input_len) };

    // Execute the generator
    let response = crate::execute(input).unwrap_codegen_response();

    // Encode the response
    let output = response.encode_to_vec();
    let output_len = output.len();

    // Store in thread-local output buffer
    OUTPUT.with(|out| {
        *out.borrow_mut() = output;
    });

    output_len
}

/// Get the pointer to the output buffer.
///
/// Must be called after `prost_execute`.
///
/// # Returns
///
/// A pointer to the output buffer, or null if no output is available.
#[no_mangle]
pub extern "C" fn prost_get_output_ptr() -> *const u8 {
    OUTPUT.with(|out| {
        let borrowed = out.borrow();
        if borrowed.is_empty() {
            std::ptr::null()
        } else {
            borrowed.as_ptr()
        }
    })
}

/// Get the length of the output buffer.
///
/// # Returns
///
/// The length of the output buffer in bytes.
#[no_mangle]
pub extern "C" fn prost_get_output_len() -> usize {
    OUTPUT.with(|out| out.borrow().len())
}

/// Clear the output buffer.
///
/// Should be called after reading the output to free memory.
#[no_mangle]
pub extern "C" fn prost_clear_output() {
    OUTPUT.with(|out| {
        out.borrow_mut().clear();
        out.borrow_mut().shrink_to_fit();
    });
}
