//! Offline solar and prayer-time calculations for Earth.
//!
//! The first release deliberately uses an explicit fixed UTC offset and a
//! research-only 15°/15° profile. It does not claim a named institutional
//! method or infer a civil time zone from coordinates.

mod civil;
mod method;
mod prayer;
mod solar;

pub use civil::{CivilDate, CivilDateTime, FixedUtcOffset, UtcInstant};
pub use method::{AsrCriterion, MethodProfile};
pub use prayer::{
    CalculationInput, CalculationRecord, Event, EventRule, PrayerTimes, UnavailableReason,
    calculate_prayer_times,
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
    NumericalFailure,
}

impl fmt::Display for CalculationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidCoordinates => f.write_str("coordinates must be finite and in range"),
            Self::InvalidDate => f.write_str("date must be valid and within 1900–2100"),
            Self::InvalidUtcOffset => f.write_str("UTC offset must be within ±14 hours"),
            Self::InvalidMethod => f.write_str("method parameters are invalid"),
            Self::NumericalFailure => f.write_str("solar calculation failed numerically"),
        }
    }
}

impl std::error::Error for CalculationError {}
