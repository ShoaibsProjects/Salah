# Salah: what we are building now

**Updated:** 30 September 2026. The [roadmap](roadmap.md) owns phase gates; this page is the plain-language map. “Implemented” means code exists and has the stated checks, not that it is approved for consumer use.

## Current piece

**Phase 2: Civil Time → F2-TZ9: Timezone Data Updates → P2b.1: Local Repository and Crash Recovery.**

The Sun gives us a UTC event. Timezone rules turn that event into a local clock reading. Governments can change those rules, so Salah needs a way to keep reviewed updates locally and recover if an installation is interrupted. P2b.1 stores signed packages separately, selects a trial through a small state record, and restores the previous package after an unconfirmed restart. It keeps the accepted update counter so recovery cannot allow an old update to be replayed. See the [repository contract](../specification/rule-pack-repository-v0.1.md).

This is a Unix storage prototype checked on macOS. The current prayer engine still calculates with its embedded rules. Connecting a stored pack to the calculation path, reviewing other platforms, and establishing production signing custody are remaining pieces of P2b.

## Names of the main parts

| Part | Code | Purpose and present state |
| --- | --- | --- |
| Prayer Kernel | `salah-core` | Calculates solar and prayer events in UTC. Dependency-free research kernel; independent astronomy/methodology review and known discrepancies remain open. |
| Civil Clock | `salah-time` | Converts UTC under a recorded IANA snapshot, handles DST/date changes, selects local-date solar cycles, and preserves missing events. |
| Location and Zone Choice | `salah-location` | Suggests zones from approximate offline boundaries; requires confirmation or manual choice. |
| Schedule Composer | `salah-engine` | Combines explicit location/zone, date, method, and Asr inputs into a localized schedule and data notices. |
| Update Authentication | `salah-update` | Verifies signed packages, exact hashes/inventory, and compatibility before parsing their payloads. P2a is implemented. |
| Update Storage and Recovery | `salah-update-store` | Archives signed packages and manages trial, confirmation, restart recovery, and counter preservation. P2b.1 is the current implemented slice. |
| Calculation Lab Command | `salah-cli` | Runs the existing fixed-offset research interface. Named-zone orchestration and stored packages are not exposed by this CLI yet. |

## How to name work

Use **phase → workstream → piece**, followed by a plain name in every handoff. Preserve the existing `F2-TZ*` identifiers for continuity; `F2` refers to Phase 2 work, not a separate roadmap.

| Piece | Plain name | State |
| --- | --- | --- |
| F2-TZ9-P1 | Pack Identity and Integrity | Implemented; exact bytes and zone inventory are identified. |
| F2-TZ9-P2a | Signed Candidate Verification | Implemented; no production key configured. |
| F2-TZ9-P2b.1 | Local Repository and Crash Recovery | Implemented prototype; macOS evidence and explicit filesystem limits. |
| F2-TZ9-P2b.2 | Stored-Pack Runtime and Platform Integration | Next bounded review/design: ensure a selected verified snapshot is the one used and recorded by every civil-time result, and review platform persistence. |
| F2-TZ9-P2b.3 | Production Signing Stewardship | Open: maintainers, key custody/rotation/revocation, release pipeline, retention, and distribution policy. |

Phase 1 (accuracy and method integrity) and Phase 2 remain open. High-latitude alternatives, Qibla, broader independent comparisons, portable bindings, and user apps remain on the roadmap. A working update subsystem does not settle those other gates.

## Source discipline

The project may consult [quran.ai](https://mcp.quran.ai/) for verified Qur'an text, named translations, and attributed tafsir during religious-methodology research. Record the exact verse, edition, retrieved content, and date when using it; fetch tafsir for interpretation and preserve scholarly differences. Numerical solar algorithms, timezone law, and physical error budgets require independently checkable scientific/civil sources. Neither an AI inference nor an unreviewed interpretation may silently become a calculation parameter or religious ruling. Research tools remain optional; daily calculation stays offline.
