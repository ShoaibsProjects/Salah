use crate::solar::{self, altitude_degrees, crossing, position};
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
pub const ASTRONOMY_MODEL: &str = "NOAA-MEEUS-SOLAR-1";

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

/// Calculate a local solar cycle with one explicit fixed UTC offset. There is
/// no time-zone lookup, DST rule, atmospheric observation, or high-latitude
/// substitution in this first core slice.
pub fn calculate_prayer_times(input: CalculationInput) -> Result<PrayerTimes, CalculationError> {
    let method = input.method.validate()?;
    let local_day_start_utc = input.local_date.days_since_unix_epoch() * 86_400
        - i64::from(input.utc_offset.minutes_east()) * 60;
    let transit = solar::upper_transit(local_day_start_utc, input.coordinates)?;
    let morning_start = transit - HALF_SOLAR_DAY_SECONDS;
    let evening_end = transit + HALF_SOLAR_DAY_SECONDS;

    let apparent_sunrise = crossing(morning_start, transit, |t| {
        altitude_degrees(t, input.coordinates) - APPARENT_HORIZON_DEGREES
    })?;
    let apparent_sunset = crossing(transit, evening_end, |t| {
        altitude_degrees(t, input.coordinates) - APPARENT_HORIZON_DEGREES
    })?;
    let fajr_crossing = crossing(morning_start, transit, |t| {
        altitude_degrees(t, input.coordinates) + method.fajr_depression_degrees
    })?;
    let isha_crossing = crossing(transit, evening_end, |t| {
        altitude_degrees(t, input.coordinates) + method.isha_depression_degrees
    })?;

    let asr_factor = input.asr_criterion.shadow_factor();
    let noon_declination = position(transit).declination_radians;
    let noon_zenith_distance =
        (input.coordinates.latitude_degrees().to_radians() - noon_declination).abs();
    let asr_crossing = if noon_zenith_distance >= core::f64::consts::FRAC_PI_2 {
        None
    } else {
        crossing(transit, evening_end, |t| {
            let declination = position(t).declination_radians;
            let zenith_distance =
                (input.coordinates.latitude_degrees().to_radians() - declination).abs();
            let noon_shadow = zenith_distance.tan();
            let target_altitude = (1.0 / (asr_factor + noon_shadow)).atan().to_degrees();
            altitude_degrees(t, input.coordinates) - target_altitude
        })?
    };

    let sunrise = astronomical_event(apparent_sunrise, EventRule::ApparentHorizon);
    let sunset = astronomical_event(apparent_sunset, EventRule::ApparentHorizon);
    let fajr = astronomical_event(
        fajr_crossing,
        EventRule::SolarDepression {
            degrees: method.fajr_depression_degrees,
        },
    );
    let isha = astronomical_event(
        isha_crossing,
        EventRule::SolarDepression {
            degrees: method.isha_depression_degrees,
        },
    );
    let asr = astronomical_event(asr_crossing, EventRule::AsrShadow { factor: asr_factor });

    let dhuhr = Event::Occurs {
        utc: UtcInstant {
            unix_seconds: transit.round() as i64 + i64::from(method.dhuhr_adjustment_seconds),
        },
        unrounded_utc_unix_seconds: transit + f64::from(method.dhuhr_adjustment_seconds),
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
        } => Event::Occurs {
            utc: UtcInstant {
                unix_seconds: utc.unix_seconds + i64::from(method.maghrib_adjustment_seconds),
            },
            unrounded_utc_unix_seconds: unrounded_utc_unix_seconds
                + f64::from(method.maghrib_adjustment_seconds),
            rule: EventRule::SunsetWithAdjustment {
                seconds: method.maghrib_adjustment_seconds,
            },
        },
        Event::Unavailable { reason } => Event::Unavailable { reason },
    };

    Ok(PrayerTimes {
        fajr,
        sunrise,
        dhuhr,
        asr,
        sunset,
        maghrib,
        isha,
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

fn astronomical_event(candidate: Option<f64>, rule: EventRule) -> Event {
    match candidate {
        Some(utc) => Event::Occurs {
            utc: UtcInstant {
                unix_seconds: utc.round() as i64,
            },
            unrounded_utc_unix_seconds: utc,
            rule,
        },
        None => Event::Unavailable {
            reason: UnavailableReason::NoCrossingInSolarCycle,
        },
    }
}
