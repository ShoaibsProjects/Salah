# Guidance for contributors and coding agents

This repository is the canonical workspace for Salah. Read these files before changing architecture or prayer calculations:

1. `docs/vision-and-architecture.md` — product promise and long-term constraints.
2. `docs/roadmap.md` — active phases, evidence gates, release scope, and current position.
3. `docs/phase-1-validation.md` — active validation tasks and next bounded work item.
4. `docs/decisions.md` — accepted decisions versus options still under evaluation.
5. `specification/calculation-contract-v0.3.md` — current calculation interface and unresolved definitions. Keep earlier contracts for historical results.
6. `specification/accuracy-budget.md` — current comparison protocol, bounded implementation properties, and unmeasured limits.
7. `specification/reference-cases.md` — independently sourced comparison data and limits.
8. `data/reference/README.md` and `specification/usno-matrix-v1-report.md` — versioned solar source provenance, raw UTC comparison, and known disagreement.
9. `data/reference/prayer-library-v1.tsv` and `specification/prayer-library-v1-report.md` — pinned prayer-library comparison and unresolved Asr/high-latitude differences.
10. `docs/director-handoff.md` — delegation backlog, agent prompts, evidence requirements, and review protocol. `docs/roadmap.md` remains authoritative for phase gates.
11. `specification/presentation-contract-v0.1.md` — architect-approved research-preview minute-display policy; keep it separate from historical calculation contract v0.3.
12. `specification/phase-1-gate-report-v0.1.md` — current evidence, open ledger, reviewer vacancies, and Phase 1 claim boundary.
13. `docs/civil-time-spike.md` — reviewed offline UTC-to-local conversion probe and remaining civil-time boundaries; Phase 1 remains open.
14. `docs/civil-time-date-selector.md` — bounded six-zone local-date transit selector and its completeness argument.
15. `docs/civil-time-local-schedule.md` — local event conversion slice and its research limits.
16. `specification/civil-date-existence-contract-v0.1.md` — exact transition-interval classifier; keep skipped civil days distinct from zero solar-cycle matches.
17. `specification/coordinate-zone-selection-contract-v0.1.md` — approximate offline boundary suggestions, mandatory explicit confirmation/manual override, data provenance, and limits.
18. `specification/selected-local-day-engine-contract-v0.1.md` — explicit-input composition of zone selection, solar calculation, local-date selection, and event localization.
19. `docs/civil-time-data-lifecycle.md` and `specification/civil-time-data-assessment-contract-v0.1.md` — standard offline data rule, date-sensitive notices, and future pack-update path.
20. `specification/offline-tzif-pack-interface-v0.1.md` — schema 1 rule-pack identity, offline validator, and boundary before signed activation.
21. `specification/signed-rule-pack-candidate-v0.1.md` — exact signed bytes, pinned-key verification, limits, and activation boundary.
22. `specification/rule-pack-repository-v0.1.md` — Unix app-private signed archive, trial/confirmation, atomic state replacement, recovery, and filesystem limits.
23. `specification/runtime-rule-snapshot-v0.1.md` — immutable bundled/authenticated runtime selection, owned identities, shared payload validation, and acceptance gaps.
24. `docs/current-work.md` — plain-language component names and the current phase/workstream/piece; roadmap gate authority is unchanged.
25. `specification/named-zone-interface-contract-v0.1.md` — explicit manual zone without lookup, named-zone CLI, and shared schedule JSON; no display-minute or notification rule.
26. `specification/wasm-bridge-contract-v0.1.md` — bounded explicit JSON bridge, Web Worker/browser preview, reproducible package build, and pending browser/cross-target acceptance.
27. `docs/offline-product-contract.md` — canonical offline operating vision, hardware/clock limits, automatic setup, manual fallback, and long-term direction. Read this before extending device inputs or app startup.
28. `specification/offline-setup-contract-v0.1.md` — additive zone lookup/clock/selection bridge, selected-zone device date, explicit confirmation, and hash-checked public browser package.

## Project intent

Build a free-to-use, private, offline-capable prayer-time system for Earth. The durable center is a documented Rust calculation core, versioned method definitions, reference cases, and clear explanations. iOS, Android, web, and later clients consume the core. No account, cloud service, paid API, or internet connection may be required to calculate prayer times or the Qibla bearing.

The [offline product contract](docs/offline-product-contract.md) records the accepted operating model: fill routine inputs from usable local readings and embedded data, preserve uncertainty and choice, and offer manual correction. A compass cannot supply coordinates; a source-agnostic browser location provider cannot be declared GNSS-only or guaranteed offline. Use the caller's UTC instant and the selected location's Rust timezone rules to derive today, with an explicitly unverified device clock.

## Rules for implementation

- Keep astronomical calculations, prayer-method parameters, civil-time conversion, location lookup, and presentation in separate modules. Do not put prayer mathematics in Flutter or UI code.
- Represent calculation results as UTC instants and typed statuses. Convert to local civil time using an explicitly chosen time-zone rule set. Never assume the device zone is the location's zone.
- Keep date, coordinate, and offset invariants behind checked constructors; do not expose a path that lets callers pass impossible civil input into the engine.
- Preserve the selected method, its parameters and version, adjustment chain, astronomy model, and time-zone data version with each result. Make output explainable and reproducible.
- Return an explicit unavailable event when the chosen solar condition does not occur. Apply high-latitude alternatives only as separately labeled, documented rules.
- Keep calculated prayer beginning, mosque timetable, and iqamah as separate data. Do not present one method as the universal religious answer or issue religious rulings.
- Never invent prayer times, test vectors, source citations, scholarly quotes, or a numerical confidence score. Example times in planning material are illustrative until verified.
- When consulting Qur'an or tafsir, use verified source tools such as quran.ai and record exact verse, edition, and retrieval date. Attribute interpretations and preserve disagreements; scientific/civil calculations still need independent reference evidence. Research tools are never runtime calculation dependencies.
- Use established libraries and published astronomical references as comparison sources. A library is not automatically correct for every method or edge case.
- Minimize external dependencies and document each one. No mandatory external runtime service. Prefer explicit manual coordinates when location services are unavailable.
- Keep the core free of AI-generated calculation. Any later AI feature can explain documented results, never determine times or issue fatwas.
- Do not add Mars, blockchain, social feeds, ads, or an account system to the initial implementation.

## How to work here

- Take one bounded task from the active phase in `docs/roadmap.md`. The first contract, core, CLI, and reference slice already exist. Do not treat a phase gate as passed without its recorded evidence.
- Identify progress as phase → workstream → piece with a plain-language name. Keep `docs/current-work.md` aligned with the implemented scope and remaining boundaries.
- Take delegation packets for delegated Phase 1 work only from `docs/director-handoff.md` §3 with the prompt in §4 and the review protocol in §5. Do not redefine P1.4/P1.5 scope elsewhere.
- Add a reference case with provenance whenever implementing a calculation rule or fixing a discrepancy. Compare UTC instants before comparing rounded local display times. Preserve a prior model's manifest/report when behavior changes; create a new versioned comparison rather than rewriting historical evidence.
- Record assumptions and open scholarly or astronomical questions in the specification. Do not silently choose defaults that affect religious practice.
- Keep platform adapters thin. Probe the Rust-to-mobile and Rust-to-WASM boundary early, before committing to a UI framework.
- Update the specification and relevant documentation when behavior changes. Report what was verified and any remaining limitations.

## Current state

F3-W1 adds `salah-wasm`, a bounded explicit JSON bridge, and `apps/web`, the first local worker-based schedule screen. No JavaScript prayer calculation, local-time reinterpretation, defaults, signed activation, or new tests are introduced. Native/WASM builds and one direct module use are recorded; browser acceptance remains pending. Read the bridge contract and current-work map before extending this client.

F4-C1 refines the browser research preview with an opt-in energy-conscious Geolocation API request, a user-selected higher-accuracy retry after failure, and plain-language profile/Asr explanations. The browser/OS may need connectivity; do not describe the result as compass-derived, exact, or guaranteed offline. Keep timezone selection separate, preserve manual coordinate entry, and do not add continuous location tracking or collect coordinates remotely. Browser and real-device review remain pending; no tests were added or run locally in this piece.

F4-C2 now fills coordinate-derived timezone suggestions through the embedded Rust map and derives today from a caller-supplied UTC clock instant in the confirmed location zone. One suggestion still requires confirmation because point geometry is approximate and the fix error footprint is unchecked. v1 schedule input remains available; additive v2 records manual versus confirmed-map choice. F3-W2 adds opt-in, hash-checked public-asset caching and explicit browser package updates. Browser restart/update/eviction and native sensor acceptance remain open; no tests were added or run locally. Follow the offline setup contract rather than inferring a universal device capability.

F3-C1 adds `salah-cli schedule`, `salah-cli zones`, and `salah_engine::schedule_document` under the named-zone interface contract. Direct manual selection records an unperformed map lookup separately from no coverage. Native build/static checks are recorded; the user requested no new tests, and no tests were run locally in this slice. The next build is a portable binding probe, with all earlier phase gates still open.

F2-TZ1 adds a reviewed UTC-anchor solar-cycle API to `salah-core`; see `specification/utc-anchor-contract-v0.1.md`. F2-TZ2 is implemented in `salah-time`; see `specification/civil-date-transit-selector-v0.2.md` for the expanded named-zone scope and v0.1 for the original six-zone evidence. F2-TZ3 localizes all seven events, F2-TZ4 classifies whether the local civil date exists, and F2-TZ5 bundles 598 named IANA zones; these extensions are implemented research code pending phase-gate review. F2-TZ6 adds an experimental offline suggestion layer in `salah-location`; suggestions require explicit confirmation or manual override and do not establish authoritative timezone boundaries. F2-TZ7 composes a selected zone and explicit calculation inputs through `salah-engine`; see the selected-local-day contract. F2-TZ8 adds a separate, caller-dated data assessment with typed notices; see the civil-time data lifecycle and assessment contract. F2-TZ9-P1 adds exact bundled pack identity and a local integrity validator. F2-TZ9-P2a adds a signed-candidate verifier in `salah-update`. F2-TZ9-P2b.1 adds experimental Unix signed archives and trial/recovery state in `salah-update-store`; see its repository contract and `docs/current-work.md`. F2-TZ9-P2b.2a adds explicit verified-snapshot calculations with exact owned identity and signed source metadata; build/lint checks are recorded, focused acceptance evidence remains open. No production key or automatic update activation exists. Zero/one/multiple solar-cycle matches remain separate from skipped/existing date status. Neither phase gate has passed.

The repository now contains `salah-core` and `salah-cli` as a research preview. They implement fixed-offset, sea-level, angle-based daily calculations with no high-latitude substitution. Available profiles are `research-15` and `mwl-angles-18-17`, the latter sourced to a secondary PrayTimes parameter table without claiming institutional endorsement. Solar events are checked against USNO and a selected independent Horizons near-grazing audit; selected prayer outputs are compared with Adhan JS and PrayTimes v2. The P1.3-A1-R1 audit reproduces the main Adhan Asr gap contribution and measures an unresolved near-fixed-target remainder. The separate experimental `salah-time` crate carries an embedded 598-name IANA 2026d pack, selects local-date solar cycles, classifies civil-date existence, and localizes all seven schedule events. `salah-location` suggests zones from approximate community polygons and requires user confirmation or manual override. `salah-engine` composes that selection with explicit date, method, and Asr inputs to produce a localized schedule. No institutional certification or consumer notification behavior exists. The USNO-side near-grazing cause, Asr model/solver residual, and polar-night Asr policy remain open. P1.4 technical provenance is recorded; primary institutional confirmation and qualified methodology review remain open. P1.5's optional, versioned research-preview minute display adapter is implemented and reviewed, without changing calculation outputs. The interim Phase 1 gate report and decision record keep the gate open. Independent astronomy and Islamic-methodology review are the next gate actions. Civil-time boundaries, the local schedule slice, coordinate-to-zone limits, and orchestration behavior are documented in the respective contracts. Do not make consumer accuracy claims.
