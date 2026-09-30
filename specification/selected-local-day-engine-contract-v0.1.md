# Selected local-day engine contract v0.1

**Interface additions:** the [runtime snapshot contract](runtime-rule-snapshot-v0.1.md) extends snapshot selection and owned provenance; [F3-C1](named-zone-interface-contract-v0.1.md) adds direct manual zone selection, named-zone CLI, and a shared JSON encoder using pinned `serde_json`. Historical calculation behavior below is preserved.

**Status:** experimental orchestration in `salah-engine`; not a Phase 2 gate decision.

## Purpose

Compose the existing solar/prayer kernel, explicit time-zone choice, pinned civil-time rules, and local-date scheduler into one public Rust call. The facade adds no astronomy, prayer formula, calculation method, location guess, or display rounding.

## Required inputs

`calculate_selected_local_day_schedule` requires:

1. A `ZoneSelection` created by explicitly confirming a `salah-location` suggestion or manually choosing a supported IANA identifier.
2. The requested Gregorian local date.
3. A `MethodProfile` supplied by the caller.
4. An `AsrCriterion` supplied by the caller.

There is no default method, Asr criterion, time zone, or local date. The facade does not read device settings, current time, network state, or host zoneinfo.

## Processing and output

1. Check that the zone-selection TZDB version and boundary-data version agree with the engine's pinned IANA pack.
2. Resolve the selected identifier to its TZif bytes from `salah-time`'s offline 2026d pack.
3. Call `calculate_local_date_prayer_schedule` using the coordinates and IANA zone carried in the explicit selection.
4. Return both the original selection evidence and the local schedule.

The local schedule preserves its separate civil-date existence result and solar-cycle cardinality (`Zero`, `One`, `Multiple`). Occurring events preserve the core UTC second, raw solver value, and event rule when localized. Unavailable events remain unavailable. No schedule candidate is chosen when there are zero or multiple cycles.

Errors remain typed as `EngineError::IncompatibleZoneSelection` or `EngineError::Time(TimeError)`. There is no fallback to a device zone or remote service.

## Boundaries

- The caller explicitly chooses the date and method; the facade does not establish a religious default or current-day behavior.
- A single boundary-map candidate is still not a zone selection until explicitly confirmed. A manual override is preserved as such in the returned result.
- The selected zone controls local civil rendering and date matching; it does not change solar geometry at the selected coordinates.
- High-latitude substitutions remain absent. A core event can be unavailable.
- DST, skipped civil dates, and zero/multiple transit outcomes remain explicit.
- The facade does not provide an app UI, persistent location storage, notifications, mosque timetable, or an accuracy/certification claim.

## Validation

The integration checks exercise a confirmed Minneapolis schedule, a manual Kathmandu zone override on the same coordinates, an Apia skipped date, an existing spring-DST date with zero matching transits, and explicit method/Asr inputs. These verify composition and provenance only. They do not independently validate the astronomy, boundary source, religious method, or every IANA transition.

## Dependency and maintenance

`salah-engine` depends only on the workspace's `salah-core`, `salah-time`, and `salah-location` crates. It adds no external dependency. `salah-location` embeds the ODbL timezone-boundary dataset described by [coordinate-zone selection contract v0.1](coordinate-zone-selection-contract-v0.1.md). The data and rule version must be updated together through a reviewed release process.

The separate [civil-time data assessment v0.1](civil-time-data-assessment-contract-v0.1.md) adds advisory notices against a caller-supplied observation date. It does not change this calculation contract or its returned schedule.
