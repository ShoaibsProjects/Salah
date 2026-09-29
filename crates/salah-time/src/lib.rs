//! Offline IANA civil-time adapter and local-date solar-cycle selector.
//!
//! This crate converts UTC instants produced by `salah-core` into local civil
//! date and clock readings under caller-supplied IANA time-zone rules. Its
//! research selector also uses `salah-core`'s explicit UTC-anchor API to find
//! all solar cycles whose transit matches a requested local date. It remains
//! separate from the legacy fixed-offset calculation interface.
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
//! - No zone is inferred from coordinates. The research date selector returns
//!   every solar cycle whose transit maps to a requested local date; zero or
//!   multiple matches remain explicit. It does not yet localize every event
//!   or provide a consumer daily-schedule facade.

use core::fmt;

use jiff::{Timestamp, tz::TimeZone};
use salah_core::{
    ASTRONOMY_MODEL, AsrCriterion, CORE_ENGINE_VERSION, CalculationError, CivilDate, CivilDateTime,
    Coordinates, MethodProfile, UTC_ANCHOR_MAX_UNIX_SECONDS, UTC_ANCHOR_MIN_UNIX_SECONDS,
    UtcAnchorInput, UtcAnchorTimes, UtcInstant, calculate_utc_anchor_times,
};

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

/// Identity of the policy that matches a solar transit to a requested local date.
pub const LOCAL_DATE_TRANSIT_POLICY_ID: &str = "local-date-solar-transit";
/// Revision of [`LOCAL_DATE_TRANSIT_POLICY_ID`].
pub const LOCAL_DATE_TRANSIT_POLICY_REVISION: &str = "0.1";

const MAX_ABSOLUTE_OFFSET_SECONDS: i64 = 25 * 3600 + 59 * 60 + 59;
const SECONDS_PER_DAY: i64 = 86_400;
const MAX_ANCHOR_STEP_SECONDS: i64 = 6 * 3600;

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

/// Provenance shared by every outcome of a requested-date transit selection.
#[derive(Debug, Clone)]
pub struct LocalDateTransitRecord {
    pub requested_date: CivilDate,
    pub coordinates: Coordinates,
    pub zone_id: String,
    pub tzdb_version: &'static str,
    pub method: MethodProfile,
    pub asr_criterion: AsrCriterion,
    pub astronomy_model: &'static str,
    pub engine_version: &'static str,
    pub policy_id: &'static str,
    pub policy_revision: &'static str,
}

/// One solar cycle whose rounded, unadjusted transit falls on the requested
/// local date under the recorded zone rules.
#[derive(Debug, Clone)]
pub struct LocalDateTransitCandidate {
    pub cycle: UtcAnchorTimes,
    pub local_transit: LocalCivilTime,
}

/// Explicit cardinality of all matching solar cycles for a requested date.
#[derive(Debug, Clone)]
pub enum LocalDateTransitMatches {
    /// No selected solar transit maps to the requested date. This alone does
    /// not establish that the civil date itself was skipped.
    Zero,
    /// Exactly one cycle maps to the requested date.
    One(Box<LocalDateTransitCandidate>),
    /// More than one cycle maps to the requested date; values are UTC-sorted.
    Multiple(Vec<LocalDateTransitCandidate>),
}

/// Complete result of selecting solar cycle(s) for one zone-local date.
#[derive(Debug, Clone)]
pub struct LocalDateTransitSelection {
    pub record: LocalDateTransitRecord,
    pub matches: LocalDateTransitMatches,
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

/// Explicit failure modes of civil-time conversion and date selection.
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
    /// The core rejected a UTC anchor or could not calculate its solar cycle.
    Calculation(CalculationError),
    /// Checked arithmetic could not form a bounded date search interval.
    DateWindowOverflow,
    /// No in-range anchor exists to cover a possible transit for the date.
    IncompleteCoverage(CivilDate),
    /// The canonical in-range anchor did not reproduce the discovered cycle.
    TransitNotStable { discovered: i64, canonical: i64 },
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
            Self::Calculation(error) => write!(f, "solar-cycle calculation failed: {error}"),
            Self::DateWindowOverflow => f.write_str("local-date UTC search window overflowed"),
            Self::IncompleteCoverage(date) => {
                write!(f, "no supported UTC-anchor range covers local date {date}")
            }
            Self::TransitNotStable {
                discovered,
                canonical,
            } => write!(
                f,
                "canonical anchor selected transit {canonical}, but search discovered {discovered}"
            ),
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
    let zone = parse_pinned_zone(zone_id, tzif_bytes)?;
    convert_with_zone(utc, zone_id, &zone)
}

fn parse_pinned_zone(zone_id: &str, tzif_bytes: &[u8]) -> Result<TimeZone, TimeError> {
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
    Ok(zone)
}

fn convert_with_zone(
    utc: UtcInstant,
    zone_id: &str,
    zone: &TimeZone,
) -> Result<LocalCivilTime, TimeError> {
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

/// Select every core solar cycle whose rounded, unadjusted upper transit
/// converts to `date` under the exact pinned rules supplied by the caller.
///
/// The search covers the full UTC interval allowed by Jiff's supported TZif
/// offset bounds, intersects it with `salah-core`'s anchor range, and samples
/// anchors no more than six hours apart. The equation-of-time bound and grid
/// completeness argument are specified in
/// `specification/civil-date-transit-selector-v0.1.md`.
///
/// `Zero` means no matching transit was found; it does not by itself mean that
/// the civil date was skipped. Every returned candidate is independently
/// converted using the same zone bytes and reports the pinned data version.
pub fn select_local_date_transits(
    date: CivilDate,
    coordinates: Coordinates,
    zone_id: &str,
    tzif_bytes: &[u8],
    method: MethodProfile,
    asr_criterion: AsrCriterion,
) -> Result<LocalDateTransitSelection, TimeError> {
    let zone = parse_pinned_zone(zone_id, tzif_bytes)?;
    let method = method.validate().map_err(TimeError::Calculation)?;
    let window = local_date_utc_window(date)?;
    let search_start = window.0.max(UTC_ANCHOR_MIN_UNIX_SECONDS);
    let search_end = window.1.min(UTC_ANCHOR_MAX_UNIX_SECONDS);
    if search_start > search_end {
        return Err(TimeError::IncompleteCoverage(date));
    }

    let mut anchors = Vec::new();
    let mut anchor = search_start;
    anchors.push(anchor);
    while anchor < search_end {
        let remaining = search_end - anchor;
        anchor += remaining.min(MAX_ANCHOR_STEP_SECONDS);
        anchors.push(anchor);
    }

    let mut discovered_transits = std::collections::BTreeSet::new();
    for anchor in anchors {
        let cycle = calculate_utc_anchor_times(UtcAnchorInput {
            coordinates,
            anchor: UtcInstant {
                unix_seconds: anchor,
            },
            method,
            asr_criterion,
        })
        .map_err(TimeError::Calculation)?;
        let transit = cycle.selected_transit.unix_seconds;
        if (window.0..=window.1).contains(&transit) {
            discovered_transits.insert(transit);
        }
    }

    let mut candidates = Vec::new();
    for discovered in discovered_transits {
        // Recalculate from a canonical anchor so the same transit always has
        // the same floating-point solver result, independent of which grid
        // point first discovered it. Clamp only at the supported range edges.
        let canonical_anchor =
            discovered.clamp(UTC_ANCHOR_MIN_UNIX_SECONDS, UTC_ANCHOR_MAX_UNIX_SECONDS);
        let cycle = calculate_utc_anchor_times(UtcAnchorInput {
            coordinates,
            anchor: UtcInstant {
                unix_seconds: canonical_anchor,
            },
            method,
            asr_criterion,
        })
        .map_err(TimeError::Calculation)?;
        if cycle.selected_transit.unix_seconds != discovered {
            return Err(TimeError::TransitNotStable {
                discovered,
                canonical: cycle.selected_transit.unix_seconds,
            });
        }
        let local_transit = match convert_with_zone(cycle.selected_transit, zone_id, &zone) {
            Ok(local) => local,
            // The requested date is in 1900–2100. A different candidate
            // outside that range cannot match it, so it is safely excluded.
            Err(TimeError::LocalDateOutOfRange(_)) => continue,
            Err(error) => return Err(error),
        };
        if local_transit.local.date != date {
            continue;
        }
        candidates.push(LocalDateTransitCandidate {
            cycle,
            local_transit,
        });
    }
    candidates.sort_by_key(|candidate| candidate.cycle.selected_transit.unix_seconds);

    let record = LocalDateTransitRecord {
        requested_date: date,
        coordinates,
        zone_id: zone_id.to_owned(),
        tzdb_version: TZDB_VERSION,
        method,
        asr_criterion,
        astronomy_model: ASTRONOMY_MODEL,
        engine_version: CORE_ENGINE_VERSION,
        policy_id: LOCAL_DATE_TRANSIT_POLICY_ID,
        policy_revision: LOCAL_DATE_TRANSIT_POLICY_REVISION,
    };
    let matches = match candidates.len() {
        0 => LocalDateTransitMatches::Zero,
        1 => LocalDateTransitMatches::One(Box::new(candidates.pop().expect("one candidate"))),
        _ => LocalDateTransitMatches::Multiple(candidates),
    };
    Ok(LocalDateTransitSelection { record, matches })
}

fn local_date_utc_window(date: CivilDate) -> Result<(i64, i64), TimeError> {
    let utc_midnight = date
        .days_since_unix_epoch()
        .checked_mul(SECONDS_PER_DAY)
        .ok_or(TimeError::DateWindowOverflow)?;
    let earliest = utc_midnight
        .checked_sub(MAX_ABSOLUTE_OFFSET_SECONDS)
        .ok_or(TimeError::DateWindowOverflow)?;
    let latest = utc_midnight
        .checked_add(SECONDS_PER_DAY - 1)
        .and_then(|value| value.checked_add(MAX_ABSOLUTE_OFFSET_SECONDS))
        .ok_or(TimeError::DateWindowOverflow)?;
    Ok((earliest, latest))
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
