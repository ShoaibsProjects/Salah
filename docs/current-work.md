# Salah: what we are building now

**Updated:** 30 September 2026. The [roadmap](roadmap.md) owns phase gates; this page is the plain-language map. “Implemented” means code exists and has the stated checks, not that it is approved for consumer use.

## Current piece

**Phase 3: Portable Calculation Platform → F3-W1: Browser Bridge → WebAssembly Engine and First Schedule Screen.**

The same Rust engine now builds into WebAssembly. A strict JSON bridge accepts explicit coordinates, date, zone, method, and Asr settings and returns the existing shared schedule. The first browser screen runs that module in a worker, displays each event, explains its rule, and offers a local JSON download. It performs no prayer mathematics in JavaScript. See the [bridge contract](../specification/wasm-bridge-contract-v0.1.md).

The earlier named-zone CLI remains available. Direct manual zone choice bypasses polygon lookup, and skipped dates, zero/multiple cycles, and unavailable events remain visible in the bridge. No formula, method profile, solver threshold, or historical reference result was changed.

**Evidence so far:** native build/static checks, pinned-compiler WASM release build and binding generation, JavaScript syntax checks, and one direct use of the generated module in Node.js. The observed WASM is about 726 KB before compression. The local server serves the files; the in-app browser timed out and the Mac was reported locked, so browser visual/interaction review remains pending. No tests were added or run locally. Earlier test counts and GitHub's existing-suite runs are prior/unchanged-suite evidence; they do not cover the new bridge. Phase 1, Phase 2, and the Phase 3 cross-target gate remain open.

**Next piece:** F3-W2: Browser Reliability and Offline Loading. Complete browser visual/interaction review and durable offline startup before adding next-prayer state or reminders. Mobile binding and signed-pack platform integration remain separate open work.

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
| Browser Engine Bridge | `salah-wasm` | Validates bounded explicit JSON requests and exposes the same schedule document and bundled zone inventory. |
| First Schedule Screen | `apps/web` | Static local browser preview with worker calculation, event explanations, and JSON download; browser acceptance pending. |

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
| F3-W1 | WebAssembly Bridge and First Schedule Screen | Implemented; native/WASM builds and direct module use; browser acceptance pending. |
| F3-W2 | Browser Reliability and Offline Loading | Next: browser review and durable offline startup. |

Phase 1 (accuracy and method integrity) and Phase 2 remain open. High-latitude alternatives, Qibla, broader independent comparisons, portable bindings, and user apps remain on the roadmap. A working update subsystem does not settle those other gates.

## Source discipline

The project may consult [quran.ai](https://mcp.quran.ai/) for verified Qur'an text, named translations, and attributed tafsir during religious-methodology research. Record the exact verse, edition, retrieved content, and date when using it; fetch tafsir for interpretation and preserve scholarly differences. Numerical solar algorithms, timezone law, and physical error budgets require independently checkable scientific/civil sources. Neither an AI inference nor an unreviewed interpretation may silently become a calculation parameter or religious ruling. Research tools remain optional; daily calculation stays offline.
