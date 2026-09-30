//! Thin, offline orchestration for a user-selected local prayer schedule.
//!
//! `salah-core` remains the astronomy and prayer calculation kernel;
//! `salah-location` provides an approximate zone suggestion that a user must
//! confirm or override; and `salah-time` owns local-date selection and IANA
//! conversion. This crate wires those layers together without introducing
//! method defaults, date inference, or new calculation rules.

use core::fmt;

use salah_core::{AsrCriterion, CivilDate, MethodProfile};
use salah_location::ZoneSelection;
use salah_time::{
    LocalDatePrayerSchedule, RulePackIdentity, TZDB_VERSION, TimeError,
    calculate_local_date_prayer_schedule, fixture_tzif_bytes,
};

/// Identity of the advisory assessment for installed civil-time data.
pub const DATA_ASSESSMENT_POLICY_ID: &str = "installed-civil-time-data-assessment";
/// Revision of [`DATA_ASSESSMENT_POLICY_ID`].
pub const DATA_ASSESSMENT_POLICY_REVISION: &str = "0.1";

/// One local-date schedule calculated under an explicitly confirmed or
/// manually selected time zone.
#[derive(Debug, Clone)]
pub struct SelectedLocalDaySchedule {
    /// The user's zone choice and the mapping evidence from which it came.
    pub zone_selection: ZoneSelection,
    /// The requested-date result, including skipped-date status, all matching
    /// solar cycles, local event times, method metadata, and IANA provenance.
    pub schedule: LocalDatePrayerSchedule,
}

/// A concrete reason to review the installed civil-time data before relying
/// on a displayed local clock time. Neither variant changes the calculation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CivilTimeDataNotice {
    /// The installed IANA release is from an earlier calendar year than the
    /// caller's observation date. A newer release may or may not exist.
    PackPredatesObservationYear,
    /// The requested local date is after the caller's observation date, so
    /// future civil-time legislation could change its local clock labels.
    RequestedDateIsFuture,
}

/// Deterministic advisory about the data used by a selected-day schedule.
/// An empty `notices` list is not an accuracy or legal-validity certificate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CivilTimeDataAssessment {
    pub policy_id: &'static str,
    pub policy_revision: &'static str,
    /// Caller-supplied date used for the assessment; no device clock is read.
    pub observed_on: CivilDate,
    pub requested_date: CivilDate,
    pub boundary_data_version: String,
    pub rule_pack: RulePackIdentity,
    pub notices: Vec<CivilTimeDataNotice>,
}

/// Assess the installed rule snapshot for a selected-day schedule.
///
/// The caller explicitly supplies the date on which the assessment is made.
/// This comparison is a prompt to check for newer data, not a determination
/// that any offset is wrong or that a newer IANA release exists. A future date
/// is marked even when it falls in the release year. Historical IANA coverage
/// and boundary geometry require their own review.
#[must_use]
pub fn assess_civil_time_data(
    selected: &SelectedLocalDaySchedule,
    observed_on: CivilDate,
) -> CivilTimeDataAssessment {
    let requested_date = selected.schedule.record.requested_date;
    let mut notices = Vec::new();
    if observed_on.year() > selected.schedule.record.rule_pack.release_year {
        notices.push(CivilTimeDataNotice::PackPredatesObservationYear);
    }
    if requested_date.days_since_unix_epoch() > observed_on.days_since_unix_epoch() {
        notices.push(CivilTimeDataNotice::RequestedDateIsFuture);
    }
    CivilTimeDataAssessment {
        policy_id: DATA_ASSESSMENT_POLICY_ID,
        policy_revision: DATA_ASSESSMENT_POLICY_REVISION,
        observed_on,
        requested_date,
        boundary_data_version: selected
            .zone_selection
            .candidates()
            .boundary_data_version()
            .to_owned(),
        rule_pack: selected.schedule.record.rule_pack,
        notices,
    }
}

/// Errors raised while composing the selected-zone schedule.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EngineError {
    /// The supplied selection was created against different boundary/time-zone
    /// versions from this engine build.
    IncompatibleZoneSelection {
        boundary_data_version: String,
        selection_timezone_database_version: &'static str,
        engine_timezone_database_version: &'static str,
    },
    /// The pinned zone pack or solar calculation rejected the request.
    Time(TimeError),
}

impl fmt::Display for EngineError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::IncompatibleZoneSelection {
                boundary_data_version,
                selection_timezone_database_version,
                engine_timezone_database_version,
            } => write!(
                f,
                "zone selection uses boundary data {boundary_data_version} and IANA rules {selection_timezone_database_version}; this engine uses IANA rules {engine_timezone_database_version}"
            ),
            Self::Time(error) => write!(f, "local prayer schedule failed: {error}"),
        }
    }
}

impl std::error::Error for EngineError {}

impl From<TimeError> for EngineError {
    fn from(error: TimeError) -> Self {
        Self::Time(error)
    }
}

/// Calculate a local-day schedule using an explicit user-selected IANA zone.
///
/// The caller supplies the local Gregorian date, method profile, and Asr
/// criterion. This function does not inspect device settings, current time,
/// network state, or unconfirmed coordinate suggestions. A skipped date,
/// zero/multiple matching solar cycles, and unavailable events stay explicit
/// in the returned [`LocalDatePrayerSchedule`].
pub fn calculate_selected_local_day_schedule(
    zone_selection: &ZoneSelection,
    requested_date: CivilDate,
    method: MethodProfile,
    asr_criterion: AsrCriterion,
) -> Result<SelectedLocalDaySchedule, EngineError> {
    let selection_tzdb_version = zone_selection.candidates().timezone_database_version();
    if selection_tzdb_version != TZDB_VERSION
        || zone_selection.candidates().boundary_data_version() != TZDB_VERSION
    {
        return Err(EngineError::IncompatibleZoneSelection {
            boundary_data_version: zone_selection
                .candidates()
                .boundary_data_version()
                .to_owned(),
            selection_timezone_database_version: selection_tzdb_version,
            engine_timezone_database_version: TZDB_VERSION,
        });
    }

    let zone_id = zone_selection.zone_id();
    let tzif_bytes = fixture_tzif_bytes(zone_id)?;
    let schedule = calculate_local_date_prayer_schedule(
        requested_date,
        zone_selection.coordinates(),
        zone_id,
        tzif_bytes,
        method,
        asr_criterion,
    )?;

    Ok(SelectedLocalDaySchedule {
        zone_selection: zone_selection.clone(),
        schedule,
    })
}
