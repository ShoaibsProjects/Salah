# Calculation contract v0.1

**Status:** draft; unresolved definitions are listed below. This is a shared contract for the first Rust implementation, not a claim that any prayer time has already been validated.

## Scope

Calculate one Earth location and one local civil date, offline. The first implementation may use an explicit fixed UTC offset to display local times. It must return UTC instants so that later IANA time-zone support does not require rewriting astronomy. This version covers Fajr, sunrise, Dhuhr, Asr, sunset, Maghrib, and Isha. The UI may display six prayer-related entries while preserving sunset separately from Maghrib internally.

## Input

| Field | Meaning and constraint |
| --- | --- |
| Latitude | Geographic degrees north, within -90 to +90. |
| Longitude | Geographic degrees east, within -180 to +180. |
| Local date | Gregorian civil date for which the user requests the schedule. Define the relationship to UTC search intervals before coding. |
| UTC offset | Explicit signed offset used only for the first slice's local display and local-date interpretation. It is not an IANA time-zone identifier and contains no future DST rules. |
| Method profile | Versioned data specifying Fajr and Isha rules, Dhuhr/Maghrib adjustments, and any relevant rounding convention. |
| Asr criterion | Standard or Hanafi shadow criterion, when applicable to the method. |
| High-latitude rule | Explicitly `none` for the first raw astronomical pass; later values require their own specifications. |
| Elevation | Optional in the long-term model. First-slice treatment must be stated, such as sea-level assumption. Do not silently infer elevation from GPS. |

Reject non-finite coordinates, out-of-range values, unsupported method versions, malformed dates, and impossible offsets with typed errors. Keep input and output units explicit.

## Output

For each event, return:

- Event identity and a UTC instant if the event occurs under the selected rule.
- Local clock representation under the explicit offset.
- Status: `astronomical`, `method_adjusted`, `high_latitude_adjusted`, or `unavailable`; first-slice implementation uses only statuses it actually supports.
- Base solar event or angle/interval definition and any applied offset.
- Unrounded instant or documented precision, plus separately rounded display value.

For the complete calculation, return a reproducibility record: coordinates, requested local date, offset or time-zone identifier, astronomy model/version, method ID/version and parameters, Asr selection, high-latitude selection, adjustment chain, engine version, and reference-data versions where relevant. Preserve whether an input came from GPS, manual coordinates, a city, or a mosque timetable in application metadata; the core should not infer that provenance.

## Separation of concerns

1. **Astronomy:** solar position, transit, and geometric/apparent event candidates under documented assumptions.
2. **Method rules:** selected twilight angle or fixed interval, Asr criterion, and stated offsets.
3. **Civil time:** conversion of UTC instants to a local clock under explicit rules.
4. **Presentation:** rounding, language, 12/24-hour format, and accessibility.
5. **Mosque data:** separately sourced timetable and iqamah, never substituted into the astronomical result without a label.

The core must expose missing astronomical events explicitly. It must not silently switch to a fallback, return a fabricated instant, or treat a mosque's iqamah as the calculated start.

## Definitions to settle before implementation

- Astronomical reference model, epoch/time scale, numerical method, and supported date range.
- Sunrise/sunset convention: apparent solar disk, atmospheric refraction, elevation, and horizon assumption.
- Dhuhr and Maghrib base event plus method adjustment semantics.
- Asr shadow formula and how noon shadow is included.
- Fajr/Isha search interval, date assignment, and behavior when twilight is absent.
- Rounding of display minutes and ordering of offsets and rounding.
- First method profile's authoritative source and revision.
- Numerical acceptance tolerance relative to independent references.

Do not treat these as implementation trivia. They affect displayed prayer times and must be documented with the first working code.
