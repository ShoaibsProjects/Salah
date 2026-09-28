//! Offline UTC-instant-to-local civil-time adapter (F2-TZ0 probe).
//!
//! This crate converts a [`UtcInstant`] produced by `salah-core` into a local
//! civil date and clock time under caller-supplied IANA time-zone rules. It is
//! deliberately separate from `salah-core`: the prayer engine keeps its
//! fixed-offset interface and output record unchanged, and this adapter only
//! interprets already-computed UTC instants.
//!
//! ## Dependency rationale
//!
//! TZif parsing is done with `jiff` 0.2.37 (`TimeZone::tzif`), pinned exactly
//! (`version = "=0.2.37"`) with `default-features = false, features = ["std"]`.
//! That configuration compiles out everything that could reach beyond the
//! caller's bytes:
//!
//! - no bundled zone database (`tzdb-bundle-platform`, `tzdb-bundle-always`
//!   off; the inspected `jiff-tzdb` 0.1.8 bundle identifies itself as 2026c
//!   and must never be reported as this pack's 2026d),
//! - no host database scan (`tzdb-zoneinfo`, `tzdb-concatenated` off),
//! - no implicit system-zone detection (`tz-system` off, so `TimeZone::system`
//!   cannot be reached; this crate never calls `TimeZone::get`, `system`, or
//!   any filesystem/network API).
//!
//! What remains is CPU arithmetic over the supplied slice plus `salah-core`
//! types: a build/runtime library and data dependency, not a mandatory network
//! service, which fits the project's dependency policy (offline, free core
//! use; pinned, inventoried build inputs). `jiff`/`jiff-core` are maintained
//! by Andrew Gallant (BurntSushi) under Unlicense OR MIT, support TZif
//! versions 1-4 per RFC 8536, parse and consistency-check the POSIX footer,
//! and resolve post-transition instants from that footer dynamically even for
//! slim files without the `tz-fat` pre-expansion (inspected in the resolved
//! `jiff-core` 0.1.1 `tz/tzif/parser.rs` and `tz/tzif/query.rs`). The `alloc`
//! requirement of `TimeZone::tzif` is satisfied through `std`. Mobile and WASM
//! target builds remain unverified because those targets are not installed.
//!
//! ## Scope limits
//!
//! - UTC-to-local only. The reverse mapping is ambiguous around transitions
//!   and is not implemented here.
//! - Only the six fixture zones in [`SUPPORTED_ZONE_IDS`] and their exact
//!   pinned TZif bytes are accepted. Different valid TZif bytes are rejected
//!   so they cannot be mislabeled with this pack's [`TZDB_VERSION`].
//! - The resolved offset is reported in whole seconds and is never coerced
//!   into `salah_core::FixedUtcOffset`, which is minute-only input state.
//! - No zone is inferred from coordinates, and the result is never fed back
//!   into `CalculationInput`. A later packet must define zone-aware prayer-date
//!   selection (DST gaps/overlaps, skipped civil dates, date-line cases)
//!   before any engine integration.

use core::fmt;

use jiff::{Timestamp, tz::TimeZone};
use salah_core::{CivilDate, CivilDateTime, UtcInstant};

/// IANA database version of the fixture pack under `fixtures/`.
///
/// This constant describes the bytes served by [`fixture_tzif_bytes`], never a
/// host setting. It must change only together with the fixture bytes and
/// `fixtures/manifest.json`.
pub const TZDB_VERSION: &str = "2026d";

/// Canonical zone identifiers covered by the pinned fixture pack.
pub const SUPPORTED_ZONE_IDS: [&str; 6] = [
    "Etc/UTC",
    "America/Chicago",
    "Europe/London",
    "Asia/Kathmandu",
    "Pacific/Kiritimati",
    "Pacific/Apia",
];

/// Local civil reading of one UTC instant under one zone's pinned rules.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocalCivilTime {
    /// The kernel instant that was converted.
    pub utc: UtcInstant,
    /// Caller-chosen canonical zone identifier, echoed back verbatim.
    pub zone_id: String,
    /// IANA data version of the pack that supplied the rules ([`TZDB_VERSION`]).
    pub tzdb_version: &'static str,
    /// Resolved UTC offset at `utc`, in whole seconds east of UTC.
    ///
    /// Seconds precision is required: historical IANA offsets can contain a
    /// non-minute remainder, so this must not be narrowed to a minute grid.
    pub offset_seconds_east: i32,
    /// Local civil date and clock time under that offset.
    pub local: CivilDateTime,
}

impl fmt::Display for LocalCivilTime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {} ({}s, {}@{})",
            self.local,
            self.zone_id,
            self.offset_seconds_east,
            self.utc.unix_seconds,
            self.tzdb_version
        )
    }
}

/// Explicit failure modes of [`convert_utc_to_local`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TimeError {
    /// The zone identifier is not one of [`SUPPORTED_ZONE_IDS`].
    InvalidZoneId(String),
    /// The supplied bytes are empty or not recognized as valid TZif data.
    MalformedTzif(String),
    /// Valid TZif bytes differ from this zone's pinned 2026d fixture.
    UnpinnedTzif(String),
    /// The instant is outside the parser's supported timestamp range.
    UnsupportedInstant(i64),
    /// The resulting local date falls outside `salah-core`'s 1900-2100 range.
    LocalDateOutOfRange(String),
}

impl fmt::Display for TimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidZoneId(id) => write!(
                f,
                "unsupported zone id {id:?}; expected one of Etc/UTC, America/Chicago, Europe/London, Asia/Kathmandu, Pacific/Kiritimati, Pacific/Apia"
            ),
            Self::MalformedTzif(detail) => write!(f, "invalid TZif bytes: {detail}"),
            Self::UnpinnedTzif(id) => {
                write!(
                    f,
                    "TZif bytes for {id:?} do not match the pinned 2026d pack"
                )
            }
            Self::UnsupportedInstant(s) => {
                write!(
                    f,
                    "UTC instant {s} is outside the supported timestamp range"
                )
            }
            Self::LocalDateOutOfRange(detail) => {
                write!(f, "local civil date out of range: {detail}")
            }
        }
    }
}

impl std::error::Error for TimeError {}

fn check_zone_id(zone_id: &str) -> Result<(), TimeError> {
    if SUPPORTED_ZONE_IDS.contains(&zone_id) {
        Ok(())
    } else {
        Err(TimeError::InvalidZoneId(zone_id.to_owned()))
    }
}

/// Pinned TZif bytes for one of the [`SUPPORTED_ZONE_IDS`] zones.
///
/// This is a convenience accessor for the fixture pack recorded in
/// `fixtures/manifest.json`. Callers must still pass the returned bytes
/// explicitly into [`convert_utc_to_local`]; nothing is loaded implicitly.
pub fn fixture_tzif_bytes(zone_id: &str) -> Result<&'static [u8], TimeError> {
    check_zone_id(zone_id)?;
    let bytes: &'static [u8] = match zone_id {
        "Etc/UTC" => include_bytes!("../fixtures/tzif/Etc_UTC.tzif"),
        "America/Chicago" => include_bytes!("../fixtures/tzif/America_Chicago.tzif"),
        "Europe/London" => include_bytes!("../fixtures/tzif/Europe_London.tzif"),
        "Asia/Kathmandu" => include_bytes!("../fixtures/tzif/Asia_Kathmandu.tzif"),
        "Pacific/Kiritimati" => {
            include_bytes!("../fixtures/tzif/Pacific_Kiritimati.tzif")
        }
        "Pacific/Apia" => include_bytes!("../fixtures/tzif/Pacific_Apia.tzif"),
        _ => unreachable!("zone id checked above"),
    };
    Ok(bytes)
}

/// Convert a kernel UTC instant to local civil time under caller-supplied rules.
///
/// `zone_id` must be one of [`SUPPORTED_ZONE_IDS`] and `tzif_bytes` must match
/// the exact pinned TZif image for that zone (for example from
/// [`fixture_tzif_bytes`]). This prevents arbitrary bytes from being labeled
/// with this pack's [`TZDB_VERSION`].
/// Conversion is unique for a valid instant: UTC-to-local never has gaps or
/// folds. No host time zone, device setting, network lookup, or coordinate is
/// consulted; behavior depends only on the three arguments.
///
/// Returns the local date/clock, the resolved offset in seconds east of UTC,
/// and the pack's [`TZDB_VERSION`] as provenance.
pub fn convert_utc_to_local(
    utc: UtcInstant,
    zone_id: &str,
    tzif_bytes: &[u8],
) -> Result<LocalCivilTime, TimeError> {
    check_zone_id(zone_id)?;
    if tzif_bytes.is_empty() {
        return Err(TimeError::MalformedTzif("empty TZif slice".to_owned()));
    }
    // Caller-supplied bytes only. Never TimeZone::get/system: those would read
    // the host database or environment.
    let zone =
        TimeZone::tzif(zone_id, tzif_bytes).map_err(|e| TimeError::MalformedTzif(e.to_string()))?;
    if tzif_bytes != fixture_tzif_bytes(zone_id)? {
        return Err(TimeError::UnpinnedTzif(zone_id.to_owned()));
    }
    let instant = Timestamp::from_second(utc.unix_seconds)
        .map_err(|_| TimeError::UnsupportedInstant(utc.unix_seconds))?;
    let offset_seconds_east = zone.to_offset(instant).seconds();
    let civil = zone.to_datetime(instant);
    let date = CivilDate::new(
        i32::from(civil.year()),
        civil.month() as u8,
        civil.day() as u8,
    )
    .map_err(|_| {
        TimeError::LocalDateOutOfRange(format!(
            "{}-{:02}-{:02}",
            civil.year(),
            civil.month(),
            civil.day()
        ))
    })?;
    Ok(LocalCivilTime {
        utc,
        zone_id: zone_id.to_owned(),
        tzdb_version: TZDB_VERSION,
        offset_seconds_east,
        local: CivilDateTime {
            date,
            hour: civil.hour() as u8,
            minute: civil.minute() as u8,
            second: civil.second() as u8,
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pack_version_is_pinned() {
        assert_eq!(TZDB_VERSION, "2026d");
    }

    #[test]
    fn rejects_unknown_zone_id() {
        let utc = UtcInstant { unix_seconds: 0 };
        let bytes = fixture_tzif_bytes("Etc/UTC").unwrap();
        assert!(matches!(
            convert_utc_to_local(utc, "Mars/Olympus", bytes),
            Err(TimeError::InvalidZoneId(_))
        ));
        // Case-sensitive: lowercase alias is not canonical here.
        assert!(matches!(
            convert_utc_to_local(utc, "america/chicago", bytes),
            Err(TimeError::InvalidZoneId(_))
        ));
    }

    #[test]
    fn rejects_empty_and_garbage_bytes() {
        let utc = UtcInstant { unix_seconds: 0 };
        assert!(matches!(
            convert_utc_to_local(utc, "Etc/UTC", &[]),
            Err(TimeError::MalformedTzif(_))
        ));
        assert!(matches!(
            convert_utc_to_local(utc, "Etc/UTC", b"not a tzif file"),
            Err(TimeError::MalformedTzif(_))
        ));
    }

    #[test]
    fn rejects_unsupported_instant_and_out_of_range_local_date() {
        let chicago = fixture_tzif_bytes("America/Chicago").unwrap();
        assert!(matches!(
            convert_utc_to_local(
                UtcInstant {
                    unix_seconds: i64::MAX
                },
                "America/Chicago",
                chicago
            ),
            Err(TimeError::UnsupportedInstant(_))
        ));
        // 1800-01-01T00:00:00Z predates salah-core's civil range.
        assert!(matches!(
            convert_utc_to_local(
                UtcInstant {
                    unix_seconds: -5_364_662_400
                },
                "America/Chicago",
                chicago
            ),
            Err(TimeError::LocalDateOutOfRange(_))
        ));
    }

    #[test]
    fn rejects_valid_bytes_for_another_zone() {
        let chicago = fixture_tzif_bytes("America/Chicago").unwrap();
        let utc = UtcInstant { unix_seconds: 0 };
        assert!(matches!(
            convert_utc_to_local(utc, "Etc/UTC", chicago),
            Err(TimeError::UnpinnedTzif(_))
        ));
    }
}
