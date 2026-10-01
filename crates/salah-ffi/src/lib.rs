//! Small owned-buffer C boundary. JSON validation and calculations belong to
//! salah-bridge; this crate owns only UTF-8 validation and allocation lifetime.

use std::panic::{AssertUnwindSafe, catch_unwind};

pub struct SalahResponse {
    bytes: Box<[u8]>,
}

fn boundary_error(code: &str) -> String {
    // All codes are static ASCII constants chosen below, never caller input.
    format!(
        "{{\"schema\":\"salah-native-error-v1\",\"status\":\"error\",\"error\":{{\"code\":\"{code}\",\"message\":\"The native request could not be processed.\"}}}}"
    )
}

#[unsafe(no_mangle)]
pub extern "C" fn salah_native_abi_version() -> u32 {
    1
}

/// Execute one explicit request. The response must be released exactly once.
///
/// # Safety
/// For nonzero lengths, request must point to that many readable, immutable
/// bytes for this call. Null is permitted only for zero length. Input beyond
/// the bound is rejected before any memory read. Foreign pointer validity is
/// the caller's responsibility; this API cannot validate arbitrary pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn salah_execute(
    operation: u32,
    request: *const u8,
    length: usize,
) -> *mut SalahResponse {
    let result = catch_unwind(AssertUnwindSafe(|| {
        if length > salah_bridge::MAX_REQUEST_BYTES {
            return boundary_error("request_too_large");
        }
        if operation > 3 {
            return boundary_error("unsupported_operation");
        }
        if request.is_null() && length != 0 {
            return boundary_error("null_request");
        }
        let bytes = if length == 0 {
            &[][..]
        } else {
            // SAFETY: caller contract; bound/null checked before constructing.
            unsafe { std::slice::from_raw_parts(request, length) }
        };
        let input = match std::str::from_utf8(bytes) {
            Ok(input) => input,
            Err(_) => return boundary_error("invalid_utf8"),
        };
        match operation {
            0 if length == 0 => salah_bridge::zone_inventory_json(),
            0 => boundary_error("unexpected_request"),
            1 => salah_bridge::lookup_timezone_json(input),
            2 => salah_bridge::local_clock_json(input),
            3 => salah_bridge::calculate_schedule_with_selection_json(input),
            _ => unreachable!(),
        }
    }));
    let document = result.unwrap_or_else(|_| boundary_error("engine_failure"));
    Box::into_raw(Box::new(SalahResponse {
        bytes: document.into_bytes().into_boxed_slice(),
    }))
}

/// # Safety
/// response must be null or a live response allocated by salah_execute; it
/// must not be freed during this call or while the returned bytes are read.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn salah_response_data(response: *const SalahResponse) -> *const u8 {
    if response.is_null() {
        return std::ptr::null();
    }
    unsafe { &*response }.bytes.as_ptr()
}

/// # Safety
/// response must be null or a live response allocated by salah_execute.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn salah_response_length(response: *const SalahResponse) -> usize {
    if response.is_null() {
        return 0;
    }
    unsafe { &*response }.bytes.len()
}

/// # Safety
/// response must be null or a live owned response from salah_execute. Release
/// it once, after every reader has finished; use after free is invalid.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn salah_response_free(response: *mut SalahResponse) {
    if !response.is_null() {
        drop(unsafe { Box::from_raw(response) });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(operation: u32, request: &[u8]) -> String {
        unsafe {
            let result = salah_execute(operation, request.as_ptr(), request.len());
            let text = std::str::from_utf8(std::slice::from_raw_parts(
                salah_response_data(result),
                salah_response_length(result),
            ))
            .unwrap()
            .to_owned();
            salah_response_free(result);
            text
        }
    }

    #[test]
    fn ffi_copies_the_exact_shared_result_and_owns_it_independently() {
        let request = r#"{"schema":"salah-schedule-request-v2","latitude_degrees":44.9778,"longitude_degrees":-93.265,"local_date":"2026-10-01","zone_id":"America/Chicago","method_id":"mwl-angles-18-17","asr":"hanafi","zone_choice":"manual"}"#;
        assert_eq!(
            run(3, request.as_bytes()),
            salah_bridge::calculate_schedule_with_selection_json(request)
        );
        assert_eq!(run(0, &[]), salah_bridge::zone_inventory_json());
    }

    #[test]
    fn foreign_input_bounds_and_utf8_fail_before_engine_execution() {
        assert!(run(1, &[0xff]).contains("invalid_utf8"));
        assert!(run(5, &[]).contains("unsupported_operation"));
        assert!(run(0, b"{}").contains("unexpected_request"));
        unsafe {
            let result = salah_execute(1, std::ptr::null(), 1);
            let bytes = std::slice::from_raw_parts(
                salah_response_data(result),
                salah_response_length(result),
            );
            assert!(std::str::from_utf8(bytes).unwrap().contains("null_request"));
            salah_response_free(result);
            // A null pointer plus oversized length must never be dereferenced.
            let result = salah_execute(1, std::ptr::null(), usize::MAX);
            let bytes = std::slice::from_raw_parts(
                salah_response_data(result),
                salah_response_length(result),
            );
            assert!(
                std::str::from_utf8(bytes)
                    .unwrap()
                    .contains("request_too_large")
            );
            salah_response_free(result);
            assert_eq!(salah_response_length(std::ptr::null()), 0);
            assert!(salah_response_data(std::ptr::null()).is_null());
            salah_response_free(std::ptr::null_mut());
        }
    }
}
