//! Solar position using the NOAA solar-calculation equations derived from
//! Jean Meeus, *Astronomical Algorithms*. The Julian day is based on UTC as
//! a civil approximation to UT1; this model is intended for minute-level
//! prayer-time work in 1900–2100, not sub-second ephemeris research.

use crate::{CalculationError, Coordinates};

const SECONDS_PER_DAY: f64 = 86_400.0;
const JULIAN_DAY_UNIX_EPOCH: f64 = 2_440_587.5;

#[derive(Debug, Clone, Copy)]
pub(crate) struct SolarPosition {
    pub declination_radians: f64,
    pub equation_of_time_minutes: f64,
}

pub(crate) fn position(utc_unix_seconds: f64) -> SolarPosition {
    let julian_day = utc_unix_seconds / SECONDS_PER_DAY + JULIAN_DAY_UNIX_EPOCH;
    let centuries = (julian_day - 2_451_545.0) / 36_525.0;
    let mean_longitude =
        (280.46646 + centuries * (36_000.769_83 + 0.0003032 * centuries)).rem_euclid(360.0);
    let mean_anomaly = (357.52911 + centuries * (35_999.050_29 - 0.0001537 * centuries))
        .rem_euclid(360.0)
        .to_radians();
    let eccentricity = 0.016708634 - centuries * (0.000042037 + 0.0000001267 * centuries);

    let equation_of_center = mean_anomaly.sin()
        * (1.914602 - centuries * (0.004817 + 0.000014 * centuries))
        + (2.0 * mean_anomaly).sin() * (0.019993 - 0.000101 * centuries)
        + (3.0 * mean_anomaly).sin() * 0.000289;
    let true_longitude = mean_longitude + equation_of_center;
    let omega = (125.04 - 1934.136 * centuries).to_radians();
    let apparent_longitude = (true_longitude - 0.00569 - 0.00478 * omega.sin()).to_radians();

    let mean_obliquity = 23.0
        + (26.0
            + (21.448 - centuries * (46.815 + centuries * (0.00059 - centuries * 0.001813)))
                / 60.0)
            / 60.0;
    let obliquity = (mean_obliquity + 0.00256 * omega.cos()).to_radians();
    let declination = (obliquity.sin() * apparent_longitude.sin()).asin();

    let y = (obliquity / 2.0).tan().powi(2);
    let l0 = mean_longitude.to_radians();
    let equation_of_time = 4.0
        * (y * (2.0 * l0).sin() - 2.0 * eccentricity * mean_anomaly.sin()
            + 4.0 * eccentricity * y * mean_anomaly.sin() * (2.0 * l0).cos()
            - 0.5 * y * y * (4.0 * l0).sin()
            - 1.25 * eccentricity * eccentricity * (2.0 * mean_anomaly).sin())
        .to_degrees();

    SolarPosition {
        declination_radians: declination,
        equation_of_time_minutes: equation_of_time,
    }
}

pub(crate) fn altitude_degrees(utc_unix_seconds: f64, coordinates: Coordinates) -> f64 {
    let solar = position(utc_unix_seconds);
    let utc_minutes_of_day = utc_unix_seconds.rem_euclid(SECONDS_PER_DAY) / 60.0;
    let true_solar_minutes = (utc_minutes_of_day
        + solar.equation_of_time_minutes
        + 4.0 * coordinates.longitude_degrees())
    .rem_euclid(1440.0);
    let hour_angle = (true_solar_minutes / 4.0 - 180.0).to_radians();
    let latitude = coordinates.latitude_degrees().to_radians();
    let sin_altitude = latitude.sin() * solar.declination_radians.sin()
        + latitude.cos() * solar.declination_radians.cos() * hour_angle.cos();
    sin_altitude.clamp(-1.0, 1.0).asin().to_degrees()
}

/// Find the upper solar transit closest to local clock noon. The offset only
/// selects a solar cycle; it does not affect astronomical position.
pub(crate) fn upper_transit(
    local_day_start_utc: i64,
    coordinates: Coordinates,
) -> Result<f64, CalculationError> {
    let local_noon_utc = local_day_start_utc as f64 + 43_200.0;
    let longitude_seconds = coordinates.longitude_degrees() * 240.0;
    let initial_equation = position(local_noon_utc).equation_of_time_minutes * 60.0;
    let solar_day_index =
        ((local_noon_utc + initial_equation + longitude_seconds) / SECONDS_PER_DAY - 0.5).round();
    let mut estimate = local_noon_utc;
    for _ in 0..8 {
        let equation_seconds = position(estimate).equation_of_time_minutes * 60.0;
        estimate = (solar_day_index + 0.5) * SECONDS_PER_DAY - equation_seconds - longitude_seconds;
    }
    if estimate.is_finite() {
        Ok(estimate)
    } else {
        Err(CalculationError::NumericalFailure)
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum CrossingDirection {
    Rising,
    Falling,
}

/// Find the requested crossing of a solar condition. A same-sign pair of
/// endpoints can hide two crossings near a midnight extremum, so inspect the
/// interior extremum before declaring an event unavailable.
pub(crate) fn crossing(
    start: f64,
    end: f64,
    direction: CrossingDirection,
    condition: impl Fn(f64) -> f64,
) -> Result<Option<f64>, CalculationError> {
    let start_value = condition(start);
    let end_value = condition(end);
    if !start_value.is_finite() || !end_value.is_finite() || start >= end {
        return Err(CalculationError::NumericalFailure);
    }
    if start_value == 0.0 {
        return Ok(Some(start));
    }
    if end_value == 0.0 {
        return Ok(Some(end));
    }

    let bracket = match direction {
        CrossingDirection::Rising if start_value < 0.0 && end_value > 0.0 => Some((start, end)),
        CrossingDirection::Falling if start_value > 0.0 && end_value < 0.0 => Some((start, end)),
        _ if start_value > 0.0 && end_value > 0.0 => {
            let minimum = interior_extremum(start, end, &condition, false)?;
            if condition(minimum) >= 0.0 {
                None
            } else {
                match direction {
                    CrossingDirection::Rising => Some((minimum, end)),
                    CrossingDirection::Falling => Some((start, minimum)),
                }
            }
        }
        _ if start_value < 0.0 && end_value < 0.0 => {
            let maximum = interior_extremum(start, end, &condition, true)?;
            if condition(maximum) <= 0.0 {
                None
            } else {
                match direction {
                    CrossingDirection::Rising => Some((start, maximum)),
                    CrossingDirection::Falling => Some((maximum, end)),
                }
            }
        }
        _ => None,
    };
    let Some((mut low, mut high)) = bracket else {
        return Ok(None);
    };
    let mut low_value = condition(low);
    if !low_value.is_finite() {
        return Err(CalculationError::NumericalFailure);
    }
    for _ in 0..50 {
        let middle = (low + high) / 2.0;
        let middle_value = condition(middle);
        if !middle_value.is_finite() {
            return Err(CalculationError::NumericalFailure);
        }
        if middle_value.signum() == low_value.signum() {
            low = middle;
            low_value = middle_value;
        } else {
            high = middle;
        }
    }
    Ok(Some((low + high) / 2.0))
}

/// Golden-section search on a half solar cycle. Altitude is unimodal in the
/// interval in the supported model; the test suite covers the midnight-grazing
/// case that motivates this extra search.
fn interior_extremum(
    start: f64,
    end: f64,
    condition: &impl Fn(f64) -> f64,
    maximum: bool,
) -> Result<f64, CalculationError> {
    let mut low = start;
    let mut high = end;
    let golden = (5.0_f64.sqrt() - 1.0) / 2.0;
    for _ in 0..80 {
        let left = high - golden * (high - low);
        let right = low + golden * (high - low);
        let left_value = condition(left);
        let right_value = condition(right);
        if !left_value.is_finite() || !right_value.is_finite() {
            return Err(CalculationError::NumericalFailure);
        }
        if (left_value < right_value) == maximum {
            low = left;
        } else {
            high = right;
        }
    }
    Ok((low + high) / 2.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crossing_finds_direction_when_endpoints_hide_two_roots() {
        let parabola = |t: f64| (t - 0.2) * (t - 0.4);
        let falling = crossing(0.0, 1.0, CrossingDirection::Falling, parabola)
            .unwrap()
            .unwrap();
        let rising = crossing(0.0, 1.0, CrossingDirection::Rising, parabola)
            .unwrap()
            .unwrap();
        assert!((falling - 0.2).abs() < 1e-8);
        assert!((rising - 0.4).abs() < 1e-8);
    }
}
