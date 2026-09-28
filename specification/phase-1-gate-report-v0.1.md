# Phase 1 gate report v0.1 — interim review

**Decision:** Phase 1 remains open as of 2026-09-28 UTC. The current engine is a research preview with explicit inputs and unavailable statuses. This report records what the selected comparisons support and what still blocks a consumer accuracy or religious-method claim. It changes no calculation, threshold, profile, or historical contract. The [roadmap](../docs/roadmap.md) owns the gate.

## Evidence and scope

- [Accuracy budget](accuracy-budget.md): comparison protocol and investigation allowances, not release error bounds.
- [USNO solar manifest](../data/reference/solar-usno-v1.tsv) and [signed report](usno-matrix-v1-report.md): 19 selected cases, 48 numeric UTC comparisons within the pre-existing case allowances, six matching explicit no-event cells, two event-existence disagreements, and one unreported transit.
- [Horizons grazing manifest](../data/reference/grazing-horizons-v1.tsv) and [independent audit](grazing-independent-audit-v1.md): an airless Sun-center altitude comparison around the disputed latitude, sampled at 10-second spacing under the fixed −0.833° threshold.
- [Prayer-library manifest](../data/reference/prayer-library-v1.tsv) and [report](prayer-library-v1-report.md): 28 rows against pinned PrayTimes and Adhan implementations with explicit configuration and status distinctions.
- [Method register](method-register-v1.md) and [presentation contract](presentation-contract-v0.1.md): technical parameter provenance and an optional research-preview display rule. Qualified method review and consumer timetable approval are pending.

All signed differences below are **Salah raw UTC instant minus source UTC instant**. USNO values are minute formatted; their differences cannot establish a sub-minute physical error bound. Rows are selected comparisons, not a representative sample of all places, dates, elevations, or weather.

## Solar evidence by regime

The signed ranges and largest absolute values are computed from the 19 rows in the [USNO report](usno-matrix-v1-report.md). A range summarizes reported numeric cells only; it never converts `no_event`, `unreported`, or a status disagreement into a number.

| Regime | Numeric cells | Signed range (s) | Largest absolute Δ (s) | Matching no-event | Status disagreement | Unreported | Gate reading |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| Ordinary | 15 | −21.1 to +27.0 | 27.0 | 0 | 0 | 0 | Selected cases agree within their investigation allowance; no global bound. |
| High latitude, existing events | 6 | −29.0 to +15.2 | 29.0 | 0 | 0 | 0 | Two holdout cases only; not a polar policy. |
| Date line | 9 | −20.9 to +25.9 | 25.9 | 0 | 0 | 0 | Explicit fixed offsets and selected-cycle handling only; no IANA-zone claim. |
| Boundary years | 9 | −28.6 to +20.7 | 28.6 | 0 | 0 | 0 | Selected 1900, 2050, and 2100 cases; no full-range accuracy bound. |
| Leap year | 3 | −19.3 to +7.1 | 19.3 | 0 | 0 | 0 | One selected date. |
| Polar no-event | 2 | −10.5 to −0.5 | 10.5 | 6 | 0 | 1 | Solar rise/set statuses match selected USNO cases; winter transit was not reported. |
| Near grazing | 4 | −94.3 to +111.0 | 111.0 | 0 | 2 | 0 | Rise/set at 65.735° N disagree in existence; a seconds allowance cannot decide that mismatch. |

At 65.735° N, the [Horizons audit](grazing-independent-audit-v1.md) sampled a selected-window minimum of −0.830944439° at 2026-06-22 00:02:00 UT, +0.002055561° above the kernel's fixed −0.833° threshold. Its sampled arc supports Salah's no-crossing status under that threshold. Raising the comparison threshold by 0.005° reverses the sampled classification. USNO's effective horizon/refraction rule, ephemeris geometry, and time scale remain unreported in the pinned response, so the **USNO-side cause remains unresolved**. The 10-second samples are not a formal continuous-curve or physical-observation bound.

## Prayer-rule evidence by event

The maxima are selected-case absolute differences from the [prayer-library report](prayer-library-v1-report.md). The signed Minneapolis examples show direction; all signed row/event values and source statuses are in the offline matrix test. Neither library is an astronomical or religious oracle.

| Event | PrayTimes maximum absolute Δ | Adhan maximum absolute Δ | Signed example (Salah − source) | Status or method limit |
| --- | ---: | ---: | --- | --- |
| Fajr | 20.155 s | 2.516 s | Minneapolis 18°: −0.070 s vs PrayTimes; −0.314 s vs Adhan | London summer 18° has no selected-angle crossing in Salah/PrayTimes; Adhan substitutes a labeled fallback. |
| Dhuhr | 22.285 s | 1.573 s | See signed source matrix | Zero adjustment in the compared configurations; fixed-offset input only. |
| Standard/Hanafi Asr | 19.203 s | 63.817 s | Minneapolis Standard: −14.766 s vs PrayTimes; +63.817 s vs Adhan | Main Adhan difference is traced to shadow-target declination epoch; about 16 s residual and polar-night status remain open. |
| Maghrib | 25.177 s | 1.859 s | Kiritimati: +25.177 s vs PrayTimes; +1.625 s vs Adhan | No sunset/Maghrib in selected Tromsø winter case; zero adjustment. |
| Isha | 23.630 s | 8.496 s | See signed source matrix | London summer 17° has no selected-angle crossing in Salah/PrayTimes; Adhan substitutes a labeled fallback. |

The +63.817-second Minneapolis Standard Asr result exceeds the earlier ±60-second Adhan investigation allowance by 3.817 seconds. The allowance is unchanged. The controlled declination-epoch substitution explains +47.815 seconds of that gap; the residual is not yet apportioned to model or solver terms. A source's fallback or polar-night formula output is recorded as a **status disagreement**, not averaged into a numeric time comparison.

## Open ledger and owners

| ID / issue | Current evidence | Required next decision | Review owner and state |
| --- | --- | --- | --- |
| P1.2-G1, 65.735° USNO event existence | Horizons supports Salah under a matched fixed threshold; USNO reports events. | Determine USNO-side effective definition if available, or explicitly exclude near-grazing event-existence claims. | Astronomy reviewer: vacant. Technical maintainer: unassigned. Open. |
| P1.3-A1, Asr residual | Most of the Adhan gap explained by declination epoch; about 16 s remains. | Assess the shadow definition and residual across more seasons/latitudes before a stronger Asr claim. | Astronomy reviewer: vacant. Open. |
| P1.3-H1, missing twilight | London angle condition absent; Adhan supplies its own fallback. | Define a separately named, reviewed high-latitude option and user wording. | Islamic-methodology reviewer: vacant. Open; no fallback in Salah. |
| P1.3-P1, polar-night Asr | Salah returns unavailable; two source formulas return instants despite the absent above-horizon shadow. | Review astronomical event semantics and any permissible religious policy separately. | Astronomy and Islamic-methodology reviewers: vacant. Open. |
| P1.3-D1, Kiritimati cycle | Library date/cycle normalization reproduced for this selected case. | Broaden date-line cases during civil-time work. | Technical maintainer: unassigned. Resolved for the cited case only. |
| P1.4, named method provenance | PrayTimes secondary table supports the 18°/17° parameter label; primary institutional confirmation was not established. | Obtain primary institutional source or retain a strictly secondary parameter label; review user wording and regional applicability. | Islamic-methodology reviewer: vacant. Open. |
| P1.5, display minute | Optional typed adapter follows the versioned research-preview ceiling rule. | Review method-specific conventions and local-time/notification behavior before consumer use. | Islamic-methodology and product reviewers: vacant. Technical implementation reviewed. |

## Gate criteria and claim boundary

| Criterion | Interim state |
| --- | --- |
| Reproducible selected-case sources and signed differences | Met for the recorded USNO and prayer-library matrices; Horizons request URLs, hashes, selected values, and limitations are recorded. Raw Horizons response bodies are not archived because rights were not established. |
| Supported regimes and defensible release tolerance | Not met. Investigation allowances are not release error bounds; grazing event existence and Asr residual need further review or a narrower, explicit exclusion. |
| Every material discrepancy resolved, reproduced, or explicitly excluded | Partly met. Several differences are reproduced and classified; the USNO-side grazing cause, Asr residual, and polar Asr policy remain open. |
| Named profiles sourced and reviewed for consumer use | Not met. The 18°/17° set has secondary provenance only; qualified review is pending. `research-15` is engineering-only. |
| Review sign-off | Not met. No independent astronomy, Islamic-methodology, civil-time/data, or product/accessibility reviewer has signed. AI-assisted comparison does not replace these roles. |

**Allowed statement:** the research engine calculates specified solar and prayer rules offline, returns UTC results or explicit unavailable statuses, and exposes its inputs and limits. **Unsupported statements:** globally accurate prayer times, institutionally endorsed MWL timetable, observed sunrise precision, a universal high-latitude ruling, consumer-ready local times, or a 1900–2100 accuracy guarantee.

The next evidence work should investigate P1.3-A1's residual and polar-night event semantics as separate bounded packets. The gate is reviewed again when those results and qualified method/astronomy reviews exist; no formula should change solely to match one source.
