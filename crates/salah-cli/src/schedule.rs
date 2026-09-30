//! Thin named-zone interface; all numerical work belongs to the engine.

use std::collections::BTreeSet;
use std::io::{self, Write};

use salah_core::{AsrCriterion, CivilDate, Coordinates, MethodProfile};
use salah_engine::{calculate_selected_local_day_schedule, schedule_document};
use salah_location::select_manual_zone;
use salah_time::{
    LocalDatePrayerScheduleCandidate, LocalDatePrayerScheduleMatches, LocalDateStatus,
    LocalizedEvent, SUPPORTED_ZONE_IDS,
};

const USAGE: &str = "Usage: salah-cli schedule --lat DEGREES --lon DEGREES --date YYYY-MM-DD \
--zone IANA_ID --method <research-15|mwl-angles-18-17> --asr <standard|hanafi> [--json]\n\n\
Every calculation input is explicit. The zone is your manual choice; no polygon\n\
lookup or device setting is used. Dates must be Gregorian, 1900–2100.\n\
Output is a research preview with seconds, UTC, and exact timezone data identity.\n\
No high-latitude substitution, mosque timetable, or notification rule is applied.\n\
research-15 is an engineering-only profile; mwl-angles-18-17 is a published\n\
parameter set, not an institutional endorsement. Use `salah-cli zones` to list names.";

/// Buffer the complete response before writing; JSON never mixes with a text
/// banner or a partial calculation. A pipe closed by its reader is ordinary.
fn write_output(output: &str) -> Result<(), String> {
    match io::stdout().lock().write_all(output.as_bytes()) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == io::ErrorKind::BrokenPipe => Ok(()),
        Err(error) => Err(format!("cannot write output: {error}")),
    }
}

pub(super) fn zones(args: impl Iterator<Item = String>) -> Result<(), String> {
    let args: Vec<_> = args.collect();
    if args == ["--help"] || args == ["-h"] {
        return write_output(
            "Usage: salah-cli zones\nLists every identifier in the bundled IANA pack, one per line.\n",
        );
    }
    if !args.is_empty() {
        return Err("zones accepts no options; use `salah-cli zones`".to_owned());
    }
    let mut output = SUPPORTED_ZONE_IDS.join("\n");
    output.push('\n');
    write_output(&output)
}

pub(super) fn run(args: impl Iterator<Item = String>) -> Result<(), String> {
    let args: Vec<_> = args.collect();
    if args == ["--help"] || args == ["-h"] {
        return write_output(&format!("{USAGE}\n"));
    }
    let mut args = args.into_iter();
    let mut seen = BTreeSet::new();
    let mut latitude = None;
    let mut longitude = None;
    let mut date = None;
    let mut zone_id = None;
    let mut method = None;
    let mut asr = None;
    let mut json = false;
    while let Some(flag) = args.next() {
        if !matches!(
            flag.as_str(),
            "--lat" | "--lon" | "--date" | "--zone" | "--method" | "--asr" | "--json"
        ) {
            return Err(format!("unknown schedule option: {flag}"));
        }
        if !seen.insert(flag.clone()) {
            return Err(format!("repeated schedule option: {flag}"));
        }
        if flag == "--json" {
            json = true;
            continue;
        }
        let value = args
            .next()
            .ok_or_else(|| format!("missing value for {flag}"))?;
        if value.starts_with("--") {
            return Err(format!("missing value for {flag} (found option {value})"));
        }
        match flag.as_str() {
            "--lat" => latitude = Some(super::parse_number(&value, "latitude")?),
            "--lon" => longitude = Some(super::parse_number(&value, "longitude")?),
            "--date" => {
                let parsed: CivilDate = super::parse_date(&value)?;
                if parsed.to_string() != value {
                    return Err("schedule date must use YYYY-MM-DD format".to_owned());
                }
                date = Some(parsed);
            }
            "--zone" => zone_id = Some(value),
            "--method" => {
                method = Some(match value.as_str() {
                    "research-15" => MethodProfile::research_15(),
                    "mwl-angles-18-17" => MethodProfile::mwl_angles_18_17(),
                    _ => return Err(format!("unsupported method: {value}")),
                })
            }
            "--asr" => {
                asr = Some(match value.as_str() {
                    "standard" => AsrCriterion::Standard,
                    "hanafi" => AsrCriterion::Hanafi,
                    _ => return Err(format!("unsupported Asr criterion: {value}")),
                })
            }
            _ => unreachable!("option was checked above"),
        }
    }
    // Validate every input before loading a zone or doing numerical work.
    let coordinates = Coordinates::new(
        latitude.ok_or("missing --lat")?,
        longitude.ok_or("missing --lon")?,
    )
    .map_err(|error| error.to_string())?;
    let requested_date = date.ok_or("missing --date")?;
    let zone_id = zone_id.ok_or("missing --zone")?;
    let method = method.ok_or("missing --method")?;
    let asr = asr.ok_or("missing --asr")?;
    let selection = select_manual_zone(coordinates, &zone_id).map_err(|error| error.to_string())?;
    let selected = calculate_selected_local_day_schedule(&selection, requested_date, method, asr)
        .map_err(|error| error.to_string())?;
    // The same finite-value check guards text output and JSON output.
    let document = schedule_document(&selected).map_err(|error| error.to_string())?;
    if json {
        let mut output =
            serde_json::to_string_pretty(&document).map_err(|error| error.to_string())?;
        output.push('\n');
        return write_output(&output);
    }

    let schedule = &selected.schedule;
    let record = &schedule.record;
    let mut output = format!(
        "Salah local schedule · research preview\n\
Kernel: {} · {}\n\
Location: {:.6}°, {:.6}° · requested local date {}\n\
Zone: {} · manually selected, boundary map not consulted\n\
Rules: IANA {} · bundled snapshot\n\
Pack SHA-256: {}\n\
Inventory SHA-256: {}\n\
Method: {} v{} · Fajr {}° · Isha {}° · Asr {:?}\n\
Source: {}\n\
Parameter profile; not an endorsed timetable. No high-latitude fallback.\n\
Local event dates and offsets are resolved separately for each UTC instant.\n",
        record.engine_version,
        record.astronomy_model,
        coordinates.latitude_degrees(),
        coordinates.longitude_degrees(),
        requested_date,
        record.zone_id,
        record.rule_pack.tzdb_version,
        record.rule_pack.sha256,
        record.rule_pack.inventory_sha256,
        method.id,
        method.revision,
        method.fajr_depression_degrees,
        method.isha_depression_degrees,
        asr,
        method.source,
    );
    match &schedule.civil_date.status {
        LocalDateStatus::Skipped => {
            output.push_str("Civil date: skipped under this zone's rules; no local schedule.\n")
        }
        LocalDateStatus::Exists(_) => {
            output.push_str("Civil date: exists under this zone's rules.\n")
        }
    }
    match &schedule.matches {
        LocalDatePrayerScheduleMatches::Zero => {
            output.push_str("Solar cycles: zero matches; no cycle was substituted.\n")
        }
        LocalDatePrayerScheduleMatches::One(candidate) => {
            output.push_str("Solar cycles: one match.\n");
            append_cycle(&mut output, candidate);
        }
        LocalDatePrayerScheduleMatches::Multiple(candidates) => {
            output.push_str(&format!(
                "Solar cycles: {} matches; all shown, no winner selected.\n",
                candidates.len()
            ));
            for (index, candidate) in candidates.iter().enumerate() {
                output.push_str(&format!("\nCycle {}\n", index + 1));
                append_cycle(&mut output, candidate);
            }
        }
    }
    write_output(&output)
}

fn append_cycle(output: &mut String, candidate: &LocalDatePrayerScheduleCandidate) {
    output.push_str(&format!(
        "Selected solar transit: {}\n\n",
        candidate.local_transit.local
    ));
    let e = &candidate.events;
    for (name, event) in [
        ("Fajr", &e.fajr),
        ("Sunrise", &e.sunrise),
        ("Dhuhr", &e.dhuhr),
        ("Asr", &e.asr),
        ("Sunset", &e.sunset),
        ("Maghrib", &e.maghrib),
        ("Isha", &e.isha),
    ] {
        match event {
            LocalizedEvent::Occurs { local, .. } => output.push_str(&format!(
                "{name:<8} {} {} (UTC {}Z)\n",
                local.local,
                format_offset(local.offset_seconds_east),
                local.utc.to_utc(),
            )),
            LocalizedEvent::Unavailable { reason } => {
                output.push_str(&format!("{name:<8} unavailable: {reason}\n"))
            }
        }
    }
}

/// Keep historical sub-minute offsets intact, including a negative sign.
fn format_offset(seconds_east: i32) -> String {
    let sign = if seconds_east < 0 { '-' } else { '+' };
    let magnitude = i64::from(seconds_east).abs();
    format!(
        "{sign}{:02}:{:02}:{:02}",
        magnitude / 3600,
        (magnitude % 3600) / 60,
        magnitude % 60
    )
}
