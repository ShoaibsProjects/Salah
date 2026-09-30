use salah_core::{AsrCriterion, CivilDate, Coordinates, Event, MethodProfile, UtcInstant};
use salah_time::{
    LocalDatePrayerScheduleMatches, LocalDateStatus, LocalizedEvent, TimeError,
    calculate_local_date_prayer_schedule, fixture_tzif_bytes,
};

fn date(year: i32, month: u8, day: u8) -> CivilDate {
    CivilDate::new(year, month, day).unwrap()
}

fn coordinates(latitude: f64, longitude: f64) -> Coordinates {
    Coordinates::new(latitude, longitude).unwrap()
}

fn schedule(
    year: i32,
    month: u8,
    day: u8,
    latitude: f64,
    longitude: f64,
    zone: &str,
) -> Result<salah_time::LocalDatePrayerSchedule, TimeError> {
    calculate_local_date_prayer_schedule(
        date(year, month, day),
        coordinates(latitude, longitude),
        zone,
        fixture_tzif_bytes(zone)?,
        MethodProfile::research_15(),
        AsrCriterion::Standard,
    )
}

fn assert_localization_matches_core(source: Event, localized: &LocalizedEvent, zone: &str) {
    match (source, localized) {
        (
            Event::Occurs {
                utc,
                unrounded_utc_unix_seconds,
                rule,
            },
            LocalizedEvent::Occurs {
                local,
                unrounded_utc_unix_seconds: localized_raw,
                rule: localized_rule,
            },
        ) => {
            assert_eq!(local.utc, utc);
            assert_eq!(*localized_raw, unrounded_utc_unix_seconds);
            assert_eq!(*localized_rule, rule);
            assert_eq!(local.zone_id, zone);
            assert_eq!(local.tzdb_version, "2026d");
        }
        (
            Event::Unavailable { reason },
            LocalizedEvent::Unavailable {
                reason: localized_reason,
            },
        ) => assert_eq!(*localized_reason, reason),
        (source, localized) => panic!("event status changed: {source:?} -> {localized:?}"),
    }
}

#[test]
fn ordinary_chicago_schedule_localizes_all_seven_events_without_changing_them() {
    let result = schedule(2026, 9, 27, 44.9778, -93.2650, "America/Chicago").unwrap();
    assert_eq!(result.record.requested_date, date(2026, 9, 27));
    assert_eq!(result.record.zone_id, "America/Chicago");
    assert_eq!(result.record.tzdb_version, "2026d");
    assert!(matches!(
        result.civil_date.status,
        LocalDateStatus::Exists(_)
    ));
    assert_eq!(result.policy_id, "local-date-prayer-schedule");
    assert_eq!(result.policy_revision, "0.1");
    let LocalDatePrayerScheduleMatches::One(candidate) = result.matches else {
        panic!("expected one schedule candidate");
    };
    assert_eq!(candidate.local_transit.local.date, date(2026, 9, 27));
    let checks = [
        (candidate.cycle.fajr, &candidate.events.fajr),
        (candidate.cycle.sunrise, &candidate.events.sunrise),
        (candidate.cycle.dhuhr, &candidate.events.dhuhr),
        (candidate.cycle.asr, &candidate.events.asr),
        (candidate.cycle.sunset, &candidate.events.sunset),
        (candidate.cycle.maghrib, &candidate.events.maghrib),
        (candidate.cycle.isha, &candidate.events.isha),
    ];
    for (source, localized) in checks {
        assert_localization_matches_core(source, localized, "America/Chicago");
    }
}

#[test]
fn london_isha_keeps_its_next_day_date_and_second_precision() {
    // This UTC instant is the existing London 2026-06-21 research-15 reference
    // vector in salah-core/tests/reference_cases.rs. The local zone is UTC+1.
    let result = schedule(2026, 6, 21, 51.5072, -0.1276, "Europe/London").unwrap();
    let LocalDatePrayerScheduleMatches::One(candidate) = result.matches else {
        panic!("expected one London schedule candidate");
    };
    let Event::Occurs { utc, .. } = candidate.cycle.isha else {
        panic!("reference Isha should occur");
    };
    let LocalizedEvent::Occurs { local, .. } = candidate.events.isha else {
        panic!("reference Isha should be localized");
    };
    assert_eq!(local.utc, utc);
    assert_eq!(
        utc,
        UtcInstant {
            unix_seconds: 1_782_085_755
        }
    );
    assert_eq!(local.local.date, date(2026, 6, 22));
    assert_eq!(
        (local.local.hour, local.local.minute, local.local.second),
        (0, 49, 15)
    );
    assert_eq!(local.offset_seconds_east, 3_600);
}

#[test]
fn zero_and_multiple_transit_outcomes_remain_explicit_in_schedule_api() {
    let zero = schedule(2026, 3, 8, 44.9778, 97.5, "America/Chicago").unwrap();
    assert!(matches!(zero.matches, LocalDatePrayerScheduleMatches::Zero));
    assert!(matches!(zero.civil_date.status, LocalDateStatus::Exists(_)));

    let multiple = schedule(2026, 11, 1, 44.9778, 97.5, "America/Chicago").unwrap();
    let LocalDatePrayerScheduleMatches::Multiple(candidates) = multiple.matches else {
        panic!("expected multiple schedule cycles for the long date");
    };
    assert_eq!(candidates.len(), 2);
    assert!(matches!(
        multiple.civil_date.status,
        LocalDateStatus::Exists(_)
    ));
    assert!(candidates[0].cycle.selected_transit < candidates[1].cycle.selected_transit);
    for candidate in candidates {
        assert_eq!(candidate.local_transit.local.date, date(2026, 11, 1));
        let checks = [
            (candidate.cycle.fajr, &candidate.events.fajr),
            (candidate.cycle.sunrise, &candidate.events.sunrise),
            (candidate.cycle.dhuhr, &candidate.events.dhuhr),
            (candidate.cycle.asr, &candidate.events.asr),
            (candidate.cycle.sunset, &candidate.events.sunset),
            (candidate.cycle.maghrib, &candidate.events.maghrib),
            (candidate.cycle.isha, &candidate.events.isha),
        ];
        for (source, localized) in checks {
            assert_localization_matches_core(source, localized, "America/Chicago");
        }
    }
}

#[test]
fn polar_unavailable_events_stay_unavailable_without_fabricated_local_times() {
    // At the prime meridian the deliberately selected UTC zone is explicit;
    // this test checks the solar no-crossing status, not coordinate mapping.
    let result = schedule(2026, 12, 21, 69.6492, 0.0, "Etc/UTC").unwrap();
    let LocalDatePrayerScheduleMatches::One(candidate) = result.matches else {
        panic!("expected one Tromso solar cycle");
    };
    assert!(matches!(candidate.cycle.sunrise, Event::Unavailable { .. }));
    assert!(matches!(
        candidate.events.sunrise,
        LocalizedEvent::Unavailable { .. }
    ));
    assert!(matches!(candidate.cycle.sunset, Event::Unavailable { .. }));
    assert!(matches!(
        candidate.events.sunset,
        LocalizedEvent::Unavailable { .. }
    ));
    assert_localization_matches_core(candidate.cycle.fajr, &candidate.events.fajr, "Etc/UTC");
    assert_localization_matches_core(candidate.cycle.isha, &candidate.events.isha, "Etc/UTC");
}
