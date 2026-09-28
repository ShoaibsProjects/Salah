# Salah delivery roadmap

**Status:** active execution plan, 27 September 2026. This document governs phase order and release claims. The [product vision](vision-and-architecture.md) describes the destination; the [decision record](decisions.md) tracks choices; calculation contracts describe implemented behavior. Review this roadmap whenever a phase gate changes.

## Product contract

Salah aims to provide free-to-use, private prayer times for Earth on iOS, Android, and web. Its durable center is an offline Rust calculation engine with versioned method definitions, civil-time data, reference evidence, and an explanation for every result. Core use must not require an account, paid API, subscription, or server. Calculated prayer beginning, a mosque timetable, and iqamah remain distinct concepts. The app does not decide a disputed religious question or silently invent a time when its selected rule has no event.

“Useful through 2050” requires maintenance and succession. An unchanged 2026 binary cannot know future time-zone law, platform policy, or method revisions. We will preserve reproducible historical results while allowing current data and software to be updated. This is a continuing obligation across every phase, not a final feature.

## Current position

The repository contains `salah-core` and `salah-cli` v0.3.0. It calculates one solar cycle offline with explicit coordinates, a local Gregorian date, a **manually supplied fixed UTC offset**, one of two angle profiles, and Standard/Hanafi Asr. It returns UTC instants or typed unavailable events. Selected solar results are compared with USNO and selected prayer results with Adhan JS and PrayTimes. The Rust workspace has no third-party crate dependencies. CI checks formatting, lints, and reference cases.

This is a **research preview**. It has no IANA time-zone lookup, coordinate-to-zone data, reviewed regional default, high-latitude fallback, Qibla implementation, platform binding, or consumer app. Passing selected comparisons does not establish a globally accurate timetable or religious endorsement.

## How phases work

Seven phases are numbered 0–6. Each phase has a deliverable, an evidence gate, and a named decision to advance. A gate limits **claims and release scope**; it does not prohibit a small exploratory UI, binding, or data-source probe while earlier work continues. The critical path is validation → civil-time integrity → portable core → consumer release. Governance, accessibility, privacy, and maintainability start now and continue throughout.

A phase is complete only when its evidence is committed, limitations are written down, and open discrepancies have an owner. No date or version number automatically passes a gate. The maintainer records the gate decision in [decisions.md](decisions.md) with a link to the evidence.

| Phase | Result | Required gate evidence | Current state |
| --- | --- | --- | --- |
| **0. Research foundation** | Versioned contract, offline Rust core/CLI, first reference cases, CI, agent guidance. | A reproducible daily result with explicit inputs, assumptions, and unavailable status. | **Complete for research use**, not certified. |
| **1. Validation and method integrity** | Numerical error budget, source manifest, wider reference matrix, discrepancy ledger, method provenance, rounding/event definitions. | Supported regimes and comparison limits are stated; every material discrepancy is resolved, reproduced, or explicitly excluded. Named profiles carry sourced parameters and review status. | **Active.** Follow [Phase 1 plan](phase-1-validation.md). |
| **2. Civil time and difficult geography** | Versioned offline IANA rules, coordinate-to-zone data with ambiguity handling and manual choice, DST/date-line behavior, high-latitude policies, offline Qibla, stated elevation/horizon policy. | UTC events convert to local dates/times under a recorded data version; gaps, overlaps, borders, stale data, polar cases, and fallbacks have explicit outcomes. No silent zone or religious-rule guess. | Planned after Phase 1 evidence; data-source research can start now. |
| **3. Portable calculation platform** | Reviewed public Rust API and reproducibility record; CLI, WASM, and mobile binding adapters. | The same versioned input vectors produce equivalent UTC results and statuses on native, browser, iOS, and Android targets. No client duplicates prayer mathematics. | Planned; small binding probes may run earlier. |
| **4. Trustworthy experience** | Accessible interaction model and design system for today, settings, location, Qibla, reminders, and “Why this time?”. | Usability review covers older users, screen readers, RTL, uncertainty, manual correction, and clear distinction between calculated and local-mosque times. A prototype explains a result without hiding its assumptions. | Early sketches may run now; full design follows a stable result model. |
| **5. Offline consumer beta** | iOS, Android, and responsive web clients using the same core; local settings and opt-in reminders. | Real-device/browser checks cover offline use, travel, DST, permission denial, notification scheduling limits, accessibility, and restart/update behavior. Manual location remains usable without GPS. | Planned. |
| **6. Public release and stewardship** | Independent review, release evidence, privacy/security review, data-update and rollback process, documented ownership, license, and funding path. | Reviewers sign off on the supported scope and wording; archived code/data reproduce released results; release notes state known limits and an issue-report path. A maintainer can update civil-time/method data without replacing the calculation engine. | Governance work starts now; public release waits for this gate. |

### Scope of the first public release

The target is a calm daily schedule for a selected location with Fajr, sunrise, Dhuhr, Asr, Maghrib, and Isha; current/next prayer; Qibla bearing; manual and permission-based location; a visible zone and method; “Why this time?”; and optional local reminders. It should work offline after installation with bundled data. Web notification and background behavior will be described according to the browser's actual limits.

Monthly exports, Hijri calendar claims, mosque timetable integration, adhan audio, educational content, widgets, watches, vehicles, and AI explanations are later decisions. None may silently alter a calculated prayer beginning. A small mosque timetable comparison can be evaluated after source and labeling rules exist; an unreviewed feed is not a launch dependency.

### Scope and accuracy rules

- Compare **UTC instants and definitions** before local display. A library's named method, rounded clock time, or historical device time zone is not an oracle by itself.
- Keep numerical solver error, solar-model error, apparent-horizon/refraction assumptions, method differences, civil-time data error, and presentation rounding separate. A single “accuracy” number would hide the cause.
- A coordinate and date do not uniquely determine a civil-time zone at borders, at sea, in flight, or after a law changes. Record the zone source, data version, and user override. Ask the user to choose when offline mapping is ambiguous.
- High-latitude substitutions require a named, reviewed rule and a visibly adjusted status. If no accepted rule is selected, report the missing event.
- Reference coverage will be expanded before claiming support across the full 1900–2100 date range or all latitudes. The current date-range validation is a small boundary sample plus calendar invariants.
- “Free” means core access has no paywall or per-calculation service dependency. Sustainable updates, security, platform fees, and hosting still need maintainers and funding.

## Parallel work and decision owners

The project needs roles even before people are assigned: a **technical maintainer** owns code and release gates; an **astronomy reviewer** checks model definitions and comparisons; an **Islamic-methodology reviewer** checks named methods and user-facing religious wording; a **civil-time/data maintainer** owns TZDB sources and updates; and a **product/accessibility reviewer** checks comprehension and usability. One person may hold multiple roles, but an unfilled review role must be shown as unfilled. No profile becomes a regional default merely because a table lists its angles.

Start license and stewardship discussions during Phase 1. Investigate TZDB licensing, offline polygon data, and mobile/WASM binding feasibility while validation proceeds. These probes can identify a blocked architecture early; they do not establish a phase gate by themselves.

## Work unit for contributors and agents

Take one bounded task from the active phase. Every task description should state:

1. **Objective and reason:** the user-visible or scientific question it answers.
2. **Scope and exclusions:** exact behavior, data, and platforms affected.
3. **Source and assumptions:** primary source where possible, version/date, time scale, units, and religious review status.
4. **Acceptance evidence:** reference vectors or observed behavior, tolerance rationale, test method, and documented discrepancies.
5. **Change record:** contract/model/profile/data version changes if results change; migration and historical reproduction impact.
6. **Gate effect:** which phase criterion moves forward, and what remains open.

Do not change a formula just to make one reference clock time match. Reproduce the discrepancy, identify its cause, then change the specification and code together. Do not publish a consumer accuracy claim from a small selected case set.

## Immediate execution

Phase 1 is the active work. **P1.1** produced the draft [accuracy budget](../specification/accuracy-budget.md); **P1.2** added the [USNO source manifest](../data/reference/solar-usno-v1.tsv) and [signed report](../specification/usno-matrix-v1-report.md); **P1.3** added the [prayer-library matrix](../data/reference/prayer-library-v1.tsv) and [discrepancy report](../specification/prayer-library-v1-report.md). **P1.4** has a [method register](../specification/method-register-v1.md) recording technical provenance, while primary institutional confirmation and qualified review remain open. **P1.5** has a [rounding survey](../specification/rounding-survey-v1.md), architect-selected research-preview [presentation policy](../specification/presentation-contract-v0.1.md), and reviewed optional Rust adapter in `0be7aee`; calculation outputs and consumer timetable status remain unchanged. The next technical priority is independent investigation of the near-grazing event-existence disagreement, with a bounded packet to be specified before delegation. The delegation backlog and review protocol are in [director-handoff.md](director-handoff.md) §3–§5; this roadmap remains authoritative for gates.

The next gate review will answer: which regimes are supported, what comparison tolerance is defensible under matched assumptions, which cases remain unresolved, and which method labels are suitable for user-facing use. Only then should Phase 2's local-time outputs be treated as candidate consumer behavior.
