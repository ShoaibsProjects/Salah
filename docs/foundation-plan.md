# Foundation plan: from vision to a working core

**Status:** historical foundation plan for the first Rust slice. The active phase plan is [Delivery roadmap](roadmap.md), with current work in [Phase 1 validation](phase-1-validation.md). Delegation packets live in [Director handoff](director-handoff.md).
**Working rule:** finish small, independently checkable capabilities before expanding platforms or feature count.

## Current progress

The repository now has a dependency-free `salah-core` crate and an offline `salah-cli`. It calculates a solar cycle with an explicit fixed UTC offset and two angle profiles: `research-15` and the PrayTimes-documented `mwl-angles-18-17`. It preserves UTC results and typed unavailable events. Selected solar outputs are compared with USNO; selected prayer outputs are compared with Adhan JS and PrayTimes v2. The civil-date type now requires validated construction, and USNO cases cover the supported 1900 and 2100 year boundaries. See [reference cases](../specification/reference-cases.md). This remains a research prototype, not a released global prayer timetable. Institutional method endorsement, broader validation, IANA time zones, Qibla, and high-latitude fallback remain open.

## Decision on the proposed seven-milestone plan

Keep its central order: **Rust calculation core → independent validation → platform clients**. Four changes make the start safer:

1. **Specify behavior before formulas.** Define what a date, time zone, method, adjustment, rounding rule, and unavailable event mean. Otherwise two implementations can produce different times while both claim to follow the same named method.
2. **Validate as each rule lands.** Validation begins with the first solar event. Waiting until an entire engine exists makes numerical and method errors harder to isolate.
3. **Resolve the time-zone sequence.** The proposed plan requests a time zone in M1, then adds time zones in M3. The first slice will accept an explicit fixed UTC offset and output UTC instants; a versioned IANA zone and offline coordinate-to-zone lookup follow before claiming global civil-time accuracy.
4. **Use evidence gates, not a 30-day promise.** A trustworthy calculation and review may take longer than a week. The initial goal is a demonstrable offline slice, not a calendar deadline or a version-number ladder.

A lightweight UI sketch can inform usability at any time. Production screens, notifications, and a full design system should follow a validated calculation interface. A small Rust-to-WASM and mobile binding probe should happen early enough to reveal integration trouble, before a large UI investment.

## Milestones and gates

| Gate | Build | Evidence needed to advance |
| --- | --- | --- |
| F0 — Contract and sources | Record the calculation contract, first method profile and its source, astronomical reference, expected units, statuses, and rounding policy. Set up the Rust workspace and simple CLI shape. | Reviewers can state exactly what an input means and how one result should be checked. |
| F1 — First vertical slice | Given coordinates, a local date, an explicit UTC offset, and one documented parameter profile, calculate the six daily entries offline. Return UTC instants, unrounded details, and explicit event statuses. | CLI produces a reproducible result for several ordinary locations and dates; reference provenance is recorded. No illustrative time is copied into expected output. Institutional method names await source review. |
| F2 — Independent validation | Compare solar events with an astronomical reference and prayer outputs with at least two mature implementations under matched parameters. Add seasonal, hemisphere, Asr, leap-year, and numerical edge cases. | Every material discrepancy is explained or tracked; tolerances are stated for UTC instants, separate from method differences. |
| F3 — Civil time and difficult geography | Add versioned IANA rules, offline coordinate-to-zone mapping, DST/date-line cases, unavailable twilight statuses, selected high-latitude fallbacks, and Qibla bearing. | Tests cover transitions and missing events; the CLI shows zone source and any fallback applied. |
| F4 — Portable core | Stabilize typed Rust API and output record. Add CLI and WASM boundary; probe mobile binding. Keep calendars as a separate, documented module. | Same reference cases pass through the Rust API and WASM; consumers need no duplicate calculation logic. |
| F5 — First user experience | Build accessible iOS/Android daily view, manual and device location, method settings, explanation screen, Qibla, and opt-in local reminders. Build responsive web client using the core. | Real-device and browser reviews cover offline use, permissions, local-time display, travel, accessibility, and notification limits. |
| F6 — Long-term operations | Ship compatible, authenticated data updates for time zones and methods where needed; archive releases and reference data; establish review and funding governance. | A new maintainer can reproduce an old result and update data without rewriting the calculation engine. |

F0 through F3 are the near-term priority. F4 through F6 remain design constraints, not work to scaffold prematurely.

## The first buildable slice

Create a small Rust workspace with one core crate and one CLI crate. Avoid splitting astronomy, methods, time, and Qibla into separate crates until their interfaces are stable. The CLI should take latitude, longitude, local civil date, explicit UTC offset, method ID, and Asr option. Its output should include the requested daily events, UTC instants, local display times under the explicit offset, and the calculation record. An event that does not exist under the chosen definition should return a status, never an invented clock time.

The implementation order is:

1. Typed coordinates, dates, offsets, methods, result/status model, and input validation.
2. Documented solar position and transit under one astronomy model.
3. Sunrise and sunset with stated apparent-horizon assumptions.
4. Dhuhr and Maghrib as method-defined transformations of transit/sunset.
5. Fajr and Isha by the chosen method's angle or interval, including no-event status.
6. Asr under standard and Hanafi shadow criteria.
7. CLI explanation output and reference comparisons after each rule.

Do not use a sample printed in a planning conversation as a test oracle. For each expected value, record its source, method parameters, time basis, rounding, and acceptable discrepancy. The first method can be an explicitly named **research profile** if authoritative source review is pending; do not label it ISNA, MWL, or another institution until its actual parameters are confirmed.

## Initial repository shape

```text
Salah/
├── AGENTS.md
├── README.md
├── docs/
│   ├── vision-and-architecture.md
│   ├── decisions.md
│   └── foundation-plan.md
├── specification/
│   ├── calculation-contract-v0.1.md
│   ├── calculation-contract-v0.2.md
│   ├── calculation-contract-v0.3.md
│   └── reference-cases.md
└── crates/
    ├── salah-core/
    └── salah-cli/
```

CI checks formatting, lints, and reference cases on pushes and pull requests. A release tag means a documented, validated behavior; version numbers should follow actual compatibility needs. Avoid publishing `v1.0.0` simply because a checklist reached its last item.

## Work that can proceed independently

- Identify qualified reviewers for astronomy and Islamic-method descriptions.
- Confirm source documents and revision dates for the first regional methods.
- Research reusable time-zone boundary data, license, size, and update process.
- Probe Rust bindings for iOS, Android, and WASM with a tiny calculation-independent function.
- Sketch the today screen and “Why this time?” view with accessibility needs in mind.

These tasks inform decisions; they do not change the principle that the core must work offline.

## Open decisions before moving beyond the research slice

- Which institutional method profile is first, and who will review its parameters?
- What is the broader numerical error budget beyond the selected USNO and Adhan comparisons?
- Which high-latitude and near-tangency cases need an independent astronomical adjudication?
- How should display-minute rounding differ by use case and regional method?
- What license and maintainership model will let others sustain the free core?

The first core slice is implemented. Follow the active [Phase 1 validation plan](phase-1-validation.md) for the next bounded tasks. Do not present `research-15` as a named institution's timetable.
