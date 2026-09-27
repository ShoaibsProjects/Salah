use crate::CalculationError;

/// The shadow criterion used for Asr. The factor is added to the noon shadow.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AsrCriterion {
    Standard,
    Hanafi,
}

impl AsrCriterion {
    pub fn shadow_factor(self) -> f64 {
        match self {
            Self::Standard => 1.0,
            Self::Hanafi => 2.0,
        }
    }
}

/// A versioned parameter set. This first slice supports angle-based Fajr and Isha only.
/// The built-in research profile is not attributed to a religious institution.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MethodProfile {
    pub id: &'static str,
    pub revision: &'static str,
    pub fajr_depression_degrees: f64,
    pub isha_depression_degrees: f64,
    pub dhuhr_adjustment_seconds: i32,
    pub maghrib_adjustment_seconds: i32,
}

impl MethodProfile {
    pub const fn research_15() -> Self {
        Self {
            id: "research-15",
            revision: "0.1",
            fajr_depression_degrees: 15.0,
            isha_depression_degrees: 15.0,
            dhuhr_adjustment_seconds: 0,
            maghrib_adjustment_seconds: 0,
        }
    }

    pub fn validate(self) -> Result<Self, CalculationError> {
        if self.id.is_empty()
            || self.revision.is_empty()
            || !self.fajr_depression_degrees.is_finite()
            || !self.isha_depression_degrees.is_finite()
            || !(0.0..=30.0).contains(&self.fajr_depression_degrees)
            || !(0.0..=30.0).contains(&self.isha_depression_degrees)
            || self.fajr_depression_degrees == 0.0
            || self.isha_depression_degrees == 0.0
            || !(-7200..=7200).contains(&self.dhuhr_adjustment_seconds)
            || !(-7200..=7200).contains(&self.maghrib_adjustment_seconds)
        {
            return Err(CalculationError::InvalidMethod);
        }
        Ok(self)
    }
}
