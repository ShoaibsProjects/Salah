use salah_core::{AsrCriterion, CivilDate, Coordinates, MethodProfile};
use salah_engine::{
    CivilTimeDataNotice, assess_civil_time_data, calculate_selected_local_day_schedule,
};
use salah_location::{SelectionOrigin, lookup_timezone_candidates};
use salah_time::{LocalDatePrayerScheduleMatches, LocalDateStatus, LocalizedEvent};

fn date(year: i32, month: u8, day: u8) -> CivilDate {
    CivilDate::new(year, month, day).expect("test date is valid")
}

fn coordinates(latitude: f64, longitude: f64) -> Coordinates {
    Coordinates::new(latitude, longitude).expect("test coordinates are valid")
}

#[test]
fn confirmed_zone_flows_through_to_a_complete_local_schedule() {
    let point = coordinates(44.9778, -93.2650);
    let candidates = lookup_timezone_candidates(point).unwrap();
    let selection = candidates.confirm_suggestion("America/Chicago").unwrap();

    let result = calculate_selected_local_day_schedule(
        &selection,
        date(2026, 9, 27),
        MethodProfile::research_15(),
        AsrCriterion::Hanafi,
    )
    .unwrap();

    assert_eq!(result.zone_selection.zone_id(), "America/Chicago");
    assert_eq!(
        result.zone_selection.origin(),
        SelectionOrigin::UserConfirmedSuggestion
    );
    assert_eq!(
        result.schedule.record.coordinates, point,
        "the schedule uses the coordinates from the explicit zone selection"
    );
    assert_eq!(result.schedule.record.requested_date, date(2026, 9, 27));
    assert_eq!(result.schedule.record.zone_id, "America/Chicago");
    assert_eq!(result.schedule.record.tzdb_version, "2026d");
    assert_eq!(
        result.schedule.record.rule_pack,
        result.schedule.civil_date.rule_pack
    );
    assert_eq!(result.schedule.record.method.id, "research-15");
    assert_eq!(result.schedule.record.asr_criterion, AsrCriterion::Hanafi);

    let same_day = assess_civil_time_data(&result, date(2026, 9, 27));
    assert!(same_day.notices.is_empty());
    assert_eq!(same_day.rule_pack.tzdb_version, "2026d");
    assert_eq!(same_day.rule_pack.release_year, 2026);
    assert_eq!(same_day.rule_pack.sha256.len(), 64);
    assert_eq!(same_day.boundary_data_version, "2026d");
    let future_day = assess_civil_time_data(&result, date(2026, 9, 26));
    assert_eq!(
        future_day.notices,
        vec![CivilTimeDataNotice::RequestedDateIsFuture]
    );
    let old_pack = assess_civil_time_data(&result, date(2027, 1, 1));
    assert_eq!(
        old_pack.notices,
        vec![CivilTimeDataNotice::PackPredatesObservationYear]
    );

    let LocalDatePrayerScheduleMatches::One(candidate) = result.schedule.matches else {
        panic!("expected exactly one solar cycle for this ordinary local date");
    };
    for event in [
        candidate.events.fajr,
        candidate.events.sunrise,
        candidate.events.dhuhr,
        candidate.events.asr,
        candidate.events.sunset,
        candidate.events.maghrib,
        candidate.events.isha,
    ] {
        if let LocalizedEvent::Occurs { local, .. } = event {
            assert_eq!(local.zone_id, "America/Chicago");
            assert_eq!(local.tzdb_version, "2026d");
            assert_eq!(local.rule_pack, result.schedule.record.rule_pack);
        }
    }
}

#[test]
fn manual_zone_choice_is_respected_and_retained_as_manual() {
    let point = coordinates(44.9778, -93.2650);
    let candidates = lookup_timezone_candidates(point).unwrap();
    let selection = candidates.manual_override("Asia/Kathmandu").unwrap();

    let result = calculate_selected_local_day_schedule(
        &selection,
        date(2026, 9, 27),
        MethodProfile::mwl_angles_18_17(),
        AsrCriterion::Standard,
    )
    .unwrap();

    assert_eq!(result.zone_selection.zone_id(), "Asia/Kathmandu");
    assert_eq!(
        result.zone_selection.origin(),
        SelectionOrigin::ManualOverride
    );
    assert_eq!(result.schedule.record.zone_id, "Asia/Kathmandu");
    assert_eq!(result.schedule.record.method.id, "mwl-angles-18-17");

    let LocalDatePrayerScheduleMatches::One(candidate) = result.schedule.matches else {
        panic!("expected one solar cycle under the explicitly selected zone");
    };
    let LocalizedEvent::Occurs { local, .. } = candidate.events.dhuhr else {
        panic!("Dhuhr should be available at this test location and date");
    };
    assert_eq!(local.zone_id, "Asia/Kathmandu");
    assert_eq!(local.offset_seconds_east, 20_700);
}

#[test]
fn skipped_civil_date_stays_distinct_from_a_missing_solar_cycle() {
    let point = coordinates(-13.8333, -171.75);
    let candidates = lookup_timezone_candidates(point).unwrap();
    let selection = candidates.manual_override("Pacific/Apia").unwrap();

    let result = calculate_selected_local_day_schedule(
        &selection,
        date(2011, 12, 30),
        MethodProfile::research_15(),
        AsrCriterion::Standard,
    )
    .unwrap();

    assert!(matches!(
        result.schedule.civil_date.status,
        LocalDateStatus::Skipped
    ));
    assert!(matches!(
        result.schedule.matches,
        LocalDatePrayerScheduleMatches::Zero
    ));
}

#[test]
fn selected_schedule_records_the_manual_override_provenance() {
    let candidates = lookup_timezone_candidates(coordinates(0.0, 0.0)).unwrap();
    let selection = candidates.manual_override("America/Chicago").unwrap();
    let result = calculate_selected_local_day_schedule(
        &selection,
        date(2026, 9, 27),
        MethodProfile::research_15(),
        AsrCriterion::Standard,
    )
    .unwrap();

    assert_eq!(
        result.zone_selection.origin(),
        SelectionOrigin::ManualOverride
    );
    assert_eq!(
        result.zone_selection.candidates().zone_ids(),
        &["Etc/GMT".to_owned()]
    );
    assert_eq!(result.schedule.record.zone_id, "America/Chicago");
}

#[test]
fn supported_method_and_asr_choices_are_always_explicit_inputs() {
    let selection = lookup_timezone_candidates(coordinates(39.9289, 116.3883))
        .unwrap()
        .confirm_suggestion("Asia/Shanghai")
        .unwrap();

    let result = calculate_selected_local_day_schedule(
        &selection,
        date(2026, 9, 27),
        MethodProfile::mwl_angles_18_17(),
        AsrCriterion::Hanafi,
    )
    .unwrap();

    assert_eq!(result.schedule.record.method.id, "mwl-angles-18-17");
    assert_eq!(result.schedule.record.asr_criterion, AsrCriterion::Hanafi);
}

#[test]
fn zero_solar_cycle_on_an_existing_date_stays_explicit() {
    let point = coordinates(44.9778, 97.5);
    let selection = lookup_timezone_candidates(point)
        .unwrap()
        .manual_override("America/Chicago")
        .unwrap();

    let result = calculate_selected_local_day_schedule(
        &selection,
        date(2026, 3, 8),
        MethodProfile::research_15(),
        AsrCriterion::Standard,
    )
    .unwrap();

    assert!(matches!(
        result.schedule.civil_date.status,
        LocalDateStatus::Exists(_)
    ));
    assert!(matches!(
        result.schedule.matches,
        LocalDatePrayerScheduleMatches::Zero
    ));
}
