# Prayer-start presentation contract v0.1

**Status:** architect decision for a research-preview display adapter, 2026-09-27. Implementing this contract must not change `salah-core` v0.3 calculation outputs or its historical [calculation contract](calculation-contract-v0.3.md). The [rounding survey](rounding-survey-v1.md) is evidence and comparison material, not a religious authority. Qualified Islamic-methodology and product review remain pending; this policy is not an institutional timetable rule or a consumer release claim.

## Decision and scope

The versioned presentation policy is `prayer-start-ceil-minute` revision `0.1`. It applies **only** to the five calculated prayer beginnings: Fajr, Dhuhr, Asr, Maghrib, and Isha. It produces a local **date and `HH:MM` label**, separate from the actual UTC event instant. Sunrise and sunset retain their existing second-precision event presentation; this contract assigns them no whole-minute timetable label. It also defines no fasting cutoff, prayer end, mosque timetable, iqamah, high-latitude fallback, or notification scheduling rule. In particular, the Fajr prayer-start label must not be reused as a suhoor deadline.

This is a Salah research-preview **display** policy, independent of the selected method profile. A later sourced method-specific display convention requires its own policy ID/revision and review. No profile parameter or method revision changes here.

## Exact order

1. Calculate the solar condition under the existing v0.3 contract. The kernel rounds the raw UTC candidate to the nearest integer second with `f64::round` (halves away from zero), **then** adds any whole-second Dhuhr or Maghrib method adjustment. Its existing `Event::Occurs.utc` is the resulting adjusted beginning for display purposes. Never add the adjustment again. Its `unrounded_utc_unix_seconds` remains preserved; for Dhuhr and Maghrib that field already includes the adjustment, so it is not necessarily the raw astronomical instant.
2. For an occurring prayer beginning, convert that adjusted UTC second to local seconds using the explicit `FixedUtcOffset` in the calculation record. Let `L = event.utc.unix_seconds + 60 × offset_minutes_east`.
3. Set `M = 60 × ceil(L / 60)` using mathematical ceiling or Euclidean integer division. `M` is the start of the **first local whole minute at or after** the adjusted beginning. If `L` is already on a minute boundary, keep that minute. Convert `M` to a local Gregorian date and `HH:MM`; carry a date change, including a midnight crossing. Do not discard the date or infer it from the requested local date. This fixed-offset rule must be revisited when Phase 2 adds IANA time-zone transitions; naive local-minute arithmetic must not be assumed across a gap or fold.
4. Return a separate, typed display receipt containing the prayer name, occurrence/unavailable status, actual adjusted `UtcInstant` when present, local displayed date and minute when present, `EventRule`, method ID/revision, and display-policy ID/revision. Preserve the original `Event` and `CalculationRecord`; the displayed label does not replace `Event.utc` or become a new astronomical result.

For `Event::Unavailable`, return its reason and **no** display minute. A future fallback, if ever implemented, must carry its own reviewed rule and label; this policy never converts one into an ordinary calculated beginning. Sunrise and sunset can be shown with their existing seconds, but receive no `prayer-start-ceil-minute` receipt.

## Rationale and limits

Ceiling makes the displayed prayer-start minute no earlier than the **computed adjusted second** and at most 59 seconds later. It is a deterministic presentation choice, not a claim that the underlying solar model, method, atmosphere, location, or device clock is exact; it is not a religious safety guarantee. Floor can display a prayer start before the computed event, while nearest can do so for part of a minute. The source survey found no verified per-method rounding or adjustment-ordering mandate, so the policy must be labeled as Salah's own versioned rule and remain subject to qualified review before consumer use.

The displayed minute is never the basis for a future notification instant. A notification policy may later schedule against the actual adjusted `Event.utc` plus a declared lead offset and platform rules. No notification behavior is decided or implemented by this contract.

## Cited examples for implementation review

The following **kernel** seconds are recorded in [rounding survey §3](rounding-survey-v1.md#3-worked-examples-from-cited-vectors-all-adjustments-zero), derived there from the current CLI and linked source rows. They are research examples, not independent physical references. Each current built-in profile has zero Dhuhr/Maghrib adjustment.

| Case | Adjusted kernel beginning | Fixed offset | Local exact second | Display under policy 0.1 |
| --- | --- | --- | --- | --- |
| Minneapolis 2026-09-27, Dhuhr | 2026-09-27T18:03:59Z | UTC−05:00 | 2026-09-27 13:03:59 | 2026-09-27 13:04 |
| Minneapolis 2026-09-27, Maghrib | 2026-09-28T00:00:50Z | UTC−05:00 | 2026-09-27 19:00:50 | 2026-09-27 19:01 |
| Makkah 2026-03-20, Dhuhr | 2026-03-20T09:28:10Z | UTC+03:00 | 2026-03-20 12:28:10 | 2026-03-20 12:29 |
| London 2026-06-21, research-15 Isha | 2026-06-21T23:49:15Z | UTC+01:00 | 2026-06-22 00:49:15 | 2026-06-22 00:50 |
| Tromsø 2026-12-21, Maghrib or Asr | `Unavailable` | UTC+01:00 | none | none; preserve reason |

Additional boundary checks for the implementing packet: exactly `HH:MM:00` stays in that minute; `HH:MM:01` advances one minute; `23:59:59` carries to the next local date; `1969-12-31T23:59:59Z` at UTC+00:00 displays `1970-01-01 00:00` under Euclidean division. A hypothetical nonzero Dhuhr/Maghrib adjustment must be applied by the existing kernel before the display function receives `Event.utc`; compare it with the raw solar event and show the adjustment only once. Never modify a real profile or claim the hypothetical value is a reference observation.

## Versioning and gate effect

Implementation may add an opt-in presentation API and CLI output, with a regression test for each cited or boundary case; it must leave existing calculation fields and second-precision CLI lines intact. A behavioral change to this mapping mints a new display-policy revision and explicit migration note. The receipt must reveal the rule that produced its displayed minute. The Phase 1 P1.5 acceptance line remains open until implementation and review verify the schema and examples. P1.6 and consumer claims remain blocked by the wider validation and reviewer gaps.
