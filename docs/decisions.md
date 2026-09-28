# Decision record

**Status:** living record. Change a decision deliberately and explain why; do not infer a settled choice from an illustrative example.

## Accepted project direction

| Decision | Reason |
| --- | --- |
| Support Earth first. | The prayer-time domain and validation work are substantial already. Future celestial settings would require separate physical and Islamic research. |
| Use Rust for the calculation core. | One portable implementation can serve native clients and WebAssembly while remaining independent of a UI framework. |
| Make core calculations work offline. | A person must be able to calculate times and Qibla without an account, remote API, or subscription. |
| Publish a versioned calculation specification and reference cases. | Results need to be explainable and reproducible, including after the original maintainers or interface change. |
| Use phase gates tied to evidence, with one bounded task at a time. | The active [roadmap](roadmap.md) prevents dates, version numbers, or a passing sample from being mistaken for product readiness. |
| Keep Islamic methods configurable and sourced. | Astronomy alone does not settle all jurisprudential and regional choices. |
| Keep mosque prayer-start and iqamah times separate from calculated events. | They answer different questions and may differ intentionally. |
| Keep the core free to users and private by default. | Access to daily prayer information must not depend on data collection or payment. |
| Exclude blockchain, social feeds, ads, and AI-generated times from the initial product. | They add cost or risk without improving the core calculation. |

## Working implementation choices

| Choice | Current position | Revisit when |
| --- | --- | --- |
| Mobile interface | Flutter is a candidate, not a dependency of the Rust engine. | After a small Rust binding probe and accessibility review. |
| Web interface | Rust-to-WASM with a responsive client is the leading approach. | After checking offline, browser, and deployment behavior. |
| First time input | Explicit fixed UTC offset for the initial research CLI; IANA zone support is required before claiming global civil-time accuracy. | At the Phase 2 civil-time milestone (see roadmap.md Phase 2). |
| First method profile | `research-15` uses 15° Fajr and Isha angles with zero adjustments. It carries no institutional attribution. | Replace or supplement only after source and scholarly review. |
| First published angle set | `mwl-angles-18-17` reproduces the Fajr and Isha angles in the PrayTimes MWL table with zero Dhuhr/Maghrib adjustment. It is a sourced parameter set, not an endorsement or complete regional timetable. | Revisit when primary institutional specifications and regional practice are reviewed. |
| Research-preview prayer-start minute display | The architect selected [`prayer-start-ceil-minute` revision `0.1`](../specification/presentation-contract-v0.1.md) on 2026-09-27: use the already adjusted UTC second, then show the first whole local minute at or after it. This is Salah's display rule for the five prayer beginnings, not a method or religious ruling. Sunrise/sunset remain second-precision events; fasting cutoffs and notifications need separate rules. | Revisit for sourced method conventions, IANA time-zone transitions, qualified methodology/product review, or any consumer release. |
| Crate layout | `salah-core` owns prayer calculations, `salah-cli` exposes the research command line, and experimental `salah-time` owns offline civil-time conversion. | Split further only when real module boundaries and independent reuse are clear. |
| Civil-time TZif parser probe | Jiff 0.2.37 is accepted for the separate [six-zone F2-TZ0 probe](civil-time-spike.md), with pinned caller-supplied TZif bytes and no host-zone fallback. Its license is Unlicense OR MIT; `salah-core` remains independent. | Review mobile/WASM builds, full data-pack and update design, dependency inventory, and performance before adopting for consumer use. |
| Update system | Versioned, authenticated data packs are a long-term target. | When method or time-zone update requirements and platform limits are known. |

## Unresolved decisions

- First calculation profile and authoritative parameter sources.
- Broader numerical accuracy budget and physical model limits; the optional research-preview prayer-start presentation policy is implemented but not approved for consumer use.
- High-latitude defaults by region and the scholarly review process.
- Time-zone boundary dataset, licensing, and update mechanism.
- Product license, maintainers, funding, and governance through 2050.
- Launch languages, platform minimums, and notification behavior by OS.

The repository name is **Salah**. “2050” describes the maintenance horizon, not a guarantee that software or civil-time data can remain unchanged until that year.

## Interim Phase 1 gate review — 2026-09-28

The [Phase 1 gate report v0.1](../specification/phase-1-gate-report-v0.1.md) records the selected-case comparisons and reviewer vacancies. Phase 1 remains **open**. The independent Horizons audit supports Salah's near-grazing status under its fixed threshold, but the USNO-side cause is unresolved; the Asr residual and polar-night policy also remain open. No consumer accuracy claim, regional method endorsement, or Phase 2 gate approval follows from this review.

The later [P1.3-A1-R1 Asr audit](../specification/asr-residual-audit-v1.md) reproduced the Adhan shadow-target epoch contribution and measured the remaining near-fixed-target gap in three selected cases. It does not apportion the solar ephemeris and root-method terms or supply independent astronomy or Islamic-methodology review. The Phase 1 decision remains **open**; no threshold or calculation behavior changes.
