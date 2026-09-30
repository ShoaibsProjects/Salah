# Civil-time data lifecycle and operating rule

**Working decision, 29 September 2026.** This governs how the research engine reports the limits of an installed time-zone snapshot. Phase 1 and Phase 2 gates remain open. The [roadmap](roadmap.md) governs release claims.

## Standard rule for a usable offline engine

1. **Calculate deterministically.** Coordinates, requested local date, confirmed or manually selected IANA zone, method, and Asr criterion are explicit. The engine calculates UTC solar and prayer events, then applies one recorded IANA rule pack to produce local clock labels. No account, internet call, or device-zone inference is needed.
2. **Keep the basis visible.** Return the selected zone and its source, boundary-data and IANA versions, method and model versions, UTC instants, local offsets, event rules, and unavailable statuses. A user can see why a clock label appeared and reproduce it later.
3. **Do not turn uncertainty into a time.** An ambiguous boundary needs a user choice. A skipped civil date, zero or multiple solar cycles, or a missing solar crossing stays explicit. Any later high-latitude alternative must be a separately named, reviewed, user-visible rule.
4. **Treat future civil rules as changeable.** The bundled 2026d pack works offline even for later dates, but later local clock labels are projections under that snapshot. The [v0.1 assessment](../specification/civil-time-data-assessment-contract-v0.1.md) prompts review when the pack year precedes an explicit observation date or when the requested date is future. Neither condition changes a calculated result. Absence of a notice is not certification.
5. **Update data without requiring a service for each calculation.** A newer reviewed IANA/boundary pack may be offered when a connection exists; the installed pack remains available offline. Results must retain the exact pack identity that produced them. A pack update must never silently rewrite a saved historical result.
6. **Make correction explicit.** If the mapped zone is wrong, the user can already select another supported IANA zone. If the civil-time law changed but no updated pack is available, a future single-date, user-supplied fixed-offset mode should provide a labeled emergency correction. It must not silently predict future transitions or replace the stored IANA result.

## Implementation sequence

| Step | Deliverable | Decision gate |
| --- | --- | --- |
| F2-TZ8 — implemented | Pure assessment of installed data against a caller-supplied observation date; typed notices and version metadata. | Verify the notices and wording; no freshness or accuracy claim. |
| F2-TZ9-P1 — implemented | [Schema 1 manifest validator and exact rule-pack identity](../specification/offline-tzif-pack-interface-v0.1.md) in every civil-time result. Only the compiled pack is active. | Verify inventory, byte hashes, TZif parsing, and identity propagation; no authentication claim. |
| F2-TZ9-P2a — implemented | [Offline signed-candidate verifier](../specification/signed-rule-pack-candidate-v0.1.md) with application-pinned trust keys, sequence check, strict signature, bounded format validation, and private verified-candidate type. | Reject altered or incompatible candidates; no candidate activation claim. |
| F2-TZ9-P2b.1 — implemented prototype | [Local repository and crash recovery](../specification/rule-pack-repository-v0.1.md): signed archives, atomic state replacement, trial/confirmation, and high-water preservation in a separate Unix adapter. | macOS process-interruption/error-path evidence; no hardware power-loss, other-target, or calculation-activation claim. |
| F2-TZ9-P2b.2a — implemented, acceptance pending | [Explicit verified runtime snapshot](../specification/runtime-rule-snapshot-v0.1.md) and exact result identity; shared immutable payload validation. | Local build/lint checks; focused old/new snapshot and stored-selection acceptance cases remain open. |
| F2-TZ9-P2b.2b — integration task | Target persistence review and application trial-health/confirmation policy. | Verify platform packaging/storage semantics and selected-runtime startup/recovery. |
| F2-TZ9-P2b.3 — stewardship task | Named release maintainers, production key custody/rotation/revocation, retention, and optional distribution channel. | No production update release before reviewed signing and operational policy. |
| F2-TZ10 — later correction task | Explicit single-date fixed-offset path with separate provenance, supported range, and no inferred DST. | Demonstrate that it changes only civil rendering/date selection for the chosen date and cannot be confused with an IANA schedule. |
| Validation before release | Broader independent IANA transitions and boundary cases, astronomy and Islamic-methodology review, high-latitude rule decisions, and portable-target checks. | Record the supported scope and reviewers' decisions in the roadmap and decision record. |

The data-pack interface keeps calculation code independent of any download mechanism. The application can retrieve an update optionally; the Rust engine may consume it only after authentication, compatibility validation, and atomic activation. A bundled pack is always the recovery path. The signed-candidate format is specified, while production key custody, durable activation, and platform distribution still need reviewed decisions before network updates are enabled. No update service or fee may be required for a daily calculation.

The 2026d lookup validates its bundled boundary/rule release pair. The signed runtime separately authenticates compatibility with the exact boundary artifact and requires every currently supported identifier in its rule inventory; a later compatible rule release need not share the boundary release label. A saved zone choice must be revalidated when its rule pack changes.

## Why this supports a 2050 horizon

The formulas and historical results can remain reproducible while civil rules, boundary data, and methods evolve. Maintenance is still necessary: no 2026 snapshot can know every law that governments will pass by 2050. The product promise is an offline engine with transparent data and a practical update path, not an unchanged binary guaranteed to know future law.
