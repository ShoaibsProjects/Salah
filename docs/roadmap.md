# Salah delivery roadmap

**Status:** active execution plan, refreshed 1 October 2026. This document governs phase order and release claims. The [product vision](vision-and-architecture.md) describes the destination; the [decision record](decisions.md) tracks choices; calculation contracts describe implemented behavior. Review this roadmap whenever a phase gate changes.

## Product contract

Salah aims to provide free-to-use, private prayer times for Earth on iOS, Android, and web. Its durable center is an offline Rust calculation engine with versioned method definitions, civil-time data, reference evidence, and an explanation for every result. Core use must not require an account, paid API, subscription, or server. Calculated prayer beginning, a mosque timetable, and iqamah remain distinct concepts. The app does not decide a disputed religious question or silently invent a time when its selected rule has no event.

“Useful through 2050” requires maintenance and succession. An unchanged 2026 binary cannot know future time-zone law, platform policy, or method revisions. We will preserve reproducible historical results while allowing current data and software to be updated. This is a continuing obligation across every phase, not a final feature.

## Current position

The repository contains `salah-core` and `salah-cli` v0.3.0. It calculates one solar cycle offline with explicit coordinates, a local Gregorian date, a **manually supplied fixed UTC offset**, one of two angle profiles, and Standard/Hanafi Asr. It returns UTC instants or typed unavailable events. Selected solar results are compared with USNO and selected prayer results with Adhan JS and PrayTimes. `salah-core` has no third-party crate dependencies. The separate experimental `salah-time` crate uses a pinned IANA 2026d pack of 598 named zones and Jiff 0.2.37; its research APIs enumerate local-date cycles, classify date existence, and localize events. It does not infer a zone from coordinates, and independent transition evidence remains much narrower than the pack. CI checks formatting, lints, and reference cases.

This is a **research preview**. The repository now includes the global IANA zone pack, an experimental coordinate-to-zone suggestion layer, and a thin selected-zone schedule facade; none is a completed Phase 2 gate. There is still no reviewed regional method default, high-latitude fallback, Qibla implementation, platform binding, or consumer app. Passing selected comparisons does not establish a globally accurate timetable or religious endorsement.

F3-C1 now exposes that facade through `salah-cli schedule` with explicit named-zone inputs, full local/UTC event labels, and optional versioned JSON. `salah-cli zones` lists the bundled identifiers. Direct manual selection avoids polygon lookup and records that fact. See the [interface contract](../specification/named-zone-interface-contract-v0.1.md) and [current-work map](current-work.md). Local build/static checks are recorded; no new tests were added or run locally in this piece, as requested by the user. Earlier phase gates and cross-target equivalence remain open.

F3-W1 adds the [WebAssembly bridge and first local browser screen](../specification/wasm-bridge-contract-v0.1.md). The pinned-compiler WASM build and direct generated-module use in Node.js succeeded; browser interaction/visual review is pending. A worker calls the same engine, with strict explicit JSON inputs and the existing schedule document. CI builds an archived research preview. Durable offline installation, mobile bindings, and the Phase 3 equivalence gate remain open.

F4-C1 is an exploratory browser-screen refinement: a user-initiated, energy-conscious device-location request with visible provider uncertainty and an optional higher-accuracy retry, plus plain-language Fajr/Isha profile and Asr shadow-ratio guidance. It does not turn a compass into a location source, guarantee offline positioning, infer a timezone, or advance the Phase 3 gate. Browser/device review remains pending.

## How phases work

Seven phases are numbered 0–6. Each phase has a deliverable, an evidence gate, and a named decision to advance. A gate limits **claims and release scope**; it does not prohibit a small exploratory UI, binding, or data-source probe while earlier work continues. The critical path is validation → civil-time integrity → portable core → consumer release. Governance, accessibility, privacy, and maintainability start now and continue throughout.

A phase is complete only when its evidence is committed, limitations are written down, and open discrepancies have an owner. No date or version number automatically passes a gate. The maintainer records the gate decision in [decisions.md](decisions.md) with a link to the evidence.

| Phase | Result | Required gate evidence | Current state |
| --- | --- | --- | --- |
| **0. Research foundation** | Versioned contract, offline Rust core/CLI, first reference cases, CI, agent guidance. | A reproducible daily result with explicit inputs, assumptions, and unavailable status. | **Complete for research use**, not certified. |
| **1. Validation and method integrity** | Numerical error budget, source manifest, wider reference matrix, discrepancy ledger, method provenance, rounding/event definitions. | Supported regimes and comparison limits are stated; every material discrepancy is resolved, reproduced, or explicitly excluded. Named profiles carry sourced parameters and review status. | **Active.** Follow [Phase 1 plan](phase-1-validation.md). |
| **2. Civil time and difficult geography** | Versioned offline IANA rules, coordinate-to-zone data with ambiguity handling and manual choice, DST/date-line behavior, high-latitude policies, offline Qibla, stated elevation/horizon policy. | UTC events convert to local dates/times under a recorded data version; gaps, overlaps, borders, stale data, polar cases, and fallbacks have explicit outcomes. No silent zone or religious-rule guess. | 598-name 2026d pack, research schedule APIs, approximate coordinate-zone suggestions, and explicit-input orchestration are implemented research code pending phase-gate review; broader independent transition evidence and high-latitude/Qibla work remain open. |
| **3. Portable calculation platform** | Reviewed public Rust API and reproducibility record; CLI, WASM, and mobile binding adapters. | The same versioned input vectors produce equivalent UTC results and statuses on native, browser, iOS, and Android targets. No client duplicates prayer mathematics. | Named-zone CLI/shared JSON and WASM bridge/local screen are implemented; native/WASM builds and one direct module use are recorded. Browser acceptance, mobile bindings, and cross-target evidence remain open. |
| **4. Trustworthy experience** | Accessible interaction model and design system for today, settings, location, Qibla, reminders, and “Why this time?”. | Usability review covers older users, screen readers, RTL, uncertainty, manual correction, and clear distinction between calculated and local-mosque times. A prototype explains a result without hiding its assumptions. | **Exploratory F4-C1 implemented in the browser preview:** one-shot device-location request and plain-language method/Asr guidance. Visual, accessibility, and real-device review remain open; Phase 3 is still ungated. |
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

Phase 1 is the active work. **P1.1** produced the draft [accuracy budget](../specification/accuracy-budget.md); **P1.2** added the [USNO source manifest](../data/reference/solar-usno-v1.tsv), [signed report](../specification/usno-matrix-v1-report.md), and [independent near-grazing audit](../specification/grazing-independent-audit-v1.md). **P1.3** added the [prayer-library matrix](../data/reference/prayer-library-v1.tsv), [discrepancy report](../specification/prayer-library-v1-report.md), and [Asr residual audit](../specification/asr-residual-audit-v1.md). **P1.4** has a [method register](../specification/method-register-v1.md) recording technical provenance, while primary institutional confirmation and qualified review remain open. **P1.5** has a [rounding survey](../specification/rounding-survey-v1.md), architect-selected research-preview [presentation policy](../specification/presentation-contract-v0.1.md), and reviewed optional Rust adapter in `0be7aee`; calculation outputs and consumer timetable status remain unchanged. The [interim Phase 1 gate report](../specification/phase-1-gate-report-v0.1.md) and [decision record](decisions.md) keep the gate open. Independent astronomy and Islamic-methodology review are the next gate actions; civil-time data-source feasibility may proceed in parallel under the roadmap's research allowance. [Director handoff](director-handoff.md) §3–§5 retains the completed packet protocol; this roadmap remains authoritative for gates.

The next gate review will answer: which regimes are supported, what comparison tolerance is defensible under matched assumptions, which cases remain unresolved, and which method labels are suitable for user-facing use. Only then should Phase 2's local-time outputs be treated as candidate consumer behavior.

The [F2-TZ0 offline civil-time conversion](civil-time-spike.md) probe is implemented and reviewed. It loads explicitly pinned IANA rules and converts UTC instants for manually selected zones without changing the prayer engine or claiming global civil-time coverage. The subsequent [F2-TZ1 UTC solar-cycle anchor](civil-time-next-slice.md) is now implemented and enables zone-aware selection of a requested prayer date across DST changes and skipped days. Later slices and the current piece are recorded below and in the [work map](current-work.md). Phase 1 gate review and independent method/astronomy review remain open.

F2-TZ1 status (architect-reviewed): the additive UTC-anchor API is implemented in `salah-core` with the versioned selector and [UTC-anchor contract v0.1](../specification/utc-anchor-contract-v0.1.md). Legacy results, CLI output, and historical contract v0.3 are preserved. Phase 1 and Phase 2 gates remain open; no consumer claim follows.

F2-TZ2 local-date transit selection is implemented for the six pinned zones; see the [selector contract](../specification/civil-date-transit-selector-v0.1.md). It enumerates matching cycles explicitly and remains under review; the Phase 1 and Phase 2 gates remain open.

F2-TZ3 adds a bounded local-schedule research facade; see the [schedule contract](../specification/local-prayer-schedule-contract-v0.1.md). It localizes all seven events for each transit match without display rounding or fallback. This remains research-only and does not close either phase gate.

F2-TZ4 adds exact civil-date existence classification from the pinned zone's transition intervals; see the [classifier contract](../specification/civil-date-existence-contract-v0.1.md). The schedule distinguishes skipped dates from existing dates with zero transit matches. It uses the named-zone pack but still has focused independent evidence and does not close either phase gate.

F2-TZ5 adds a compact, reproducible pack containing all 598 named identifiers emitted by IANA 2026d source inputs; see the [pack contract](../specification/global-timezone-pack-v0.1.md) and the F2-TZ2 [coverage amendment](../specification/civil-date-transit-selector-v0.2.md). Runtime use remains offline, and broad pack inclusion is not independent validation of every zone's historical rules.

F2-TZ6 adds an experimental `salah-location` layer that returns all approximate timezone-boundary matches from the timezone-boundary-builder 2026d dataset. It refuses boundary/rule version mismatch, requires explicit confirmation or a manual override, and retains candidate provenance; see the [coordinate-to-zone contract](../specification/coordinate-zone-selection-contract-v0.1.md). The ODbL data, boundary approximation, target-specific package impact, and map-to-IANA-name coverage remain under review. This is not the Phase 2 gate decision.

F2-TZ7 adds a thin `salah-engine` facade: a caller supplies the explicit `ZoneSelection`, local date, method profile, and Asr criterion, and receives the existing local schedule result paired with zone-selection provenance. It does not infer user choices or change calculation formulas; see the [selected local-day contract](../specification/selected-local-day-engine-contract-v0.1.md). It remains experimental and does not pass the Phase 2 gate.

F2-TZ8 adds the [civil-time data assessment](../specification/civil-time-data-assessment-contract-v0.1.md): a separate, deterministic advisory using a caller-supplied observation date. It identifies a pack from an earlier year and a future requested date without altering the schedule or claiming legal freshness. The [data lifecycle rule](civil-time-data-lifecycle.md) sets the next path: versioned authenticated packs with rollback, then a separately labeled manual single-date offset correction. This does not pass the Phase 2 gate.

F2-TZ9-P1 adds a schema 1 [offline TZif pack interface](../specification/offline-tzif-pack-interface-v0.1.md): exact bundled pack identity accompanies civil-time results, and an offline validator rejects inconsistent manifests, hashes, slices, and TZif images. The runtime still accepts only compiled 2026d bytes. Signed activation, old-pack archives, and rollback remain the F2-TZ9-P2 decision; P1 does not pass the Phase 2 gate.

F2-TZ9-P2a adds an offline [signed-candidate verifier](../specification/signed-rule-pack-candidate-v0.1.md) in `salah-update`. It checks a strict Ed25519 signature before parsing bounded update bytes, requires a pinned boundary artifact and all currently supported zone IDs, and rejects a nonincreasing sequence. No production trust key or activation channel exists; F2-TZ9-P2b owns durable activation and rollback. This does not pass the Phase 2 gate.

F2-TZ9-P2b.1 adds a separate experimental Unix [local repository and crash-recovery adapter](../specification/rule-pack-repository-v0.1.md) in `salah-update-store`. It persists exact signed archives and a small state record, reverts interrupted trials, and preserves both accepted sequence and release high-water records. Evidence is from macOS; other platforms, hardware power loss, runtime pack integration, and production key stewardship remain open. Stored selection does not yet change a calculated schedule. The [current-work map](current-work.md) names the parts and remaining pieces; neither phase gate is passed.

F2-TZ9-P2b.2a implements the additive [verified runtime snapshot](../specification/runtime-rule-snapshot-v0.1.md) path. A shared immutable payload loader retains schema/integrity/TZif validation; the verifier still authenticates before parsing. The engine uses one bundled or verified snapshot for all civil-time operations and preserves its exact identity plus signed source metadata. Local build/lint checks pass; focused runtime acceptance evidence, target integration, and trial-health policy remain open. No phase gate is passed.
