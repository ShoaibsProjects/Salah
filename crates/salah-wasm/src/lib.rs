//! Thin explicit-input WebAssembly boundary. All astronomical, local-date,
//! and civil-time calculations remain in the existing Rust engine.
//! No device clock, location API, network, storage, or JavaScript Date is read
//! by Rust. Setup operations accept explicit caller readings and bundled data.

use salah_core::{AsrCriterion, CivilDate, Coordinates, MethodProfile};
use salah_engine::{calculate_selected_local_day_schedule, schedule_document};
use salah_location::{lookup_timezone_candidates, select_manual_zone};
use salah_time::{BUNDLED_RULE_PACK_IDENTITY, SUPPORTED_ZONE_IDS};
use serde::Deserialize;
use serde_json::{Value, json};
use wasm_bindgen::prelude::*;

mod setup;
pub use setup::{local_clock_json, lookup_timezone_json};

pub const REQUEST_SCHEMA: &str = "salah-schedule-request-v1";
pub const RESPONSE_SCHEMA: &str = "salah-schedule-response-v1";
pub const MAX_REQUEST_BYTES: usize = 8 * 1024;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Request {
    schema: String,
    latitude_degrees: f64,
    longitude_degrees: f64,
    local_date: String,
    zone_id: String,
    method_id: String,
    asr: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SelectionRequest {
    schema: String,
    latitude_degrees: f64,
    longitude_degrees: f64,
    local_date: String,
    zone_id: String,
    method_id: String,
    asr: String,
    zone_choice: ZoneChoice,
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
enum ZoneChoice {
    Manual,
    ConfirmedSuggestion,
}

struct BridgeError {
    code: &'static str,
    message: String,
}

fn error(code: &'static str, message: impl ToString) -> BridgeError {
    BridgeError {
        code,
        message: message.to_string(),
    }
}

fn date(text: &str) -> Result<CivilDate, BridgeError> {
    let bytes = text.as_bytes();
    if bytes.len() != 10
        || !bytes.iter().enumerate().all(|(index, byte)| {
            if index == 4 || index == 7 {
                *byte == b'-'
            } else {
                byte.is_ascii_digit()
            }
        })
    {
        return Err(error(
            "invalid_date",
            "Choose a Gregorian date in YYYY-MM-DD format, from 1900–2100.",
        ));
    }
    // The ASCII shape check above makes these UTF-8 boundaries safe.
    let year = text[..4].parse().map_err(|e| error("invalid_date", e))?;
    let month = text[5..7].parse().map_err(|e| error("invalid_date", e))?;
    let day = text[8..].parse().map_err(|e| error("invalid_date", e))?;
    CivilDate::new(year, month, day).map_err(|e| error("invalid_date", e))
}

fn calculate(input: &str) -> Result<Value, BridgeError> {
    if input.len() > MAX_REQUEST_BYTES {
        return Err(error(
            "request_too_large",
            "The schedule request exceeds 8192 UTF-8 bytes.",
        ));
    }
    // Serde rejects duplicate fields, unknown fields, missing fields, wrong
    // types, non-JSON numeric values, trailing data, and excessive nesting.
    let request: Request = serde_json::from_str(input).map_err(|e| error("invalid_request", e))?;
    if request.schema != REQUEST_SCHEMA {
        return Err(error(
            "unsupported_schema",
            format!("Expected {REQUEST_SCHEMA}."),
        ));
    }
    calculate_request(request, ZoneChoice::Manual)
}

fn calculate_request(request: Request, choice: ZoneChoice) -> Result<Value, BridgeError> {
    let coordinates = Coordinates::new(request.latitude_degrees, request.longitude_degrees)
        .map_err(|e| error("invalid_coordinates", e))?;
    let requested_date = date(&request.local_date)?;
    let method = match request.method_id.as_str() {
        "research-15" => MethodProfile::research_15(),
        "mwl-angles-18-17" => MethodProfile::mwl_angles_18_17(),
        _ => {
            return Err(error(
                "unsupported_method",
                "Choose an explicitly supported method profile.",
            ));
        }
    };
    let asr = match request.asr.as_str() {
        "standard" => AsrCriterion::Standard,
        "hanafi" => AsrCriterion::Hanafi,
        _ => {
            return Err(error(
                "unsupported_asr",
                "Choose standard or hanafi explicitly.",
            ));
        }
    };
    let selection = match choice {
        ZoneChoice::Manual => select_manual_zone(coordinates, &request.zone_id)
            .map_err(|e| error("unsupported_zone", e))?,
        ZoneChoice::ConfirmedSuggestion => lookup_timezone_candidates(coordinates)
            .map_err(|e| error("zone_lookup_failed", e))?
            .confirm_suggestion(&request.zone_id)
            .map_err(|e| error("invalid_zone_confirmation", e))?,
    };
    let selected = calculate_selected_local_day_schedule(&selection, requested_date, method, asr)
        .map_err(|e| error("calculation_failed", e))?;
    schedule_document(&selected).map_err(|e| error("encoding_failed", e))
}

/// Additive v2 request preserving whether the person explicitly confirmed a
/// bundled-map suggestion. Recheck the coordinates and candidate in Rust;
/// client-provided candidate arrays are never accepted as evidence.
#[wasm_bindgen]
pub fn calculate_schedule_with_selection_json(request_json: &str) -> String {
    let result = (|| {
        if request_json.len() > MAX_REQUEST_BYTES {
            return Err(error(
                "request_too_large",
                "The request exceeds 8192 UTF-8 bytes.",
            ));
        }
        let request: SelectionRequest =
            serde_json::from_str(request_json).map_err(|e| error("invalid_request", e))?;
        if request.schema != "salah-schedule-request-v2" {
            return Err(error(
                "unsupported_schema",
                "Expected salah-schedule-request-v2.",
            ));
        }
        calculate_request(
            Request {
                schema: REQUEST_SCHEMA.to_owned(),
                latitude_degrees: request.latitude_degrees,
                longitude_degrees: request.longitude_degrees,
                local_date: request.local_date,
                zone_id: request.zone_id,
                method_id: request.method_id,
                asr: request.asr,
            },
            request.zone_choice,
        )
    })();
    match result {
        Ok(schedule) => json!({"schema": RESPONSE_SCHEMA, "status": "ok", "schedule": schedule}),
        Err(e) => json!({"schema": RESPONSE_SCHEMA, "status": "error", "error": {"code": e.code, "message": e.message}}),
    }.to_string()
}

/// One bounded JSON request, one JSON envelope. Handled failures return
/// `status: error` and no schedule. Traps are platform failures, not caught and
/// relabeled as valid astronomical results. No current-date/method default.
#[wasm_bindgen]
pub fn calculate_schedule_json(request_json: &str) -> String {
    match calculate(request_json) {
        Ok(schedule) => json!({"schema": RESPONSE_SCHEMA, "status": "ok", "schedule": schedule}),
        Err(e) => json!({"schema": RESPONSE_SCHEMA, "status": "error", "error": {"code": e.code, "message": e.message}}),
    }.to_string()
}

/// Names in the bundled inventory, with the exact data identity. This returns
/// possible manual choices, not coordinate-derived timezone suggestions.
#[wasm_bindgen]
pub fn zone_inventory_json() -> String {
    let pack = BUNDLED_RULE_PACK_IDENTITY;
    json!({
        "schema": "salah-zone-inventory-v1",
        "scope": "research_preview",
        "zone_ids": SUPPORTED_ZONE_IDS,
        "rule_pack": {
            "schema_version": pack.schema_version,
            "tzdb_version": pack.tzdb_version,
            "release_year": pack.release_year,
            "sha256": pack.sha256,
            "inventory_sha256": pack.inventory_sha256,
        },
    })
    .to_string()
}
