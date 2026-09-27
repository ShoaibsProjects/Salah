//! Offline audit of the versioned prayer-library snapshot.
//! Run with `cargo test --locked --offline -p salah-core --test prayer_library_matrix -- --nocapture`.
use salah_core::{
    AsrCriterion, CalculationInput, CivilDate, Coordinates, Event, FixedUtcOffset, MethodProfile,
    calculate_prayer_times,
};

fn parse_date(value: &str) -> CivilDate {
    let mut parts = value.split('-');
    CivilDate::new(
        parts.next().unwrap().parse().unwrap(),
        parts.next().unwrap().parse().unwrap(),
        parts.next().unwrap().parse().unwrap(),
    )
    .unwrap()
}

fn source_seconds(value: &str) -> Option<f64> {
    if value == "no_event" {
        return None;
    }
    let (date, time) = value.split_once('T').unwrap();
    let time = time.strip_suffix('Z').unwrap();
    let mut parts = time.split(':');
    let hour: f64 = parts.next().unwrap().parse().unwrap();
    let minute: f64 = parts.next().unwrap().parse().unwrap();
    let second: f64 = parts.next().unwrap().parse().unwrap();
    Some(
        parse_date(date).days_since_unix_epoch() as f64 * 86_400.0
            + hour * 3600.0
            + minute * 60.0
            + second,
    )
}

fn signed_difference(event: Event, reference: &str) -> String {
    match (
        event.unrounded_utc_unix_seconds(),
        source_seconds(reference),
    ) {
        (Some(actual), Some(expected)) => format!("{:+.3}s", actual - expected),
        (None, None) => "both_no_event".to_string(),
        (Some(_), None) => "salah_only_event".to_string(),
        (None, Some(_)) => "source_only_event".to_string(),
    }
}

#[test]
fn source_labeled_prayer_rules() {
    let manifest = include_str!("../../../data/reference/prayer-library-v1.tsv");
    let mut lines = manifest.lines();
    let header = lines.next().unwrap();
    assert_eq!(header.split('\t').count(), 24);
    let mut count = 0;
    let mut fallback_count = 0;
    for line in lines {
        let columns: Vec<_> = line.split('\t').collect();
        assert_eq!(columns.len(), 24);
        let id = columns[0];
        let date = parse_date(columns[2]);
        let praytimes_shift: i64 = columns[22].parse().unwrap();
        let adhan_date = parse_date(columns[23]);
        let expected_shift = if id == "kiritimati-date-line" { -1 } else { 0 };
        assert_eq!(praytimes_shift, expected_shift);
        assert_eq!(
            adhan_date.days_since_unix_epoch() - date.days_since_unix_epoch(),
            expected_shift
        );
        let coordinates =
            Coordinates::new(columns[3].parse().unwrap(), columns[4].parse().unwrap()).unwrap();
        let offset = FixedUtcOffset::from_minutes_east(columns[5].parse().unwrap()).unwrap();
        let method = match columns[6] {
            "research-15" => MethodProfile::research_15(),
            "mwl-angles-18-17" => MethodProfile::mwl_angles_18_17(),
            other => panic!("unexpected profile {other}"),
        };
        assert_eq!(
            columns[7].parse::<f64>().unwrap(),
            method.fajr_depression_degrees
        );
        assert_eq!(
            columns[8].parse::<f64>().unwrap(),
            method.isha_depression_degrees
        );
        let asr = match columns[9] {
            "Standard" => AsrCriterion::Standard,
            "Hanafi" => AsrCriterion::Hanafi,
            other => panic!("unexpected asr {other}"),
        };
        for kind in [columns[10], columns[11]] {
            assert!(kind == "angle" || kind == "fallback");
            fallback_count += usize::from(kind == "fallback");
        }
        let output = calculate_prayer_times(CalculationInput {
            coordinates,
            local_date: date,
            utc_offset: offset,
            method,
            asr_criterion: asr,
        })
        .unwrap();
        let events = [
            ("fajr", output.fajr),
            ("dhuhr", output.dhuhr),
            ("asr", output.asr),
            ("maghrib", output.maghrib),
            ("isha", output.isha),
        ];
        for (index, (name, event)) in events.into_iter().enumerate() {
            let praytimes = signed_difference(event, columns[12 + index * 2]);
            let adhan = signed_difference(event, columns[13 + index * 2]);
            println!(
                "{id} {} {:?} {name} praytimes={praytimes} adhan={adhan}",
                method.id, asr
            );
            if id == "tromso-winter" && name == "asr" {
                assert_eq!(praytimes, "source_only_event");
                assert_eq!(adhan, "source_only_event");
            }
            if id == "tromso-winter" && name == "maghrib" {
                assert_eq!(praytimes, "both_no_event");
                assert_eq!(adhan, "both_no_event");
            }
            if (name == "fajr" && columns[10] == "fallback")
                || (name == "isha" && columns[11] == "fallback")
            {
                assert_eq!(praytimes, "both_no_event");
                assert_eq!(adhan, "source_only_event");
            }
        }
        count += 1;
    }
    assert_eq!(count, 28);
    assert_eq!(fallback_count, 4);
}
