//! Offline setup over caller-supplied device readings and bundled data.
//! No sensor, system-clock, host timezone, or network access occurs here.

use salah_core::{Coordinates, UtcInstant};
use salah_location::{CandidateCardinality, lookup_timezone_candidates};
use salah_time::RuntimeZone;
use serde::Deserialize;
use serde_json::{Value, json};

use crate::{BridgeError, MAX_REQUEST_BYTES, error};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct LookupRequest {
    schema: String,
    latitude_degrees: f64,
    longitude_degrees: f64,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ClockRequest {
    schema: String,
    zone_id: String,
    utc_unix_seconds: i64,
}

fn parse<T: serde::de::DeserializeOwned>(input: &str) -> Result<T, BridgeError> {
    if input.len() > MAX_REQUEST_BYTES {
        return Err(error(
            "request_too_large",
            "The request exceeds 8192 UTF-8 bytes.",
        ));
    }
    serde_json::from_str(input).map_err(|e| error("invalid_request", e))
}

fn envelope(schema: &str, field: &str, result: Result<Value, BridgeError>) -> String {
    match result {
        Ok(value) => {
            let mut document = json!({"schema": schema, "status": "ok"});
            document[field] = value;
            document
        },
        Err(e) => json!({"schema": schema, "status": "error", "error": {"code": e.code, "message": e.message}}),
    }.to_string()
}

/// Return every bundled-map suggestion. A unique match still needs explicit
/// confirmation, and does not certify the receiver's accuracy footprint.
pub fn lookup_timezone_json(request_json: &str) -> String {
    let result = (|| {
        let request: LookupRequest = parse(request_json)?;
        if request.schema != "salah-zone-lookup-request-v1" {
            return Err(error(
                "unsupported_schema",
                "Expected salah-zone-lookup-request-v1.",
            ));
        }
        let coordinates = Coordinates::new(request.latitude_degrees, request.longitude_degrees)
            .map_err(|e| error("invalid_coordinates", e))?;
        let candidates =
            lookup_timezone_candidates(coordinates).map_err(|e| error("zone_lookup_failed", e))?;
        let cardinality = match candidates.cardinality() {
            CandidateCardinality::NoCoverage => "no_coverage",
            CandidateCardinality::OneSuggestion => "one_suggestion",
            CandidateCardinality::MultipleSuggestions => "multiple_suggestions",
            CandidateCardinality::NotLookedUp => {
                return Err(error("zone_lookup_failed", "No lookup was performed."));
            }
        };
        Ok(json!({
            "scope": "research_preview",
            "latitude_degrees": coordinates.latitude_degrees(),
            "longitude_degrees": coordinates.longitude_degrees(),
            "cardinality": cardinality,
            "zone_ids": candidates.zone_ids(),
            "requires_confirmation": true,
            "accuracy_footprint_checked": false,
            "boundary_version": candidates.boundary_data_version(),
            "boundary_sha256": candidates.boundary_data_sha256(),
            "tzdb_version": candidates.timezone_database_version(),
        }))
    })();
    envelope("salah-zone-lookup-response-v1", "candidates", result)
}

/// Derive the selected zone's Gregorian date from a supplied UTC instant using
/// the pinned Rust rules. Caller clock accuracy is neither inferred nor proved.
pub fn local_clock_json(request_json: &str) -> String {
    let result = (|| {
        let request: ClockRequest = parse(request_json)?;
        if request.schema != "salah-local-clock-request-v1" {
            return Err(error(
                "unsupported_schema",
                "Expected salah-local-clock-request-v1.",
            ));
        }
        let zone =
            RuntimeZone::bundled(&request.zone_id).map_err(|e| error("unsupported_zone", e))?;
        let local = zone
            .convert_utc(UtcInstant {
                unix_seconds: request.utc_unix_seconds,
            })
            .map_err(|e| error("invalid_clock_instant", e))?;
        Ok(json!({
            "source": "caller_clock_unverified",
            "utc_unix_seconds": local.utc.unix_seconds,
            "zone_id": local.zone_id,
            "local_date": local.local.date.to_string(),
            "local_time": format!("{:02}:{:02}:{:02}", local.local.hour, local.local.minute, local.local.second),
            "offset_seconds_east": local.offset_seconds_east,
            "rule_pack": {
                "tzdb_version": local.rule_pack.tzdb_version,
                "sha256": local.rule_pack.sha256,
                "inventory_sha256": local.rule_pack.inventory_sha256,
            },
        }))
    })();
    envelope("salah-local-clock-response-v1", "clock", result)
}
