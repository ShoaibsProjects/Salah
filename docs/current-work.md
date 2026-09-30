# Salah: what we are building now

**Updated:** 30 September 2026. The [roadmap](roadmap.md) owns phase gates; this page is the plain-language map. “Implemented” means code exists and has the stated checks, not that it is approved for consumer use.

## Current piece

**Phase 2: Civil Time → F2-TZ9: Timezone Data Updates → P2b.2a: Verified Snapshot in Calculations.**

The Prayer Kernel calculates UTC events. The Civil Clock turns those events into local readings under one timezone snapshot. The new runtime path lets the Schedule Composer consume a `VerifiedRulePack`, including one loaded from update storage, and use that snapshot consistently for the requested date and every event. Its exact identity stays with the result. See the [runtime contract](../specification/runtime-rule-snapshot-v0.1.md).

The runtime owns its bytes and labels. A subsequent update, rollback, or repository close cannot change a calculation already using that handle. Signature verification still happens before payload parsing, and calculation never automatically confirms a stored trial. The legacy bundled entry point remains available.

**Evidence so far:** local workspace/all-target build and lint checks. Focused runtime acceptance cases remain open; no new runtime regression suite was added or executed in this slice. Earlier checkpoint `1be80bc` has 100 passing Rust tests and six validator tests; those are prior evidence. Mobile/WASM builds and target persistence review remain separate P2b.2 work. Neither phase gate has passed.

## Names of the main parts

| Part | Code | Purpose and present state |
| --- | --- | --- |
| Prayer Kernel | `salah-core` | Calculates solar and prayer events in UTC. Dependency-free research kernel; independent astronomy/methodology review and known discrepancies remain open. |
| Civil Clock | `salah-time` | Converts UTC under a recorded IANA snapshot, handles DST/date changes, selects local-date solar cycles, and preserves missing events. |
| Location and Zone Choice | `salah-location` | Suggests zones from approximate offline boundaries; requires confirmation or manual choice. |
| Schedule Composer | `salah-engine` | Combines explicit inputs and a bundled or verified immutable rule snapshot into a local schedule with exact identity and source metadata. |
| Update Authentication | `salah-update` | Verifies signed packages, exact hashes/inventory, and compatibility before parsing their payloads. P2a is implemented. |
| Update Storage and Recovery | `salah-update-store` | Archives signed packages and manages trial, confirmation, restart recovery, and counter preservation. P2b.1 is implemented; runtime selection and trial confirmation remain explicit caller actions. |
| Calculation Lab Command | `salah-cli` | Runs the existing fixed-offset research interface. Named-zone orchestration and stored packages are not exposed by this CLI yet. |

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

Phase 1 (accuracy and method integrity) and Phase 2 remain open. High-latitude alternatives, Qibla, broader independent comparisons, portable bindings, and user apps remain on the roadmap. A working update subsystem does not settle those other gates.

## Source discipline

The project may consult [quran.ai](https://mcp.quran.ai/) for verified Qur'an text, named translations, and attributed tafsir during religious-methodology research. Record the exact verse, edition, retrieved content, and date when using it; fetch tafsir for interpretation and preserve scholarly differences. Numerical solar algorithms, timezone law, and physical error budgets require independently checkable scientific/civil sources. Neither an AI inference nor an unreviewed interpretation may silently become a calculation parameter or religious ruling. Research tools remain optional; daily calculation stays offline.
