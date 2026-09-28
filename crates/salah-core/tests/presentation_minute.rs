//! Research-preview prayer-start minute receipts.
//!
//! Expected values derive from the committed
//! `specification/presentation-contract-v0.1.md` cited kernel seconds (which
//! reference `data/reference/prayer-library-v1.tsv` rows) and its boundary
//! checks. No source observation is invented here.

use salah_core::{
    AsrCriterion, CalculationInput, CivilDate, Coordinates, Event, EventRule, FixedUtcOffset,
    MethodProfile, PrayerStart, PrayerStartReceipt, PrayerStartStatus, UnavailableReason,
    UtcInstant, calculate_prayer_times, prayer_start_minute,
};

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

fn kernel_input(
    year: i32,
    month: u8,
    day: u8,
    latitude: f64,
    longitude: f64,
    offset_minutes: i16,
    method: MethodProfile,
) -> CalculationInput {
    CalculationInput {
        coordinates: Coordinates::new(latitude, longitude).unwrap(),
        local_date: CivilDate::new(year, month, day).unwrap(),
        utc_offset: FixedUtcOffset::from_minutes_east(offset_minutes).unwrap(),
        method,
        asr_criterion: AsrCriterion::Standard,
    }
}

/// Build a synthetic result whose Fajr occurs at `utc_seconds` with the given
/// offset, for pure adapter boundary checks without any solar calculation.
fn synthetic_fajr_result(utc_seconds: i64, offset_minutes: i16) -> salah_core::PrayerTimes {
    let method = MethodProfile::research_15();
    let event = Event::Occurs {
        utc: UtcInstant {
            unix_seconds: utc_seconds,
        },
        unrounded_utc_unix_seconds: utc_seconds as f64,
        rule: EventRule::SolarDepression { degrees: 15.0 },
    };
    let unavailable = Event::Unavailable {
        reason: UnavailableReason::NoCrossingInSolarCycle,
    };
    salah_core::PrayerTimes {
        fajr: event,
        sunrise: unavailable,
        dhuhr: unavailable,
        asr: unavailable,
        sunset: unavailable,
        maghrib: unavailable,
        isha: unavailable,
        record: salah_core::CalculationRecord {
            coordinates: Coordinates::new(0.0, 0.0).unwrap(),
            local_date: CivilDate::new(2026, 1, 1).unwrap(),
            utc_offset: FixedUtcOffset::from_minutes_east(offset_minutes).unwrap(),
            method,
            asr_criterion: AsrCriterion::Standard,
            astronomy_model: "test-only",
            engine_version: "test-only",
            high_latitude_rule: "none",
            assumed_elevation_meters: 0.0,
        },
    }
}

fn assert_receipt_identity(receipt: &PrayerStartReceipt, prayer: PrayerStart) {
    assert_eq!(receipt.prayer, prayer);
    assert_eq!(receipt.display_policy_id, "prayer-start-ceil-minute");
    assert_eq!(receipt.display_policy_revision, "0.1");
    assert_eq!(receipt.display_policy_id, salah_core::DISPLAY_POLICY_ID);
    assert_eq!(
        receipt.display_policy_revision,
        salah_core::DISPLAY_POLICY_REVISION
    );
}

fn occurs_parts(receipt: &PrayerStartReceipt) -> (UtcInstant, String, EventRule) {
    match receipt.status {
        PrayerStartStatus::Occurs { utc, display, rule } => (utc, display.to_string(), rule),
        PrayerStartStatus::Unavailable { reason } => {
            panic!("expected Occurs, got Unavailable({reason:?})")
        }
    }
}

#[test]
fn minneapolis_dhuhr_ceils_to_next_minute() {
    // presentation-contract-v0.1 row: kernel 18:03:59Z, offset -300.
    let times = calculate_prayer_times(kernel_input(
        2026,
        9,
        27,
        44.9778,
        -93.2650,
        -300,
        MethodProfile::mwl_angles_18_17(),
    ))
    .unwrap();
    assert_eq!(times.dhuhr.utc(), Some(utc(2026, 9, 27, 18, 3, 59)));
    let receipt = prayer_start_minute(&times, PrayerStart::Dhuhr);
    assert_receipt_identity(&receipt, PrayerStart::Dhuhr);
    assert_eq!(receipt.method_id, "mwl-angles-18-17");
    assert_eq!(receipt.method_revision, "0.1");
    let (preserved, display, rule) = occurs_parts(&receipt);
    assert_eq!(preserved, utc(2026, 9, 27, 18, 3, 59));
    assert_eq!(display, "2026-09-27 13:04");
    assert_eq!(rule, EventRule::SolarTransit);
}

#[test]
fn minneapolis_maghrib_ceils_to_next_minute() {
    // presentation-contract-v0.1 row: kernel 2026-09-28T00:00:50Z, offset -300.
    let times = calculate_prayer_times(kernel_input(
        2026,
        9,
        27,
        44.9778,
        -93.2650,
        -300,
        MethodProfile::mwl_angles_18_17(),
    ))
    .unwrap();
    assert_eq!(times.maghrib.utc(), Some(utc(2026, 9, 28, 0, 0, 50)));
    let receipt = prayer_start_minute(&times, PrayerStart::Maghrib);
    assert_receipt_identity(&receipt, PrayerStart::Maghrib);
    assert_eq!(receipt.method_id, "mwl-angles-18-17");
    assert_eq!(receipt.method_revision, "0.1");
    let (preserved, display, rule) = occurs_parts(&receipt);
    assert_eq!(preserved, utc(2026, 9, 28, 0, 0, 50));
    assert_eq!(display, "2026-09-27 19:01");
    assert_eq!(rule, EventRule::SunsetWithAdjustment { seconds: 0 });
}

#[test]
fn makkah_dhuhr_ceils_to_next_minute() {
    // presentation-contract-v0.1 row: kernel 09:28:10Z, offset +180.
    let times = calculate_prayer_times(kernel_input(
        2026,
        3,
        20,
        21.4225,
        39.8262,
        180,
        MethodProfile::mwl_angles_18_17(),
    ))
    .unwrap();
    assert_eq!(times.dhuhr.utc(), Some(utc(2026, 3, 20, 9, 28, 10)));
    let receipt = prayer_start_minute(&times, PrayerStart::Dhuhr);
    assert_receipt_identity(&receipt, PrayerStart::Dhuhr);
    let (preserved, display, rule) = occurs_parts(&receipt);
    assert_eq!(preserved, utc(2026, 3, 20, 9, 28, 10));
    assert_eq!(display, "2026-03-20 12:29");
    assert_eq!(rule, EventRule::SolarTransit);
}

#[test]
fn london_isha_carries_converted_date_past_midnight() {
    // presentation-contract-v0.1 row: research-15 kernel 23:49:15Z, offset +60.
    // The displayed local date (06-22) differs from the requested date (06-21).
    let times = calculate_prayer_times(kernel_input(
        2026,
        6,
        21,
        51.5072,
        -0.1276,
        60,
        MethodProfile::research_15(),
    ))
    .unwrap();
    assert_eq!(times.isha.utc(), Some(utc(2026, 6, 21, 23, 49, 15)));
    let receipt = prayer_start_minute(&times, PrayerStart::Isha);
    assert_receipt_identity(&receipt, PrayerStart::Isha);
    assert_eq!(receipt.method_id, "research-15");
    assert_eq!(receipt.method_revision, "0.1");
    let (preserved, display, rule) = occurs_parts(&receipt);
    assert_eq!(preserved, utc(2026, 6, 21, 23, 49, 15));
    assert_eq!(display, "2026-06-22 00:50");
    assert_eq!(rule, EventRule::SolarDepression { degrees: 15.0 });
}

#[test]
fn tromso_unavailable_events_gain_no_minute() {
    // presentation-contract-v0.1 row: polar-night Maghrib/Asr Unavailable.
    let times = calculate_prayer_times(kernel_input(
        2026,
        12,
        21,
        69.6492,
        18.9553,
        60,
        MethodProfile::mwl_angles_18_17(),
    ))
    .unwrap();
    assert_eq!(times.maghrib.utc(), None);
    assert_eq!(times.asr.utc(), None);
    for prayer in [PrayerStart::Maghrib, PrayerStart::Asr] {
        let receipt = prayer_start_minute(&times, prayer);
        assert_receipt_identity(&receipt, prayer);
        match receipt.status {
            PrayerStartStatus::Unavailable { reason } => {
                assert_eq!(reason, UnavailableReason::NoCrossingInSolarCycle);
            }
            PrayerStartStatus::Occurs { .. } => {
                panic!("{prayer:?} must stay unavailable, never gain a minute")
            }
        }
    }
    // No source-library fallback is imported: the kernel's own Fajr/Isha may
    // occur, but Maghrib/Asr remain unavailable with no clock value.
}

#[test]
fn exact_minute_stays_and_next_second_advances() {
    let base = utc(2026, 1, 1, 10, 0, 0).unix_seconds;
    let on_minute = prayer_start_minute(&synthetic_fajr_result(base, 0), PrayerStart::Fajr);
    let (_, display, _) = occurs_parts(&on_minute);
    assert_eq!(display, "2026-01-01 10:00");

    let one_second_later =
        prayer_start_minute(&synthetic_fajr_result(base + 1, 0), PrayerStart::Fajr);
    let (_, display, _) = occurs_parts(&one_second_later);
    assert_eq!(display, "2026-01-01 10:01");
}

#[test]
fn end_of_day_carries_into_next_local_date() {
    let last_second = utc(2026, 1, 1, 23, 59, 59).unix_seconds;
    let receipt = prayer_start_minute(&synthetic_fajr_result(last_second, 0), PrayerStart::Fajr);
    let (_, display, _) = occurs_parts(&receipt);
    assert_eq!(display, "2026-01-02 00:00");
}

#[test]
fn pre_epoch_second_uses_euclidean_ceiling() {
    // 1969-12-31T23:59:59Z is Unix second -1. The whole-minute ceiling
    // crosses the epoch and carries the date to 1970-01-01.
    let receipt = prayer_start_minute(&synthetic_fajr_result(-1, 0), PrayerStart::Fajr);
    let (_, display, _) = occurs_parts(&receipt);
    assert_eq!(display, "1970-01-01 00:00");
}

#[test]
fn hypothetical_nonzero_dhuhr_adjustment_applies_once_before_display() {
    // Hypothetical illustration only: not a real method, profile, or source
    // observation. The existing kernel adds the adjustment; the adapter must
    // use the adjusted Event.utc exactly once and never reapply it.
    let hypothetical = MethodProfile {
        id: "test-hypothetical-plus-120s",
        revision: "0.1-test",
        source: "hypothetical ordering illustration only; not a real method",
        fajr_depression_degrees: 18.0,
        isha_depression_degrees: 17.0,
        dhuhr_adjustment_seconds: 120,
        maghrib_adjustment_seconds: 0,
    };
    let base = calculate_prayer_times(kernel_input(
        2026,
        9,
        27,
        44.9778,
        -93.2650,
        -300,
        MethodProfile::mwl_angles_18_17(),
    ))
    .unwrap();
    let adjusted = calculate_prayer_times(kernel_input(
        2026,
        9,
        27,
        44.9778,
        -93.2650,
        -300,
        hypothetical,
    ))
    .unwrap();
    let base_utc = base.dhuhr.utc().unwrap();
    let adjusted_utc = adjusted.dhuhr.utc().unwrap();
    assert_eq!(adjusted_utc.unix_seconds - base_utc.unix_seconds, 120);

    let receipt = prayer_start_minute(&adjusted, PrayerStart::Dhuhr);
    assert_receipt_identity(&receipt, PrayerStart::Dhuhr);
    assert_eq!(receipt.method_id, "test-hypothetical-plus-120s");
    let (preserved, display, rule) = occurs_parts(&receipt);
    assert_eq!(preserved, adjusted_utc);
    // Base kernel Dhuhr 18:03:59Z + 120 s = 18:05:59Z -> local 13:05:59 -> 13:06.
    assert_eq!(adjusted_utc, utc(2026, 9, 27, 18, 5, 59));
    assert_eq!(display, "2026-09-27 13:06");
    assert_eq!(rule, EventRule::TransitWithAdjustment { seconds: 120 });
}

#[test]
fn hypothetical_nonzero_maghrib_adjustment_applies_once_before_display() {
    // Hypothetical illustration only: not a real method, profile, or source
    // observation.
    let hypothetical = MethodProfile {
        id: "test-hypothetical-maghrib-plus-60s",
        revision: "0.1-test",
        source: "hypothetical ordering illustration only; not a real method",
        fajr_depression_degrees: 18.0,
        isha_depression_degrees: 17.0,
        dhuhr_adjustment_seconds: 0,
        maghrib_adjustment_seconds: 60,
    };
    let base = calculate_prayer_times(kernel_input(
        2026,
        9,
        27,
        44.9778,
        -93.2650,
        -300,
        MethodProfile::mwl_angles_18_17(),
    ))
    .unwrap();
    let adjusted = calculate_prayer_times(kernel_input(
        2026,
        9,
        27,
        44.9778,
        -93.2650,
        -300,
        hypothetical,
    ))
    .unwrap();
    let base_utc = base.maghrib.utc().unwrap();
    let adjusted_utc = adjusted.maghrib.utc().unwrap();
    assert_eq!(adjusted_utc.unix_seconds - base_utc.unix_seconds, 60);

    let receipt = prayer_start_minute(&adjusted, PrayerStart::Maghrib);
    let (preserved, display, rule) = occurs_parts(&receipt);
    assert_eq!(preserved, adjusted_utc);
    // Base kernel Maghrib 2026-09-28T00:00:50Z + 60 s -> local 19:01:50 -> 19:02.
    assert_eq!(display, "2026-09-27 19:02");
    assert_eq!(rule, EventRule::SunsetWithAdjustment { seconds: 60 });
}

#[test]
fn maghrib_and_sunset_share_instant_but_keep_distinct_rules() {
    let times = calculate_prayer_times(kernel_input(
        2026,
        9,
        27,
        44.9778,
        -93.2650,
        -300,
        MethodProfile::mwl_angles_18_17(),
    ))
    .unwrap();
    let sunset_utc = times.sunset.utc().unwrap();
    let maghrib_utc = times.maghrib.utc().unwrap();
    assert_eq!(sunset_utc, maghrib_utc);
    let sunset_rule = match times.sunset {
        Event::Occurs { rule, .. } => rule,
        Event::Unavailable { .. } => panic!("sunset occurs in this case"),
    };
    assert_eq!(sunset_rule, EventRule::ApparentHorizon);
    let receipt = prayer_start_minute(&times, PrayerStart::Maghrib);
    let (_, _, rule) = occurs_parts(&receipt);
    assert_eq!(rule, EventRule::SunsetWithAdjustment { seconds: 0 });
    assert_ne!(rule, sunset_rule);
}

#[test]
fn all_five_prayer_starts_have_receipts_in_display_order() {
    let times = calculate_prayer_times(kernel_input(
        2026,
        9,
        27,
        44.9778,
        -93.2650,
        -300,
        MethodProfile::mwl_angles_18_17(),
    ))
    .unwrap();
    let all = PrayerStart::all();
    assert_eq!(all.len(), 5);
    assert_eq!(
        all.map(PrayerStart::name),
        ["Fajr", "Dhuhr", "Asr", "Maghrib", "Isha"]
    );
    // The typed enum offers no Sunrise or Sunset variant: every constructible
    // prayer-start receipt is one of the five above, so sunset/sunrise can
    // never be requested through this adapter.
    for prayer in all {
        let receipt = prayer_start_minute(&times, prayer);
        assert_receipt_identity(&receipt, prayer);
        assert_eq!(receipt.prayer.name(), receipt.prayer.to_string());
    }
}
