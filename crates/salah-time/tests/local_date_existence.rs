use salah_core::CivilDate;
use salah_time::{
    LOCAL_DATE_EXISTENCE_POLICY_ID, LocalDateStatus, classify_local_date, fixture_tzif_bytes,
};

fn date(year: i32, month: u8, day: u8) -> CivilDate {
    CivilDate::new(year, month, day).unwrap()
}

fn classify(year: i32, month: u8, day: u8, zone: &str) -> salah_time::LocalDateExistence {
    classify_local_date(
        date(year, month, day),
        zone,
        fixture_tzif_bytes(zone).unwrap(),
    )
    .unwrap()
}

fn total_seconds(intervals: &[salah_time::LocalDateUtcInterval]) -> i64 {
    intervals
        .iter()
        .map(|interval| interval.end.unix_seconds - interval.start.unix_seconds + 1)
        .sum()
}

#[test]
fn chicago_dst_dates_have_their_actual_23_and_25_hour_lengths() {
    let spring = classify(2026, 3, 8, "America/Chicago");
    assert_eq!(spring.policy_id, LOCAL_DATE_EXISTENCE_POLICY_ID);
    assert_eq!(spring.policy_revision, "0.1");
    assert_eq!(spring.tzdb_version, "2026d");
    let LocalDateStatus::Exists(spring_intervals) = spring.status else {
        panic!("Chicago spring-forward date exists");
    };
    assert_eq!(total_seconds(&spring_intervals), 23 * 60 * 60);
    assert_eq!(spring_intervals.len(), 2);
    assert_eq!(spring_intervals[0].offset_seconds_east, -6 * 60 * 60);
    assert_eq!(spring_intervals[1].offset_seconds_east, -5 * 60 * 60);

    let fall = classify(2026, 11, 1, "America/Chicago");
    let LocalDateStatus::Exists(fall_intervals) = fall.status else {
        panic!("Chicago fall-back date exists");
    };
    assert_eq!(total_seconds(&fall_intervals), 25 * 60 * 60);
    assert_eq!(fall_intervals.len(), 2);
    assert_eq!(fall_intervals[0].offset_seconds_east, -5 * 60 * 60);
    assert_eq!(fall_intervals[1].offset_seconds_east, -6 * 60 * 60);
}

#[test]
fn apia_date_line_jump_is_classified_as_a_skipped_civil_date() {
    let apia = classify(2011, 12, 30, "Pacific/Apia");
    assert_eq!(apia.requested_date, date(2011, 12, 30));
    assert_eq!(apia.zone_id, "Pacific/Apia");
    assert!(matches!(apia.status, LocalDateStatus::Skipped));
}

#[test]
fn ordinary_and_fractional_offset_zones_have_existing_dates() {
    for (year, month, day, zone) in [
        (2026, 9, 27, "America/Chicago"),
        (2026, 1, 1, "Asia/Kathmandu"),
        (2026, 9, 27, "Pacific/Kiritimati"),
    ] {
        let result = classify(year, month, day, zone);
        let LocalDateStatus::Exists(intervals) = result.status else {
            panic!("expected {zone} date to exist");
        };
        assert!(!intervals.is_empty());
        assert_eq!(result.tzdb_version, "2026d");
    }
}

#[test]
fn chicago_posix_footer_transitions_apply_after_the_explicit_tzif_table() {
    // The fixture's footer supplies recurring U.S. transitions in this year.
    let result = classify(2099, 3, 8, "America/Chicago");
    let LocalDateStatus::Exists(intervals) = result.status else {
        panic!("2099 spring-forward date exists");
    };
    assert_eq!(total_seconds(&intervals), 23 * 60 * 60);
    assert_eq!(intervals.len(), 2);
    assert_eq!(intervals[0].offset_seconds_east, -6 * 60 * 60);
    assert_eq!(intervals[1].offset_seconds_east, -5 * 60 * 60);
}
