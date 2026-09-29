use salah_core::{
    AsrCriterion, CivilDate, Coordinates, MethodProfile, UtcAnchorInput, UtcInstant,
    calculate_utc_anchor_times,
};
use salah_time::{
    LOCAL_DATE_TRANSIT_POLICY_ID, LocalDateTransitMatches, TimeError, convert_utc_to_local,
    fixture_tzif_bytes, select_local_date_transits,
};

fn date(year: i32, month: u8, day: u8) -> CivilDate {
    CivilDate::new(year, month, day).unwrap()
}

fn coordinates(latitude: f64, longitude: f64) -> Coordinates {
    Coordinates::new(latitude, longitude).unwrap()
}

fn select(
    year: i32,
    month: u8,
    day: u8,
    latitude: f64,
    longitude: f64,
    zone: &str,
) -> salah_time::LocalDateTransitSelection {
    select_local_date_transits(
        date(year, month, day),
        coordinates(latitude, longitude),
        zone,
        fixture_tzif_bytes(zone).unwrap(),
        MethodProfile::research_15(),
        AsrCriterion::Standard,
    )
    .unwrap()
}

#[test]
fn ordinary_chicago_date_has_one_matching_cycle() {
    let result = select(2026, 9, 27, 44.9778, -93.2650, "America/Chicago");
    assert_eq!(result.record.policy_id, LOCAL_DATE_TRANSIT_POLICY_ID);
    assert_eq!(result.record.policy_revision, "0.1");
    assert_eq!(result.record.tzdb_version, "2026d");
    assert_eq!(result.record.astronomy_model, "NOAA-MEEUS-SOLAR-2");
    assert_eq!(
        result.record.engine_version,
        salah_core::CORE_ENGINE_VERSION
    );
    let LocalDateTransitMatches::One(candidate) = result.matches else {
        panic!("expected exactly one cycle, got {:?}", result.matches);
    };
    assert_eq!(candidate.local_transit.local.date, date(2026, 9, 27));
    assert_eq!(candidate.local_transit.zone_id, "America/Chicago");
    assert_eq!(
        candidate.cycle.record.method.id,
        MethodProfile::research_15().id
    );
}

#[test]
fn short_chicago_spring_date_can_have_no_transit_without_being_skipped() {
    // At this longitude, the adjacent solar transits fall outside the 23-hour
    // Chicago civil date. The local date still exists.
    let result = select(2026, 3, 8, 44.9778, 97.5, "America/Chicago");
    assert!(matches!(result.matches, LocalDateTransitMatches::Zero));
    let local_midnight = convert_utc_to_local(
        UtcInstant {
            unix_seconds: 1_772_949_600,
        },
        "America/Chicago",
        fixture_tzif_bytes("America/Chicago").unwrap(),
    )
    .unwrap();
    assert_eq!(local_midnight.local.date, date(2026, 3, 8));
    assert_eq!(local_midnight.local.hour, 0);
}

#[test]
fn long_chicago_fall_date_can_have_two_matching_transits() {
    // This intentionally mismatched longitude places consecutive transits
    // near the beginning and end of Chicago's 25-hour fall-back date.
    let result = select(2026, 11, 1, 44.9778, 97.5, "America/Chicago");
    let LocalDateTransitMatches::Multiple(candidates) = result.matches else {
        panic!("expected multiple cycles, got {:?}", result.matches);
    };
    assert_eq!(candidates.len(), 2);
    assert!(candidates[0].cycle.selected_transit < candidates[1].cycle.selected_transit);
    for candidate in candidates {
        assert_eq!(candidate.local_transit.local.date, date(2026, 11, 1));
        assert_eq!(candidate.local_transit.zone_id, "America/Chicago");
        assert_eq!(candidate.local_transit.tzdb_version, "2026d");
    }
}

#[test]
fn london_dst_date_still_selects_its_local_transit() {
    let result = select(2026, 3, 29, 51.5072, -0.1276, "Europe/London");
    let LocalDateTransitMatches::One(candidate) = result.matches else {
        panic!("expected one London cycle");
    };
    assert_eq!(candidate.local_transit.local.date, date(2026, 3, 29));
    assert_eq!(candidate.local_transit.zone_id, "Europe/London");
    assert_eq!(candidate.local_transit.tzdb_version, "2026d");
}

#[test]
fn selection_handles_anchors_on_both_sides_of_a_transit_boundary() {
    let coordinates = coordinates(44.9778, -93.2650);
    let zone = "America/Chicago";
    let bytes = fixture_tzif_bytes(zone).unwrap();
    let before = calculate_utc_anchor_times(UtcAnchorInput {
        coordinates,
        anchor: UtcInstant {
            unix_seconds: 1_790_575_428,
        },
        method: MethodProfile::research_15(),
        asr_criterion: AsrCriterion::Standard,
    })
    .unwrap();
    let after = calculate_utc_anchor_times(UtcAnchorInput {
        coordinates,
        anchor: UtcInstant {
            unix_seconds: 1_790_575_429,
        },
        method: MethodProfile::research_15(),
        asr_criterion: AsrCriterion::Standard,
    })
    .unwrap();
    assert_ne!(before.selected_transit, after.selected_transit);
    assert_eq!(
        convert_utc_to_local(before.selected_transit, zone, bytes)
            .unwrap()
            .local
            .date,
        date(2026, 9, 27)
    );
    assert_eq!(
        convert_utc_to_local(after.selected_transit, zone, bytes)
            .unwrap()
            .local
            .date,
        date(2026, 9, 28)
    );
    let result = select(2026, 9, 27, 44.9778, -93.2650, zone);
    let LocalDateTransitMatches::One(candidate) = result.matches else {
        panic!("expected exactly the Sep 27 cycle");
    };
    assert_eq!(candidate.cycle.selected_transit, before.selected_transit);
}

#[test]
fn apia_skipped_date_has_no_matching_transit() {
    let result = select(2011, 12, 30, -13.8333, -171.75, "Pacific/Apia");
    assert!(matches!(result.matches, LocalDateTransitMatches::Zero));
}

#[test]
fn kiritimati_and_kathmandu_resolve_transits_with_their_zone_rules() {
    let kiritimati = select(2026, 9, 27, 1.8721, -157.4278, "Pacific/Kiritimati");
    let LocalDateTransitMatches::One(kiritimati) = kiritimati.matches else {
        panic!("expected one Kiritimati cycle");
    };
    assert_eq!(kiritimati.local_transit.local.date, date(2026, 9, 27));
    assert_eq!(kiritimati.local_transit.offset_seconds_east, 50_400);

    let kathmandu = select(2026, 1, 1, 27.7172, 85.3240, "Asia/Kathmandu");
    let LocalDateTransitMatches::One(kathmandu) = kathmandu.matches else {
        panic!("expected one Kathmandu cycle");
    };
    assert_eq!(kathmandu.local_transit.local.date, date(2026, 1, 1));
    assert_eq!(kathmandu.local_transit.offset_seconds_east, 20_700);
}

#[test]
fn endpoint_dates_are_covered_by_the_clipped_anchor_window() {
    for (year, month, day) in [(1900, 1, 1), (2100, 12, 31)] {
        let result = select(year, month, day, 0.0, 0.0, "Etc/UTC");
        let LocalDateTransitMatches::One(candidate) = result.matches else {
            panic!("expected one cycle at endpoint {year}-{month}-{day}");
        };
        assert_eq!(candidate.local_transit.local.date, date(year, month, day));
    }
}

#[test]
fn invalid_zone_bytes_cannot_become_a_zero_match() {
    let date = date(2011, 12, 30);
    let err = select_local_date_transits(
        date,
        coordinates(-13.8333, -171.75),
        "Pacific/Apia",
        b"not TZif",
        MethodProfile::research_15(),
        AsrCriterion::Standard,
    )
    .unwrap_err();
    assert!(matches!(err, TimeError::MalformedTzif(_)));

    let unknown_zone = select_local_date_transits(
        date,
        coordinates(-13.8333, -171.75),
        "Mars/Olympus",
        b"not TZif",
        MethodProfile::research_15(),
        AsrCriterion::Standard,
    )
    .unwrap_err();
    assert!(matches!(unknown_zone, TimeError::InvalidZoneId(_)));

    let wrong_zone_bytes = select_local_date_transits(
        date,
        coordinates(-13.8333, -171.75),
        "Pacific/Apia",
        fixture_tzif_bytes("Etc/UTC").unwrap(),
        MethodProfile::research_15(),
        AsrCriterion::Standard,
    )
    .unwrap_err();
    assert!(matches!(wrong_zone_bytes, TimeError::UnpinnedTzif(_)));
}
