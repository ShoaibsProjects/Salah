//! Thin WebAssembly exports of the shared, platform-neutral JSON boundary.
//! Astronomy, local-date selection, civil-time conversion, strict input
//! validation, and response encoding remain in the existing Rust operations.

use wasm_bindgen::prelude::*;

pub use salah_bridge::{MAX_REQUEST_BYTES, REQUEST_SCHEMA, RESPONSE_SCHEMA};

/// One explicit bounded v1 request, one shared schedule/error envelope.
#[wasm_bindgen]
pub fn calculate_schedule_json(request_json: &str) -> String {
    salah_bridge::calculate_schedule_json(request_json)
}

/// Additive v2 calculation preserving the caller's zone-selection provenance.
#[wasm_bindgen]
pub fn calculate_schedule_with_selection_json(request_json: &str) -> String {
    salah_bridge::calculate_schedule_with_selection_json(request_json)
}

/// Names and exact identity of the bundled manual timezone inventory.
#[wasm_bindgen]
pub fn zone_inventory_json() -> String {
    salah_bridge::zone_inventory_json()
}

/// Embedded-map candidates, requiring explicit caller confirmation.
#[wasm_bindgen]
pub fn lookup_timezone_json(request_json: &str) -> String {
    salah_bridge::lookup_timezone_json(request_json)
}

/// Selected-zone conversion of an explicit, unverified caller clock reading.
#[wasm_bindgen]
pub fn local_clock_json(request_json: &str) -> String {
    salah_bridge::local_clock_json(request_json)
}
