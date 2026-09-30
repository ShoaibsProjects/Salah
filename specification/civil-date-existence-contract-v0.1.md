# Local civil-date existence contract v0.1

**Status:** F2-TZ4 research implementation under review. The classifier accepts exact IANA 2026d TZif entries from the 598-name pack. It composes with F2-TZ3 schedule data but does not alter prayer-event calculation, transit selection, or presentation. Phase 1 and Phase 2 remain open.

## Public operation and result

```rust
classify_local_date(date, zone_id, tzif_bytes)
```

The caller supplies a validated Gregorian `CivilDate`, one supported canonical zone ID, and its exact pinned TZif bytes. The result records the requested date, zone, TZDB version, and policy `local-date-existence` revision `0.1`.

`LocalDateStatus::Exists` carries every inclusive UTC-second interval that maps to the local date. Intervals are partitioned at time-zone transitions and carry the corresponding offset in seconds east of UTC. More than one interval is allowed, since historical offset changes can repeat parts or all of a date. `Skipped` means the interval set is empty under the supplied TZif rules.

This status is independent of whether a solar transit matches the date. A schedule can therefore report either:

- `Exists` plus zero transit matches: the date exists, but the selected solar-cycle policy found no transit on it;
- `Skipped` plus zero transit matches: the zone's rules skip the civil date;
- `Exists` plus one or multiple transit matches: return each cycle explicitly.

The classifier does not infer the zone from coordinates and does not label a prayer event.

## Complete transition-interval algorithm

Let `D0` be the checked Unix-second value for midnight at date `D`, interpreted as a numeric civil-day boundary. Jiff offsets are bounded to ±93,599 seconds, so every UTC second that could map to `D` lies within:

```text
[D0 - 93_599, D0 + 86_399 + 93_599]
```

The implementation obtains the actual offset at the first second in that window from the parsed pinned zone. It enumerates `TimeZone::following` transitions through the inclusive last second, including rules derived from the TZif POSIX footer. These transition instants partition the UTC window into inclusive integer-second segments with constant offsets. For a segment `[s,e]` and offset `o`, its intersection with local date `D` is:

```text
[max(s, D0 - o), min(e, D0 + 86_399 - o)]
```

If the lower endpoint is not greater than the upper endpoint, the inclusive UTC interval and `o` are recorded. If no segment intersects, the date is skipped. All day-boundary and interval arithmetic is checked; errors remain typed failures and cannot become `Skipped`.

At a transition second, the new offset applies. The preceding segment therefore ends one second before the transition and the next segment starts at the transition. The public interval is exact at second precision, consistent with the TZif transition format and `UtcInstant` API. This classifier does not model leap seconds.

## Composition with prayer schedules

`LocalDatePrayerSchedule::civil_date` carries the classifier result. F2-TZ3's transit selection still uses the rounded, unadjusted selected transit and keeps zero/one/multiple results explicit. Classification adds information about the civil calendar only; it does not select a cycle, repair a missing solar event, apply a high-latitude rule, or round clock output.

The implementation parses and verifies the pinned TZif bytes once, then reuses the resulting zone object for date classification, cycle enumeration, and all seven event conversions. No host zone database, network, filesystem lookup, device timezone, or coordinate-to-zone mapping is consulted.

## Evidence and limits

`crates/salah-time/tests/local_date_existence.rs` checks Chicago's 23-hour 2026 spring date, 25-hour 2026 fall date, the Apia 2011 skipped date, ordinary and fractional-offset dates, and Chicago's 2099 POSIX-footer transition. The schedule tests verify that a zero-transit but existing 23-hour date remains distinct from a skipped date.

The classifier's integration suite uses `Etc/UTC`, `America/Chicago`, `Europe/London`, `Asia/Kathmandu`, `Pacific/Kiritimati`, and `Pacific/Apia` plus Chicago's 2099 footer rule. The pack contains 598 named IANA IDs, but this does not establish independent historical validation of every zone, correctness of coordinate-to-zone lookup, religious-method acceptance, or production timetable accuracy.
