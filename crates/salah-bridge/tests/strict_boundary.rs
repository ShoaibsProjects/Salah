//! Portable boundary regressions: a client cannot weaken the existing request
//! contract or turn unavailable astronomical events into made-up readings.

use salah_bridge::{
    MAX_REQUEST_BYTES, calculate_schedule_json, calculate_schedule_with_selection_json,
    local_clock_json, lookup_timezone_json, zone_inventory_json,
};
use serde_json::Value;

type Operation = fn(&str) -> String;

fn requests() -> [(&'static str, Operation, &'static str); 4] {
    [
        (
            "schedule_v1",
            calculate_schedule_json,
            r#"{"schema":"salah-schedule-request-v1","latitude_degrees":44.9778,"longitude_degrees":-93.265,"local_date":"2026-10-01","zone_id":"America/Chicago","method_id":"mwl-angles-18-17","asr":"hanafi"}"#,
        ),
        (
            "schedule_v2",
            calculate_schedule_with_selection_json,
            r#"{"schema":"salah-schedule-request-v2","latitude_degrees":44.9778,"longitude_degrees":-93.265,"local_date":"2026-10-01","zone_id":"America/Chicago","method_id":"mwl-angles-18-17","asr":"hanafi","zone_choice":"manual"}"#,
        ),
        (
            "lookup",
            lookup_timezone_json,
            r#"{"schema":"salah-zone-lookup-request-v1","latitude_degrees":44.9778,"longitude_degrees":-93.265}"#,
        ),
        (
            "clock",
            local_clock_json,
            r#"{"schema":"salah-local-clock-request-v1","zone_id":"America/Chicago","utc_unix_seconds":1790856000}"#,
        ),
    ]
}

fn assert_error(name: &str, output: &str, code: &str) {
    let document: Value = serde_json::from_str(output).expect("JSON response");
    assert_eq!(document["status"], "error", "{name}: {output}");
    assert_eq!(document["error"]["code"], code, "{name}: {output}");
    assert!(document.get("schedule").is_none(), "{name}: {output}");
    assert!(document.get("clock").is_none(), "{name}: {output}");
    assert!(document.get("candidates").is_none(), "{name}: {output}");
}

#[test]
fn all_explicit_operations_reject_oversized_malformed_duplicate_and_unknown_fields() {
    for (name, operation, request) in requests() {
        assert_error(
            name,
            &operation(&" ".repeat(MAX_REQUEST_BYTES + 1)),
            "request_too_large",
        );
        assert_error(name, &operation("{"), "invalid_request");
        assert_error(
            name,
            &operation(&format!(
                "{},\"schema\":\"duplicate\"}}",
                &request[..request.len() - 1]
            )),
            "invalid_request",
        );
        assert_error(
            name,
            &operation(&format!(
                "{},\"extra\":true}}",
                &request[..request.len() - 1]
            )),
            "invalid_request",
        );
        assert_error(
            name,
            &operation(&format!("{request}null")),
            "invalid_request",
        );
    }
}

#[test]
fn v1_and_explicit_manual_v2_preserve_the_same_existing_schedule_bytes() {
    let operations = requests();
    let v1 = (operations[0].1)(operations[0].2);
    let v2 = (operations[1].1)(operations[1].2);
    assert_eq!(v1, v2);
    let document: Value = serde_json::from_str(&v1).unwrap();
    assert_eq!(document["schema"], "salah-schedule-response-v1");
    assert_eq!(document["status"], "ok");
    assert_eq!(document["schedule"]["schema"], "salah-local-schedule-v1");
    assert_eq!(document["schedule"]["scope"], "research_preview");
    assert_eq!(
        document["schedule"]["zone_selection"]["origin"],
        "manual_without_lookup"
    );
    assert_eq!(
        document["schedule"]["cycles"][0]["events"]
            .as_array()
            .unwrap()
            .len(),
        7
    );
}

#[test]
fn unavailable_polar_events_stay_successful_without_time_readings() {
    let document: Value = serde_json::from_str(&calculate_schedule_json(
        r#"{"schema":"salah-schedule-request-v1","latitude_degrees":69.6492,"longitude_degrees":18.9553,"local_date":"2026-06-21","zone_id":"Europe/Oslo","method_id":"mwl-angles-18-17","asr":"hanafi"}"#,
    ))
    .unwrap();
    assert_eq!(document["status"], "ok");
    let events = document["schedule"]["cycles"][0]["events"]
        .as_array()
        .unwrap();
    for name in ["fajr", "sunrise", "sunset", "maghrib", "isha"] {
        let event = events.iter().find(|event| event["name"] == name).unwrap();
        assert_eq!(event["status"], "unavailable");
        assert_eq!(event["reason"], "no_crossing_in_solar_cycle");
        assert!(event.get("reading").is_none());
        assert!(event.get("unrounded_utc_unix_seconds").is_none());
    }
}

#[test]
fn timezone_inventory_keeps_named_zone_data_identity() {
    let document: Value = serde_json::from_str(&zone_inventory_json()).unwrap();
    assert_eq!(document["schema"], "salah-zone-inventory-v1");
    assert_eq!(document["scope"], "research_preview");
    assert_eq!(document["zone_ids"].as_array().unwrap().len(), 598);
    assert_eq!(document["rule_pack"]["tzdb_version"], "2026d");
    assert_eq!(document["rule_pack"]["sha256"].as_str().unwrap().len(), 64);
}
