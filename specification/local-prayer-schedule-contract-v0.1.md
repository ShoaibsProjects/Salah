# Local prayer schedule contract v0.1

**Status:** F2-TZ3 research implementation under review, with F2-TZ4 civil-date existence classification composed into the result. It accepts the 598 named IANA 2026d identifiers in the embedded pack; most independent schedule vectors still cover the original six-zone probe. It does not amend calculation contract v0.3, UTC-anchor contract v0.1, or the F2-TZ2 transit-membership rule. Phase 1 and Phase 2 remain open.

## Public operation

```rust
calculate_local_date_prayer_schedule(
    date,
    coordinates,
    zone_id,
    tzif_bytes,
    method,
    asr_criterion,
)
```

Inputs retain the F2-TZ2 meanings. The caller supplies the Gregorian date, validated coordinates, canonical zone ID, exact bytes from the pinned 2026d fixture, method profile, and Asr criterion. The operation parses and validates the zone bytes once, selects every solar cycle whose **rounded, unadjusted UTC transit second** maps to the requested local date, and converts that cycle's events with the same parsed zone rules.

The result carries the F2-TZ2 selection record, an independent `LocalDateExistence` classification, and schedule policy `local-date-prayer-schedule` revision `0.1`. It preserves `Zero | One | Multiple` cardinality. It never chooses among multiple cycles or invents a cycle for zero matches. `civil_date.status` distinguishes a skipped date from an existing date with no matched transit.

## Event representation

Each candidate retains the complete `UtcAnchorTimes` value and local transit. Its `LocalPrayerEvents` contains Fajr, sunrise, Dhuhr, Asr, sunset, Maghrib, and Isha as `LocalizedEvent` values:

- `Occurs` contains the unchanged core UTC second, local civil date/time, resolved offset seconds east of UTC, zone ID, TZDB version, raw unrounded solver value, and original `EventRule`.
- `Unavailable` contains the original `UnavailableReason` and no UTC or local timestamp.

There is no minute rounding in this operation. Local event dates are not forced to equal the requested date: an event such as Isha may occur after midnight on the following date. Existing rule distinctions remain intact, including sunset and Maghrib sharing an instant under separate core rules.

## Errors and coverage

Errors from pinned zone validation, civil date-window arithmetic, F2-TZ2 completeness, UTC-anchor calculations, and UTC-to-local range conversion propagate as `TimeError`. They are never converted into `Zero` or an unavailable prayer event. In particular, an occurring event whose local date falls outside `CivilDate`'s 1900–2100 range returns `LocalDateOutOfRange`; the adapter does not clamp it.

`Zero` retains F2-TZ2's meaning: no selected transit maps to the requested date. It does not by itself mean that the civil date was skipped. `Multiple` retains every matching cycle in transit-UTC order, with all seven events localized for each cycle.

## Determinism and privacy boundary

For identical inputs and fixture bytes, output depends only on the Salah core and the supplied TZif bytes. No host timezone, geolocation service, network, filesystem lookup, device setting, or coordinates-to-zone mapping is consulted. The result records the caller-selected zone and fixture version. Supported IDs and their exact 2026d byte ranges are listed in the global pack manifest.

## Evidence

`crates/salah-time/tests/local_prayer_schedule.rs` verifies:

- all seven Chicago events preserve their original UTC instant, raw solver value, rule/status, and pinned zone provenance;
- London Isha on 2026-06-21 remains the core's `2026-06-21T23:49:15Z` instant and localizes to `2026-06-22 00:49:15` under the 2026d London rules;
- the F2-TZ2 zero and multiple outcomes remain explicit and each returned event is localized;
- the 23-hour Chicago date with zero matched transits is still classified as existing;
- unavailable polar sunrise/sunset statuses remain unavailable without fabricated local timestamps.

These cases validate composition for a small fixture set. They do not establish global time-zone coverage, coordinate-to-zone correctness, accepted high-latitude rules, religious endorsement, or production timetable accuracy. Mobile, WASM, full offline data distribution, and notification behavior remain outside this contract.
