//! Offline solar and prayer-time calculations for Earth.
//!
//! This research kernel uses an explicit fixed UTC offset and source-labeled
//! angle profiles. It does not claim institutional endorsement or infer a
//! civil time zone from coordinates.

mod civil;
mod method;
mod prayer;
mod presentation;
mod solar;

pub use civil::{CivilDate, CivilDateTime, FixedUtcOffset, UtcInstant};
pub use method::{AsrCriterion, MethodProfile};
pub use prayer::{
    CalculationInput, CalculationRecord, Event, EventRule, PrayerTimes,
    UTC_ANCHOR_MAX_UNIX_SECONDS, UTC_ANCHOR_MIN_UNIX_SECONDS, UTC_ANCHOR_SELECTION_POLICY_ID,
    UTC_ANCHOR_SELECTION_POLICY_REVISION, UnavailableReason, UtcAnchorInput, UtcAnchorRecord,
    UtcAnchorTimes, calculate_prayer_times, calculate_utc_anchor_times,
};
pub use presentation::{
    DISPLAY_POLICY_ID, DISPLAY_POLICY_REVISION, LocalDisplayMinute, PrayerStart,
    PrayerStartReceipt, PrayerStartStatus, prayer_start_minute,
};

use core::fmt;

/// Validated geographic coordinates, with east-positive longitude.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Coordinates {
    latitude_degrees: f64,
    longitude_degrees: f64,
}

impl Coordinates {
    pub fn new(latitude_degrees: f64, longitude_degrees: f64) -> Result<Self, CalculationError> {
        if !latitude_degrees.is_finite()
            || !longitude_degrees.is_finite()
            || !(-90.0..=90.0).contains(&latitude_degrees)
            || !(-180.0..=180.0).contains(&longitude_degrees)
        {
            return Err(CalculationError::InvalidCoordinates);
        }
        Ok(Self {
            latitude_degrees,
            longitude_degrees,
        })
    }

    pub fn latitude_degrees(self) -> f64 {
        self.latitude_degrees
    }

    pub fn longitude_degrees(self) -> f64 {
        self.longitude_degrees
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CalculationError {
    InvalidCoordinates,
    InvalidDate,
    InvalidUtcOffset,
    InvalidMethod,
    InvalidUtcAnchor,
    NumericalFailure,
}

impl fmt::Display for CalculationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidCoordinates => f.write_str("coordinates must be finite and in range"),
            Self::InvalidDate => f.write_str("date must be valid and within 1900–2100"),
            Self::InvalidUtcOffset => f.write_str("UTC offset must be within ±14 hours"),
            Self::InvalidMethod => f.write_str("method parameters are invalid"),
            Self::InvalidUtcAnchor => f.write_str(
                "UTC anchor must be within 1899-12-31T00:00:00Z through 2101-01-01T23:59:59Z",
            ),
            Self::NumericalFailure => f.write_str("solar calculation failed numerically"),
        }
    }
}

impl std::error::Error for CalculationError {}
