# UTC-anchor contract v0.1

**Status:** additive research interface in `salah-core` (F2-TZ1). This
contract does not amend, reinterpret, or replace the historical
[calculation contract v0.3](calculation-contract-v0.3.md). The legacy
`CalculationInput` / `PrayerTimes` / `CalculationRecord` /
`calculate_prayer_times` API, its method IDs/revisions, formulas, event
adjustment/rounding, high-latitude behavior, and historical results are
unchanged. No civil-date policy, time-zone lookup, or consumer accuracy claim
follows from this contract. Phase 1 and Phase 2 gates remain open.

## Purpose

The legacy core selects a solar cycle from a civil date plus a
minute-precision fixed UTC offset by way of a local-noon UTC estimate. An
IANA zone can change offset inside that date, and historical zone offsets can
carry seconds, so feeding a zone-resolved offset into the legacy input would
misstate its meaning. The time-zone layer needs a core operation whose
selection input is an **explicit UTC instant**. This contract defines that
operation without choosing any civil-date policy.

## Public interface

```rust
pub const UTC_ANCHOR_SELECTION_POLICY_ID: &str = "utc-anchor-nearest-transit";
pub const UTC_ANCHOR_SELECTION_POLICY_REVISION: &str = "0.1";
pub const UTC_ANCHOR_MIN_UNIX_SECONDS: i64 = -2_209_075_200; // 1899-12-31T00:00:00Z
pub const UTC_ANCHOR_MAX_UNIX_SECONDS: i64 = 4_134_067_199;  // 2101-01-01T23:59:59Z

pub struct UtcAnchorInput {
    pub coordinates: Coordinates,   // validated; degrees, east-positive longitude
    pub anchor: UtcInstant,          // validated range below; Unix seconds (i64)
    pub method: MethodProfile,       // validated; same IDs/revisions as v0.3
    pub asr_criterion: AsrCriterion, // Standard | Hanafi
}

pub struct UtcAnchorTimes {
    pub fajr: Event,
    pub sunrise: Event,
    pub dhuhr: Event,
    pub asr: Event,
    pub sunset: Event,
    pub maghrib: Event,
    pub isha: Event,
    pub selected_transit: UtcInstant,                  // rounded upper transit
    pub unrounded_selected_transit_unix_seconds: f64,  // solver value before rounding
    pub record: UtcAnchorRecord,
}

pub struct UtcAnchorRecord {
    pub coordinates: Coordinates,
    pub anchor: UtcInstant,
    pub selected_transit: UtcInstant,
    pub unrounded_selected_transit_unix_seconds: f64,
    pub selection_policy_id: &'static str,       // "utc-anchor-nearest-transit"
    pub selection_policy_revision: &'static str, // "0.1"
    pub method: MethodProfile,
    pub asr_criterion: AsrCriterion,
    pub astronomy_model: &'static str, // "NOAA-MEEUS-SOLAR-2", unchanged
    pub engine_version: &'static str,
    pub high_latitude_rule: &'static str, // "none"
    pub assumed_elevation_meters: f64,    // 0.0
}

pub fn calculate_utc_anchor_times(input: UtcAnchorInput)
    -> Result<UtcAnchorTimes, CalculationError>;
```

`Event`, `EventRule`, `UnavailableReason`, `Coordinates`, `UtcInstant`,
`MethodProfile`, and `AsrCriterion` are the existing `salah-core` types with
unchanged meaning. The new record carries **no requested local date and no
fixed offset** by construction.

## Input units and supported range

| Field | Units and constraint |
| --- | --- |
| Coordinates | Validated decimal degrees; latitude −90…+90, longitude −180…+180; non-finite rejected with `InvalidCoordinates`. |
| Anchor | `UtcInstant.unix_seconds` (`i64`), whole SI seconds since 1970-01-01T00:00:00Z, in `UTC_ANCHOR_MIN_UNIX_SECONDS..=UTC_ANCHOR_MAX_UNIX_SECONDS`, i.e. 1899-12-31T00:00:00Z through 2101-01-01T23:59:59Z inclusive. Outside this range the function returns `CalculationError::InvalidUtcAnchor` before any astronomy. |
| Method profile | Same validation as v0.3 (`InvalidMethod` on failure); IDs/revisions/parameters unchanged. |
| Asr criterion | Standard (factor 1) or Hanafi (factor 2). |

The anchor range covers every legacy local-noon UTC estimate
(−2_208_996_000 for 1900-01-01 at +14:00 through 4_133_988_000 for
2100-12-31 at −14:00) with roughly a day of margin at each end, while
keeping the transit iteration, half-cycle offsets (±43_200 s), and
`round()`-to-`i64` conversions inside finite `f64`/`i64` arithmetic. Unix
values in this range are exactly representable as `f64` (magnitude well
below 2⁵³), so `anchor as f64` is exact.

## UTC-anchor selection algorithm

Let `A` be the anchor in Unix seconds as `f64`, `lon_sec` be
`longitude_degrees * 240.0`, and `E(t)` be the equation of time in seconds
from the unchanged `NOAA-MEEUS-SOLAR-2` position at `t`:

1. `index = round((A + E(A) + lon_sec) / 86_400 − 0.5)`.
2. Starting from `estimate = A`, iterate eight times:
   `estimate = (index + 0.5) * 86_400 − E(estimate) − lon_sec`.
3. If the final estimate is non-finite, return `NumericalFailure`. Otherwise
   it is the unrounded selected upper transit.

This is the same iteration as legacy `upper_transit`, with the legacy
local-noon estimate (`local_day_start_utc as f64 + 43_200.0`) replaced by `A`.
Both paths call one shared `upper_transit_from_noon_estimate` routine, and
the seven events are produced by one shared `events_from_transit` routine, so
there is a single copy of the transit solver and prayer formulas.

Equivalence property: when `A` equals the legacy local-noon UTC estimate
(`days_since_unix_epoch * 86_400 − offset_minutes * 60 + 43_200`), the
anchor path selects the identical `f64` transit and returns
event-for-event identical rounded UTC instants, unrounded seconds, rules,
and unavailable reasons as the legacy path. Both `f64` noon values are exact
integer arithmetic below 2⁵³, so the substitution is bitwise identical.
`crates/salah-core/tests/utc_anchor.rs` asserts exact equality (not a time
tolerance) for the Minneapolis/Makkah/Sydney/Quito/leap-day reference
inputs, Hanafi Asr, the `mwl-angles-18-17` profile, a custom
Dhuhr/Maghrib-adjustment profile, a pre-epoch case (Sydney 1900-12-21 at
+10:00), date-line/offset extremes (Kiritimati +14:00, London 2100-06-21),
and polar-day/night unavailable patterns.

## Boundary behavior

The selector partitions the anchor line into solar-cycle cells about one
solar day wide. The cell index changes where
`(A + E(A) + lon_sec) / 86_400` crosses an integer; because `E(A)` depends
on `A`, the boundary is implicit rather than a fixed clock time. The
implementation uses Rust `f64::round`, whose half-way rule is away from
zero. Therefore an anchor exactly on a boundary belongs to the higher-index
cell when the rounded operand is nonnegative and the lower-index cell when
it is negative; this tie rule is part of policy revision 0.1. Adjacent cells
select adjacent transits roughly 82_800–90_000 s apart. The implementation
exposes the discontinuity: two anchors one second apart on opposite sides
of a boundary return different transits and correspondingly different event
sets. It does not blend, interpolate, or snap them.

Observed example (Minneapolis 44.9778°, −93.2650°; `research-15`; Standard;
`--nocapture` prints the boundary in `utc_anchor.rs`):

- transit 0: Unix 1790532239 (2026-09-27T18:03:59Z);
- transit 1: Unix 1790618618 (2026-09-28T18:03:38Z); gap 86_379 s;
- anchor 1790575428 (2026-09-28T06:03:48Z) selects transit 0;
- anchor 1790575429 (2026-09-28T06:03:49Z) selects transit 1.

Anchors well inside a cell are stable: `transit − 6 h`, `transit`, and
`transit + 6 h` all select the same transit in the test.

## Output provenance

- `selected_transit` is the rounded upper transit; with the built-in
  zero-adjustment profiles it equals Dhuhr's UTC instant and its unrounded
  value equals Dhuhr's unrounded seconds. With nonzero `dhuhr_adjustment_seconds`,
  Dhuhr is the rounded transit plus that adjustment; `selected_transit`
  remains the unadjusted transit.
- The seven events reuse the v0.3 solar conditions, Asr shadow rule fixed at
  the calculated transit, Dhuhr/Maghrib adjustments, nearest-second rounding,
  `EventRule` values, and `UnavailableReason::NoCrossingInSolarCycle`. No
  high-latitude substitution is added (`high_latitude_rule: "none"`).
- `astronomy_model` remains `NOAA-MEEUS-SOLAR-2`; the interface addition is
  not a new physical model. `engine_version` is the `salah-core` package
  version, as in v0.3 records.
- Overflow is reported, never panicked on or saturated: non-finite
  transit/half-cycle endpoints, out-of-`i64` rounded seconds, and failed
  `checked_add` of Dhuhr/Maghrib adjustments all return
  `CalculationError::NumericalFailure`. The legacy path uses the same checked
  conversions; all historical in-range results are unchanged (full workspace
  suite passes).

## What this API does not assert

A UTC anchor alone does **not** prove that the selected transit belongs to
any requested local date under any civil time zone. Date membership requires
converting candidate transits through explicitly versioned IANA rules and
handling zero or multiple matches (skipped days, overlaps, date-line
carriage). That selector is F2-TZ2 and is intentionally absent here. Callers
must not relabel `anchor` as a local date, feed a zone-resolved offset into
`CalculationInput` as if it carried zone meaning, or present anchor results
as a consumer timetable.

## Review gate

Accept F2-TZ1 only if the legacy API and CLI bytes are preserved, the new
record carries truthful UTC-only provenance with the versioned selector ID,
selection boundaries are explicit and discontinuous, range/overflow behavior
is typed, and the seam suffices for F2-TZ2. Passing this packet does not
pass Phase 1 or Phase 2.
