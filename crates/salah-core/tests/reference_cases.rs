//! Independent reference cases. Provenance and comparison limits are in
//! `../../../specification/reference-cases.md`.

use salah_core::{
    AsrCriterion, CalculationInput, CivilDate, Coordinates, Event, FixedUtcOffset, MethodProfile,
    UtcInstant, calculate_prayer_times,
};

fn input(
    year: i32,
    month: u8,
    day: u8,
    latitude: f64,
    longitude: f64,
    offset_minutes: i16,
    asr_criterion: AsrCriterion,
) -> CalculationInput {
    CalculationInput {
        coordinates: Coordinates::new(latitude, longitude).unwrap(),
        local_date: CivilDate::new(year, month, day).unwrap(),
        utc_offset: FixedUtcOffset::from_minutes_east(offset_minutes).unwrap(),
        method: MethodProfile::research_15(),
        asr_criterion,
    }
}

fn utc(year: i32, month: u8, day: u8, hour: i64, minute: i64, second: i64) -> UtcInstant {
    UtcInstant {
        unix_seconds: CivilDate::new(year, month, day)
            .unwrap()
            .days_since_unix_epoch()
            * 86_400
            + hour * 3600
            + minute * 60
            + second,
    }
}

fn near(actual: Event, expected: UtcInstant, tolerance_seconds: i64) {
    let actual = actual.utc().expect("expected an astronomical event");
    let discrepancy = (actual.unix_seconds - expected.unix_seconds).abs();
    assert!(
        discrepancy <= tolerance_seconds,
        "actual {actual:?}, expected {expected:?}, discrepancy {discrepancy}s"
    );
}

#[test]
fn usno_solar_events_across_seasons_and_hemispheres() {
    // USNO rounds its API results to the minute. ±90 s allows for that
    // reporting precision and differences in apparent-horizon assumptions.
    let minneapolis = calculate_prayer_times(input(
        2026,
        9,
        27,
        44.9778,
        -93.2650,
        -300,
        AsrCriterion::Standard,
    ))
    .unwrap();
    near(minneapolis.sunrise, utc(2026, 9, 27, 12, 6, 0), 90);
    near(minneapolis.dhuhr, utc(2026, 9, 27, 18, 4, 0), 90);
    near(minneapolis.sunset, utc(2026, 9, 28, 0, 1, 0), 90);

    let makkah = calculate_prayer_times(input(
        2026,
        3,
        20,
        21.4225,
        39.8262,
        180,
        AsrCriterion::Standard,
    ))
    .unwrap();
    near(makkah.sunrise, utc(2026, 3, 20, 3, 25, 0), 90);
    near(makkah.dhuhr, utc(2026, 3, 20, 9, 28, 0), 90);
    near(makkah.sunset, utc(2026, 3, 20, 15, 32, 0), 90);

    let sydney = calculate_prayer_times(input(
        2026,
        12,
        21,
        -33.8688,
        151.2093,
        660,
        AsrCriterion::Standard,
    ))
    .unwrap();
    near(sydney.sunrise, utc(2026, 12, 20, 18, 41, 0), 90);
    near(sydney.dhuhr, utc(2026, 12, 21, 1, 53, 0), 90);
    near(sydney.sunset, utc(2026, 12, 21, 9, 5, 0), 90);

    let quito_2050 = calculate_prayer_times(input(
        2050,
        9,
        27,
        0.1807,
        -78.4678,
        -300,
        AsrCriterion::Standard,
    ))
    .unwrap();
    near(quito_2050.sunrise, utc(2050, 9, 27, 11, 2, 0), 90);
    near(quito_2050.dhuhr, utc(2050, 9, 27, 17, 5, 0), 90);
    near(quito_2050.sunset, utc(2050, 9, 27, 23, 8, 0), 90);

    let leap_day = calculate_prayer_times(input(
        2028,
        2,
        29,
        21.4225,
        39.8262,
        180,
        AsrCriterion::Standard,
    ))
    .unwrap();
    near(leap_day.sunrise, utc(2028, 2, 29, 3, 42, 0), 90);
    near(leap_day.dhuhr, utc(2028, 2, 29, 9, 33, 0), 90);
    near(leap_day.sunset, utc(2028, 2, 29, 15, 25, 0), 90);

    let sydney_1900 = calculate_prayer_times(input(
        1900,
        12,
        21,
        -33.8688,
        151.2093,
        600,
        AsrCriterion::Standard,
    ))
    .unwrap();
    near(sydney_1900.sunrise, utc(1900, 12, 20, 18, 41, 0), 90);
    near(sydney_1900.dhuhr, utc(1900, 12, 21, 1, 53, 0), 90);
    near(sydney_1900.sunset, utc(1900, 12, 21, 9, 6, 0), 90);

    let london_2100 = calculate_prayer_times(input(
        2100,
        6,
        21,
        51.5072,
        -0.1276,
        0,
        AsrCriterion::Standard,
    ))
    .unwrap();
    near(london_2100.sunrise, utc(2100, 6, 21, 3, 43, 0), 90);
    near(london_2100.dhuhr, utc(2100, 6, 21, 12, 3, 0), 90);
    near(london_2100.sunset, utc(2100, 6, 21, 20, 22, 0), 90);
}

#[test]
fn mwl_angle_profile_matches_independent_praytimes_parameters() {
    let mut minneapolis_input = input(2026, 9, 27, 44.9778, -93.2650, -300, AsrCriterion::Standard);
    minneapolis_input.method = MethodProfile::mwl_angles_18_17();
    let minneapolis = calculate_prayer_times(minneapolis_input).unwrap();
    // PrayTimes v2 Float output, converted from UTC−05:00 to UTC.
    near(minneapolis.fajr, utc(2026, 9, 27, 10, 28, 2), 60);
    near(minneapolis.dhuhr, utc(2026, 9, 27, 18, 3, 57), 60);
    near(minneapolis.asr, utc(2026, 9, 27, 21, 22, 11), 60);
    near(minneapolis.isha, utc(2026, 9, 28, 1, 33, 6), 60);

    let mut makkah_input = input(2026, 3, 20, 21.4225, 39.8262, 180, AsrCriterion::Standard);
    makkah_input.method = MethodProfile::mwl_angles_18_17();
    let makkah = calculate_prayer_times(makkah_input).unwrap();
    near(makkah.fajr, utc(2026, 3, 20, 2, 11, 4), 60);
    near(makkah.isha, utc(2026, 3, 20, 16, 41, 21), 60);
}

#[test]
fn near_midnight_sunrise_is_distinct_from_continuous_daylight() {
    let rising =
        calculate_prayer_times(input(2026, 6, 21, 65.72, 0.0, 0, AsrCriterion::Standard)).unwrap();
    // USNO reports 00:10 sunrise and 23:53 sunset; mean-refraction models
    // are especially sensitive here, so this case allows ±3 minutes.
    near(rising.sunrise, utc(2026, 6, 21, 0, 10, 0), 180);
    near(rising.sunset, utc(2026, 6, 21, 23, 53, 0), 180);

    let continuous =
        calculate_prayer_times(input(2026, 6, 21, 65.74, 0.0, 0, AsrCriterion::Standard)).unwrap();
    assert!(matches!(continuous.sunrise, Event::Unavailable { .. }));
    assert!(matches!(continuous.sunset, Event::Unavailable { .. }));
}

#[test]
fn adhan_reference_for_angle_and_asr_rules() {
    // Adhan JS v4.4.6, custom 15°/15°, no method adjustments, no rounding.
    let makkah = calculate_prayer_times(input(
        2026,
        3,
        20,
        21.4225,
        39.8262,
        180,
        AsrCriterion::Standard,
    ))
    .unwrap();
    near(makkah.fajr, utc(2026, 3, 20, 2, 24, 1), 60);
    near(makkah.asr, utc(2026, 3, 20, 12, 53, 21), 60);
    near(makkah.isha, utc(2026, 3, 20, 16, 32, 43), 60);

    let minneapolis_hanafi = calculate_prayer_times(input(
        2026,
        9,
        27,
        44.9778,
        -93.2650,
        -300,
        AsrCriterion::Hanafi,
    ))
    .unwrap();
    near(minneapolis_hanafi.fajr, utc(2026, 9, 27, 10, 45, 34), 60);
    near(minneapolis_hanafi.asr, utc(2026, 9, 27, 22, 10, 46), 60);
    near(minneapolis_hanafi.isha, utc(2026, 9, 28, 1, 21, 16), 60);

    let standard = calculate_prayer_times(input(
        2026,
        9,
        27,
        44.9778,
        -93.2650,
        -300,
        AsrCriterion::Standard,
    ))
    .unwrap();
    assert!(standard.asr.utc() < minneapolis_hanafi.asr.utc());
}

#[test]
fn polar_day_reports_missing_solar_events() {
    let tromso = calculate_prayer_times(input(
        2026,
        6,
        21,
        69.6492,
        18.9553,
        120,
        AsrCriterion::Standard,
    ))
    .unwrap();
    for event in [
        tromso.fajr,
        tromso.sunrise,
        tromso.sunset,
        tromso.maghrib,
        tromso.isha,
    ] {
        assert!(matches!(event, Event::Unavailable { .. }));
    }
    near(tromso.dhuhr, utc(2026, 6, 21, 10, 46, 0), 90);
}

#[test]
fn date_line_uses_requested_local_solar_cycle() {
    let kiritimati = calculate_prayer_times(input(
        2026,
        9,
        27,
        1.8721,
        -157.4278,
        840,
        AsrCriterion::Standard,
    ))
    .unwrap();
    near(kiritimati.sunrise, utc(2026, 9, 26, 16, 18, 0), 90);
    near(kiritimati.dhuhr, utc(2026, 9, 26, 22, 21, 0), 90);
    near(kiritimati.sunset, utc(2026, 9, 27, 4, 24, 0), 90);
    assert_eq!(
        kiritimati
            .sunrise
            .utc()
            .unwrap()
            .to_local(kiritimati.record.utc_offset)
            .date,
        CivilDate::new(2026, 9, 27).unwrap()
    );
}

#[test]
fn near_solstice_twilight_can_cross_local_midnight() {
    let london = calculate_prayer_times(input(
        2026,
        6,
        21,
        51.5072,
        -0.1276,
        60,
        AsrCriterion::Standard,
    ))
    .unwrap();
    near(london.fajr, utc(2026, 6, 21, 0, 15, 18), 60);
    near(london.isha, utc(2026, 6, 21, 23, 49, 15), 60);
    assert_eq!(
        london
            .isha
            .utc()
            .unwrap()
            .to_local(london.record.utc_offset)
            .date,
        CivilDate::new(2026, 6, 22).unwrap()
    );
}

#[test]
fn polar_night_can_have_twilight_without_sunrise() {
    let tromso = calculate_prayer_times(input(
        2026,
        12,
        21,
        69.6492,
        18.9553,
        60,
        AsrCriterion::Standard,
    ))
    .unwrap();
    assert!(matches!(tromso.sunrise, Event::Unavailable { .. }));
    assert!(matches!(tromso.sunset, Event::Unavailable { .. }));
    assert!(tromso.fajr.utc().is_some());
    assert!(tromso.isha.utc().is_some());
    assert!(matches!(tromso.asr, Event::Unavailable { .. }));
}

#[test]
fn invalid_inputs_do_not_enter_the_engine() {
    assert!(Coordinates::new(f64::NAN, 0.0).is_err());
    assert!(Coordinates::new(0.0, 181.0).is_err());
    assert!(CivilDate::new(2100, 2, 29).is_err());
    assert!(FixedUtcOffset::from_minutes_east(841).is_err());
    let mut invalid_method = MethodProfile::research_15();
    invalid_method.fajr_depression_degrees = f64::INFINITY;
    assert!(invalid_method.validate().is_err());
}

#[test]
fn raw_result_is_preserved_before_display_rounding() {
    let result = calculate_prayer_times(input(
        2026,
        9,
        27,
        44.9778,
        -93.2650,
        -300,
        AsrCriterion::Standard,
    ))
    .unwrap();
    for event in [
        result.fajr,
        result.sunrise,
        result.dhuhr,
        result.asr,
        result.maghrib,
        result.isha,
    ] {
        let rounded = event.utc().unwrap().unix_seconds as f64;
        let raw = event.unrounded_utc_unix_seconds().unwrap();
        assert!((rounded - raw).abs() <= 0.5);
    }
}
