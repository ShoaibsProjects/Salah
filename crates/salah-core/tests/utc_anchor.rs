//! F2-TZ1 UTC-anchor regression coverage.
//!
//! The legacy [`CalculationInput`] selects a solar cycle from a civil date
//! plus a minute-precision fixed offset by way of a local-noon UTC estimate.
//! The additive [`UtcAnchorInput`] selects the same transit directly from an
//! explicit UTC anchor. These tests prove the two paths agree exactly when
//! the anchor equals the legacy noon estimate, expose the transit-selection
//! boundary, and prove unavailable statuses are preserved.

use salah_core::{
    AsrCriterion, CalculationError, CalculationInput, CivilDate, Coordinates, Event,
    FixedUtcOffset, MethodProfile, UtcAnchorInput, UtcInstant, calculate_prayer_times,
    calculate_utc_anchor_times,
};

fn legacy_input(
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

fn legacy_noon_anchor(input: CalculationInput) -> UtcInstant {
    UtcInstant {
        unix_seconds: input.local_date.days_since_unix_epoch() * 86_400
            - i64::from(input.utc_offset.minutes_east()) * 60
            + 43_200,
    }
}

fn anchor_input(input: CalculationInput, anchor: UtcInstant) -> UtcAnchorInput {
    UtcAnchorInput {
        coordinates: input.coordinates,
        anchor,
        method: input.method,
        asr_criterion: input.asr_criterion,
    }
}

fn assert_event_equal(old: Event, new: Event, label: &str) {
    match (old, new) {
        (
            Event::Occurs {
                utc: old_utc,
                unrounded_utc_unix_seconds: old_raw,
                rule: old_rule,
            },
            Event::Occurs {
                utc: new_utc,
                unrounded_utc_unix_seconds: new_raw,
                rule: new_rule,
            },
        ) => {
            assert_eq!(old_utc, new_utc, "{label}: rounded UTC differs");
            assert!(
                old_raw == new_raw,
                "{label}: unrounded seconds differ: {old_raw} vs {new_raw}"
            );
            assert_eq!(old_rule, new_rule, "{label}: rule differs");
        }
        (Event::Unavailable { reason: old_r }, Event::Unavailable { reason: new_r }) => {
            assert_eq!(old_r, new_r, "{label}: unavailable reason differs");
        }
        _ => panic!("{label}: availability differs: old={old:?} new={new:?}"),
    }
}

fn assert_cycle_equal(input: CalculationInput) {
    let anchor = legacy_noon_anchor(input);
    let old = calculate_prayer_times(input).unwrap();
    let new = calculate_utc_anchor_times(anchor_input(input, anchor)).unwrap();

    for (label, old_event, new_event) in [
        ("fajr", old.fajr, new.fajr),
        ("sunrise", old.sunrise, new.sunrise),
        ("dhuhr", old.dhuhr, new.dhuhr),
        ("asr", old.asr, new.asr),
        ("sunset", old.sunset, new.sunset),
        ("maghrib", old.maghrib, new.maghrib),
        ("isha", old.isha, new.isha),
    ] {
        assert_event_equal(old_event, new_event, label);
    }

    // UTC-only provenance: same astronomy/model/method, explicit anchor and
    // selection policy, no local date or fixed offset on the new record.
    assert_eq!(new.record.anchor, anchor);
    assert_eq!(
        new.record.selection_policy_id,
        salah_core::UTC_ANCHOR_SELECTION_POLICY_ID
    );
    assert_eq!(
        new.record.selection_policy_revision,
        salah_core::UTC_ANCHOR_SELECTION_POLICY_REVISION
    );
    assert_eq!(new.record.astronomy_model, old.record.astronomy_model);
    assert_eq!(new.record.astronomy_model, "NOAA-MEEUS-SOLAR-2");
    assert_eq!(new.record.method, old.record.method);
    assert_eq!(new.record.asr_criterion, old.record.asr_criterion);
    assert_eq!(new.record.engine_version, old.record.engine_version);
    assert_eq!(new.record.high_latitude_rule, "none");
    assert_eq!(new.record.assumed_elevation_meters, 0.0);
    assert_eq!(new.selected_transit, new.record.selected_transit);
    assert_eq!(
        new.unrounded_selected_transit_unix_seconds,
        new.record.unrounded_selected_transit_unix_seconds
    );
    // With the built-in zero-adjustment profiles the rounded transit is Dhuhr.
    if input.method.dhuhr_adjustment_seconds == 0 {
        assert_eq!(new.selected_transit, new.dhuhr.utc().unwrap());
        assert_eq!(
            new.unrounded_selected_transit_unix_seconds,
            new.dhuhr.unrounded_utc_unix_seconds().unwrap()
        );
    }
}

#[test]
fn legacy_noon_anchor_reproduces_reference_cycles_exactly() {
    // Existing reference locations: Minneapolis, Makkah, Sydney, Quito,
    // leap day, and the MWL angle profile is covered separately below.
    assert_cycle_equal(legacy_input(
        2026,
        9,
        27,
        44.9778,
        -93.2650,
        -300,
        AsrCriterion::Standard,
    ));
    assert_cycle_equal(legacy_input(
        2026,
        3,
        20,
        21.4225,
        39.8262,
        180,
        AsrCriterion::Standard,
    ));
    assert_cycle_equal(legacy_input(
        2026,
        12,
        21,
        -33.8688,
        151.2093,
        660,
        AsrCriterion::Standard,
    ));
    assert_cycle_equal(legacy_input(
        2050,
        9,
        27,
        0.1807,
        -78.4678,
        -300,
        AsrCriterion::Standard,
    ));
    assert_cycle_equal(legacy_input(
        2028,
        2,
        29,
        21.4225,
        39.8262,
        180,
        AsrCriterion::Standard,
    ));
    assert_cycle_equal(legacy_input(
        2026,
        9,
        27,
        44.9778,
        -93.2650,
        -300,
        AsrCriterion::Hanafi,
    ));
}

#[test]
fn legacy_noon_anchor_covers_pre_epoch_and_date_line_extremes() {
    // Pre-epoch case: Sydney 1900-12-21 at +10:00.
    assert_cycle_equal(legacy_input(
        1900,
        12,
        21,
        -33.8688,
        151.2093,
        600,
        AsrCriterion::Standard,
    ));
    // Date-line/offset extreme: Kiritimati 2026-09-27 at +14:00; its sunrise
    // UTC instant falls on the previous UTC date.
    assert_cycle_equal(legacy_input(
        2026,
        9,
        27,
        1.8721,
        -157.4278,
        840,
        AsrCriterion::Standard,
    ));
    // Late-boundary case: London 2100-06-21 at +00:00.
    assert_cycle_equal(legacy_input(
        2100,
        6,
        21,
        51.5072,
        -0.1276,
        0,
        AsrCriterion::Standard,
    ));
}

#[test]
fn anchor_path_preserves_method_adjustments_without_drift() {
    let mut input = legacy_input(2026, 9, 27, 44.9778, -93.2650, -300, AsrCriterion::Standard);
    input.method = MethodProfile {
        dhuhr_adjustment_seconds: 120,
        maghrib_adjustment_seconds: 60,
        ..MethodProfile::research_15()
    };
    assert_cycle_equal(input);

    let anchor = legacy_noon_anchor(input);
    let new = calculate_utc_anchor_times(anchor_input(input, anchor)).unwrap();
    let dhuhr_utc = new.dhuhr.utc().unwrap().unix_seconds;
    assert_eq!(
        dhuhr_utc,
        new.selected_transit.unix_seconds + 120,
        "Dhuhr must be the rounded transit plus its adjustment"
    );
}

#[test]
fn mwl_profile_anchor_path_matches_legacy_path() {
    let mut input = legacy_input(2026, 9, 27, 44.9778, -93.2650, -300, AsrCriterion::Standard);
    input.method = MethodProfile::mwl_angles_18_17();
    assert_cycle_equal(input);
}

#[test]
fn unavailable_events_are_preserved_through_the_anchor_path() {
    // Polar day: Fajr/sunrise/sunset/Maghrib/Isha unavailable, Dhuhr occurs.
    let polar_day = legacy_input(2026, 6, 21, 69.6492, 18.9553, 120, AsrCriterion::Standard);
    assert_cycle_equal(polar_day);
    let anchor_day =
        calculate_utc_anchor_times(anchor_input(polar_day, legacy_noon_anchor(polar_day))).unwrap();
    for event in [
        anchor_day.fajr,
        anchor_day.sunrise,
        anchor_day.sunset,
        anchor_day.maghrib,
        anchor_day.isha,
    ] {
        assert!(matches!(event, Event::Unavailable { .. }));
    }
    assert!(anchor_day.dhuhr.utc().is_some());

    // Polar night: sunrise/sunset/Asr unavailable, twilight Fajr/Isha occur.
    let polar_night = legacy_input(2026, 12, 21, 69.6492, 18.9553, 60, AsrCriterion::Standard);
    assert_cycle_equal(polar_night);
    let anchor_night =
        calculate_utc_anchor_times(anchor_input(polar_night, legacy_noon_anchor(polar_night)))
            .unwrap();
    assert!(matches!(anchor_night.sunrise, Event::Unavailable { .. }));
    assert!(matches!(anchor_night.sunset, Event::Unavailable { .. }));
    assert!(matches!(anchor_night.asr, Event::Unavailable { .. }));
    assert!(anchor_night.fajr.utc().is_some());
    assert!(anchor_night.isha.utc().is_some());
}

#[test]
fn same_cycle_anchors_select_one_transit_and_boundary_selects_the_next() {
    let coordinates = Coordinates::new(44.9778, -93.2650).unwrap();
    let method = MethodProfile::research_15();
    let solve = |anchor_unix: i64| {
        calculate_utc_anchor_times(UtcAnchorInput {
            coordinates,
            anchor: UtcInstant {
                unix_seconds: anchor_unix,
            },
            method,
            asr_criterion: AsrCriterion::Standard,
        })
        .unwrap()
    };

    let base = legacy_input(2026, 9, 27, 44.9778, -93.2650, -300, AsrCriterion::Standard);
    let base_anchor = legacy_noon_anchor(base).unix_seconds;
    let transit0 = solve(base_anchor).selected_transit.unix_seconds;
    // Anchors ±6 h around the legacy noon stay in the same solar cycle.
    for anchor in [transit0 - 6 * 3600, transit0, transit0 + 6 * 3600] {
        assert_eq!(
            solve(anchor).selected_transit.unix_seconds,
            transit0,
            "anchor {anchor} should stay on transit {transit0}"
        );
    }

    // The following cycle's transit is about one solar day later.
    let transit1 = solve(transit0 + 86_400).selected_transit.unix_seconds;
    assert!(transit1 > transit0, "expected a later transit");
    let gap = transit1 - transit0;
    assert!(
        (82_800..=90_000).contains(&gap),
        "adjacent transits should be about a day apart, got {gap}s"
    );

    // Scan for the selection boundary between the two transits, then refine
    // to adjacent whole seconds on either side.
    let midpoint = (transit0 + transit1) / 2;
    let mut coarse: Option<i64> = None;
    let mut previous = solve(midpoint - 7200).selected_transit.unix_seconds;
    let mut anchor = midpoint - 7200 + 60;
    while anchor <= midpoint + 7200 {
        let current = solve(anchor).selected_transit.unix_seconds;
        assert!(
            current == transit0 || current == transit1,
            "unexpected third transit {current} near the boundary"
        );
        if current != previous {
            coarse = Some(anchor);
            break;
        }
        previous = current;
        anchor += 60;
    }
    let coarse = coarse.expect("expected a transit-selection boundary nearby");
    let mut boundary: Option<(i64, i64)> = None;
    for candidate in (coarse - 60)..=(coarse + 60) {
        let low = solve(candidate).selected_transit.unix_seconds;
        let high = solve(candidate + 1).selected_transit.unix_seconds;
        if low != high {
            assert_eq!(low, transit0, "low side must select the first transit");
            assert_eq!(high, transit1, "high side must select the next transit");
            boundary = Some((candidate, candidate + 1));
            break;
        }
    }
    let (low_anchor, high_anchor) =
        boundary.expect("expected adjacent anchors selecting different transits");
    println!(
        "Minneapolis boundary: low anchor {low_anchor} selects transit {transit0}; \
         high anchor {high_anchor} selects transit {transit1} (gap {gap}s)"
    );
}

#[test]
fn anchor_range_is_enforced_with_a_typed_error() {
    let coordinates = Coordinates::new(44.9778, -93.2650).unwrap();
    let valid = |anchor_unix: i64| {
        calculate_utc_anchor_times(UtcAnchorInput {
            coordinates,
            anchor: UtcInstant {
                unix_seconds: anchor_unix,
            },
            method: MethodProfile::research_15(),
            asr_criterion: AsrCriterion::Standard,
        })
    };
    assert!(valid(salah_core::UTC_ANCHOR_MIN_UNIX_SECONDS).is_ok());
    assert!(valid(salah_core::UTC_ANCHOR_MAX_UNIX_SECONDS).is_ok());
    assert_eq!(
        valid(salah_core::UTC_ANCHOR_MIN_UNIX_SECONDS - 1).unwrap_err(),
        CalculationError::InvalidUtcAnchor
    );
    assert_eq!(
        valid(salah_core::UTC_ANCHOR_MAX_UNIX_SECONDS + 1).unwrap_err(),
        CalculationError::InvalidUtcAnchor
    );
}
