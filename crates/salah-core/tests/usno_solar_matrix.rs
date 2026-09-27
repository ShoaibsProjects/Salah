//! Offline comparison against a versioned USNO source manifest.
//! Run with `cargo test -p salah-core --test usno_solar_matrix -- --nocapture`
//! to see signed raw-UTC discrepancies by case, event, cohort, and regime.

use std::collections::HashSet;

use salah_core::{
    AsrCriterion, CalculationInput, CivilDate, Coordinates, Event, FixedUtcOffset, MethodProfile,
    calculate_prayer_times,
};

const MANIFEST: &str = include_str!("../../../data/reference/solar-usno-v1.tsv");
const HEADER: &str = "case_id\tcohort\tregime\tlocal_date\tlatitude_deg\tlongitude_deg\tutc_offset_minutes\tcoordinate_datum\tobserver_elevation_m\tsource_name\tsource_version\tprimary_url\tprimary_sha256\tsecondary_url\tsecondary_sha256\tretrieved_date_utc\tsource_clock_basis\tsource_ephemeris_time_scale\tsource_event_definition\tsource_horizon_assumption\tsource_precision\trise_utc\ttransit_utc\tset_utc\tinclusion_reason";

struct Case<'a> {
    id: &'a str,
    cohort: &'a str,
    regime: &'a str,
    date: CivilDate,
    coordinates: Coordinates,
    offset: FixedUtcOffset,
    rise: &'a str,
    transit: &'a str,
    sunset: &'a str,
    precision: &'a str,
}

fn parse_date(value: &str) -> CivilDate {
    let parts: Vec<_> = value.split('-').collect();
    assert_eq!(parts.len(), 3, "invalid date {value}");
    CivilDate::new(
        parts[0].parse().unwrap(),
        parts[1].parse().unwrap(),
        parts[2].parse().unwrap(),
    )
    .unwrap()
}

fn parse_utc(value: &str) -> f64 {
    assert_eq!(value.len(), 20, "invalid UTC instant {value}");
    assert_eq!(&value[10..11], "T");
    assert_eq!(&value[19..20], "Z");
    let date = parse_date(&value[..10]);
    let hour: i64 = value[11..13].parse().unwrap();
    let minute: i64 = value[14..16].parse().unwrap();
    let second: i64 = value[17..19].parse().unwrap();
    assert!(hour < 24 && minute < 60 && second < 60);
    (date.days_since_unix_epoch() * 86_400 + hour * 3600 + minute * 60 + second) as f64
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn parse_case(line: &str) -> Case<'_> {
    let fields: Vec<_> = line.split('\t').collect();
    assert_eq!(
        fields.len(),
        25,
        "manifest row has wrong column count: {line}"
    );
    let id = fields[0];
    assert!(!id.is_empty());
    assert!(matches!(fields[1], "development" | "holdout"), "{id}");
    assert!(!fields[2].is_empty(), "{id}");
    assert_eq!(fields[7], "unreported", "{id}");
    assert_eq!(fields[8], "unreported", "{id}");
    assert_eq!(fields[9], "USNO_rise_set_transit_API", "{id}");
    assert_eq!(fields[10], "4.0.1", "{id}");
    assert!(
        fields[11].starts_with("https://aa.usno.navy.mil/api/rstt/oneday?"),
        "{id}"
    );
    assert!(valid_sha256(fields[12]), "{id}");
    assert_eq!(fields[13] == "-", fields[14] == "-", "{id}");
    if fields[13] != "-" {
        assert!(
            fields[13].starts_with("https://aa.usno.navy.mil/api/rstt/oneday?"),
            "{id}"
        );
        assert!(valid_sha256(fields[14]), "{id}");
    }
    parse_date(fields[15]);
    assert_eq!(fields[16], "supplied_fixed_offset", "{id}");
    assert_eq!(fields[17], "unreported", "{id}");
    assert_eq!(fields[18], "USNO_API_Rise_Upper_Transit_Set_labels", "{id}");
    assert_eq!(fields[19], "unreported_in_API_response", "{id}");
    assert_eq!(fields[20], "minute_formatted", "{id}");
    assert!(!fields[24].is_empty(), "{id}");
    for expected in &fields[21..24] {
        if !matches!(*expected, "no_event" | "unreported") {
            parse_utc(expected);
        }
    }
    Case {
        id,
        cohort: fields[1],
        regime: fields[2],
        date: parse_date(fields[3]),
        coordinates: Coordinates::new(fields[4].parse().unwrap(), fields[5].parse().unwrap())
            .unwrap(),
        offset: FixedUtcOffset::from_minutes_east(fields[6].parse().unwrap()).unwrap(),
        precision: fields[20],
        rise: fields[21],
        transit: fields[22],
        sunset: fields[23],
    }
}

fn compare_event(
    case: &Case<'_>,
    name: &str,
    expected: &str,
    actual: Event,
    failures: &mut Vec<String>,
) {
    let label = format!(
        "{}\t{}\t{}\t{}\t{}",
        case.id, case.cohort, case.regime, name, case.precision
    );
    if expected == "unreported" {
        println!("{label}\tunreported\t-");
        return;
    }
    if expected == "no_event" {
        match actual {
            Event::Unavailable { .. } => println!("{label}\tno_event_matches\t-"),
            Event::Occurs { .. } => {
                println!("{label}\tunexpected_event\t-");
                failures.push(format!("{} {name}: source reports no event", case.id));
            }
        }
        return;
    }
    match actual {
        Event::Occurs {
            unrounded_utc_unix_seconds,
            ..
        } => {
            let signed_delta = unrounded_utc_unix_seconds - parse_utc(expected);
            let allowance_seconds = if case.regime == "near-grazing" {
                180.0
            } else {
                90.0
            };
            let status = if signed_delta.abs() <= allowance_seconds {
                "within_case_allowance"
            } else {
                failures.push(format!(
                    "{} {name}: signed difference {signed_delta:.1}s exceeds {allowance_seconds:.0}s",
                    case.id
                ));
                "outside_case_allowance"
            };
            println!("{label}\t{status}\t{signed_delta:+.1}");
        }
        Event::Unavailable { .. }
            if case.id == "usno-2026-65735-grazing-disagreement"
                && matches!(name, "rise" | "set") =>
        {
            println!("{label}\tknown_model_boundary_disagreement\t-");
        }
        Event::Unavailable { .. } => {
            println!("{label}\tmissing_event\t-");
            failures.push(format!("{} {name}: source reports an event", case.id));
        }
    }
}

#[test]
fn sourced_solar_matrix_has_visible_differences_and_expected_statuses() {
    let mut lines = MANIFEST.lines();
    assert_eq!(lines.next(), Some(HEADER));
    let mut ids = HashSet::new();
    let mut holdout_count = 0;
    let mut failures = Vec::new();
    println!(
        "case_id\tcohort\tregime\tevent\tsource_precision\tstatus\tsigned_raw_utc_delta_seconds"
    );
    for line in lines {
        let case = parse_case(line);
        assert!(ids.insert(case.id), "duplicate case ID {}", case.id);
        if case.cohort == "holdout" {
            holdout_count += 1;
        }
        let result = calculate_prayer_times(CalculationInput {
            coordinates: case.coordinates,
            local_date: case.date,
            utc_offset: case.offset,
            method: MethodProfile::research_15(),
            asr_criterion: AsrCriterion::Standard,
        })
        .unwrap();
        compare_event(&case, "rise", case.rise, result.sunrise, &mut failures);
        // research-15 has zero Dhuhr adjustment, so Dhuhr is upper transit.
        compare_event(&case, "transit", case.transit, result.dhuhr, &mut failures);
        compare_event(&case, "set", case.sunset, result.sunset, &mut failures);
    }
    assert_eq!(
        ids.len(),
        19,
        "manifest case count changed; review coverage"
    );
    assert_eq!(holdout_count, 7, "holdout cohort changed; review split");
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
