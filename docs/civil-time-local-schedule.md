# F2-TZ3/F2-TZ4 — local schedule and civil-date classification

**Status:** F2-TZ3 local event conversion and F2-TZ4 local-date existence are implemented in the working tree for architect review. They build on the F2-TZ2 selector and accept the 598 identifiers in the pinned IANA 2026d pack. Independent transition and schedule evidence remains concentrated in the original six-zone probe. They pass neither Phase 1 nor Phase 2.

## Why this slice

F2-TZ2 answers which solar cycle belongs to a requested zone-local date and explicitly returns zero, one, or multiple matches. Its cycle events remain UTC. This slice converts each of those seven events under the same validated TZif rules and keeps the event's original UTC second, raw solver value, event rule, and unavailable status.

The result is now a usable local schedule data structure for research and later presentation work. It also says whether the requested civil date exists in that zone, separately from whether a solar transit matched. It is not a consumer UI: it applies no display-minute rounding, selects no winner when multiple cycles match, and does not change an unavailable event into a guessed fallback.

## Implementation boundary

The public operation is `salah_time::calculate_local_date_prayer_schedule`. It takes the same explicit date, coordinates, zone ID and exact fixture bytes, method, and Asr criterion as the F2-TZ2 selector. It validates/parses the pinned zone once, reuses the complete transit enumeration, then converts each occurring event through that same parsed zone. Unavailable events carry their original reason without a timestamp.

Each schedule result carries the selected date, location, method, core/model version, zone and TZDB provenance, transit selector policy, date-existence policy, and schedule policy `local-date-prayer-schedule` revision `0.1`. Every occurring event includes its actual local date; it may fall on an adjacent date. F2-TZ2 zero and multiple outcomes remain explicit. The classifier uses TZif transition intervals to distinguish a skipped date from an existing date with no matching transit.

See the [local prayer schedule contract](../specification/local-prayer-schedule-contract-v0.1.md) and [civil-date existence contract](../specification/civil-date-existence-contract-v0.1.md) for the exact data semantics and errors.

## Still out of scope

- Inferring a timezone from coordinates or using a global zone pack.
- Choosing regional defaults or claiming that a calculation is religiously authoritative.
- High-latitude substitution, elevation, horizon/refraction changes, or formula changes.
- Display-minute policy, next-prayer UI, notifications, app bindings, or a consumer release.
- Global IANA data distribution, coordinate-to-zone mapping, and policy for locations at borders, at sea, or in flight.

The next engineering work is coordinate-to-zone source/licensing and border policy, followed by a Rust API portability probe and broader independent transition evidence. The embedded named-zone pack removes the six-zone data limit but does not answer which zone belongs to a coordinate. These tasks do not replace the still-open Phase 1 astronomy and method reviews.
