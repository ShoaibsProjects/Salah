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

## Project intent

Build a free-to-use, private, offline-capable prayer-time system for Earth. The durable center is a documented Rust calculation core, versioned method definitions, reference cases, and clear explanations. iOS, Android, web, and later clients consume the core. No account, cloud service, paid API, or internet connection may be required to calculate prayer times or the Qibla bearing.

## Rules for implementation

- Keep astronomical calculations, prayer-method parameters, civil-time conversion, location lookup, and presentation in separate modules. Do not put prayer mathematics in Flutter or UI code.
- Represent calculation results as UTC instants and typed statuses. Convert to local civil time using an explicitly chosen time-zone rule set. Never assume the device zone is the location's zone.
- Keep date, coordinate, and offset invariants behind checked constructors; do not expose a path that lets callers pass impossible civil input into the engine.
- Preserve the selected method, its parameters and version, adjustment chain, astronomy model, and time-zone data version with each result. Make output explainable and reproducible.
- Return an explicit unavailable event when the chosen solar condition does not occur. Apply high-latitude alternatives only as separately labeled, documented rules.
- Keep calculated prayer beginning, mosque timetable, and iqamah as separate data. Do not present one method as the universal religious answer or issue religious rulings.
- Never invent prayer times, test vectors, source citations, scholarly quotes, or a numerical confidence score. Example times in planning material are illustrative until verified.
- Use established libraries and published astronomical references as comparison sources. A library is not automatically correct for every method or edge case.
- Minimize external dependencies and document each one. No mandatory external runtime service. Prefer explicit manual coordinates when location services are unavailable.
- Keep the core free of AI-generated calculation. Any later AI feature can explain documented results, never determine times or issue fatwas.
- Do not add Mars, blockchain, social feeds, ads, or an account system to the initial implementation.

## How to work here

- Take one bounded task from the active phase in `docs/roadmap.md`. The first contract, core, CLI, and reference slice already exist. Do not treat a phase gate as passed without its recorded evidence.
- Take delegation packets for delegated Phase 1 work only from `docs/director-handoff.md` §3 with the prompt in §4 and the review protocol in §5. Do not redefine P1.4/P1.5 scope elsewhere.
- Add a reference case with provenance whenever implementing a calculation rule or fixing a discrepancy. Compare UTC instants before comparing rounded local display times. Preserve a prior model's manifest/report when behavior changes; create a new versioned comparison rather than rewriting historical evidence.
- Record assumptions and open scholarly or astronomical questions in the specification. Do not silently choose defaults that affect religious practice.
- Keep platform adapters thin. Probe the Rust-to-mobile and Rust-to-WASM boundary early, before committing to a UI framework.
- Update the specification and relevant documentation when behavior changes. Report what was verified and any remaining limitations.

## Current state

The repository now contains `salah-core` and `salah-cli` as a research preview. They implement fixed-offset, sea-level, angle-based daily calculations with no high-latitude substitution. Available profiles are `research-15` and `mwl-angles-18-17`, the latter sourced to a secondary PrayTimes parameter table without claiming institutional endorsement. Solar events are checked against USNO; selected prayer outputs are compared with Adhan JS and PrayTimes v2. No time-zone lookup, institutional certification, or consumer notification behavior exists. The 19-case USNO source matrix and report include an unresolved near-grazing event-existence disagreement. The 28-row prayer-rule matrix exposes the Minneapolis Asr gap, Adhan high-latitude fallback, and polar-night Asr source disagreement. P1.4 technical provenance is recorded; primary institutional confirmation and qualified methodology review remain open. P1.5's optional, versioned research-preview minute display adapter is implemented and reviewed, without changing calculation outputs. The next bounded packet is P1.2-G1, an independent near-grazing event-existence audit in `docs/director-handoff.md`; it makes no formula change. Phase 1 remains open; do not make consumer accuracy claims.
