# Civil-time data assessment contract v0.1

**Status:** implemented advisory in `salah-engine`; not a Phase 2 gate or an accuracy certificate.

## Purpose

The installed IANA rules and boundary map are snapshots. A schedule remains reproducible under their recorded versions, but a future government decision can change its local clock labels. `assess_civil_time_data` gives a caller two deterministic prompts to review the installed data. It does not alter UTC prayer events, offsets, zone selection, or the schedule.

## Inputs and result

The function receives a `SelectedLocalDaySchedule` and an explicit `observed_on: CivilDate`. It reads no system clock or network source. The result records policy `installed-civil-time-data-assessment` revision `0.1`, the observed and requested dates, the boundary version, the exact [rule-pack identity](offline-tzif-pack-interface-v0.1.md) (IANA version, release year, schema, blob hash, and zone-inventory hash), and an ordered list of notices:

1. `PackPredatesObservationYear` when `observed_on.year() > selected.schedule.record.rule_pack.release_year`. The installed IANA and matching boundary snapshots are from an earlier year. This is a prompt to check for a newer reviewed pack; it does not assert that one exists or that an offset is wrong.
2. `RequestedDateIsFuture` when the requested Gregorian date is later than `observed_on`. Future civil-time legislation may change that date's local clock labels, including within the release year.

The comparison uses checked Gregorian dates and the core's day-number conversion. Both notices may occur. An empty list means neither condition is met; it does **not** establish that the boundary, historical rule, future law, device clock, or prayer method is correct.

The caller must supply the actual date it intends to use for this assessment. If a device clock is uncertain, the caller should allow the user to correct it. The assessment does not infer the user's location or current date.

## Compatibility and limits

`TZDB_RELEASE_YEAR = 2026` belongs to the same embedded 2026d release as `TZDB_VERSION`. Both must change with a future pack and manifest. `salah-engine` already rejects a selected boundary map whose IANA version differs from the installed rule pack. Its returned zone selection retains the original candidate list and selection origin.

The advisory does not inspect each zone's transition table, verify a government's latest announcement, establish whether a newer IANA release exists, or claim that dates through 2026 are settled. Slim TZif files can project POSIX footer rules far beyond the data release; this policy does not treat those projections as known future law. It also does not rate historical IANA coverage or the approximate boundary geometry. Those are separate validation and update concerns.

## Evidence

The selected-day integration case checks an empty notice list on its observation date, the future-date notice for the prior observation date, and the earlier-pack-year notice for a 2027 observation date. This tests the advisory rule and version propagation, not independent civil-time accuracy.

See the [data lifecycle decision](../docs/civil-time-data-lifecycle.md) for the update and correction path.
