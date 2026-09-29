use crate::solar::{self, CrossingDirection, altitude_degrees, crossing, position};
use crate::{
    AsrCriterion, CalculationError, CivilDate, Coordinates, FixedUtcOffset, MethodProfile,
    UtcInstant,
};
use core::fmt;

/// Standard apparent sunrise/sunset altitude at sea level: solar center is
/// about 0.833° below the geometric horizon. This combines mean refraction
/// and apparent solar radius; it does not model local terrain or weather.
const APPARENT_HORIZON_DEGREES: f64 = -0.833;
const HALF_SOLAR_DAY_SECONDS: f64 = 43_200.0;
pub const ASTRONOMY_MODEL: &str = "NOAA-MEEUS-SOLAR-2";

/// Selection-policy identity for the additive UTC-anchor interface.
///
/// The selector returns the upper solar transit nearest to the caller-supplied
/// UTC anchor using the same equation-of-time iteration as the legacy
/// local-noon estimate. It does not consult a local date, fixed offset, or
/// time-zone rule.
pub const UTC_ANCHOR_SELECTION_POLICY_ID: &str = "utc-anchor-nearest-transit";
/// Selection-policy revision for the additive UTC-anchor interface.
pub const UTC_ANCHOR_SELECTION_POLICY_REVISION: &str = "0.1";

/// Minimum supported UTC-anchor Unix seconds: 1899-12-31T00:00:00Z.
///
/// The bound covers the earliest legacy local-noon UTC estimate
/// (1899-12-31T22:00:00Z for 1900-01-01 at +14:00) with margin while keeping
/// the transit search and Unix-second rounding inside finite `f64`/`i64`
/// arithmetic.
pub const UTC_ANCHOR_MIN_UNIX_SECONDS: i64 = -2_209_075_200;
/// Maximum supported UTC-anchor Unix seconds: 2101-01-01T23:59:59Z.
///
/// The bound covers the latest legacy local-noon UTC estimate
/// (2101-01-01T02:00:00Z for 2100-12-31 at −14:00) with margin while keeping
/// the transit search and Unix-second rounding inside finite `f64`/`i64`
/// arithmetic.
pub const UTC_ANCHOR_MAX_UNIX_SECONDS: i64 = 4_134_067_199;

#[derive(Debug, Clone, Copy)]
pub struct CalculationInput {
    pub coordinates: Coordinates,
    pub local_date: CivilDate,
    pub utc_offset: FixedUtcOffset,
    pub method: MethodProfile,
    pub asr_criterion: AsrCriterion,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EventRule {
    SolarTransit,
    ApparentHorizon,
    SolarDepression { degrees: f64 },
    AsrShadow { factor: f64 },
    SunsetWithAdjustment { seconds: i32 },
    TransitWithAdjustment { seconds: i32 },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Event {
    Occurs {
        utc: UtcInstant,
        unrounded_utc_unix_seconds: f64,
        rule: EventRule,
    },
    Unavailable {
        reason: UnavailableReason,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnavailableReason {
    NoCrossingInSolarCycle,
}

impl fmt::Display for UnavailableReason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoCrossingInSolarCycle => {
                f.write_str("the solar condition has no crossing in this local solar cycle")
            }
        }
    }
}

impl Event {
    pub fn utc(self) -> Option<UtcInstant> {
        match self {
            Self::Occurs { utc, .. } => Some(utc),
            Self::Unavailable { .. } => None,
        }
    }

    pub fn unrounded_utc_unix_seconds(self) -> Option<f64> {
        match self {
            Self::Occurs {
                unrounded_utc_unix_seconds,
                ..
            } => Some(unrounded_utc_unix_seconds),
            Self::Unavailable { .. } => None,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct CalculationRecord {
    pub coordinates: Coordinates,
    pub local_date: CivilDate,
    pub utc_offset: FixedUtcOffset,
    pub method: MethodProfile,
    pub asr_criterion: AsrCriterion,
    pub astronomy_model: &'static str,
    pub engine_version: &'static str,
    pub high_latitude_rule: &'static str,
    pub assumed_elevation_meters: f64,
}

#[derive(Debug, Clone, Copy)]
pub struct PrayerTimes {
    pub fajr: Event,
    pub sunrise: Event,
    pub dhuhr: Event,
    pub asr: Event,
    pub sunset: Event,
    pub maghrib: Event,
    pub isha: Event,
    pub record: CalculationRecord,
}

/// Input for one solar cycle selected by an explicit UTC instant anchor.
///
/// Unlike [`CalculationInput`], this carries no civil date and no fixed UTC
/// offset. The anchor does not claim membership in any local date.
#[derive(Debug, Clone, Copy)]
pub struct UtcAnchorInput {
    pub coordinates: Coordinates,
    pub anchor: UtcInstant,
    pub method: MethodProfile,
    pub asr_criterion: AsrCriterion,
}

/// Provenance for one UTC-anchored solar-cycle calculation.
///
/// There is deliberately no requested local date and no fixed offset: the
/// selected transit is identified by the anchor, the versioned
/// UTC-anchor selection policy, and the astronomy/method records.
#[derive(Debug, Clone, Copy)]
pub struct UtcAnchorRecord {
    pub coordinates: Coordinates,
    pub anchor: UtcInstant,
    pub selected_transit: UtcInstant,
    pub unrounded_selected_transit_unix_seconds: f64,
    pub selection_policy_id: &'static str,
    pub selection_policy_revision: &'static str,
    pub method: MethodProfile,
    pub asr_criterion: AsrCriterion,
    pub astronomy_model: &'static str,
    pub engine_version: &'static str,
    pub high_latitude_rule: &'static str,
    pub assumed_elevation_meters: f64,
}

/// One solar cycle selected by an explicit UTC anchor, with the seven
/// existing event kinds and their existing rules and unavailable statuses.
#[derive(Debug, Clone, Copy)]
pub struct UtcAnchorTimes {
    pub fajr: Event,
    pub sunrise: Event,
    pub dhuhr: Event,
    pub asr: Event,
    pub sunset: Event,
    pub maghrib: Event,
    pub isha: Event,
    pub selected_transit: UtcInstant,
    pub unrounded_selected_transit_unix_seconds: f64,
    pub record: UtcAnchorRecord,
}

/// Calculate a local solar cycle with one explicit fixed UTC offset. There is
/// no time-zone lookup, DST rule, atmospheric observation, or high-latitude
/// substitution in this first core slice.
pub fn calculate_prayer_times(input: CalculationInput) -> Result<PrayerTimes, CalculationError> {
    let method = input.method.validate()?;
    let local_day_start_utc = input.local_date.days_since_unix_epoch() * 86_400
        - i64::from(input.utc_offset.minutes_east()) * 60;
    let transit = solar::upper_transit(local_day_start_utc, input.coordinates)?;
    let events = events_from_transit(transit, input.coordinates, method, input.asr_criterion)?;

    Ok(PrayerTimes {
        fajr: events.fajr,
        sunrise: events.sunrise,
        dhuhr: events.dhuhr,
        asr: events.asr,
        sunset: events.sunset,
        maghrib: events.maghrib,
        isha: events.isha,
        record: CalculationRecord {
            coordinates: input.coordinates,
            local_date: input.local_date,
            utc_offset: input.utc_offset,
            method,
            asr_criterion: input.asr_criterion,
            astronomy_model: ASTRONOMY_MODEL,
            engine_version: env!("CARGO_PKG_VERSION"),
            high_latitude_rule: "none",
            assumed_elevation_meters: 0.0,
        },
    })
}

/// Calculate one solar cycle from an explicit UTC instant anchor.
///
/// The anchor selects the nearest upper solar transit through the versioned
/// [`UTC_ANCHOR_SELECTION_POLICY_ID`] selector; the seven returned events use
/// the same astronomy, method rules, adjustments, rounding, and unavailable
/// statuses as [`calculate_prayer_times`]. The result carries UTC-only
/// provenance and asserts nothing about which local date the selected transit
/// belongs to under any civil time zone.
pub fn calculate_utc_anchor_times(
    input: UtcAnchorInput,
) -> Result<UtcAnchorTimes, CalculationError> {
    if input.anchor.unix_seconds < UTC_ANCHOR_MIN_UNIX_SECONDS
        || input.anchor.unix_seconds > UTC_ANCHOR_MAX_UNIX_SECONDS
    {
        return Err(CalculationError::InvalidUtcAnchor);
    }
    let method = input.method.validate()?;
    let transit = solar::upper_transit_from_noon_estimate(
        input.anchor.unix_seconds as f64,
        input.coordinates,
    )?;
    let events = events_from_transit(transit, input.coordinates, method, input.asr_criterion)?;
    let selected_transit = UtcInstant {
        unix_seconds: checked_round_unix_seconds(transit)?,
    };

    Ok(UtcAnchorTimes {
        fajr: events.fajr,
        sunrise: events.sunrise,
        dhuhr: events.dhuhr,
        asr: events.asr,
        sunset: events.sunset,
        maghrib: events.maghrib,
        isha: events.isha,
        selected_transit,
        unrounded_selected_transit_unix_seconds: transit,
        record: UtcAnchorRecord {
            coordinates: input.coordinates,
            anchor: input.anchor,
            selected_transit,
            unrounded_selected_transit_unix_seconds: transit,
            selection_policy_id: UTC_ANCHOR_SELECTION_POLICY_ID,
            selection_policy_revision: UTC_ANCHOR_SELECTION_POLICY_REVISION,
            method,
            asr_criterion: input.asr_criterion,
            astronomy_model: ASTRONOMY_MODEL,
            engine_version: env!("CARGO_PKG_VERSION"),
            high_latitude_rule: "none",
            assumed_elevation_meters: 0.0,
        },
    })
}

struct CycleEvents {
    fajr: Event,
    sunrise: Event,
    dhuhr: Event,
    asr: Event,
    sunset: Event,
    maghrib: Event,
    isha: Event,
}

fn events_from_transit(
    transit: f64,
    coordinates: Coordinates,
    method: MethodProfile,
    asr_criterion: AsrCriterion,
) -> Result<CycleEvents, CalculationError> {
    if !transit.is_finite() {
        return Err(CalculationError::NumericalFailure);
    }
    let morning_start = transit - HALF_SOLAR_DAY_SECONDS;
    let evening_end = transit + HALF_SOLAR_DAY_SECONDS;
    if !morning_start.is_finite() || !evening_end.is_finite() {
        return Err(CalculationError::NumericalFailure);
    }

    let apparent_sunrise = crossing(morning_start, transit, CrossingDirection::Rising, |t| {
        altitude_degrees(t, coordinates) - APPARENT_HORIZON_DEGREES
    })?;
    let apparent_sunset = crossing(transit, evening_end, CrossingDirection::Falling, |t| {
        altitude_degrees(t, coordinates) - APPARENT_HORIZON_DEGREES
    })?;
    let fajr_crossing = crossing(morning_start, transit, CrossingDirection::Rising, |t| {
        altitude_degrees(t, coordinates) + method.fajr_depression_degrees
    })?;
    let isha_crossing = crossing(transit, evening_end, CrossingDirection::Falling, |t| {
        altitude_degrees(t, coordinates) + method.isha_depression_degrees
    })?;

    let asr_factor = asr_criterion.shadow_factor();
    let noon_declination = position(transit).declination_radians;
    let noon_zenith_distance =
        (coordinates.latitude_degrees().to_radians() - noon_declination).abs();
    let asr_crossing = if noon_zenith_distance >= core::f64::consts::FRAC_PI_2 {
        None
    } else {
        let noon_shadow = noon_zenith_distance.tan();
        let target_altitude = (1.0 / (asr_factor + noon_shadow)).atan().to_degrees();
        crossing(transit, evening_end, CrossingDirection::Falling, |t| {
            altitude_degrees(t, coordinates) - target_altitude
        })?
    };

    let sunrise = astronomical_event(apparent_sunrise, EventRule::ApparentHorizon)?;
    let sunset = astronomical_event(apparent_sunset, EventRule::ApparentHorizon)?;
    let fajr = astronomical_event(
        fajr_crossing,
        EventRule::SolarDepression {
            degrees: method.fajr_depression_degrees,
        },
    )?;
    let isha = astronomical_event(
        isha_crossing,
        EventRule::SolarDepression {
            degrees: method.isha_depression_degrees,
        },
    )?;
    let asr = astronomical_event(asr_crossing, EventRule::AsrShadow { factor: asr_factor })?;

    let transit_rounded = checked_round_unix_seconds(transit)?;
    let dhuhr_unrounded = transit + f64::from(method.dhuhr_adjustment_seconds);
    if !dhuhr_unrounded.is_finite() {
        return Err(CalculationError::NumericalFailure);
    }
    let dhuhr = Event::Occurs {
        utc: UtcInstant {
            unix_seconds: transit_rounded
                .checked_add(i64::from(method.dhuhr_adjustment_seconds))
                .ok_or(CalculationError::NumericalFailure)?,
        },
        unrounded_utc_unix_seconds: dhuhr_unrounded,
        rule: if method.dhuhr_adjustment_seconds == 0 {
            EventRule::SolarTransit
        } else {
            EventRule::TransitWithAdjustment {
                seconds: method.dhuhr_adjustment_seconds,
            }
        },
    };
    let maghrib = match sunset {
        Event::Occurs {
            utc,
            unrounded_utc_unix_seconds,
            ..
        } => {
            let maghrib_unrounded =
                unrounded_utc_unix_seconds + f64::from(method.maghrib_adjustment_seconds);
            if !maghrib_unrounded.is_finite() {
                return Err(CalculationError::NumericalFailure);
            }
            Event::Occurs {
                utc: UtcInstant {
                    unix_seconds: utc
                        .unix_seconds
                        .checked_add(i64::from(method.maghrib_adjustment_seconds))
                        .ok_or(CalculationError::NumericalFailure)?,
                },
                unrounded_utc_unix_seconds: maghrib_unrounded,
                rule: EventRule::SunsetWithAdjustment {
                    seconds: method.maghrib_adjustment_seconds,
                },
            }
        }
        Event::Unavailable { reason } => Event::Unavailable { reason },
    };

    Ok(CycleEvents {
        fajr,
        sunrise,
        dhuhr,
        asr,
        sunset,
        maghrib,
        isha,
    })
}

fn checked_round_unix_seconds(value: f64) -> Result<i64, CalculationError> {
    if !value.is_finite() {
        return Err(CalculationError::NumericalFailure);
    }
    let rounded = value.round();
    if rounded < i64::MIN as f64 || rounded > i64::MAX as f64 {
        return Err(CalculationError::NumericalFailure);
    }
    Ok(rounded as i64)
}

fn astronomical_event(candidate: Option<f64>, rule: EventRule) -> Result<Event, CalculationError> {
    match candidate {
        Some(utc) => Ok(Event::Occurs {
            utc: UtcInstant {
                unix_seconds: checked_round_unix_seconds(utc)?,
            },
            unrounded_utc_unix_seconds: utc,
            rule,
        }),
        None => Ok(Event::Unavailable {
            reason: UnavailableReason::NoCrossingInSolarCycle,
        }),
    }
}
