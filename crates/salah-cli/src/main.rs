use std::env;
use std::process::ExitCode;

use salah_core::{
    AsrCriterion, CalculationInput, CivilDate, Coordinates, Event, FixedUtcOffset, MethodProfile,
    calculate_prayer_times,
};

const USAGE: &str = "Usage: salah-cli --lat DEGREES --lon DEGREES --date YYYY-MM-DD \
--utc-offset <+HH:MM|-HH:MM> --method research-15 --asr <standard|hanafi>\n\n\
This is an offline research calculation. The fixed UTC offset is not a time zone.\n\
The research-15 profile is not a named institutional prayer method.";

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("{message}\n\n{USAGE}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), String> {
    let mut latitude = None;
    let mut longitude = None;
    let mut date = None;
    let mut offset = None;
    let mut method = None;
    let mut asr = None;
    let mut args = env::args().skip(1);
    if args.len() == 0 {
        println!("{USAGE}");
        return Ok(());
    }

    while let Some(flag) = args.next() {
        if flag == "--help" || flag == "-h" {
            println!("{USAGE}");
            return Ok(());
        }
        let value = args
            .next()
            .ok_or_else(|| format!("missing value for {flag}"))?;
        match flag.as_str() {
            "--lat" => latitude = Some(parse_number(&value, "latitude")?),
            "--lon" => longitude = Some(parse_number(&value, "longitude")?),
            "--date" => date = Some(parse_date(&value)?),
            "--utc-offset" => offset = Some(parse_offset(&value)?),
            "--method" if value == "research-15" => {
                method = Some(MethodProfile::research_15());
            }
            "--method" => return Err(format!("unsupported method: {value}")),
            "--asr" if value == "standard" => asr = Some(AsrCriterion::Standard),
            "--asr" if value == "hanafi" => asr = Some(AsrCriterion::Hanafi),
            "--asr" => return Err(format!("unsupported Asr criterion: {value}")),
            _ => return Err(format!("unknown option: {flag}")),
        }
    }

    let coordinates = Coordinates::new(
        latitude.ok_or("missing --lat")?,
        longitude.ok_or("missing --lon")?,
    )
    .map_err(|error| error.to_string())?;
    let input = CalculationInput {
        coordinates,
        local_date: date.ok_or("missing --date")?,
        utc_offset: offset.ok_or("missing --utc-offset")?,
        method: method.ok_or("missing --method")?,
        asr_criterion: asr.ok_or("missing --asr")?,
    };
    let result = calculate_prayer_times(input).map_err(|error| error.to_string())?;

    println!(
        "Salah kernel {} · {}",
        result.record.engine_version, result.record.astronomy_model
    );
    println!(
        "Location: {:.4}°, {:.4}° · local date {} · fixed UTC offset {}",
        coordinates.latitude_degrees(),
        coordinates.longitude_degrees(),
        input.local_date,
        input.utc_offset
    );
    println!(
        "Method: {} v{} · Fajr {}° · Isha {}° · Asr {:?}",
        input.method.id,
        input.method.revision,
        input.method.fajr_depression_degrees,
        input.method.isha_depression_degrees,
        input.asr_criterion
    );
    println!("Research profile; not an institutional timetable. No high-latitude fallback.\n");

    for (name, event) in [
        ("Fajr", result.fajr),
        ("Sunrise", result.sunrise),
        ("Dhuhr", result.dhuhr),
        ("Asr", result.asr),
        ("Sunset", result.sunset),
        ("Maghrib", result.maghrib),
        ("Isha", result.isha),
    ] {
        match event {
            Event::Occurs { utc, .. } => println!(
                "{name:<8} {} (UTC {}Z)",
                utc.to_local(input.utc_offset),
                utc.to_utc()
            ),
            Event::Unavailable { reason } => println!("{name:<8} unavailable: {reason}"),
        }
    }
    Ok(())
}

fn parse_number(value: &str, label: &str) -> Result<f64, String> {
    value
        .parse::<f64>()
        .map_err(|_| format!("invalid {label}: {value}"))
}

fn parse_date(value: &str) -> Result<CivilDate, String> {
    let parts: Vec<_> = value.split('-').collect();
    if parts.len() != 3 {
        return Err(format!("invalid date: {value}"));
    }
    let year = parts[0]
        .parse::<i32>()
        .map_err(|_| format!("invalid date: {value}"))?;
    let month = parts[1]
        .parse::<u8>()
        .map_err(|_| format!("invalid date: {value}"))?;
    let day = parts[2]
        .parse::<u8>()
        .map_err(|_| format!("invalid date: {value}"))?;
    CivilDate::new(year, month, day).map_err(|error| error.to_string())
}

fn parse_offset(value: &str) -> Result<FixedUtcOffset, String> {
    let (sign, digits) = match value.as_bytes().first() {
        Some(b'+') => (1, &value[1..]),
        Some(b'-') => (-1, &value[1..]),
        _ => return Err(format!("invalid UTC offset: {value}")),
    };
    let parts: Vec<_> = digits.split(':').collect();
    if parts.len() != 2 {
        return Err(format!("invalid UTC offset: {value}"));
    }
    let hour = parts[0]
        .parse::<i16>()
        .map_err(|_| format!("invalid UTC offset: {value}"))?;
    let minute = parts[1]
        .parse::<i16>()
        .map_err(|_| format!("invalid UTC offset: {value}"))?;
    if !(0..=14).contains(&hour) || !(0..=59).contains(&minute) || (hour == 14 && minute != 0) {
        return Err(format!("invalid UTC offset: {value}"));
    }
    FixedUtcOffset::from_minutes_east(sign * (hour * 60 + minute))
        .map_err(|error| error.to_string())
}
