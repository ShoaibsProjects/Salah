# Civil-date solar-transit selector v0.1

**Status:** research implementation in `salah-time`; F2-TZ2. It supports only the six exact IANA 2026d TZif fixtures recorded in `crates/salah-time/fixtures/manifest.json`. It does not amend calculation contract v0.3 or UTC-anchor contract v0.1. Phase 1 and Phase 2 remain open.

## Public contract

`select_local_date_transits(date, coordinates, zone_id, tzif_bytes, method, asr_criterion)` returns a `LocalDateTransitSelection` with:

- `LocalDateTransitRecord`: requested Gregorian date, coordinates, caller-selected zone ID, tzdb version, validated method and Asr criterion, astronomy model ID, core engine version, and selector policy `local-date-solar-transit` revision `0.1`;
- `LocalDateTransitMatches::Zero`, `One(candidate)`, or `Multiple(candidates)`;
- each candidate's complete `UtcAnchorTimes` from `salah-core` and matching `LocalCivilTime` for its selected transit.

Candidates are sorted by rounded transit UTC seconds and deduplicated by that value. Invalid zone IDs, malformed or non-pinned bytes, calculation failures, unstable canonicalization, arithmetic overflow, or incomplete coverage return explicit errors; none is converted into `Zero`.

Membership uses the **rounded, unadjusted `selected_transit`** UTC second, then converts that instant to local time using the same pinned TZif bytes. Dhuhr's method adjustment is not used to choose a date. This follows the core's versioned second-precision event; rounding can affect date membership only within one second of local midnight. Other prayer events remain unchanged UTC events in each `UtcAnchorTimes` record. This selector does not yet format every event locally.

`Zero` means no matching selected transit. It does not prove that the civil date was skipped. Apia 2011-12-30 is independently known to be skipped from the pinned TZif evidence. An existing short DST date can also contain zero transits for a deliberately mismatched coordinate and zone. A future API may separately classify whether the date itself exists.

## Complete bounded enumeration

For validated date `D`, let `D0 = days_since_unix_epoch(D) * 86_400`, using checked integer arithmetic. Jiff's `Offset` is bounded to −25:59:59 through +25:59:59, or ±93,599 seconds. Every UTC integer second that could convert to local date `D` lies in the inclusive interval:

```text
[D0 - 93_599, D0 + 86_399 + 93_599]
```

The implementation intersects this interval with `UTC_ANCHOR_MIN_UNIX_SECONDS..=UTC_ANCHOR_MAX_UNIX_SECONDS` and places anchors no more than 21,600 seconds apart, including both endpoints. `CivilDate` supports years 1900 through 2100. At the first supported date, the earliest possible matching transit is less than two hours before the minimum anchor; at the last supported date, the latest possible matching transit is less than two hours after the maximum anchor. Other requested dates' candidate intervals lie inside the anchor range.

The grid completeness argument uses the selected core model and its published selector. Let:

```text
q(A) = (A + E(A) + longitude_degrees * 240) / 86_400
index = round(q(A) - 0.5)
```

where `E` is the equation of time in seconds. For Julian centuries covering this search, the implemented coefficients give `|eccentricity| < 0.017`, `|obliquity| < 24°`, and `y = tan(obliquity/2)^2 < 0.046`. Bounding every sine/cosine by one bounds the equation-of-time bracket by:

```text
0.046 + 2(0.017) + 4(0.017)(0.046) + 0.5(0.046)^2 + 1.25(0.017)^2 < 0.085 radians
```

The model multiplies this bracket in degrees by four minutes per degree, giving `|E| < 20 minutes`; the proof uses the looser bound `|E| < 1 hour`.

For the exact fixed point `t` at solar-day index `n`, the transit equation gives `q(t) = n + 0.5`. The eight-iteration solver's returned value is within two hours of `t`: the last iterate is `C - E(x)` and `t = C - E(t)`, while any two equation-of-time values differ by less than two hours under the bound above. If anchor `A` is within six hours of the returned (rounded) transit, then it is within eight hours plus at most one second of `t`. Thus:

```text
|q(A) - (n + 0.5)| < (8 hours + 2 hours + 1 second) / 24 hours < 1/2
```

Therefore `round(q(A) - 0.5)` remains `n`. A grid with spacing at most six hours has an anchor within three hours of any transit inside its endpoints. A matching transit outside the clipped endpoints is within two hours of an in-range endpoint. Both are inside the six-hour guarantee, so every transit whose rounded second could map to `D` is discovered under the F2-TZ1 selector.

For each distinct discovered rounded transit, the implementation recalculates using that instant clamped to the F2-TZ1 anchor range as a canonical anchor. At the range edges the clamp is less than two hours away; the same bound guarantees selection of the same cycle. If the canonical calculation selects another transit, return `TransitNotStable`. Convert each canonical selected transit with the already-validated zone object; retain it only when the resulting local date equals `D`. A nonmatching candidate whose local date falls outside 1900–2100 is safely excluded because it cannot equal a validated requested date.

## Evidence

The integration suite covers:

- one match for an ordinary Chicago date;
- London on a DST transition date;
- zero matches on an existing 23-hour Chicago spring date using a deliberately mismatched longitude near 97.5°E;
- two matches on Chicago's 25-hour fall-back date at the same longitude;
- Apia's independently evidenced skipped 2011-12-30 date;
- Kiritimati UTC+14 and Kathmandu UTC+05:45;
- first and last supported Gregorian dates;
- a UTC-anchor selection boundary, candidate sorting, method/model/zone provenance, and malformed TZif rejection.

No coordinates-to-zone mapping is performed. The six-zone fixture pack is a probe, not global civil-time coverage. A zero, one, or multiple result does not establish global accuracy, a religious ruling, or a consumer timetable.
