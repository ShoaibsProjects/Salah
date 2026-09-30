# Global named-zone pack v0.1

**Status:** F2-TZ5 research implementation under review. The pack contains all 598 named zones and links emitted by the pinned IANA 2026d source inputs used here. It does not contain a coordinate-to-zone map. Phase 2 remains open.

## Purpose and interface

The civil-time adapter should not depend on the phone's current timezone database or a remote service. `salah-time::fixture_tzif_bytes(zone_id)` now resolves any identifier in the embedded pack. The existing conversion, local-date selection, civil-date classification, and schedule APIs continue to require the caller to choose that identifier explicitly and pass the returned bytes.

The generated pack stores 345 unique TZif byte images in one 206,474-byte blob. A sorted static index maps 598 IANA identifiers, including backward-compatible links, to exact `(offset, length)` slices. The manifest records each identifier's byte count and SHA-256, the shared pack hash, source archive identities, compiler/tool details, and license. Exact bytes are checked before they receive `TZDB_VERSION = "2026d"` provenance.

The manifest now declares schema `1`, and results also carry the exact [rule-pack identity](offline-tzif-pack-interface-v0.1.md), including the blob SHA-256. This is an additive provenance field; the TZif bytes and offset calculations are unchanged.

The pack is a set of named civil-time rules, not a geographic database. It lets a user choose an IANA zone by name; it does not determine which zone applies to GPS coordinates, borders, ships, aircraft, or disputed locations.

## Rebuild and verification

`crates/salah-time/fixtures/generate-global-pack.sh` downloads the official `tzdata2026d.tar.gz` and `tzcode2026d.tar.gz` only for pack maintenance, verifies their pinned SHA-256 hashes, checks the extracted `version` and `zic (tzcode) 2026d` identity, and compiles the named source files with `zic -b slim`. It rejects any generated output that is not TZif data.

The default `verify` mode rebuilds in a temporary directory, compares manifest data and per-zone hashes while retaining the original build-host metadata, and byte-compares the blob, license, and generated Rust index. The exact `zic` release must match; a different compiler/host is acceptable only if the generated pack bytes still match. The explicit `--write` mode replaces those generated artifacts after source hash and tool-version checks. Runtime conversion uses the compile-time blob and static index; it performs no network or filesystem access and has no new compression dependency.

`python3 tools/validate_tzif_pack.py crates/salah-time/fixtures/global/manifest.json --boundary-version 2026d` checks the local schema, exact hashes, slice inventory, and TZif parseability without fetching sources. The generator invokes it after either mode. A matching hash is an integrity check against the manifest, not authentication of an update source.

The compact blob deduplicates identical TZif images while retaining a separate hash and index row for every identifier. Slim TZif future rules are interpreted from their POSIX footers by the pinned Jiff parser, including beyond the explicit transition table.

## Provenance snapshot

| Item | Value |
| --- | --- |
| IANA version | 2026d |
| `tzdata2026d.tar.gz` SHA-256 | `0cb2aa8e333c3dc049badc42a0c61f21987b8cd44e107fa900bad764aacc7767` |
| `tzcode2026d.tar.gz` SHA-256 | `2f5c9f7fe29e6b8cb863583667884b8ce17b0a485355a054b591c6bdfcd81791` |
| Generator | `zic (tzcode) 2026d`, `-b slim`; exact host/compiler in `crates/salah-time/fixtures/global/manifest.json` |
| Zone identifiers | 598 |
| Unique TZif images | 345 |
| Packed TZif bytes | 206,474 |
| Pack SHA-256 | `e919bf1d69ca9948a559963bc1384576144e1513367aefcd4340c83a79b8f816` |
| Zone inventory SHA-256 | `17d496af9df7ca6997fd279d3b8f78c4d854c1e02e12300bbb7eb03e8fcc4317` |
| License text SHA-256 | `0613408568889f5739e5ae252b722a2659c02002839ad970a63dc5e9174b27cf` |

The IANA archive's license text is included at `crates/salah-time/fixtures/global/IANA-LICENSE`. It distinguishes the database files from named optional C source files; the compiled TZif pack does not redistribute those C sources. Review the exact license and packaging context before public distribution.

## Accuracy and maintenance boundary

All 598 indexed identifiers are parsed in the integration suite. Selected Chicago, London, Kathmandu, Kiritimati, and Apia cases exercise transition behavior; Chicago's 2099 case exercises POSIX-footer behavior. This proves pack construction and parser acceptance for the named entries, not independent validation of every zone's historical rules or location mapping.

TZDB 2026d is a snapshot, not a promise that 2026 rules remain correct through 2050. A maintained release must be able to ship newer versioned data, preserve older manifests for reproducibility, and explain when local-time outputs change. The app must remain usable offline with its installed pack; future data updates may be distributed when a connection is available but are never a per-calculation service dependency.

The Rust crate now has broad named-zone data, but Phase 2 still needs reviewed coordinate-to-zone data or a clear manual-selection experience, ambiguity handling, stale-pack behavior, and broader transition evidence. No global prayer-time accuracy or religious endorsement claim follows from this pack.
