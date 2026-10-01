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
| Apple interface | [F3-A1/F4-A1](../specification/apple-foundation-v0.1.md) uses native SwiftUI, iOS 17+, with a static Rust C ABI/XCFramework. One shared strict JSON bridge serves Swift and WASM; no UI prayer mathematics. | Native controls/accessibility and physical offline positioning acceptance; Android UI remains undecided. |
| Web interface | Rust-to-WASM with a responsive client is the leading approach. | After checking offline, browser, and deployment behavior. |
| First time input | Explicit fixed UTC offset for the initial research CLI; IANA zone support is required before claiming global civil-time accuracy. | At the Phase 2 civil-time milestone (see roadmap.md Phase 2). |
| First method profile | `research-15` uses 15° Fajr and Isha angles with zero adjustments. It carries no institutional attribution. | Replace or supplement only after source and scholarly review. |
| First published angle set | `mwl-angles-18-17` reproduces the Fajr and Isha angles in the PrayTimes MWL table with zero Dhuhr/Maghrib adjustment. It is a sourced parameter set, not an endorsement or complete regional timetable. | Revisit when primary institutional specifications and regional practice are reviewed. |
| Research-preview prayer-start minute display | The architect selected [`prayer-start-ceil-minute` revision `0.1`](../specification/presentation-contract-v0.1.md) on 2026-09-27: use the already adjusted UTC second, then show the first whole local minute at or after it. This is Salah's display rule for the five prayer beginnings, not a method or religious ruling. Sunrise/sunset remain second-precision events; fasting cutoffs and notifications need separate rules. | Revisit for sourced method conventions, IANA time-zone transitions, qualified methodology/product review, or any consumer release. |
| Crate layout | `salah-core` owns prayer calculations, `salah-cli` exposes the research command line, and experimental `salah-time` owns offline civil-time conversion. | Split further only when real module boundaries and independent reuse are clear. |
| Civil-time TZif parser probe | Jiff 0.2.37 is accepted for the separate [F2-TZ0 probe](civil-time-spike.md) and F2-TZ5 named-zone data experiment, with pinned caller-supplied TZif bytes and no host-zone fallback. Its license is Unlicense OR MIT; `salah-core` remains independent. | Review mobile/WASM builds, pack update design, dependency inventory, licensing, and performance before consumer use. |
| Update system | Versioned, authenticated data packs are a long-term target. | When method or time-zone update requirements and platform limits are known. |
| Installed civil-time data notice | [`installed-civil-time-data-assessment` v0.1](../specification/civil-time-data-assessment-contract-v0.1.md) compares an explicit observation date with the pinned pack year and requested date. Its typed notices do not change a schedule or certify civil-time accuracy. | Revisit when the pack interface and actual update channel exist. |
| Bundled rule-pack identity | The [schema 1 TZif interface](../specification/offline-tzif-pack-interface-v0.1.md) records the exact pack hash in civil-time results and checks the local manifest and zone slices. This is integrity, not source authentication. | Revisit when signed activation, multi-pack storage, and rollback are designed. |
| Signed update candidate | The [v0.1 verifier](../specification/signed-rule-pack-candidate-v0.1.md) uses strict Ed25519 under public keys supplied by a trusted application release, verifies before parsing, and yields an inert candidate. No production key or activation path is configured. | Revisit for key custody/rotation, persistent rollback protection, platform storage, and independent security review. |
| Local update repository | The [P2b.1 Unix prototype](../specification/rule-pack-repository-v0.1.md) stores separate signed archives and atomically replaces a small trial/confirmation state. Recovery retains both sequence and IANA-release high-water records. No new third-party package enters the lockfile, and storage stays outside `salah-core`. | Review other targets, filesystem/hardware durability, exact runtime pack integration, OS-protected rollback requirements, and archive retention before consumer use. |

| Explicit runtime snapshot | The [P2b.2a runtime contract](../specification/runtime-rule-snapshot-v0.1.md) uses immutable bundled or verified snapshots, one parsed zone per schedule, and owned exact identities. Signature authentication precedes shared payload validation. | Focused runtime acceptance evidence, target builds, storage integration, and trial-health policy remain open. |

## Offline operating decision — 2026-10-01

The maintainer requested essential use without online dependencies, automatic device input where available, and manual correction when automation is missing or uncertain. The [offline product contract](offline-product-contract.md) is the canonical operating vision alongside the original architecture; the roadmap still owns gates.

- Coordinates feed bundled boundary geometry and IANA rules locally. Civil timezone law is never inferred from longitude/15, solar position, or a device's displayed zone.
- The current approximate point map may fill a unique suggestion, but explicit confirmation remains required. The full location-error footprint is not checked. Multiple/no-coverage results require a choice; manual selection always remains possible.
- Today's Gregorian date is the caller's UTC epoch instant converted by Rust in the confirmed location zone. The device wall clock is labeled unverified; plausible clock error cannot be universally detected offline. Manual date correction remains available.
- A browser Geolocation result remains an optional source-agnostic reading. No portable GNSS-only/no-network guarantee exists in that API. Strict offline sensor acquisition needs native/embedded capability evidence; a compass gives no coordinates. No IP or cloud-geocoding fallback is permitted.
- F4-C2 adds bounded lookup/clock/selection exports and the automatic setup flow. F3-W2 adds opt-in, hash-checked public app caching with explicit updates; browser cache eviction and offline restart acceptance remain open. Neither artifact hashing nor a successful build is publisher authentication or a passed accuracy/portability/religious gate.

See the [offline setup contract](../specification/offline-setup-contract-v0.1.md). No astronomy, method, tolerance, reference, or phase-gate decision changes. No tests were added or run locally in these pieces, following the user's instruction.

## Unresolved decisions

- First calculation profile and authoritative parameter sources.
- Broader numerical accuracy budget and physical model limits; the optional research-preview prayer-start presentation policy is implemented but not approved for consumer use.
- High-latitude defaults by region and the scholarly review process.
- Time-zone boundary dataset, licensing, and update mechanism.
- Product license, maintainers, funding, and governance through 2050.
- Launch languages, platform minimums, and notification behavior by OS.

The repository name is **Salah**. “2050” describes the maintenance horizon, not a guarantee that software or civil-time data can remain unchanged until that year.

The [civil-time data lifecycle](civil-time-data-lifecycle.md) records the standard offline operating rule and the sequence for compatible, authenticated pack updates and a separately labeled manual correction. Strict Ed25519 candidate verification and an experimental Unix archive/recovery adapter are implemented. Production signing stewardship, runtime pack activation, other platform storage, and distribution remain open decisions.

## Interim Phase 1 gate review — 2026-09-28

The [Phase 1 gate report v0.1](../specification/phase-1-gate-report-v0.1.md) records the selected-case comparisons and reviewer vacancies. Phase 1 remains **open**. The independent Horizons audit supports Salah's near-grazing status under its fixed threshold, but the USNO-side cause is unresolved; the Asr residual and polar-night policy also remain open. No consumer accuracy claim, regional method endorsement, or Phase 2 gate approval follows from this review.

The later [P1.3-A1-R1 Asr audit](../specification/asr-residual-audit-v1.md) reproduced the Adhan shadow-target epoch contribution and measured the remaining near-fixed-target gap in three selected cases. It does not apportion the solar ephemeris and root-method terms or supply independent astronomy or Islamic-methodology review. The Phase 1 decision remains **open**; no threshold or calculation behavior changes.
