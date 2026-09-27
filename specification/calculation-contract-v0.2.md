# Calculation contract v0.2

**Status:** research implementation in `salah-core` v0.2.0. The source and [reference cases](reference-cases.md) define the currently implemented behavior. This is not a reviewed institutional timetable. [Contract v0.1](calculation-contract-v0.1.md) remains the record for earlier output.

## Changes from v0.1

- The solver selects rising and falling crossings explicitly and inspects an interior extremum before reporting no crossing near a grazing horizon.
- Asr fixes the noon-shadow baseline at calculated upper transit rather than allowing it to drift during the afternoon.
- `mwl-angles-18-17` adds a published, source-labeled Fajr/Isha parameter set alongside `research-15`.
- The astronomy model identifier changes to `NOAA-MEEUS-SOLAR-2`; the engine version changes to `0.2.0`. Historical results must retain their original model and engine identifiers.

## Scope

Calculate one Earth location and one local civil date, offline. The first implementation uses an explicit fixed UTC offset to display local times and returns UTC instants so that later IANA time-zone support does not require rewriting astronomy. This version covers Fajr, sunrise, Dhuhr, Asr, sunset, Maghrib, and Isha. The UI may display six prayer-related entries while preserving sunset separately from Maghrib internally.

## Input

| Field | Meaning and constraint |
| --- | --- |
| Latitude | Geographic degrees north, within -90 to +90. |
| Longitude | Geographic degrees east, within -180 to +180. |
| Local date | Gregorian civil date, supported from 1900 through 2100. The explicit offset selects a local-noon UTC estimate; the engine chooses the nearest upper solar transit and searches its preceding and following half-cycles. Evening Isha can therefore fall after local midnight. |
| UTC offset | Explicit signed offset used only for the first slice's local display and local-date interpretation. It is not an IANA time-zone identifier and contains no future DST rules. |
| Method profile | Versioned data specifying Fajr and Isha rules, Dhuhr/Maghrib adjustments, and any relevant rounding convention. |
| Asr criterion | Standard or Hanafi shadow criterion, when applicable to the method. |
| High-latitude rule | `none`; no substituted or adjusted time is generated. |
| Elevation | Assumed zero meters for the first slice. The engine does not infer elevation from GPS. |

Reject non-finite coordinates, out-of-range values, malformed dates, impossible offsets, and invalid profile parameters with typed errors. The library accepts valid caller-supplied profile IDs and revisions; it does not authenticate them against an institutional registry. The CLI currently exposes only the two built-in profiles. Keep input and output units explicit.

## Output

For each event, the core currently returns either:

- `Occurs`: a UTC instant rounded to the nearest second, raw UTC Unix seconds, and an `EventRule` describing the solar condition or adjustment.
- `Unavailable`: a typed reason when the selected condition has no crossing in the selected solar cycle.

The CLI converts occurring UTC instants to local civil clock text with the explicit fixed offset. It displays seconds; it does not round to a prayer timetable minute. The core has no separate `astronomical` or `method_adjusted` status enum yet; the `EventRule` records that distinction. No high-latitude-adjusted result is produced.

`CalculationRecord` currently contains coordinates, requested local date, fixed offset, astronomy model ID, method ID/revision/source/parameters, Asr selection, high-latitude rule `none`, assumed elevation, and engine version. It does not contain a time-zone database version or external reference-data version because neither is used. An application should preserve whether coordinates came from GPS, manual entry, a city, or a mosque timetable; the core does not infer that provenance.

## Separation of concerns

1. **Astronomy:** solar position, transit, and geometric/apparent event candidates under documented assumptions.
2. **Method rules:** selected twilight angle or fixed interval, Asr criterion, and stated offsets.
3. **Civil time:** conversion of UTC instants to a local clock under explicit rules.
4. **Presentation:** rounding, language, 12/24-hour format, and accessibility.
5. **Mosque data:** separately sourced timetable and iqamah, never substituted into the astronomical result without a label.

The core must expose missing astronomical events explicitly. It must not silently switch to a fallback, return a fabricated instant, or treat a mosque's iqamah as the calculated start.

## Implemented astronomy and rule decisions

- Solar declination and equation of time follow the NOAA equations derived from Meeus; Julian day uses UTC as a practical approximation to UT1. Model ID: `NOAA-MEEUS-SOLAR-2`.
- The apparent horizon is solar-center altitude −0.833° at sea level, combining mean refraction and solar radius. Terrain and actual atmospheric conditions are not modeled.
- Upper transit is solved iteratively from the equation of time. Morning and evening altitude crossings use direction-aware bracketing and bisection over the adjacent half-cycles. An interior extremum check prevents a same-sign endpoint pair from automatically being treated as no event.
- The built-in profile `research-15` v0.1 uses 15° depression for Fajr and Isha, zero Dhuhr and Maghrib adjustment, and no high-latitude fallback. It is a research parameter set, not an institutional method claim.
- The `mwl-angles-18-17` v0.1 profile uses the [PrayTimes MWL parameter table](https://praytimes.org/docs/methods): 18° Fajr, 17° Isha, and zero Dhuhr/Maghrib adjustment. Its source is stored with the profile. The name describes a published angle set, not institutional endorsement or an implementation of every PrayTimes rule.
- Dhuhr is upper solar transit plus its profile adjustment. Maghrib is apparent sunset plus its profile adjustment. Both built-in profiles currently use zero adjustment. Adjustment and presentation rounding must remain separately documented as profiles expand.
- Asr uses the chosen standard or Hanafi factor added to the shadow ratio fixed at the calculated upper transit; the resulting target altitude is solved on the afternoon side. A missing crossing is reported as unavailable.
- The engine rejects invalid or non-finite coordinates, invalid dates or offsets, and invalid method parameters. There is no network call or third-party crate dependency.
- Gregorian date conversion follows [Howard Hinnant's civil-date algorithms](https://howardhinnant.github.io/date_algorithms.html). The calculation record includes the fixed offset, model ID, method ID/revision and parameters, engine version, Asr criterion, and the absence of a high-latitude rule.

## Definitions to settle before production use

- Institutional method parameter sources and review process; `research-15` has no such attribution.
- Full time-zone and DST behavior, including versioned data and offline coordinate-to-zone mapping.
- High-latitude alternatives and more precise classification of unavailable events near grazing/tangency cases.
- Elevation, local horizon, and atmospheric adjustment policy.
- Display-minute rounding and notification scheduling policy.
- Broader reference coverage, numerical error budget, and peer review beyond the initial cases.

Do not treat these as implementation trivia. They affect displayed prayer times and must be documented with the first working code.
