# Accuracy budget and comparison protocol — draft 0.1

**Status:** Phase 1 engineering protocol for `salah-core` v0.3.0, model `NOAA-MEEUS-SOLAR-2`. This document records what can be bounded now and what still requires independent measurement and review. It is not a claim that the kernel is globally accurate or that a particular method is religiously authoritative. See the [Phase 1 plan](../docs/phase-1-validation.md), [calculation contract](calculation-contract-v0.3.md), and [reference cases](reference-cases.md).

## What is being compared

A prayer result is a chain, not one number:

```text
coordinates + date
  → solar position and event definition
  → method rule and adjustment
  → raw UTC instant
  → second or timetable-minute rounding
  → civil zone and local clock display
  → platform notification
```

For a reference comparison, freeze and record each stage. Compare **raw UTC event definitions** before a rounded or local display. A discrepancy in a selected twilight angle, a mosque timetable, or a civil-time offset is not evidence of numerical solver error. An observed sunrise also depends on atmosphere, observer elevation, and terrain; the current core uses a fixed sea-level mean apparent horizon of −0.833°.

## Component budget

| Component | Current quantitative statement | Limit and next evidence |
| --- | --- | --- |
| Gregorian input and conversion | `CivilDate::new` accepts valid input years 1900–2100. The internal day conversion round-trips every day in that range in the current invariant check. | This checks calendar arithmetic and input validity, not solar position or future civil-time law. |
| Altitude crossing bisection | After a valid bracket is selected, 50 bisections of a half-cycle no longer than 43,200 seconds yield a nominal interval below one second (`43,200 / 2^50` seconds) before floating-point resolution is considered. | This is a bracket-width argument, **not** a global event-time accuracy bound. Bracket selection, a near-tangent event, and the solar condition itself must be checked separately. |
| Upper solar transit | Eight iterations of the equation-of-time estimate are implemented. | No independent numerical convergence bound is established yet. Record the transit residual and compare with a separate ephemeris/USNO matrix. |
| Stored instant rounding | An occurring event preserves raw UTC seconds and rounds to the nearest integer second. The existing check requires a difference of at most 0.5 second. | Timetable-minute rounding is unspecified. Do not treat displayed seconds as ephemeris accuracy. |
| Solar model | Selected USNO rise/transit/set cases pass their stated allowances, including 1900 and 2100 samples. | No measured global maximum, percentile, or release target exists. NOAA/Meeus approximation, UTC≈UT1, and source model assumptions must be compared in matched regimes. |
| Apparent horizon, weather, elevation, terrain | The kernel fixes a sea-level −0.833° solar-center horizon. | No universal time error bound is possible from that fixed assumption. Near grazing, a small altitude difference can change event existence. Expose the assumption and classify that regime separately. |
| Prayer method | Two versioned angle parameter sets and Standard/Hanafi Asr are implemented. | Differences among angle, interval, Asr, adjustment, and high-latitude conventions are semantic choices, not one numerical error budget. The MWL-named set is not an endorsed regional default. |
| Civil time | The caller supplies a fixed UTC offset, applied arithmetically. | IANA zone resolution and DST are absent. Two permitted fixed offsets can differ by as much as 28 hours, changing the displayed date as well as time; no global local-time claim is supported. |
| Notifications | Not implemented. | OS delivery behavior is outside this kernel's accuracy budget. |

The bisection interval is a mathematical implementation property under a correct finite bracket. It says nothing about whether a source uses the same solar altitude, refraction, time scale, or prayer rule.

## Existing regression allowances

These values come from the current tests. They are **case-specific investigation triggers**, not release thresholds or global precision claims.

| Case class | Existing allowance | Meaning |
| --- | --- | --- |
| Ordinary USNO solar and date-line cases | ±90 seconds on UTC events | USNO API clock values are formatted to a minute; the kernel and source can differ in horizon/model assumptions. Applies only to cited cases. |
| 65.72° N near-midnight rise/set | ±180 seconds | Near-grazing apparent-horizon models amplify small physical and numerical differences. This does not make a three-minute error acceptable everywhere. |
| Adhan JS v4.4.6 with explicit 15°/15° parameters | ±60 seconds on selected Fajr/Asr/Isha UTC values | The implementations can differ in solar ephemeris and Asr declination treatment; it is a comparison, not a religious or astronomical oracle. |
| PrayTimes v2 MWL parameter comparison | ±60 seconds on selected UTC values | The angle profile is matched, while ephemeris and iteration details differ. The default high-latitude behavior is not being compared. |
| Polar day/night statuses | Event-existence assertions, with selected Dhuhr ±90 seconds | No invented sunrise/sunset or twilight clock value is accepted when the selected solar condition has no crossing. |

The [19-case USNO matrix report](usno-matrix-v1-report.md) records the signed raw-UTC differences and two known near-grazing event-existence disagreements. The cited sources, case inputs, and earlier selected outputs are in [reference-cases.md](reference-cases.md). Tolerances must not be widened simply to make a failing change pass. A change outside an allowance requires a discrepancy record even if the source is later found to use a different definition.

## Comparison protocol

1. **Declare the regime before running the case.** Use ordinary, high-latitude existing event, near-grazing/tangent, polar no-event, or boundary-year/date-line. Record latitude, longitude, assumed elevation, date, and the desired solar cycle.
2. **Pin the kernel input.** Record core version, astronomy model ID, method ID/revision and all parameters, Asr criterion, high-latitude rule, adjustment, fixed offset or later IANA zone/data version, and raw UTC event.
3. **Pin the reference.** Record source name/version, URL or archived response and hash, retrieval date, geographic datum, time scale, event definition, horizon/refraction/elevation model, method parameters, rounding, and stated precision. If a source omits an assumption, mark it **unknown**.
4. **Normalize to UTC.** Preserve the source's original notation and conversion rule. Do not compare local clock text until its offset/zone is independently checked.
5. **Compare like with like.** For ordinary events, report signed UTC difference in seconds and the source's precision. For near-grazing cases, first compare whether the same event definition has a crossing; do not force a time tolerance across an event/no-event boundary.
6. **Classify the difference.** Possible causes: source precision, numerical bracketing/transit, ephemeris/time scale, apparent-horizon physics, method parameter, civil-time mapping, rounding, or unresolved. Show evidence for the classification.
7. **Control changes.** If a defect is demonstrated, preserve the old contract/model/profile identity, add a regression case, document the new behavior and any migration impact, then change code. Do not tune a formula to one reference without checking other regimes.

A source manifest for Phase 1 must give each vector a stable case ID and all fields above. Where source terms permit, archive its response; otherwise keep a repeatable retrieval recipe plus hash. Set aside holdout cases that are not used to tune the model.

## Two current discrepancy examples

- At 65.735° N on 2026-06-21, [USNO reports brief rise/set events](https://aa.usno.navy.mil/api/rstt/oneday?date=2026-06-21&coords=65.735,0&tz=0), while the kernel's fixed −0.833° model reports no crossing. This is an **event-existence disagreement near the apparent-horizon boundary**. The current evidence does not justify selecting one result as universally observed or loosening an ordinary-event seconds tolerance.
- In Minneapolis on 2026-09-27, Adhan JS with custom 18°/17° parameters places Standard Asr about one minute earlier than Salah and PrayTimes v2. The documented Asr shadow-angle treatments differ. This is a **method/model-definition comparison**, not a reason to average the three clock times.

## Open release thresholds and review

Phase 1 now has a wider 19-case USNO matrix, but it still needs independent matched-assumption ephemeris evidence and broader regime coverage before setting a consumer release threshold for solar events. Record signed difference, maximum absolute difference, and distribution **by event and regime**, with source precision shown alongside. Review failures individually; a percentile cannot excuse a materially wrong event or a false event-existence result. Separate numerical convergence evidence from agreement with another approximate implementation.

The current case set cannot determine a defensible global tolerance for 1900–2100, near-polar geometry, terrain/weather, or all prayer methods. The astronomy and Islamic-methodology reviewers are not yet assigned. Those gaps remain open in the [roadmap](../docs/roadmap.md) and block a broad accuracy or religious-endorsement claim.
