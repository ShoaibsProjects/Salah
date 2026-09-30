# Civil-date solar-transit selector v0.2

**Status:** F2-TZ2 zone-coverage extension under review. This document amends the supported-data scope of [selector contract v0.1](civil-date-transit-selector-v0.1.md); all transit-search, canonicalization, date-membership, cardinality, error, and provenance rules from v0.1 remain in force unless stated here. Phase 1 and Phase 2 remain open.

## Scope change

The `select_local_date_transits` signature and `local-date-solar-transit` policy revision `0.1` do not change. The allowed `zone_id` set expands from the original six-zone probe to the 598 named IANA identifiers in [global timezone pack v0.1](global-timezone-pack-v0.1.md), generated from IANA 2026d.

For each accepted identifier, `tzif_bytes` must still match that identifier's exact byte slice from `fixture_tzif_bytes(zone_id)`. A different valid TZif image is rejected and cannot be labeled with `TZDB_VERSION`. The result records the zone ID and `tzdb_version = "2026d"` just as before. Names include IANA backward-compatible links; callers should preserve the identifier they selected instead of silently canonicalizing it.

The six original F2-TZ2 cases remain the focused transition evidence. The expanded pack suite additionally constructs every named zone through Jiff and verifies its returned provenance. That confirms each entry is loadable, not that all 598 histories have received independent astronomical or civil-time review.

## Unchanged boundaries

- No coordinates-to-zone inference is provided.
- A large named-zone list is not a border-aware geographic lookup.
- `Zero` transit matches still do not imply a skipped civil date; use the separate [civil-date existence contract](civil-date-existence-contract-v0.1.md).
- The six-zone F2-TZ0 evidence and old fixture manifest remain archived unchanged.
- No Phase 1/2 gate, production timetable, global accuracy, or religious endorsement claim follows.
