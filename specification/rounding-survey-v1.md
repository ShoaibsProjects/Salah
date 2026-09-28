# Rounding-options survey v1 (P1.5-E1)

**Status:** evidence survey for an architect decision. This document selects **no**
final rounding or ordering rule, implements nothing, and amends no existing
contract. Behavior today is defined by the [calculation contract
v0.3](calculation-contract-v0.3.md); that contract is preserved as historical
and is only described here. Phase 1 remains open; no consumer accuracy claim
follows from this survey.

**Scope of this survey:** sourced and hypothetical options for converting a raw
UTC event instant into (a) an adjusted prayer beginning, (b) a displayed
timetable minute, and (c) a later notification instant. The central ordering
question is whether Dhuhr/Maghrib adjustments apply **before or after**
minute rounding, per method where sources differ, plus any further orderings
found in sources. Every option states its exact operation order, what its
source actually establishes, and its gaps. A sourced practice is labeled
**sourced**; a comparison aid invented for this survey is labeled
**hypothetical** and is never presented as an implemented method or a
reference observation.

**Sources consulted (with identity and date):**

| # | Source | Identity / retrieval | What was used |
| --- | --- | --- | --- |
| S1 | Calculation contract v0.3 | Repo file `specification/calculation-contract-v0.3.md`, current research record for `salah-core` v0.3.0 | Second-precision output, zero adjustments, no minute rule, sunset/Maghrib separation, unavailable status |
| S2 | Accuracy budget draft 0.1 | `specification/accuracy-budget.md` | Result chain (raw UTC → rounding → civil display → notification) as separate stages |
| S3 | Reference cases | `specification/reference-cases.md` | Adhan/PrayTimes comparison configuration, Isha-after-midnight note, illustrative-vs-reference warning |
| S4 | Prayer-library matrix + report | `data/reference/prayer-library-v1.tsv`, `specification/prayer-library-v1-report.md`; source run 2026-09-27 | Second/ms UTC vectors, `Rounding.None`, zero adjustments, fallback/status labels, row IDs |
| S5 | USNO matrix + report | `data/reference/solar-usno-v1.tsv`, `specification/usno-matrix-v1-report.md`; USNO API 4.0.1 retrieved 2026-09-27 UTC | Minute-formatted solar vectors, case IDs, no-event cells |
| S6 | Method register v1 | `specification/method-register-v1.md`, created 2026-09-27 | Zero Dhuhr/Maghrib adjustments in both profiles; secondary-only provenance; review vacancies |
| S7 | Matrix generator (audit tool) | `tools/generate_prayer_matrix.cjs` | Exact library options used: PrayTimes `adjust({…, dhuhr:'0 min', maghrib:'0 min', …, highLats:'None'})` + `Float` output; Adhan `CalculationParameters('Other', fajr, isha, 0, 0)` + `rounding = Rounding.None` |
| S8 | Rust schema (read-only) | `crates/salah-core/src/prayer.rs`, `method.rs`, `civil.rs`, `lib.rs`; `crates/salah-cli/src/main.rs` — read for description only, not edited | `EventRule` variants, `Event::Occurs` fields, adjustment application order at second precision, CLI second display |
| S9 | PrayTimes artifacts (secondary) | Methods-table webpage `https://praytimes.org/docs/methods` and v2 JS file `https://praytimes.org/code/v2/js/PrayTimes.js` (SHA-256 `f0e8442410446b1bf2db613bdebd3aa3aef5cdf5483068fbd9212d5acaa180fd`, file only), both retrieved 2026-09-27 per the method register | MWL 18°/17° angles; minute-granularity adjustment settings exist upstream |
| S10 | Adhan JS (secondary) | npm package `adhan` 4.4.6 (hashes in the prayer-library report) | A `rounding` parameter exists and the matched run set it to `None`; zero adjustments in the matched run |

**Not consulted / not established:** no primary institutional document stating a
per-method rounding or adjustment-ordering rule was searched for or found in
this packet; the absence of such a document anywhere is **not** established.
Upstream PrayTimes `tune`/adjust internals and Adhan non-`None` rounding-mode
semantics were not audited here and are recorded as unknown (see §5).

## 1. Definitions used in this survey

- **Raw UTC instant:** the solver's unrounded Unix-seconds value (`f64`) for one
  solar condition in the selected cycle.
- **Second beginning:** the raw instant rounded to the nearest integer second
  (current kernel behavior, round-half-away via `f64::round`).
- **Adjusted prayer beginning:** the second beginning plus the method's
  Dhuhr/Maghrib adjustment in whole seconds (current profiles: 0 s, so the
  adjusted beginning equals the second beginning).
- **Displayed timetable minute:** a future whole-minute presentation value.
  No rule for it exists; §2 lists candidate rules without selecting one.
- **Notification instant:** a future device-side fire time derived from a
  displayed or adjusted beginning plus lead-time and OS scheduling semantics.
  It is a **separate unscheduled concept**: no notification behavior exists
  (contract v0.3; accuracy budget), and nothing in this survey schedules one.

## 2. Options table

| ID | Class | Exact operation order | Source | What the source actually establishes | Gaps |
| --- | --- | --- | --- | --- | --- |
| O0 | Sourced (current behavior) | raw `f64` → round to nearest second → add integer-second adjustment → expose both `utc` (second) and `unrounded_utc_unix_seconds` (`f64`); CLI prints seconds; statuses stay `Unavailable` | S1, S8 (`astronomical_event`; Dhuhr `transit.round() + adjustment`; Maghrib `utc.unix_seconds + adjustment`; CLI second display) | Second precision with raw seconds preserved; adjustment-after-second-rounding at second level; no minute rounding exists; sunset and Maghrib are separate events | Establishes nothing about minute display or notification; see §6 for the schema gap |
| O1 | Sourced practice (comparison configuration) | Adhan 4.4.6 run with `rounding = Rounding.None` and zero manual/method adjustments (S7) | S4, S7, S10 | No library display-minute rounding or adjustments were requested in the matched P1.3 run; its returned UTC values are comparison inputs, not a display prescription or proof of unlimited raw precision | Per-mode semantics of Adhan's other rounding settings, and whether Adhan applies adjustments before or after rounding, were not exercised; **unknown** |
| O2 | Sourced practice (comparison configuration) | PrayTimes v2 run with `dhuhr='0 min'`, `maghrib='0 min'`, no explicit `tune` call, `highLats='None'`, `Float` output (S7) | S4, S7, S9 | That the matched comparison explicitly requested zero Dhuhr/Maghrib adjustments with floating-hour output; upstream exposes those adjustment settings | The effect of any default tuning and the adjust-vs-format order were not audited here; **unknown** |
| O3 | Sourced precision property, **not** a rule | USNO API returns `HH:MM` clock strings; manifest converts them to UTC minute values (S5) | S5, `data/reference/README.md` | Source quantization: a minute-formatted source can differ from an equivalent raw-second result by quantization alone | Establishes no display rule for Salah; must not be mistaken for a rounding prescription |
| O4-F | Hypothetical comparison option | adjusted beginning (seconds) → floor to minute | None (survey construct) | Nothing normative; illustrates the most conservative minute mapping | Not sourced; interaction with adjustment order shown in §4 |
| O4-N | Hypothetical comparison option | adjusted beginning (seconds) → nearest minute, halves up | None (survey construct) | Nothing normative; illustrates wall-clock-conventional mapping | Not sourced; halves rule would itself need a decision |
| O4-C | Hypothetical comparison option | adjusted beginning (seconds) → ceiling to minute | None (survey construct) | Nothing normative; illustrates the least-early mapping | Not sourced |
| O5-A | Hypothetical ordering variant | adjustment **before** minute rounding: `display(round_minute(raw + adj))` | None (survey construct) | Nothing normative; pairs with any O4 rule | Indistinguishable from O5-B while adjustments are zero |
| O5-B | Hypothetical ordering variant | minute rounding **before** adjustment: `round_minute(round_minute(raw) + adj)` if the same rule is applied again to obtain a whole-minute result | None (survey construct) | Nothing normative; pairs with any O4 rule only after the final minute mapping is specified | Indistinguishable from O5-A while adjustments are zero; omitting the final mapping leaves a seconds-bearing value rather than a displayed minute |

**Second-rounding correction (architect review):** `MethodProfile`
adjustments are integer seconds (`dhuhr_adjustment_seconds: i32`,
`maghrib_adjustment_seconds: i32`), and the current kernel rounds the raw
instant to the nearest second using `f64::round` (halves away from zero),
**then** adds the adjustment. The proposed identity
`round(x + a) = round(x) + a` is not universal: `x = -0.5` Unix seconds and
`a = +1` second give `round(0.5) = 1` on the left but
`round(-0.5) + 1 = 0` on the right. The supported 1900–2100 date range
includes the Unix epoch, so an architect decision must preserve the actual
second-level order rather than rely on commutation. The separate
**minute-display** order (O5-A vs O5-B) remains to be decided.

## 3. Worked examples from cited vectors (all adjustments zero)

Conventions: "cited instant" values come from the TSV row stated; "kernel
beginning" values are computed with `salah-cli` v0.3.0 (`NOAA-MEEUS-SOLAR-2`)
on 2026-09-27 for illustration — they are **kernel-computed illustrations,
not reference observations**. Candidate displayed minutes apply the
hypothetical O4 rules to the cited or kernel second value. Local times use
each row's explicit fixed offset; offsets are arithmetic, not time zones.

### 3.1 Minneapolis 2026-09-27 — Dhuhr and Maghrib (`mwl-angles-18-17`, Standard)

Source row: `data/reference/prayer-library-v1.tsv` row 4
(`minneapolis-autumn`, `mwl-angles-18-17`, Standard; offset −300).

| Event | Cited instant | Cited precision | Adjustment | Resulting beginning (seconds) | Local (UTC−05:00) | Floor (O4-F) | Nearest (O4-N) | Ceiling (O4-C) |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Dhuhr (PrayTimes) | 2026-09-27T18:03:56.707Z | ms | +0 s | 18:03:57Z | 13:03:57 | 13:03 | 13:04 | 13:04 |
| Dhuhr (Adhan) | 2026-09-27T18:03:57.000Z | s | +0 s | 18:03:57Z | 13:03:57 | 13:03 | 13:04 | 13:04 |
| Dhuhr (kernel) | 2026-09-27T18:03:59Z | s | +0 s | 18:03:59Z | 13:03:59 | 13:03 | 13:04 | 13:04 |
| Maghrib (PrayTimes) | 2026-09-28T00:00:47.819Z | ms | +0 s | 2026-09-28T00:00:48Z | 19:00:48 | 19:00 | 19:01 | 19:01 |
| Maghrib (Adhan) | 2026-09-28T00:00:48.000Z | s | +0 s | 2026-09-28T00:00:48Z | 19:00:48 | 19:00 | 19:01 | 19:01 |
| Maghrib (kernel) | 2026-09-28T00:00:50Z | s | +0 s | 2026-09-28T00:00:50Z | 19:00:50 | 19:00 | 19:01 | 19:01 |

Observation: floor vs nearest already move the displayed minute (19:00 vs
19:01) with **zero** adjustment, so the display rule matters independently of
the ordering question.

**Maghrib shown separately from sunset:** the kernel returns sunset
2026-09-28T00:00:50Z under `EventRule::ApparentHorizon` and Maghrib at the
same instant under `EventRule::SunsetWithAdjustment { seconds: 0 }` — one
shared instant, two distinct rules (S8). The USNO case
`usno-2026-minneapolis-autumn` reports set at 2026-09-28T00:01:00Z at
**minute** precision under a different model; it is a separate source vector,
not Salah's sunset, and the ~10 s gap is not apportioned here.

### 3.2 Makkah 2026-03-20 — Dhuhr and Maghrib (`mwl-angles-18-17`, Standard)

Source row: `data/reference/prayer-library-v1.tsv` row 8
(`makkah-equinox`, `mwl-angles-18-17`, Standard; offset +180).

| Event | Cited instant | Adjustment | Resulting beginning (seconds) | Local (UTC+03:00) | Floor | Nearest | Ceiling |
| --- | --- | --- | --- | --- | --- | --- | --- |
| Dhuhr (PrayTimes) | 2026-03-20T09:28:11.520Z | +0 s | 09:28:12Z | 12:28:12 | 12:28 | 12:28 | 12:29 |
| Dhuhr (Adhan) | 2026-03-20T09:28:11.000Z | +0 s | 09:28:11Z | 12:28:11 | 12:28 | 12:28 | 12:29 |
| Dhuhr (kernel) | 2026-03-20T09:28:10Z | +0 s | 09:28:10Z | 12:28:10 | 12:28 | 12:28 | 12:29 |
| Maghrib (PrayTimes) | 2026-03-20T15:31:42.891Z | +0 s | 15:31:43Z | 18:31:43 | 18:31 | 18:32 | 18:32 |
| Maghrib (kernel) | 2026-03-20T15:31:41Z | +0 s | 15:31:41Z | 18:31:41 | 18:31 | 18:32 | 18:32 |

USNO cross-check (different source, minute precision): case
`usno-2026-makkah-equinox` reports rise 2026-03-20T03:25:00Z, transit
2026-03-20T09:28:00Z, set 2026-03-20T15:32:00Z. Kernel sunrise for this vector
is 03:24:57Z and sunset 15:31:41Z; no minute rule is applied to either side.

### 3.3 London 2026-06-21 — Isha crossing midnight (date preservation)

Two sub-cases from `data/reference/prayer-library-v1.tsv` (offset +60):

(a) `research-15`, Standard — row 18 (`london-summer`). Isha **occurs** after
midnight UTC but on the requested local date's evening:

| Isha | Cited instant | Beginning (seconds) | Local (UTC+01:00) | Displayed date |
| --- | --- | --- | --- | --- |
| PrayTimes | 2026-06-21T23:48:59.938Z | 2026-06-21T23:49:00Z | **2026-06-22** 00:49:00 | 06-22 (day after requested local date) |
| Adhan | 2026-06-21T23:49:15.000Z | 2026-06-21T23:49:15Z | **2026-06-22** 00:49:15 | 06-22 |
| Kernel | 2026-06-21T23:49:15Z | 2026-06-21T23:49:15Z | **2026-06-22** 00:49:15 | 06-22 |

All O4 candidates give 00:49. The local date (06-22) differs from the
requested local date (06-21); a future display rule must carry the converted
civil date with the minute, never assume the requested date (contract v0.3:
"Isha can therefore fall after local midnight").

(b) `mwl-angles-18-17`, Standard — row 20 (`london-summer`). Isha has **no**
angle crossing in Salah or PrayTimes; Adhan supplies a labeled fallback:

| Isha | Value | Kind / status |
| --- | --- | --- |
| PrayTimes | `no_event` | angle rule, no crossing (matched comparison) |
| Adhan | 2026-06-22T00:02:27.000Z | `fallback` (middle-of-night substitution per P1.3-H1); **not** an angle comparison |
| Fajr (same row) | PrayTimes `no_event`; Adhan 2026-06-21T00:02:13.000Z `fallback` | same status distinction |
| Kernel | Fajr and Isha `Unavailable` | `NoCrossingInSolarCycle`; preserved as unavailable |

A minute rule must never render a fallback or an unavailable event as an
ordinary timetable minute; fallback values stay labeled and separate.

### 3.4 Tromsø no-event (status preservation)

Prayer vectors: `data/reference/prayer-library-v1.tsv` rows 24–25
(`tromso-winter`, 2026-12-21, `mwl-angles-18-17`, offset +60). Solar vectors:
`data/reference/solar-usno-v1.tsv` case `usno-2026-tromso-winter`
(polar-no-event holdout).

| Event | PrayTimes | Adhan | Kernel | Status handling |
| --- | --- | --- | --- | --- |
| Sunrise / Sunset / Maghrib | `no_event` / `no_event` | `no_event` / `no_event` | `Unavailable` (sunrise, sunset, Maghrib) | All sides agree: no clock value exists; nothing to round |
| Asr | 2026-12-21T11:13:58.286Z (row 24) | 2026-12-21T11:13:45.000Z | `Unavailable` | **Status disagreement P1.3-P1**: libraries yield a time where Salah reports no crossing; the source times must not be adopted or minute-rounded by Salah |
| USNO rise / set | `no_event` / `no_event`; transit `unreported` | — | — | `Unreported` is not a no-event claim (README schema); transit cell stays empty, not zero-filled |

Fajr/Isha do occur in Tromsø winter (e.g. row 24 PrayTimes Fajr
2026-12-21T05:28:14.266Z, Isha 2026-12-21T15:43:52.498Z; kernel Fajr
05:28:19Z, Isha 15:43:50Z) and would follow §3.1-style minute candidates;
they are listed here only to bound the no-event claim to sunrise/sunset,
Maghrib, and Asr.

## 4. Hypothetical nonzero adjustment (ordering illustration only)

Current profiles carry zero Dhuhr/Maghrib adjustment (S6), so O5-A vs O5-B is
unobservable in evidence. The construction below applies an explicitly
**hypothetical** +30 s Dhuhr adjustment to the cited Minneapolis PrayTimes raw
instant 18:03:56.707Z (row 4, §3.1). It is **not** an implemented method, not
a reference observation, and not a proposal:

| Step | O5-A: adjust **before** floor-to-minute | O5-B: floor-to-minute **before** adjust |
| --- | --- | --- |
| Start | 18:03:56.707Z | 18:03:56.707Z |
| First operation | +30 s → 18:04:26.707Z | floor → 18:03:00Z |
| Second operation | floor → **18:04** | +30 s → 18:03:30Z; floor again → **18:03** |
| Displayed minute | 18:04 | 18:03 |

The two floor-based orders differ by a full displayed minute from a 30-second
adjustment. If nearest-minute, halves-up mapping is applied at both minute
steps instead, O5-A gives 18:04 while O5-B gives 18:05 (nearest raw 18:04,
then +30 s, then nearest again). Thus the ordering effect depends on the
display rule and the final mapping; both must be specified together.

## 5. What remains unknown by method

| Method / profile | Rounding evidence | Ordering evidence | Open question for the architect |
| --- | --- | --- | --- |
| `research-15` | None: non-institutional set, no rounding source by design (S6) | None | Any future rule is a fresh decision with no method-internal constraint |
| `mwl-angles-18-17` | Secondary sources expose minute-granularity adjustment settings, not a display-minute rule (S9, S7) | Matched run used zero adjustments with `Float`/`None` output, so no order is observable (S4, S7) | Whether a primary institutional rounding/ordering specification exists and, if so, which edition governs |
| PrayTimes v2 (external) | `Float` output was used; other formats and `tune` behavior were not audited | Unaudited in this packet | Default tuning, tune-vs-format order, and per-method differences |
| Adhan JS 4.4.6 (external) | `Rounding.None` used; other enum modes exist upstream but were not run | Unaudited in this packet | Non-`None` mode semantics; adjustment-vs-rounding order; per-method differences |
| USNO API 4.0.1 (external) | Minute formatting is a source property (O3) | Not applicable | Must not be reused as a display rule |

No per-method sourced statement of "adjust before rounding" or "round before
adjusting" was found in the consulted evidence. §4's O5-A/O5-B split is
therefore a decision the architect must make, not a fact the survey
recovered.

## 6. EventRule / result-schema gap

**What the schema exposes today (S8, read-only description):**

- `EventRule`: `SolarTransit`, `ApparentHorizon`, `SolarDepression
  { degrees }`, `AsrShadow { factor }`, `SunsetWithAdjustment { seconds }`,
  `TransitWithAdjustment { seconds }`. The adjustment-carrying variants record
  the adjustment magnitude; the zero case is still labeled (`SolarTransit`
  for Dhuhr at 0 s; `SunsetWithAdjustment { seconds: 0 }` for Maghrib at 0 s).
- `Event::Occurs { utc, unrounded_utc_unix_seconds, rule }`: second-rounded
  instant plus raw `f64` plus rule. `Event::Unavailable { reason }` carries
  `NoCrossingInSolarCycle` with no clock value.
- `CalculationRecord`: coordinates, local date, fixed offset, full method
  profile (including both adjustments), Asr criterion, astronomy model,
  engine version, high-latitude rule, elevation. No time-zone version (Phase 2
  addition per contract v0.3).
- CLI: prints per-event local civil text with seconds plus `UTC …Z`, or
  `unavailable: <reason>`.

**What a future minute-display rule would need (not implemented):**

1. A named, versioned display rule (ID + revision) covering: minute mapping
   (floor / nearest / ceiling, with halves policy), adjustment order (O5-A or
   O5-B), and per-event applicability (all events vs prayer beginnings only;
   sunset display treatment).
2. Provenance in the result: the rule ID/revision recorded alongside the
   adjustment chain (in `CalculationRecord` or a display-layer receipt), so a
   reviewer can reproduce which rule produced each displayed minute.
3. Status preservation: `Unavailable` and any labeled fallback must pass
   through unrounded and visibly distinct; a minute rule must not invent a
   clock value where `Event::Unavailable` produced none.
4. Date carriage: the converted civil date travels with the displayed minute
   (London §3.3a shows why).
5. Notification separation: a notification instant needs its own
   specification (lead offset, per-prayer enablement, OS scheduling limits
   and rescheduling triggers) and must reference the adjusted beginning, not
   silently replace it. Notification instants remain unscheduled and
   unspecified.

## 7. Tradeoffs and non-binding observation

| Consideration | Floor (O4-F) | Nearest (O4-N) | Ceiling (O4-C) |
| --- | --- | --- | --- |
| Displayed minute vs computed beginning | Never later than the beginning | Closest to wall-clock convention; halves need a policy | Never earlier than the beginning |
| Sensitivity to seconds noise | Least jitter across near-boundary instants | Most movement near :30 boundaries | Least jitter, mirrored |
| Comparison behavior | Source-vs-kernel deltas up to 59 s can share a minute or split across two; §3.1 shows floor/nearest already disagree with zero adjustment | Same, with the split at :30 instead of :00 | Same, mirrored |

**Non-binding observation (for architect review, not a selection):** the
survey's examples move a displayed minute under different candidate rules even
with zero adjustment (§3.1: 19:00 vs 19:01). Section 2 corrects the
second-rounding commutation claim; the existing kernel order is explicit.
The remaining display decision is therefore the **pair** (display rule ×
adjustment order), recorded with provenance (§6 items 1–2), with
unavailable/fallback statuses and converted dates carried through (§6 items
3–4) and notifications specified separately (§6 item 5). No option in §2 is
endorsed by this survey; O4/O5 rows are comparison constructs awaiting the
architect's choice.

## 8. Verification and limitations

- Every cited UTC value was checked against its TSV row/case ID on 2026-09-27:
  prayer-library rows 4, 8, 18, 20, 24 (plus row 25 for Hanafi parity);
  USNO cases `usno-2026-minneapolis-autumn`, `usno-2026-makkah-equinox`,
  `usno-2026-tromso-winter`. Kernel beginnings were recomputed with the
  unmodified `salah-cli` (v0.3.0, `NOAA-MEEUS-SOLAR-2`) for the four vectors;
  CLI output matched the report's signed-difference direction (kernel later
  than PrayTimes at Minneapolis Dhuhr/Maghrib, earlier at Makkah Dhuhr).
- Relative links use `calculation-contract-v0.3.md`-style same-directory
  paths from `specification/`; no new cross-directory links beyond existing
  conventions.
- `cargo fmt --all -- --check`, `cargo clippy --locked --offline --workspace
  --all-targets -- -D warnings`, and `cargo test --locked --offline
  --workspace` were run; results are reported in the handoff message. The
  diff is one new file plus one link-only line (see `docs/phase-1-validation.md`).

**Limitations.** This survey establishes no rounding rule, no ordering, no
timetable, and no accuracy claim. Primary institutional rounding sources were
not searched; upstream library rounding internals were not audited; reviewer
roles (astronomy, Islamic-methodology, civil-time/data, product/a11y) remain
vacant, so all wording is unreviewed. The hypothetical §4 construction must
not be quoted as a method behavior. Stop for architect review before any
P1.5 contract change, implementation, or P1.6 gate use.
