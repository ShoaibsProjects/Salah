//! F2-TZ0 selected-fixture evidence.
//!
//! Expected values below are derived from the pinned tzdata 2026d pack
//! (`crates/salah-time/fixtures/manifest.json`) and were cross-checked with
//! the reference C `zdump` built from the same `tzcode2026d` sources reading
//! the same slim TZif files plus POSIX footers:
//!
//! - Chicago spring: `Mar 8 07:59:59 2026 UT = 01:59:59 CST (-21600)` then
//!   `Mar 8 08:00:00 2026 UT = 03:00:00 CDT (-18000)`.
//! - Chicago fall: `Nov 1 06:59:59 2026 UT = 01:59:59 CDT (-18000)` then
//!   `Nov 1 07:00:00 2026 UT = 01:00:00 CST (-21600)`.
//! - London spring: `Mar 29 00:59:59 2026 UT = 00:59:59 GMT (0)` then
//!   `Mar 29 01:00:00 2026 UT = 02:00:00 BST (3600)`.
//! - London fall: `Oct 25 00:59:59 2026 UT = 01:59:59 BST (3600)` then
//!   `Oct 25 01:00:00 2026 UT = 01:00:00 GMT (0)`.
//! - Apia dateline jump: `Dec 30 09:59:59 2011 UT = Dec 29 23:59:59 -10
//!   (-36000)` then `Dec 30 10:00:00 2011 UT = Dec 31 00:00:00 +14 (50400)`;
//!   no local 2011-12-30 exists.
//!
//! This selected set is NOT global validation: six zones, a handful of
//! instants, one data snapshot. It proves the seam (UTC instant -> local
//! date/clock/offset-seconds/zone-id/version from caller-supplied bytes) and
//! nothing about worldwide coverage, future law, or prayer-date selection.

use salah_core::UtcInstant;
use salah_time::{TZDB_VERSION, convert_utc_to_local, fixture_tzif_bytes};

fn convert(unix_seconds: i64, zone: &str) -> salah_time::LocalCivilTime {
    let bytes = fixture_tzif_bytes(zone).unwrap();
    let got = convert_utc_to_local(UtcInstant { unix_seconds }, zone, bytes).unwrap();
    assert_eq!(got.tzdb_version, TZDB_VERSION);
    assert_eq!(got.tzdb_version, "2026d");
    assert_eq!(got.zone_id, zone);
    assert_eq!(got.utc.unix_seconds, unix_seconds);
    got
}

#[test]
fn chicago_spring_forward_2026() {
    // 2026-03-08T07:59:59Z -> 01:59:59 CST (-21600); 08:00:00Z -> 03:00:00 CDT.
    let before = convert(1_772_956_799, "America/Chicago");
    assert_eq!(before.offset_seconds_east, -21_600);
    assert_eq!(before.local.to_string(), "2026-03-08 01:59:59");
    let after = convert(1_772_956_800, "America/Chicago");
    assert_eq!(after.offset_seconds_east, -18_000);
    assert_eq!(after.local.to_string(), "2026-03-08 03:00:00");
}

#[test]
fn chicago_fall_back_2026() {
    // 2026-11-01T06:59:59Z -> 01:59:59 CDT (-18000); 07:00:00Z -> 01:00:00 CST.
    // UTC-to-local stays unique across the fold; the repeated local hour is
    // a local-to-UTC concern and out of scope for this slice.
    let before = convert(1_793_516_399, "America/Chicago");
    assert_eq!(before.offset_seconds_east, -18_000);
    assert_eq!(before.local.to_string(), "2026-11-01 01:59:59");
    let after = convert(1_793_516_400, "America/Chicago");
    assert_eq!(after.offset_seconds_east, -21_600);
    assert_eq!(after.local.to_string(), "2026-11-01 01:00:00");
}

#[test]
fn london_spring_and_fall_2026() {
    let before = convert(1_774_745_999, "Europe/London");
    assert_eq!(before.offset_seconds_east, 0);
    assert_eq!(before.local.to_string(), "2026-03-29 00:59:59");
    let after = convert(1_774_746_000, "Europe/London");
    assert_eq!(after.offset_seconds_east, 3_600);
    assert_eq!(after.local.to_string(), "2026-03-29 02:00:00");

    let before = convert(1_792_889_999, "Europe/London");
    assert_eq!(before.offset_seconds_east, 3_600);
    assert_eq!(before.local.to_string(), "2026-10-25 01:59:59");
    let after = convert(1_792_890_000, "Europe/London");
    assert_eq!(after.offset_seconds_east, 0);
    assert_eq!(after.local.to_string(), "2026-10-25 01:00:00");
}

#[test]
fn kathmandu_fractional_offset() {
    // +05:45 = 20700 s, with no DST in the 2026d rules.
    let got = convert(1_767_225_600, "Asia/Kathmandu");
    assert_eq!(got.offset_seconds_east, 20_700);
    assert_eq!(got.local.to_string(), "2026-01-01 05:45:00");
}

#[test]
fn kiritimati_plus_fourteen_carries_date() {
    // 2025-12-31T12:00:00Z is already 2026-01-01 locally at +14.
    let got = convert(1_767_182_400, "Pacific/Kiritimati");
    assert_eq!(got.offset_seconds_east, 50_400);
    assert_eq!(got.local.to_string(), "2026-01-01 02:00:00");
}

#[test]
fn apia_2011_skipped_local_date() {
    // Samoa skipped local 2011-12-30, jumping -10 -> +14 at 10:00:00Z.
    let before = convert(1_325_239_199, "Pacific/Apia");
    assert_eq!(before.offset_seconds_east, -36_000);
    assert_eq!(before.local.to_string(), "2011-12-29 23:59:59");
    let after = convert(1_325_239_200, "Pacific/Apia");
    assert_eq!(after.offset_seconds_east, 50_400);
    assert_eq!(after.local.to_string(), "2011-12-31 00:00:00");
}

#[test]
fn utc_zone_is_identity() {
    let got = convert(1_767_225_600, "Etc/UTC");
    assert_eq!(got.offset_seconds_east, 0);
    assert_eq!(got.local.to_string(), "2026-01-01 00:00:00");
}
