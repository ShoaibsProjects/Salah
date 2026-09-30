# Selected local-day engine

`salah-engine` is the thin connector between the core layers. It accepts a user-confirmed or manually selected time zone, an explicit local date, a method profile, and an Asr criterion. It then uses the pinned offline IANA rules to calculate and localize every matching solar cycle.

The flow is:

```text
coordinates → zone suggestions → explicit user selection
            → local date + method + Asr choice
            → UTC prayer events → local schedule with provenance
```

The function does not choose the date, a calculation method, Asr opinion, a zone from an ambiguous result, a winner among multiple solar cycles, or a replacement for unavailable events. Its result preserves the zone-selection evidence and the scheduler's skipped-date/zero/one/multiple status.

Example:

```rust
use salah_core::{AsrCriterion, CivilDate, Coordinates, MethodProfile};
use salah_engine::calculate_selected_local_day_schedule;
use salah_location::lookup_timezone_candidates;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let coordinates = Coordinates::new(44.9778, -93.2650)?;
    let candidates = lookup_timezone_candidates(coordinates)?;
    let zone = candidates.confirm_suggestion("America/Chicago")?;
    let date = CivilDate::new(2026, 9, 27)?;
    let result = calculate_selected_local_day_schedule(
        &zone,
        date,
        MethodProfile::research_15(),
        AsrCriterion::Hanafi,
    )?;
    println!("{}", result.schedule.record.zone_id);
    Ok(())
}
```

This is an experimental Rust API, not a consumer app. See the [versioned contract](../specification/selected-local-day-engine-contract-v0.1.md) for exact behavior and the [coordinate-to-zone contract](../specification/coordinate-zone-selection-contract-v0.1.md) for data provenance and mapping limits.

For a date-sensitive advisory, pass the result and an explicit observation date to `salah_engine::assess_civil_time_data`. It reports when the installed pack is from an earlier year and when the requested date is in the future. It does not modify the schedule or certify a local clock time. See the [data lifecycle rule](civil-time-data-lifecycle.md).
