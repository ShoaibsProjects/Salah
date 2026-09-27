# Guidance for contributors and coding agents

This repository is the canonical workspace for Salah. Read these files before changing architecture or prayer calculations:

1. `docs/vision-and-architecture.md` — product promise and long-term constraints.
2. `docs/foundation-plan.md` — current build order and milestone gates.
3. `docs/decisions.md` — accepted decisions versus options still under evaluation.
4. `specification/calculation-contract-v0.1.md` — initial calculation interface and unresolved definitions.
5. `specification/reference-cases.md` — independently sourced comparison data and limits.

## Project intent

Build a free-to-use, private, offline-capable prayer-time system for Earth. The durable center is a documented Rust calculation core, versioned method definitions, reference cases, and clear explanations. iOS, Android, web, and later clients consume the core. No account, cloud service, paid API, or internet connection may be required to calculate prayer times or the Qibla bearing.

## Rules for implementation

- Keep astronomical calculations, prayer-method parameters, civil-time conversion, location lookup, and presentation in separate modules. Do not put prayer mathematics in Flutter or UI code.
- Represent calculation results as UTC instants and typed statuses. Convert to local civil time using an explicitly chosen time-zone rule set. Never assume the device zone is the location's zone.
- Preserve the selected method, its parameters and version, adjustment chain, astronomy model, and time-zone data version with each result. Make output explainable and reproducible.
- Return an explicit unavailable event when the chosen solar condition does not occur. Apply high-latitude alternatives only as separately labeled, documented rules.
- Keep calculated prayer beginning, mosque timetable, and iqamah as separate data. Do not present one method as the universal religious answer or issue religious rulings.
- Never invent prayer times, test vectors, source citations, scholarly quotes, or a numerical confidence score. Example times in planning material are illustrative until verified.
- Use established libraries and published astronomical references as comparison sources. A library is not automatically correct for every method or edge case.
- Minimize external dependencies and document each one. No mandatory external runtime service. Prefer explicit manual coordinates when location services are unavailable.
- Keep the core free of AI-generated calculation. Any later AI feature can explain documented results, never determine times or issue fatwas.
- Do not add Mars, blockchain, social feeds, ads, or an account system to the initial implementation.

## How to work here

- Make the smallest complete vertical slice. Start with the calculation contract, one documented method profile, an offline CLI, and reference cases. Add global time-zone lookup and more methods after the first slice is validated.
- Add a reference case with provenance whenever implementing a calculation rule or fixing a discrepancy. Compare UTC instants before comparing rounded local display times.
- Record assumptions and open scholarly or astronomical questions in the specification. Do not silently choose defaults that affect religious practice.
- Keep platform adapters thin. Probe the Rust-to-mobile and Rust-to-WASM boundary early, before committing to a UI framework.
- Update the specification and relevant documentation when behavior changes. Report what was verified and any remaining limitations.

## Current state

The repository now contains `salah-core` and `salah-cli` as a research preview. They implement one fixed-offset, sea-level, angle-based daily calculation with no high-latitude substitution. Solar events are checked against USNO; selected Fajr, Isha, and Asr results are compared with Adhan JS. No institutional method, time-zone lookup, or consumer notification behavior is certified. Continue the validation and method-source work in `docs/foundation-plan.md` before building a production UI.
