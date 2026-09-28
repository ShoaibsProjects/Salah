# Director handoff — delegable execution package

**Status:** active delegation package, 27 September 2026. Research preview `salah-core`/`salah-cli` v0.3.0, model `NOAA-MEEUS-SOLAR-2`.
**Authority:** [Delivery roadmap](roadmap.md) governs phase order, gates, and release claims. This document does not replace it. It adds the concrete delegation backlog, copy-paste prompts, evidence requirements, and review protocol for the active phase.
**Source of truth for behavior:** [Calculation contract v0.3](../specification/calculation-contract-v0.3.md), [Accuracy budget](../specification/accuracy-budget.md), [Reference cases](../specification/reference-cases.md), [USNO matrix report](../specification/usno-matrix-v1-report.md), [Prayer-library report](../specification/prayer-library-v1-report.md), [Decision record](decisions.md), [Phase 1 plan](phase-1-validation.md).
**Assignment boundary:** no Rust calculation, method ID/revision, output, or release-claim changes. No UI, time-zone, notification, Qibla, or high-latitude implementation. Phase 1 is not complete. No qualified scholarly or astronomical review has occurred; all reviewer roles below are explicitly vacant unless a signed record says otherwise.

## 1. Master execution plan (pointer + remainder)

Phases, gates, and claim limits live in [roadmap.md](roadmap.md) Phases 0–6. This section summarizes the remainder so a delegator can place one packet without redefining the roadmap.

### Current state (verified)

Offline core calculates one solar cycle from explicit coordinates, local Gregorian date, manually supplied fixed UTC offset, one of two angle profiles (`research-15` 15°/15°, `mwl-angles-18-17` 18°/17° with angles as listed in the PrayTimes methods-table webpage, both zero Dhuhr/Maghrib adjustment, `high_latitude_rule="none"`), and Standard/Hanafi Asr. The audited PrayTimes v2 JS source file is a separate artifact from the webpage (see §3). Returns UTC instants rounded to the second with raw seconds preserved, or typed `Unavailable`. Comparison evidence: 19-case USNO solar matrix (48 in-allowance, 6 no-event match, 2-cell 65.735°N grazing disagreement open, 1 unreported) and 28-row prayer-library matrix (Asr gap P1.3-A1 residual ~16 s open, London fallback P1.3-H1, polar-night Asr P1.3-P1, Kiritimati cycle P1.3-D1 resolved per-case only). No IANA lookup, zone data, reviewed regional default, fallback, Qibla, binding, or consumer app. See [roadmap Current position](roadmap.md#current-position).

### Remaining path (gates owned by roadmap)

| Phase | Deliverable | Gate evidence (roadmap) | Out of scope at gate |
| --- | --- | --- | --- |
| 1. Validation/method integrity (active) | Error-budget protocol, source manifests, discrepancy ledger, method provenance + review state, event/rounding definitions, gate report | Supported regimes + limits stated; every material discrepancy resolved, reproduced, or explicitly excluded; named profiles carry sourced params + review status | No consumer accuracy claim, no regional default, no global 1900–2100 claim |
| 2. Civil time/difficult geo | Versioned offline IANA rules, coord→zone mapping with ambiguity + manual choice, DST/date-line behavior, high-latitude policies, offline Qibla, elevation/horizon policy | UTC→local under recorded data version; gaps/overlaps/borders/stale/polar/fallbacks explicit; no silent zone or rule guess | No app-store release, no mosque-feed dependency |
| 3. Portable platform | Reviewed public Rust API + reproducibility record; CLI/WASM/mobile adapters | Same input vectors → equivalent UTC + statuses on native/browser/iOS/Android; no client prayer math | No full design system |
| 4. Trustworthy experience | Accessible today/settings/location/Qibla/reminder/“Why this time?” interaction model | Usability review: older users, screen readers, RTL, uncertainty, manual correction, calculated-vs-mosque distinction | No content/social/AI-times features |
| 5. Offline consumer beta | iOS/Android/web on one core; local settings + opt-in reminders | Real-device/browser offline, travel, DST, denial, scheduling limits, restart/update | No Hijri certification, no timetable-feed launch |
| 6. Public release/stewardship | Independent review, privacy/security review, signed data update + rollback, ownership/license/funding, archive | Sign-off on scope/wording; archived code+data reproduce release; limits + issue path published; maintainer can update data without replacing engine | No “unchanged binary works to 2050” promise |

Dependencies: 1 → 2 → 3 → 5 → 6; 4 runs alongside 3 once the result model is stable; governance, accessibility, privacy, security, licensing start in Phase 1 and continue. Parallel work that must not weaken the critical path: license/stewardship discussion, TZ polygon licensing/size notes, tiny non-prayer binding probes, UX sketches — all explicitly non-gating per [roadmap](roadmap.md#parallel-work-and-decision-owners).

Three review tracks stay separate: engineering validation (maintainer + astronomy reviewer), Islamic-methodology wording/defaults (qualified reviewer, no fatwas), product release scope (product/accessibility + maintainer). One person may hold multiple roles, but an unfilled role is shown as unfilled and blocks its claim class. No calendar date or version number passes a gate.

Coverage required before release claims: civil time, high latitudes, Qibla, Rust API, WASM, mobile bindings, offline data + rollback, UI, notifications, accessibility, privacy, security, licensing, maintenance — each gated in its phase above, not in Phase 1 packets.

## 2. Architecture contracts

### Boundaries (implementation today: `crates/salah-core/src/`)

| Boundary | Owns | Must preserve across boundary |
| --- | --- | --- |
| Astronomy (`solar.rs`, private) | Declination/EoT, transit iteration, altitude, direction-aware crossing + interior-extremum check | Model ID (`NOAA-MEEUS-SOLAR-2`), horizon assumption (−0.833° sea-level, no terrain/weather), time scale (UTC≈UT1), raw UTC seconds |
| Prayer rules (`prayer.rs`, `method.rs`) | Twilight angles, Asr shadow factor (Standard 1 / Hanafi 2, baseline at transit), Dhuhr/Maghrib adjustments, `EventRule`, `Unavailable` | Method ID/revision/source/params, Asr criterion, adjustment chain, `high_latitude_rule="none"`, engine version |
| Civil time (`civil.rs`; future IANA mapper separate) | Validated `CivilDate` 1900–2100, `FixedUtcOffset` ±840 min, `UtcInstant`→local conversion | Fixed offset only today; zone ID and TZDB data version are Phase 2 additions. Local-date selection rule: nearest transit ± half-cycle; Isha may cross midnight |
| Location lookup (app-side, not core) | GPS/manual/city provenance, accuracy, saved places | Coordinate source label; core never infers provenance |
| Qibla (absent; future module) | Great-circle bearing to documented Kaaba coordinate | Bearing ≠ compass heading; sensor guidance separate with calibration warning |
| Platform adapters (future) | Permissions, storage, local notifications, accessibility hooks | Scheduling limits, permission states, rescheduling triggers; no prayer math |
| Presentation (future) | Rounding, 12/24-h, language, RTL, “Why this time?” two-level explanation | Rule that produced each displayed minute; statuses never rounded away |

Rules: UTC instants + typed statuses cross every boundary before local display; device zone is never assumed to be the location zone; mosque timetable and iqamah stay separate labeled data; high-latitude substitution only as a separately named, reviewed, visibly adjusted rule; adjustments are visible transformations after the base result.

### Reproducibility when methods and TZ data change

`CalculationRecord` today carries coordinates, local date, fixed offset only (no zone ID, no TZDB version — those are Phase 2 additions), model ID, method ID/revision/params/source, Asr, high-lat rule, elevation assumption, and engine version. Old contract/model/profile identities are immutable; a behavior change mints a new versioned specification or contract, adds a regression case, documents migration, and preserves the prior manifest/report — never rewrites history and never amends a historical contract in place. Historical reproduction uses archived code + archived data; current best-estimate for a future date is a different task because civil law can change.

### Open decisions and evidence to settle them

Institutional profile sources + reviewer process (primary doc needed; secondary table insufficient); solar-model error bound beyond selected cases (matched-assumption ephemeris comparison); near-grazing classification (separate ephemeris under explicit definitions); polar/high-latitude policy + wording (astronomy + methodology review); elevation/atmosphere policy; display-minute rounding + notification policy (P1.5); TZ boundary dataset license/size/update/rollback; license/maintainers/funding; languages/platforms/notification limits. No Mars, spacecraft, AI-calculated times, or large backend abstractions before Earth phases pass.

## 3. Delegation backlog (Phase 1 only)

A smaller model must never be asked to “make prayer times accurate” without a contract section and reference rows. Each packet below is independently reviewable. Later phases stay as [roadmap](roadmap.md) skeletons; no Phase 2 implementation packets are defined here.

### P1.4-M1 — Method provenance and review state (next; smaller model suitable; senior doc review; methodology review remains vacant)

* **Objective/reason:** trace every named profile to its source and mark review state so a secondary table cannot become implied endorsement or default. Advances [Phase 1 P1.4](phase-1-validation.md#p14--method-provenance-and-religious-review); see GitHub issue #4.
* **In scope:** new `data/reference/method-sources-v1.tsv` (one row per profile) + new `specification/method-register-v1.md` (wording + review table); one link-only addition described below. No edits to any other existing file.
* **Link-only edit (exact):** Muse may add one link-only line immediately below the P1.4 heading in `docs/phase-1-validation.md` to reference the two new files. It must not edit `AGENTS.md`, `roadmap.md`, `decisions.md`, `specification/calculation-contract-v0.3.md` (preserved as historical), or any other existing document.
* **Out of scope:** `crates/**` formulas, method IDs/revisions, outputs, thresholds, new profiles, defaults, endorsements, fatwas, quotes, scores.
* **Sources required:** per profile: exact Fajr/Isha angles, Dhuhr/Maghrib adjustments, Asr applicability, missing-event/high-lat behavior, version, source edition/URL + publication/retrieval date, region/community, Ramadan/interval behavior (note if kernel lacks it), high-lat behavior, intended use. Two PrayTimes artifacts must be recorded separately: (a) the methods-table webpage `https://praytimes.org/docs/methods` (human-readable table; retrieval date required; the stated JS-file hash does not apply) and (b) the v2 JS source file `https://praytimes.org/code/v2/js/PrayTimes.js` (SHA-256 `f0e8442410446b1bf2db613bdebd3aa3aef5cdf5483068fbd9212d5acaa180fd` — this hash identifies the JS file only, never the webpage). Prefer a primary institutional document; the two PrayTimes artifacts above are SECONDARY. State explicitly if no primary source was found or if it differs.
* **Acceptance evidence:** TSV validates (stable IDs; every cell holds a value, `not_applicable` with reason, `unknown` where looked-for-but-not-found, or `unreviewed` where review is pending — `not_applicable` and `unknown` must not be conflated); every wording claim links to source+date; `research-15` labeled non-institutional; `mwl-angles-18-17` labeled parameter set, not endorsement/default; gaps + reviewer vacancies explicit; `cargo fmt --check`, `clippy -D warnings`, `cargo test --locked --offline --workspace` green; diff touches the two new files plus the single permitted link line only.
* **Edge cases:** Ramadan interval profiles the kernel cannot express; regional variants sharing a name; secondary-vs-primary angle conflicts; wording that mixes technical definition with fiqh choice.
* **Report back:** diff stat, TSV+md paths, source table with hashes/dates, gap list, assumptions, discrepancies found, test logs, limitations paragraph.

### P1.5-E1 — Rounding options survey (smaller model gathers; architect decides; senior architecture review required)

* **Objective/reason:** survey sourced minute-rounding options and illustrate each with worked examples so the architect can approve the final ordering rule. Defines nothing normative yet. Advances [Phase 1 P1.5](phase-1-validation.md#p15--event-and-rounding-contract).
* **In scope:** new `specification/rounding-survey-v1.md` only (options + worked examples + open questions); one link-only addition: a line immediately below the P1.5 heading in `docs/phase-1-validation.md` may reference it.
* **Out of scope:** amending `specification/calculation-contract-v0.3.md` (preserved as historical — any new behavior definition belongs in a future versioned specification or contract, not in v0.3); selecting the final ordering rule (architect decision); solver/method/output changes; minute-rounding or notification implementation.
* **Ordering question (explicit, not assumed):** gather sourced options for whether Dhuhr/Maghrib adjustments apply before or after minute rounding (and any further orderings found in sources), per method where sources differ. For each option record: sources consulted, exact rule statement, and worked illustration with cited vectors (e.g. Minneapolis 2026-09-27, Makkah 2026-03-20, London 2026-06-21 Isha-after-midnight, Tromsø no-event) showing raw seconds → adjusted beginning → displayed minute under that option. Present tradeoffs and a non-binding observation; do not declare a final rule and do not default to adjustment-before-rounding.
* **Acceptance evidence:** options table with sources; worked examples derived from cited vectors for each option; Maghrib/sunset separation, missing-event rendering, and post-midnight Isha date handling each illustrated; `EventRule` gap note (what the schema shows today vs what a minute rule needs); open decision questions listed for the architect.
* **Report back:** new survey path, options table with sources, example table with UTC + source links, non-binding observation, unresolved items, limitations.

### P1.5-I1 — Implement optional prayer-start display adapter (completed; reviewed in `0be7aee`)

* **Objective/reason:** implement the architect's [presentation contract v0.1](../specification/presentation-contract-v0.1.md) without changing prayer calculation results. Advances the Phase 1 P1.5 schema and example acceptance line; does not close Phase 1.
* **In scope:** `crates/salah-core/src/presentation.rs` (new, pure presentation logic), `crates/salah-core/src/lib.rs` (export), `crates/salah-cli/src/main.rs` (opt-in `--display-minute` view), and a focused presentation regression test file. No external dependency.
* **Out of scope:** `prayer.rs`, `solar.rs`, `civil.rs`, `method.rs`, existing contracts, profile IDs/revisions, astronomy, raw/adjusted UTC outputs, existing CLI output without the new flag, time-zone lookup, notification scheduling, mosque data, or high-latitude fallback.
* **Acceptance evidence:** a typed receipt carries prayer name, occurring/unavailable status, actual adjusted UTC when present, local date and minute when present, source `EventRule`, method ID/revision, and presentation-policy ID/revision; applies only to Fajr/Dhuhr/Asr/Maghrib/Isha; preserves sunrise/sunset seconds and unavailable reasons; uses Euclidean ceiling after the existing adjusted second; never double-applies an adjustment; carries London midnight and pre-1970 date boundaries; normal CLI output remains byte-for-byte unchanged without `--display-minute`; all cited contract examples and hypothetical nonzero-adjustment case have regression assertions. No consumer accuracy or religious claim.
* **Report back:** precise diff, API shape, test outputs, preserved-output check, edge-case results, limitations. Stop for architect code review; no commit or push.

### P1.2-G1 — Independent near-grazing ephemeris audit (next; evidence gathering only)

* **Objective/reason:** investigate the documented 65.735° N sunrise/sunset **event-existence** disagreement with an independent Sun ephemeris under an explicitly stated, fixed solar-center horizon. Determine which differences are ephemeris geometry, horizon/refraction definition, civil-cycle mapping, or still unknown. Advances P1.2 evidence and the [USNO discrepancy ledger](../specification/usno-matrix-v1-report.md), without selecting a new physical model or passing the Phase 1 gate.
* **In scope:** new `data/reference/grazing-horizons-v1.tsv` (small, provenance-rich source manifest) and new `specification/grazing-independent-audit-v1.md` (definitions, comparison, limitations); one link-only line under P1.2 in `docs/phase-1-validation.md`. The external reference is NASA/JPL Horizons observer output for the Sun with user-defined Earth geodetic site, `APPARENT=AIRLESS`, quantity 4 azimuth/elevation, and source-recorded UT semantics. The already pinned USNO responses and Salah CLI provide the other two evidence streams.
* **Required cases:** 2026-06-21 at 0° longitude, 0 km site height, latitudes 65.72°, 65.735°, and 65.74°; inspect **both** adjacent solar-midnight windows so a 00:00 civil-day event from the previous cycle cannot be mistaken for the selected evening event. Resolve the selected cycle around the 2026-06-21 upper transit. Record source minimum Sun-center altitude and UTC time, margin relative to Salah's fixed −0.833° center-altitude threshold, and event/no-event status under that **fixed threshold**, separately from any source's own rise/set rule. Use fine enough sampling near the minimum to bound a missed crossing; record the sampling resolution and its limitation.
* **Source discipline:** pin each exact Horizons request URL, raw-response SHA-256, retrieval UTC date, API identity, ephemeris and Earth-orientation tags shown in the response, site-coordinate convention/datum/height, time scale, refraction setting, quantity definition, output precision, and sampling. Cite the official [Horizons API documentation](https://ssd-api.jpl.nasa.gov/doc/horizons.html) and [manual](https://ssd.jpl.nasa.gov/horizons/manual.html) for those semantics. Do not confuse output decimal places with physical accuracy. Preserve the existing USNO request URLs/hashes and its minute-formatted event values; unknown USNO horizon/refraction details stay `unknown`. Do not commit raw source bodies unless their archival rights are established; URLs/hashes/selected facts suffice for this packet.
* **Acceptance:** TSV rows have stable IDs, no empty/ragged fields, and separate source measurements from derived margins; report includes per-latitude, per-window evidence table, local-solar-cycle mapping, source-definition comparison, and sensitivity of event existence to a small stated horizon change. State what the independent source supports and what it cannot establish. Any apparent JPL agreement with Salah under −0.833° is evidence for that *chosen* threshold only, not proof of a universally observed sunrise. No code, threshold, tolerance, contract, or release-claim change.
* **Report back:** exact source URL/response hash table, JPL metadata, source vs derived values, USNO/Salah status table, discrepancies and uncertainty, verification commands/results, limitations. Stop for architect review; no commit or push.

### P1.6-G1 — Gate report skeleton (senior only; blocked until P1.4/P1.5 + open ledger items progress)

* **Objective:** summarize matrix + ledger by regime/event; state verified, excluded, and reviewer-signed wording; record whether numeric/source criteria are met. Keeps Phase 1 open or narrows scope if unmet. No consumer UI claim.
* **Out of scope:** passing the gate by editing tolerances or adding fallbacks.
* **Acceptance:** per-regime table with signed Δ, maxima, no-event/status cells, open IDs (grazing, P1.3-A1/H1/P1) with owners; reviewer-signature table showing vacancies.

## 4. Copy-paste agent prompts

### First packet prompt — P1.4-M1 (self-contained; give verbatim to Muse 1.3)

```text
You are implementing P1.4-M1 (method provenance and review state) in /Users/shoaibakthar/Documents/Salah for GitHub issue #4.

READ FIRST (source of truth): AGENTS.md; docs/roadmap.md; docs/phase-1-validation.md P1.4; docs/decisions.md; specification/calculation-contract-v0.3.md; specification/accuracy-budget.md; specification/reference-cases.md; specification/prayer-library-v1-report.md; data/reference/README.md; crates/salah-core/src/method.rs (parameters only — do not change it).

TASK: create data/reference/method-sources-v1.tsv (one row per profile: research-15, mwl-angles-18-17) and specification/method-register-v1.md (parameter tables + short in-app wording + review-state table). For each profile record: Fajr/Isha angles, Dhuhr/Maghrib adjustments, Asr applicability, missing-event/high-latitude behavior (the Salah profiles currently have no fallback; record any external source method behavior separately), version, source edition/URL + publication/retrieval date, region/community, Ramadan/interval behavior and whether the kernel implements it, intended use. Two PrayTimes artifacts must be recorded separately and never conflated: (a) the methods-table webpage https://praytimes.org/docs/methods (human-readable table; record its retrieval date; the stated JS-file hash does not apply to it) and (b) the v2 JS source file https://praytimes.org/code/v2/js/PrayTimes.js with SHA-256 f0e8442410446b1bf2db613bdebd3aa3aef5cdf5483068fbd9212d5acaa180fd (this hash identifies the JS file only, never the webpage). Prefer a primary institutional document; both PrayTimes artifacts above are SECONDARY. State explicitly if no primary source was found or if it differs. Every TSV cell must hold a value, `not_applicable` with a reason (e.g. research profile has no region), `unknown` where you looked but did not find the fact, or `unreviewed` where qualified review is pending — never conflate `not_applicable` with `unknown`. Draft short user-facing explanations for (a) method choice, (b) calculated beginning vs mosque timetable/iqamah, (c) unavailable high-latitude events; mark each Reviewed or Pending qualified Islamic-methodology review (all Pending — no such review has occurred). Keep research-15 non-institutional; describe mwl-angles-18-17 as a published parameter set, never an endorsement, default, or complete timetable. No fatwas, quotes, citations you cannot verify, or confidence scores.

DO NOT: change crates/**, method IDs/revisions, formulas, outputs, tolerances, contracts’ behavior sections, or release claims. Do not edit specification/calculation-contract-v0.3.md (historical). Do not add profiles, defaults, UI, TZ, Qibla, or notifications. Preserve unrelated files including .idea/.

UPDATE LINKS ONLY: you may add exactly one link-only line immediately below the P1.4 heading in docs/phase-1-validation.md to reference the two new files. Do not edit AGENTS.md, docs/roadmap.md, docs/decisions.md, specification/calculation-contract-v0.3.md, or any other existing file; do not rewrite the roadmap gate.

VERIFY: cargo fmt --all -- --check; cargo clippy --locked --offline --workspace --all-targets -- -D warnings; cargo test --locked --offline --workspace. All must pass; doc/data-only diff expected.

RETURN: files changed + diff stat; per-parameter source table (source, edition/URL, date, hash where applicable); primary-source gaps; assumptions; any discrepancies found (do not fix beyond scope); test logs; a limitations paragraph stating reviewer vacancies and unsupported claims. Stop after the register is reviewable; do not implement P1.5 or Phase 2.
```

### Survey packet prompt — P1.5-E1 (historical; completed)

```text
You are preparing P1.5-E1, a rounding-options SURVEY, in /Users/shoaibakthar/Documents/Salah. This is evidence gathering for an architect decision, not an implementation or a decision about which rounding rule is correct.

READ FIRST: AGENTS.md; docs/roadmap.md; docs/phase-1-validation.md P1.5; docs/director-handoff.md §3 P1.5-E1 and §5; docs/decisions.md; specification/calculation-contract-v0.3.md; specification/accuracy-budget.md; specification/reference-cases.md; specification/prayer-library-v1-report.md; specification/usno-matrix-v1-report.md; data/reference/prayer-library-v1.tsv; data/reference/solar-usno-v1.tsv; specification/method-register-v1.md. Read current EventRule/result types for schema description only; do not edit Rust.

TASK: create specification/rounding-survey-v1.md. Survey sourced options for converting raw UTC event instants into adjusted prayer beginnings, displayed timetable minutes, and later notification instants. Explicitly investigate whether Dhuhr/Maghrib adjustments are applied before or after minute rounding, whether sources specify this per method, and any other relevant orderings. Distinguish a sourced practice from a hypothetical comparison option. Present tradeoffs and a non-binding observation; do not select a final rule.

For EACH option, state its exact operation order, source URL/edition/retrieval or publication date, what the source actually establishes, and gaps. Include worked examples from cited existing vectors: Minneapolis 2026-09-27, Makkah 2026-03-20, London 2026-06-21 Isha crossing midnight, and Tromsø no-event. Show the underlying UTC instant with seconds, any method adjustment, resulting prayer beginning, and candidate displayed minute. Identify the source row/case ID for every observed value. Because current profiles have zero adjustments, include at least one explicitly hypothetical nonzero-second adjustment applied to a cited raw instant to expose any ordering difference; do not present it as an implemented method or reference observation. If an exact raw instant is unavailable in existing evidence, state the gap rather than inventing one. Show Maghrib separately from sunset, preserve an unavailable event as unavailable, and preserve the correct date when Isha crosses midnight. Describe what EventRule/result schema currently exposes and what a future minute-display rule would need; notification instants remain a separate unscheduled concept.

IN SCOPE: new specification/rounding-survey-v1.md only, plus exactly one link-only line immediately below the P1.5 heading in docs/phase-1-validation.md pointing to it. Preserve the existing P1.4 link line.

DO NOT: modify crates/**, existing contracts (especially historical v0.3), methods, formulas, outputs, tolerances, reference TSVs, roadmap gates, AGENTS.md, or decisions.md. Do not implement rounding or notifications, choose a normative order, make a scholarly ruling, make a consumer accuracy claim, start P1.6 or Phase 2, commit, or push. Preserve unrelated files including .idea/.

VERIFY: check every cited vector against its source file; check relative links and the exact file diff; run cargo fmt --all -- --check, cargo clippy --locked --offline --workspace --all-targets -- -D warnings, and cargo test --locked --offline --workspace.

RETURN: files changed and diff stat; source/options table; worked UTC examples with citations; what remains unknown by method; EventRule/schema gap; non-binding observation; verification results; limitations. Stop for architect review.
```

### Implementation packet prompt — P1.5-I1 (historical; completed)

```text
You are implementing P1.5-I1 in /Users/shoaibakthar/Documents/Salah. Read AGENTS.md; docs/roadmap.md; docs/phase-1-validation.md P1.5; docs/director-handoff.md §3 P1.5-I1 and §5; docs/decisions.md; specification/presentation-contract-v0.1.md; specification/rounding-survey-v1.md; specification/calculation-contract-v0.3.md; crates/salah-core/src/prayer.rs, civil.rs, method.rs, lib.rs; crates/salah-cli/src/main.rs. The presentation contract v0.1 is the decision. The survey is evidence; its corrected second-rounding note must not be replaced by a commutation assumption.

TASK: implement a pure, typed, optional prayer-start display adapter in the Rust core and an opt-in CLI --display-minute view. The adapter accepts a PrayerTimes result and one of the five prayer-start names, selects its Event internally, and returns a typed receipt with prayer name, actual adjusted UTC instant if occurring, local displayed date/HH:MM if occurring, source EventRule if occurring, method ID/revision, display-policy ID/revision (`prayer-start-ceil-minute`, `0.1`), or the original UnavailableReason with no minute. Use the existing Event.utc exactly once as the adjusted beginning. Convert it with the record's FixedUtcOffset, then use Euclidean integer arithmetic to choose the first whole local minute at or after that second. Preserve date rollover. Do not round the raw f64 again, reapply an adjustment, or change a calculation result. Sunrise and sunset have no prayer-start minute receipt; retain their existing second-precision CLI lines.

CLI: --display-minute is a boolean opt-in flag; existing invocations without it must keep their current output byte-for-byte. When present, append a clearly labeled research-preview section naming the display-policy ID/revision and showing the five prayer-start local date/HH:MM labels or unavailable reasons. The existing second-precision lines remain visible. Do not describe the labels as certified timetable or fasting cutoffs.

FILES IN SCOPE: new crates/salah-core/src/presentation.rs; crates/salah-core/src/lib.rs export; crates/salah-cli/src/main.rs; a new focused regression test file under crates/salah-core/tests/. Add no dependency. If a sound implementation truly requires another file, stop and report the reason rather than widening scope silently.

VERIFY with regression assertions for every cited kernel example in presentation-contract-v0.1.md: Minneapolis Dhuhr/Maghrib, Makkah Dhuhr, London Isha with 06-22 local date, and Tromsø unavailable. Also cover exact minute, one second after, 23:59:59 date rollover, a pre-1970 Unix second using Euclidean division, and a hypothetical valid nonzero Dhuhr/Maghrib adjustment without changing built-in profiles. Show that the adjusted Event.utc is used once and source sunset remains distinct from Maghrib. Check that ordinary CLI output without the flag is byte-for-byte unchanged. Run cargo fmt --all -- --check; cargo clippy --locked --offline --workspace --all-targets -- -D warnings; cargo test --locked --offline --workspace. Report exact commands/results.

DO NOT edit prayer.rs, solar.rs, civil.rs, method.rs, Cargo.toml, existing contracts, methods, outputs, thresholds, reference data, roadmap, AGENTS.md, or decisions.md. Do not add TZDB, notifications, UI, high-latitude fallback, Ramadan/fasting rules, or consumer claims. Preserve .idea/. Do not commit or push. Return the diff, typed API description, test evidence, limitations, then stop for architect review.
```

### Next packet prompt — P1.2-G1 (near-grazing audit; give verbatim to OpenCode)

```text
You are conducting P1.2-G1, an INDEPENDENT EVIDENCE AUDIT, in /Users/shoaibakthar/Documents/Salah. Start from the current main commit. Read AGENTS.md; docs/roadmap.md; docs/phase-1-validation.md P1.2; docs/director-handoff.md §3 P1.2-G1 and §5; specification/accuracy-budget.md; specification/calculation-contract-v0.3.md; specification/usno-matrix-v1-report.md; data/reference/README.md; the three grazing rows in data/reference/solar-usno-v1.tsv; and crates/salah-core/src/solar.rs and prayer.rs READ ONLY.

TASK: create data/reference/grazing-horizons-v1.tsv and specification/grazing-independent-audit-v1.md. Investigate the 2026-06-21 near-grazing apparent-horizon event-existence disagreement at longitude 0°, latitude 65.735° N, with 65.72° and 65.74° as bracketing cases. Use NASA/JPL Horizons as an independent Sun-center altitude source. Consult its official API documentation https://ssd-api.jpl.nasa.gov/doc/horizons.html and manual https://ssd.jpl.nasa.gov/horizons/manual.html for each output definition. Use observer ephemeris for Sun target 10, center coord@399, geodetic site longitude 0°, each latitude, height 0 km; request quantity 4 azimuth/elevation with APPARENT=AIRLESS, CSV output, extra precision, and explicit UT time setting. Record what Horizons actually says about altitude, time scale, geodetic site, refraction, ephemeris, and Earth-orientation data. If a setting is unclear, mark it unknown; do not infer it from its name.

For each latitude, sample both 2026-06-20/21 and 2026-06-21/22 solar-midnight windows. First locate the minima with a coarse interval; then use a narrow, finer request around each minimum. Horizons STEP_SIZE unitless mode can divide a short requested interval into sub-minute output steps; verify this in official documentation and record the exact interval and step. Use the same fixed Salah solar-center threshold h0 = −0.833° only for a MATCHED-THRESHOLD comparison: compute sampled minimum airless Sun-center elevation minus h0 and classify whether that sampled solar arc dips below h0. To assert a crossing, show a sign bracket on each relevant side of the minimum; a sampled minimum alone is insufficient. Show the classification sensitivity at h0 ±0.005° without adopting either altered threshold. Mark sampling/precision limitations; do not declare an exact tangent from a sample alone. Distinguish this derived status from Horizons' own rise/set events and from the USNO API's minute-formatted Rise/Set labels.

Compare with the pinned USNO rows usno-2026-6572-grazing, usno-2026-65735-grazing-disagreement, and usno-2026-6574-continuous. Keep their request URLs, response hashes, original civil-day event text, and selected local-solar-cycle interpretation separate. In particular, the 65.735° response also has a 00:00 Set from the preceding cycle; do not treat that as the selected evening Set. Obtain Salah's event status from the unmodified CLI for the same date, coordinates, and UTC+00:00. Do not compare a time difference where one side has no event.

TSV: use stable case IDs and one row per latitude/window/source request. Every field must have a value, or a reasoned unknown/not_applicable marker. Include exact request URL, raw response SHA-256 (hash the retrieved response bytes; state whether the hash covers the JSON wrapper or embedded Horizons result), retrieval UTC date, source/API/version, ephemeris/EOP tags, site coordinate and height, time scale, refraction, quantity, sampling, minimum altitude/time, fixed threshold, derived margin/status, and source precision. Keep derived fields labeled as derived. Do not commit full remote response bodies until rights are known. The report must cite every TSV row and explain any source or model-definition mismatch.

As an exploratory lead ONLY, a prior architect query suggested that the 65.735° airless JPL minimum near 2026-06-22 00:02 UT lies roughly 0.002° ABOVE −0.833°; this was not yet an audited reference. Re-fetch and verify independently. Do not copy that figure as expected truth, tune a threshold to it, or change Salah's formula. Quantify the threshold sensitivity in angular units and explain why a small horizon/refraction assumption may flip event existence. If the independent source does not establish enough to classify the cause, say unresolved.

FILES IN SCOPE: the two new data/specification files, plus exactly one link-only line directly below the P1.2 heading in docs/phase-1-validation.md. Do not edit any crates/**, historical contract, existing reference TSV/report, accuracy budget, tolerance, horizon constant, method, roadmap, AGENTS.md, or decisions.md. Preserve .idea/. Do not implement fallback, switch ephemerides, or claim global accuracy or religious certainty. No commit, push, PR, or P1.6 work.

VERIFY: independently re-read all cited source rows and hashes; check TSV schema and local Markdown links; run cargo fmt --all -- --check, cargo clippy --locked --offline --workspace --all-targets -- -D warnings, and cargo test --locked --offline --workspace. Return exact diff and untracked-file stat; source URL/hash table; per-latitude/window altitude and margin table; USNO/Salah status mapping; what is established vs unknown; verification results; limitations. Stop for architect review.
```

### Reusable packet template

```text
You are implementing PACKET_ID in /Users/shoaibakthar/Documents/Salah.

READ FIRST: AGENTS.md; docs/director-handoff.md §2–§3 + §5; the packet’s listed contracts/reports/manifests; the exact modules named in scope.

OBJECTIVE: <one sentence + gate effect>
IN SCOPE: <files/modules/behavior/data>
OUT OF SCOPE: <explicit exclusions — formulas, IDs, outputs, claims, platforms>
SOURCES: <primary preferred; edition/URL/date/hash; distinguish not_applicable (with reason) from unknown (looked, not found) and unreviewed (review pending)>
ACCEPTANCE: <vectors, tolerances with rationale, commands, doc updates>
EDGES: <regimes, missing events, date-line, boundaries>
DO NOT: invent times/vectors/citations/quotes/scores; widen tolerances to pass; present library agreement as physical/religious proof; infer defaults/endorsements.
VERIFY: <fmt/clippy/test + link/schema checks>
RETURN: diff stat, evidence tables, gaps, assumptions, discrepancies, logs, limitations. Stop at reviewable handoff.
```

## 5. Review and merge protocol (cheap-model work)

1. **Diff review:** P1.4: two new files plus one link-only line immediately below the `phase-1-validation.md` P1.4 heading. P1.5-E1: new survey file plus one link-only line immediately below its P1.5 heading. P1.2-G1: two new audit files plus one link-only line immediately below its P1.2 heading. For evidence packets, no `crates/**` behavior change, ID/revision/output/threshold silent edit, or amendment to a historical contract.
2. **Contract review:** does the change alter specified behavior? If yes, reject without a new versioned contract or specification + regression case + migration note.
3. **Source/provenance review:** primary preferred; secondary labeled; edition/URL/date/hash present; `unknown`/`unreviewed` where honest; no invented citation, quote, ruling, or score.
4. **Offline checks:** `cargo fmt --all -- --check`; `cargo clippy --locked --offline --workspace --all-targets -- -D warnings`; `cargo test --locked --offline --workspace`; TSV schema + relative-link check (`../specification/*`, `../data/reference/*`, `roadmap.md`, `phase-1-validation.md` anchors).
5. **Limitations statement required:** vacancies, gaps, unsupported claims restated in the PR/report.
6. **Blocking findings:** unexplained regime failure; tolerance widened after failure; two-library agreement treated as physical/religious proof; named angles presented as endorsement/default; missing reviewer sign-off presented as reviewed; any invented time, vector, source, quote, or score; gate or accuracy claim from a selected subset.
7. **Gate effect:** P1.2-G1/P1.4/P1.5 evidence may advance their checklist lines; only the maintainer records a gate decision in [decisions.md](decisions.md) with evidence links. Phase 1 stays open until P1.6 criteria are met.

## 6. Handoff summary

* **Completed technical evidence:** P1.4-M1 method-source manifest and register were reviewed and committed as `7145b2d`. The secondary-source provenance is explicit. Qualified Islamic-methodology review remains vacant, so P1.4 religious review is open and no consumer method endorsement follows.
* **Completed survey and decision:** P1.5-E1 evidence is in the [rounding survey](../specification/rounding-survey-v1.md). The architect corrected its second-rounding and post-adjustment examples, then selected a bounded research-preview [presentation policy](../specification/presentation-contract-v0.1.md). Neither document certifies an institutional method or consumer timetable.
* **Completed implementation:** P1.5-I1's optional typed display adapter and CLI view were reviewed and committed as `0be7aee`. The normal CLI output and calculation UTC results remain unchanged.
* **Open questions for human owner/expert:** acceptability of PrayTimes-secondary-only provenance for an MWL-associated label; who fills astronomy + Islamic-methodology + civil-data + product/a11y roles; license/funding path; TZ response-archival rights and mirror; Qibla reference coordinate; rounding-policy authority per method.
* **Exact next packet:** P1.2-G1 using the verbatim prompt in §4. Its JPL source is audit-only; a matched fixed-threshold comparison does not certify physical sunrise. **Ready to delegate:** P1.2-G1 evidence audit. **Blocked pending external review:** P1.6-G1 gate, any regional default or endorsement, polar/high-latitude policy, global accuracy claim, consumer release.
