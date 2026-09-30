# Coordinate-to-zone selection contract v0.1

**Status:** experimental implementation in `salah-location`; not a Phase 2 gate decision.

## Purpose

Given checked geographic coordinates, return every IANA time-zone polygon that the bundled boundary dataset says covers that point. A result is a suggestion for a user to confirm. The engine does not treat a coordinate lookup as authoritative proof of a civil time zone, even when there is only one polygon match.

This layer does not change prayer calculations, select a calculation method, or claim religious or astronomical accuracy. After a user confirms or manually chooses a zone, downstream civil-time code must still use the explicitly versioned IANA rules from `salah-time`.

## Inputs and outputs

- Input is `salah_core::Coordinates`, created through its finite, range-checked constructor.
- The lookup passes longitude first and latitude second to `tzf-rs`; this order is internal to that library.
- The output contains the original coordinates, the boundary-data version, the matching zone identifiers sorted lexicographically, and the IANA rule-data version expected by the match.
- Cardinality is explicit: no coverage, one suggestion, or multiple suggestions. The map does not pick a winner among multiple matches.
- The boundary release and local civil-time pack must both identify IANA `2026d`. A version mismatch or a boundary identifier absent from the pinned IANA pack fails closed.
- A selection exists only after an explicit API call: confirm one returned suggestion, or manually override it with any identifier in the bundled 598-name IANA pack. The selection retains the lookup result, selected identifier, and selection origin.

The pinned 2026d data includes ocean polygons, so open water may map to an `Etc/GMT` identifier or another polygon. That is a geographic-map result, not a shipboard or aircraft timekeeping policy. If a point has no polygon coverage, the empty candidate set remains explicit. A user may choose any supported IANA identifier manually; route-based and operator-specific travel rules are outside this contract.

## Data and implementation provenance

| Item | Pinned value | Role |
|---|---|---|
| Boundary source release | [timezone-boundary-builder 2026d](https://github.com/evansiroky/timezone-boundary-builder/releases/tag/2026d) | Community-maintained polygon boundaries |
| Underlying geographic source | [OpenStreetMap](https://www.openstreetmap.org/) contributors, assembled and quality-controlled by timezone-boundary-builder | Source data for most boundary geometry |
| Data distribution | [`tzf-dist` 0.0.2026-d-fix1](https://crates.io/crates/tzf-dist/0.0.2026-d-fix1) | Bundled 2026d lite polygon database, generated from `combined-with-oceans.json`; SHA-256 `c1ee211d87027ebc87904d05e52942a644908d1e4ad2d062b0dbbc032eda22e8` (release MD5 `a943bc5e4bc3b45e98be03cfb92155e4`); MIT and ODbL-1.0 |
| Lookup implementation | [`tzf-rs` 2.1.2](https://crates.io/crates/tzf-rs/2.1.2) | Rust geometry parser and point-in-polygon lookup; MIT |
| Civil-time database | IANA TZDB 2026d | Rule pack used to resolve returned names |

The exact downloaded ODbL and MIT license texts, source reports, and a machine-readable artifact manifest are kept in [`data/third-party/tzf-2026d/`](../data/third-party/tzf-2026d/). `Cargo.lock` pins the Rust packages and package checksums; the manifest also records the embedded data-file SHA-256. The source ODbL database is redistributed inside the `tzf-dist` crate; any shipped application must provide appropriate OpenStreetMap/timezone-boundary-builder attribution and preserve applicable data-license notices. This document records source facts; it is not legal advice.

The mapping runs locally from embedded data. It calls no location service, network endpoint, platform time-zone database, or backend at runtime. Updating boundary data requires a new reviewed package/app release; no user account or paid lookup service is required.

## Accuracy boundary

The boundary source describes approximate current timezone areas assembled from community geographic data. It is not a legal boundary registry or an authoritative record of historical jurisdiction boundaries. The pinned [`tzf-dist` 2026d border report](https://github.com/ringsaturn/tzf-dist/blob/v0.0.2026-d-fix1/BORDER_CHANGE.md) compares lite polygons to the source geometry and reports a length-weighted 95th-percentile displacement of 66.1 m and a certified maximum of 111.538 m with +1 m certification tolerance. This measures simplification error against the source geometry; it does not measure whether source polygons match legal boundaries or local practice.

Therefore:

- A single polygon match is a suggestion requiring explicit user confirmation; it is not described as “the correct zone.”
- A multiple match is exposed as multiple candidates. No ordering, nearest-city rule, or first-match tie-break is used.
- Device location error is not estimated by this contract. A GPS point near an approximate border may map to the neighboring zone; provide an always-available manual override.
- The time-zone boundary map is not a prayer-time source and carries no calculation-method or scholarly endorsement.
- Historical calculations use today's mapped IANA identifier unless the user selects another. Historical political boundary changes are not reconstructed by this layer.

## Dependency and long-term operation

`salah-location` pins `tzf-rs` 2.1.2 and `tzf-dist` 0.0.2026-d-fix1 directly, with `tzf-rs` default features disabled. The exact lite data file hash is recorded above; the package checksums are pinned in `Cargo.lock`. The crate currently requires Rust 1.88; `salah-location` declares that minimum without raising `salah-core`'s minimum. Its transitive geometry dependencies and the embedded 4,100,168-byte ODbL dataset increase application size and must be reviewed for each target (especially WASM/mobile) before consumer integration.

No recurring service fee or network dependency is introduced. The bundled 2026d snapshot continues to work offline, but future legal timezone changes and boundary corrections require maintained data updates. “Works until 2050” is a maintenance and reproducibility goal, not a claim that this frozen snapshot predicts future law.

For a data refresh, choose a boundary release paired with the same IANA TZDB release; inspect and archive its source statistics, boundary-change report, and license; pin exact `tzf-dist` and `tzf-rs` versions; verify the embedded data version and file hash; update the constants and manifest; and run the complete workspace validation. Keep each prior versioned provenance folder so older released selections remain explainable. Do not update boundary data independently of the IANA rule pack.

## Acceptance evidence

The implementation must preserve these properties in its automated checks:

1. Known city coordinates map to their published data-pack name with input latitude/longitude order verified.
2. Exact shared-polygon boundaries return all matching identifiers.
3. The 2026d ocean polygon at (0°, 0°) returns `Etc/GMT`; genuine no-coverage, if encountered, remains an explicit empty candidate set.
4. Every emitted boundary name exists in the pinned IANA 2026d pack.
5. One candidate cannot become a selection without an explicit confirmation call.
6. Manual overrides accept only identifiers supported by the pinned IANA pack and retain the original lookup provenance.
7. All behavior works with network access disabled after dependencies are available to the build.

Passing these implementation checks does not validate whether the community polygons match every local legal or customary time-zone boundary.
