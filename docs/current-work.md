# Salah: what we are building now

**Updated:** 30 September 2026. The [roadmap](roadmap.md) owns phase gates; this page is the plain-language map. “Implemented” means code exists and has the stated checks, not that it is approved for consumer use.

## Current piece

**Phase 3: Portable Calculation Platform → F3-C1: Engine Interface → Named-Timezone Command and Schedule Document.**

You can now ask the command-line program for a local schedule with coordinates, a Gregorian date, a named IANA timezone, a method, and an Asr choice. The command uses the existing Prayer Kernel, Civil Clock, and Schedule Composer. DST and date-line rules come from the bundled snapshot. It prints full local dates and seconds, UTC, and exact data identity. Add `--json` to obtain the reusable, versioned document for future clients. See the [interface contract](../specification/named-zone-interface-contract-v0.1.md).

An explicit zone can now be chosen without loading the approximate boundary map. The result says that no map lookup occurred. Skipped dates, zero/multiple solar cycles, and unavailable events remain visible. No formula, method profile, solver threshold, or historical reference result was changed.

**Evidence so far:** local locked offline workspace build, formatting, all-target lint checks, and diff inspection. The user requested implementation without adding tests; none were added or run locally. One existing location test fixture was updated for the new private provenance field so it still compiles. Earlier checkpoint `1be80bc` has 100 passing Rust tests and six validator tests; those are prior evidence. New interface and verified-snapshot acceptance cases remain open. Phase 1, Phase 2, and the Phase 3 cross-target gate remain open.

**Next build:** F3-W1: Portable Binding Probe. Expose the same explicit engine inputs and schedule document to a browser/WASM client, assess target dependencies, and keep the app free of prayer mathematics. A simple daily schedule screen follows the working boundary. Signed-pack platform integration and trial-health policy continue as separate open work.

## Names of the main parts

| Part | Code | Purpose and present state |
| --- | --- | --- |
| Prayer Kernel | `salah-core` | Calculates solar and prayer events in UTC. Dependency-free research kernel; independent astronomy/methodology review and known discrepancies remain open. |
| Civil Clock | `salah-time` | Converts UTC under a recorded IANA snapshot, handles DST/date changes, selects local-date solar cycles, and preserves missing events. |
| Location and Zone Choice | `salah-location` | Suggests zones from approximate offline boundaries; requires confirmation or manual choice. Direct manual selection can bypass map lookup. |
| Schedule Composer | `salah-engine` | Combines explicit inputs and a bundled or verified immutable rule snapshot into a local schedule with exact identity and source metadata. |
| Update Authentication | `salah-update` | Verifies signed packages, exact hashes/inventory, and compatibility before parsing their payloads. P2a is implemented. |
| Update Storage and Recovery | `salah-update-store` | Archives signed packages and manages trial, confirmation, restart recovery, and counter preservation. P2b.1 is implemented; runtime selection and trial confirmation remain explicit caller actions. |
| Calculation Lab Command | `salah-cli` | Runs named-zone local schedules, emits shared JSON, lists supported zones, and retains the historical fixed-offset interface. Stored packages are still Rust-API-only. |

## How to name work

Use **phase → workstream → piece**, followed by a plain name in every handoff. Preserve the existing `F2-TZ*` identifiers for continuity; `F2` refers to Phase 2 work, not a separate roadmap.

| Piece | Plain name | State |
| --- | --- | --- |
| F2-TZ9-P1 | Pack Identity and Integrity | Implemented; exact bytes and zone inventory are identified. |
| F2-TZ9-P2a | Signed Candidate Verification | Implemented; no production key configured. |
| F2-TZ9-P2b.1 | Local Repository and Crash Recovery | Implemented prototype; macOS evidence and explicit filesystem limits. |
| F2-TZ9-P2b.2a | Verified Snapshot in Calculations | Implemented; build/lint checks; focused runtime acceptance evidence pending. |
| F2-TZ9-P2b.2b | Platform Integration and Trial Health Policy | Open: mobile/WASM builds, platform persistence, and application-level startup/confirmation policy. |
| F2-TZ9-P2b.3 | Production Signing Stewardship | Open: maintainers, key custody/rotation/revocation, release pipeline, retention, and distribution policy. |
| F3-C1 | Named-Timezone Command and Schedule Document | Implemented; native build/static checks; focused acceptance evidence remains open. |
| F3-W1 | Portable Binding Probe | Next: reuse the engine and versioned document across the browser/WASM boundary. |

Phase 1 (accuracy and method integrity) and Phase 2 remain open. High-latitude alternatives, Qibla, broader independent comparisons, portable bindings, and user apps remain on the roadmap. A working update subsystem does not settle those other gates.

## Source discipline

The project may consult [quran.ai](https://mcp.quran.ai/) for verified Qur'an text, named translations, and attributed tafsir during religious-methodology research. Record the exact verse, edition, retrieved content, and date when using it; fetch tafsir for interpretation and preserve scholarly differences. Numerical solar algorithms, timezone law, and physical error budgets require independently checkable scientific/civil sources. Neither an AI inference nor an unreviewed interpretation may silently become a calculation parameter or religious ruling. Research tools remain optional; daily calculation stays offline.
