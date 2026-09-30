//! Offline IANA civil-time adapter and local-date prayer schedule.
//!
//! This crate converts UTC instants produced by `salah-core` into local civil
//! date and clock readings under caller-supplied IANA time-zone rules. Its
//! research selector also uses `salah-core`'s explicit UTC-anchor API to find
//! all solar cycles whose transit matches a requested local date. The local
//! schedule facade converts each event without display rounding and preserves
//! unavailable statuses. It remains separate from the legacy fixed-offset
//! calculation interface.
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
//! - The pack covers 598 named IANA zones in release 2026d. Different valid
//!   TZif bytes are rejected so they cannot be mislabeled with this pack's
//!   [`TZDB_VERSION`]. Coordinate-to-zone mapping is not included.
//! - The resolved offset is reported in whole seconds and is never coerced
//!   into `salah_core::FixedUtcOffset`, which is minute-only input state.
//! - No zone is inferred from coordinates. The research date selector returns
//!   every solar cycle whose transit maps to a requested local date; zero or
//!   multiple matches remain explicit. The local-schedule facade converts all
//!   seven events under the same pinned rules.

use core::fmt;

use jiff::{Timestamp, tz::TimeZone};
use salah_core::{
    ASTRONOMY_MODEL, AsrCriterion, CORE_ENGINE_VERSION, CalculationError, CivilDate, CivilDateTime,
    Coordinates, Event, EventRule, MethodProfile, UTC_ANCHOR_MAX_UNIX_SECONDS,
    UTC_ANCHOR_MIN_UNIX_SECONDS, UnavailableReason, UtcAnchorInput, UtcAnchorTimes, UtcInstant,
    calculate_utc_anchor_times,
};

mod global_zone_index;

/// IANA database version of the embedded named-zone pack.
///
/// This constant describes the bytes served by [`fixture_tzif_bytes`], never a
/// host setting. It must change only together with the pack and
/// `fixtures/global/manifest.json`.
pub const TZDB_VERSION: &str = global_zone_index::DATA_VERSION;

/// Calendar year of the pinned IANA release. This is data metadata, not a
/// promise that civil-time law is settled for any date in that year.
/// Update it together with [`TZDB_VERSION`] and the pack manifest.
pub const TZDB_RELEASE_YEAR: i32 = global_zone_index::RELEASE_YEAR;

/// Schema of the bundled pack manifest. This describes the on-disk inventory,
/// not a network-update protocol.
pub const TZDB_PACK_SCHEMA_VERSION: u32 = global_zone_index::PACK_SCHEMA_VERSION;

/// SHA-256 of the exact bundled `tzif-pack.bin` bytes. A checksum identifies
/// bytes for reproducibility; it is not a signature or source authentication.
pub const TZDB_PACK_SHA256: &str = global_zone_index::PACK_SHA256;

/// SHA-256 over the canonical zone-name/slice inventory. Together with the
/// blob hash, this identifies the exact rule bytes used for every zone ID.
pub const TZDB_PACK_INVENTORY_SHA256: &str = global_zone_index::INVENTORY_SHA256;

/// Exact identity of the rule snapshot used for a civil-time result.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RulePackIdentity {
    pub schema_version: u32,
    pub tzdb_version: &'static str,
    pub release_year: i32,
    pub sha256: &'static str,
    pub inventory_sha256: &'static str,
}

/// Identity of the immutable rule pack compiled into this build.
pub const BUNDLED_RULE_PACK_IDENTITY: RulePackIdentity = RulePackIdentity {
    schema_version: TZDB_PACK_SCHEMA_VERSION,
    tzdb_version: TZDB_VERSION,
    release_year: TZDB_RELEASE_YEAR,
    sha256: TZDB_PACK_SHA256,
    inventory_sha256: TZDB_PACK_INVENTORY_SHA256,
};

/// Named IANA identifiers covered by the pinned 2026d pack.
pub const SUPPORTED_ZONE_IDS: &[&str] = global_zone_index::SUPPORTED_ZONE_IDS;

/// Identity of the policy that matches a solar transit to a requested local date.
pub const LOCAL_DATE_TRANSIT_POLICY_ID: &str = "local-date-solar-transit";
/// Revision of [`LOCAL_DATE_TRANSIT_POLICY_ID`].
pub const LOCAL_DATE_TRANSIT_POLICY_REVISION: &str = "0.1";
/// Identity of the policy that localizes a selected cycle's seven events.
pub const LOCAL_DATE_PRAYER_SCHEDULE_POLICY_ID: &str = "local-date-prayer-schedule";
/// Revision of [`LOCAL_DATE_PRAYER_SCHEDULE_POLICY_ID`].
pub const LOCAL_DATE_PRAYER_SCHEDULE_POLICY_REVISION: &str = "0.1";
/// Identity of the policy that determines whether a local civil date exists.
pub const LOCAL_DATE_EXISTENCE_POLICY_ID: &str = "local-date-existence";
/// Revision of [`LOCAL_DATE_EXISTENCE_POLICY_ID`].
pub const LOCAL_DATE_EXISTENCE_POLICY_REVISION: &str = "0.1";

const MAX_ABSOLUTE_OFFSET_SECONDS: i64 = 25 * 3600 + 59 * 60 + 59;
const SECONDS_PER_DAY: i64 = 86_400;
const MAX_ANCHOR_STEP_SECONDS: i64 = 6 * 3600;

/// Local civil reading of one UTC instant under one zone's pinned rules.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocalCivilTime {
    /// The kernel instant that was converted.
    pub utc: UtcInstant,
    /// Caller-chosen IANA identifier, including a supported link name, echoed verbatim.
    pub zone_id: String,
    /// IANA data version of the pack that supplied the rules ([`TZDB_VERSION`]).
    pub tzdb_version: &'static str,
    /// Exact embedded rule pack that produced the civil reading.
    pub rule_pack: RulePackIdentity,
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
    pub rule_pack: RulePackIdentity,
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

/// One core event represented in local civil time without display rounding.
///
/// The original cycle remains available on
/// [`LocalDatePrayerScheduleCandidate::cycle`]. For an occurring event,
/// `local.utc` is the same rounded UTC second returned by the core and
/// `unrounded_utc_unix_seconds` preserves the core's raw solver value.
#[derive(Debug, Clone)]
pub enum LocalizedEvent {
    Occurs {
        local: LocalCivilTime,
        unrounded_utc_unix_seconds: f64,
        rule: EventRule,
    },
    Unavailable {
        reason: UnavailableReason,
    },
}

/// Local representations of the seven events returned by the core.
#[derive(Debug, Clone)]
pub struct LocalPrayerEvents {
    pub fajr: LocalizedEvent,
    pub sunrise: LocalizedEvent,
    pub dhuhr: LocalizedEvent,
    pub asr: LocalizedEvent,
    pub sunset: LocalizedEvent,
    pub maghrib: LocalizedEvent,
    pub isha: LocalizedEvent,
}

/// One selected solar cycle with its transit and every event localized.
#[derive(Debug, Clone)]
pub struct LocalDatePrayerScheduleCandidate {
    /// Original UTC cycle and complete calculation provenance.
    pub cycle: UtcAnchorTimes,
    /// Rounded, unadjusted transit used to match the requested local date.
    pub local_transit: LocalCivilTime,
    /// Events under the same pinned zone bytes. Their local dates may differ
    /// from the requested date, for example Isha after local midnight.
    pub events: LocalPrayerEvents,
}

/// Explicit cardinality of cycles selected for a local date.
#[derive(Debug, Clone)]
pub enum LocalDatePrayerScheduleMatches {
    Zero,
    One(Box<LocalDatePrayerScheduleCandidate>),
    Multiple(Vec<LocalDatePrayerScheduleCandidate>),
}

/// Local-date prayer schedule with transit and localization provenance.
#[derive(Debug, Clone)]
pub struct LocalDatePrayerSchedule {
    /// Requested date, coordinates, zone/TZDB, method and kernel provenance.
    pub record: LocalDateTransitRecord,
    /// Civil-date existence independently classified from solar-cycle matches.
    pub civil_date: LocalDateExistence,
    pub policy_id: &'static str,
    pub policy_revision: &'static str,
    pub matches: LocalDatePrayerScheduleMatches,
}

/// A UTC interval whose instants map to the classified local date under one
/// constant zone offset. Several intervals can occur for a date-line rollback.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LocalDateUtcInterval {
    pub start: UtcInstant,
    pub end: UtcInstant,
    pub offset_seconds_east: i32,
}

/// Whether a Gregorian date has any instants in the selected zone.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LocalDateStatus {
    /// All UTC intervals mapping to the date, partitioned at zone transitions.
    Exists(Vec<LocalDateUtcInterval>),
    /// No UTC instant maps to the date under the pinned rules.
    Skipped,
}

/// Date-existence classification with explicit data and policy provenance.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocalDateExistence {
    pub requested_date: CivilDate,
    pub zone_id: String,
    pub tzdb_version: &'static str,
    pub rule_pack: RulePackIdentity,
    pub policy_id: &'static str,
    pub policy_revision: &'static str,
    pub status: LocalDateStatus,
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
    /// Valid TZif bytes differ from this zone's pinned 2026d pack entry.
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
                "unsupported IANA zone id {id:?} in the pinned {TZDB_VERSION} pack"
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
    if global_zone_index::bytes(zone_id).is_some() {
        Ok(())
    } else {
        Err(TimeError::InvalidZoneId(zone_id.to_owned()))
    }
}

/// Pinned TZif bytes for one of the [`SUPPORTED_ZONE_IDS`] IANA identifiers.
///
/// This is a convenience accessor for the fixture pack recorded in
/// `fixtures/global/manifest.json`. Callers must still pass the returned bytes
/// explicitly into [`convert_utc_to_local`]; nothing is loaded implicitly.
pub fn fixture_tzif_bytes(zone_id: &str) -> Result<&'static [u8], TimeError> {
    check_zone_id(zone_id)?;
    global_zone_index::bytes(zone_id).ok_or_else(|| TimeError::InvalidZoneId(zone_id.to_owned()))
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
    if !tzif_bytes.starts_with(b"TZif") {
        return Err(TimeError::MalformedTzif("missing TZif header".to_owned()));
    }
    let pinned = fixture_tzif_bytes(zone_id)?;
    if tzif_bytes != pinned {
        return Err(TimeError::UnpinnedTzif(zone_id.to_owned()));
    }
    // Caller-supplied bytes only. Never TimeZone::get/system: those would read
    // the host database or environment.
    let zone =
        TimeZone::tzif(zone_id, pinned).map_err(|e| TimeError::MalformedTzif(e.to_string()))?;
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
        rule_pack: BUNDLED_RULE_PACK_IDENTITY,
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
    select_local_date_transits_with_zone(date, coordinates, zone_id, method, asr_criterion, &zone)
}

fn select_local_date_transits_with_zone(
    date: CivilDate,
    coordinates: Coordinates,
    zone_id: &str,
    method: MethodProfile,
    asr_criterion: AsrCriterion,
    zone: &TimeZone,
) -> Result<LocalDateTransitSelection, TimeError> {
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
        let local_transit = match convert_with_zone(cycle.selected_transit, zone_id, zone) {
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
        rule_pack: BUNDLED_RULE_PACK_IDENTITY,
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

/// Classify whether `date` exists in the caller-selected zone under the exact
/// pinned TZif bytes, and return every UTC interval that maps to it.
///
/// This uses the zone's actual transition iterator, including post-table rules
/// from its TZif footer. It does not infer the zone from coordinates or equate
/// an existing date with a matching solar transit.
pub fn classify_local_date(
    date: CivilDate,
    zone_id: &str,
    tzif_bytes: &[u8],
) -> Result<LocalDateExistence, TimeError> {
    let zone = parse_pinned_zone(zone_id, tzif_bytes)?;
    classify_local_date_with_zone(date, zone_id, &zone)
}

fn classify_local_date_with_zone(
    date: CivilDate,
    zone_id: &str,
    zone: &TimeZone,
) -> Result<LocalDateExistence, TimeError> {
    let intervals = local_date_utc_intervals(date, zone)?;
    let status = if intervals.is_empty() {
        LocalDateStatus::Skipped
    } else {
        LocalDateStatus::Exists(intervals)
    };
    Ok(LocalDateExistence {
        requested_date: date,
        zone_id: zone_id.to_owned(),
        tzdb_version: TZDB_VERSION,
        rule_pack: BUNDLED_RULE_PACK_IDENTITY,
        policy_id: LOCAL_DATE_EXISTENCE_POLICY_ID,
        policy_revision: LOCAL_DATE_EXISTENCE_POLICY_REVISION,
        status,
    })
}

fn local_date_utc_intervals(
    date: CivilDate,
    zone: &TimeZone,
) -> Result<Vec<LocalDateUtcInterval>, TimeError> {
    let (window_start, window_end) = local_date_utc_window(date)?;
    let local_day_start = date
        .days_since_unix_epoch()
        .checked_mul(SECONDS_PER_DAY)
        .ok_or(TimeError::DateWindowOverflow)?;
    let local_day_end = local_day_start
        .checked_add(SECONDS_PER_DAY - 1)
        .ok_or(TimeError::DateWindowOverflow)?;
    let start_instant = Timestamp::from_second(window_start)
        .map_err(|_| TimeError::UnsupportedInstant(window_start))?;
    let mut cursor = window_start;
    let mut offset_seconds_east = zone.to_offset(start_instant).seconds();
    let mut intervals = Vec::new();

    for transition in zone.following(start_instant) {
        let transition_second = transition.timestamp().as_second();
        if transition_second > window_end {
            break;
        }
        let segment_end = transition_second
            .checked_sub(1)
            .ok_or(TimeError::DateWindowOverflow)?;
        append_local_date_intersection(
            cursor,
            segment_end,
            offset_seconds_east,
            local_day_start,
            local_day_end,
            &mut intervals,
        )?;
        cursor = transition_second;
        offset_seconds_east = transition.offset().seconds();
    }
    append_local_date_intersection(
        cursor,
        window_end,
        offset_seconds_east,
        local_day_start,
        local_day_end,
        &mut intervals,
    )?;
    Ok(intervals)
}

fn append_local_date_intersection(
    segment_start: i64,
    segment_end: i64,
    offset_seconds_east: i32,
    local_day_start: i64,
    local_day_end: i64,
    output: &mut Vec<LocalDateUtcInterval>,
) -> Result<(), TimeError> {
    if segment_start > segment_end {
        return Ok(());
    }
    let offset = i64::from(offset_seconds_east);
    let interval_start = segment_start.max(
        local_day_start
            .checked_sub(offset)
            .ok_or(TimeError::DateWindowOverflow)?,
    );
    let interval_end = segment_end.min(
        local_day_end
            .checked_sub(offset)
            .ok_or(TimeError::DateWindowOverflow)?,
    );
    if interval_start <= interval_end {
        output.push(LocalDateUtcInterval {
            start: UtcInstant {
                unix_seconds: interval_start,
            },
            end: UtcInstant {
                unix_seconds: interval_end,
            },
            offset_seconds_east,
        });
    }
    Ok(())
}

/// Select a local-date solar cycle and convert its seven UTC events to local
/// civil time using one exact pinned TZif image.
///
/// This composes [`select_local_date_transits`] with UTC-to-local conversion;
/// it does not change the core's event instants, round displayed minutes, pick
/// a winner when a date has multiple cycles, or infer a zone from coordinates.
/// A local event can fall on a date adjacent to `date`. `Zero` has the same
/// meaning as for transit selection and does not prove that the civil date was
/// skipped.
pub fn calculate_local_date_prayer_schedule(
    date: CivilDate,
    coordinates: Coordinates,
    zone_id: &str,
    tzif_bytes: &[u8],
    method: MethodProfile,
    asr_criterion: AsrCriterion,
) -> Result<LocalDatePrayerSchedule, TimeError> {
    let zone = parse_pinned_zone(zone_id, tzif_bytes)?;
    let civil_date = classify_local_date_with_zone(date, zone_id, &zone)?;
    let selection = select_local_date_transits_with_zone(
        date,
        coordinates,
        zone_id,
        method,
        asr_criterion,
        &zone,
    )?;
    let record = selection.record;
    let matches = match selection.matches {
        LocalDateTransitMatches::Zero => LocalDatePrayerScheduleMatches::Zero,
        LocalDateTransitMatches::One(candidate) => LocalDatePrayerScheduleMatches::One(Box::new(
            localize_candidate(*candidate, zone_id, &zone)?,
        )),
        LocalDateTransitMatches::Multiple(candidates) => {
            let candidates = candidates
                .into_iter()
                .map(|candidate| localize_candidate(candidate, zone_id, &zone))
                .collect::<Result<Vec<_>, _>>()?;
            LocalDatePrayerScheduleMatches::Multiple(candidates)
        }
    };
    Ok(LocalDatePrayerSchedule {
        record,
        civil_date,
        policy_id: LOCAL_DATE_PRAYER_SCHEDULE_POLICY_ID,
        policy_revision: LOCAL_DATE_PRAYER_SCHEDULE_POLICY_REVISION,
        matches,
    })
}

fn localize_candidate(
    candidate: LocalDateTransitCandidate,
    zone_id: &str,
    zone: &TimeZone,
) -> Result<LocalDatePrayerScheduleCandidate, TimeError> {
    let cycle = candidate.cycle;
    let events = LocalPrayerEvents {
        fajr: localize_event(cycle.fajr, zone_id, zone)?,
        sunrise: localize_event(cycle.sunrise, zone_id, zone)?,
        dhuhr: localize_event(cycle.dhuhr, zone_id, zone)?,
        asr: localize_event(cycle.asr, zone_id, zone)?,
        sunset: localize_event(cycle.sunset, zone_id, zone)?,
        maghrib: localize_event(cycle.maghrib, zone_id, zone)?,
        isha: localize_event(cycle.isha, zone_id, zone)?,
    };
    Ok(LocalDatePrayerScheduleCandidate {
        cycle,
        local_transit: candidate.local_transit,
        events,
    })
}

fn localize_event(
    event: Event,
    zone_id: &str,
    zone: &TimeZone,
) -> Result<LocalizedEvent, TimeError> {
    match event {
        Event::Occurs {
            utc,
            unrounded_utc_unix_seconds,
            rule,
        } => Ok(LocalizedEvent::Occurs {
            local: convert_with_zone(utc, zone_id, zone)?,
            unrounded_utc_unix_seconds,
            rule,
        }),
        Event::Unavailable { reason } => Ok(LocalizedEvent::Unavailable { reason }),
    }
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
        assert!(matches!(
            convert_utc_to_local(utc, "Etc/UTC", b"TZif untrusted data"),
            Err(TimeError::UnpinnedTzif(_))
        ));
    }
}
