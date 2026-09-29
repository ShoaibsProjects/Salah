# F2-TZ1 — explicit UTC solar-cycle anchor

**Status:** implemented and architect-reviewed for research use on 2026-09-28. Phase 1 and Phase 2 gates remain open. This is an additive research API, not a consumer timetable.

## Why this is next

`salah-time` now converts a UTC instant under six pinned IANA zones. The existing `salah-core::CalculationInput` instead selects a solar cycle from a civil date and a minute-precision fixed offset. An IANA zone can change offset during that date, and historical offsets can include seconds. Feeding an offset resolved at one clock reading into the old input would misstate its meaning and could select the wrong cycle. The time-zone layer therefore needs a core operation whose selection input is an **explicit UTC instant**. This packet builds that operation without choosing a civil-date policy yet.

The current `upper_transit` implementation chooses a transit from a UTC anchor through its equation-of-time and longitude calculation. The new API must document and version that selector. Its output must identify the selected transit, selection policy, method, model, and UTC events. It must not claim that the transit belongs to a requested local date: that is F2-TZ2, which will convert candidate transits back through the selected IANA rules and handle zero or multiple candidates explicitly.

## Scope and acceptance

1. Add a public, typed `salah-core` operation accepting validated coordinates, a `UtcInstant` anchor, a validated method profile, and Asr criterion. Return the selected solar transit and the existing seven UTC event kinds with their existing rules and unavailable statuses. Define a distinct result/record type with no fictitious local date or fixed offset. Record a new selection-policy ID/revision. Keep `NOAA-MEEUS-SOLAR-2` if the astronomy and solver are unchanged; do not represent an interface addition as a new physical model.
2. Refactor only enough shared code to prevent two copies of the astronomy and prayer formulas. Preserve `CalculationInput`, `PrayerTimes`, `CalculationRecord`, `calculate_prayer_times`, CLI default output, event math, rounding, method IDs/revisions, and historical contract v0.3. Specify a supported UTC-anchor range and return a typed error for out-of-range or overflow instead of panicking or saturating. The old API must compute exactly the same results and records for the existing reference inputs.
3. Add a new versioned specification for the anchor interface, including input units, solar-cycle selection rule, output provenance, supported range, and what the API does **not** assert about civil dates. Do not edit the historical behavior contract. Document evidence that the old and new paths agree when the anchor equals the old local-noon UTC estimate, including a pre-epoch case and at least one date-line/offset extreme. Check adjacent anchors around a transit-selection boundary and expose any discontinuity rather than smoothing it.
4. Keep `salah-core` dependency-free and offline. Do not add IANA parsing to it. Do not modify `salah-time`, choose a date policy, infer a zone from coordinates, choose high-latitude fallbacks, alter Asr/solar formulas, or claim global accuracy or institutional endorsement.

**Review gate:** accept only if the old API and CLI behavior are preserved, the new record has truthful UTC-only provenance, selection boundaries are explicit, and the new API supplies the seam needed for F2-TZ2. Passing this packet does not pass either phase gate.

## Verbatim OpenCode implementation prompt

```text
Implement F2-TZ1, the additive UTC solar-cycle anchor interface, in /Users/shoaibakthar/Documents/Salah. Start from current main and leave all work uncommitted. Read AGENTS.md; docs/roadmap.md; docs/civil-time-spike.md; docs/civil-time-next-slice.md; docs/decisions.md; specification/calculation-contract-v0.3.md; crates/salah-core/src/prayer.rs, solar.rs, civil.rs, lib.rs; and the current core reference tests before editing. Phase 1 and Phase 2 gates remain open.

OBJECTIVE: give salah-core a public, typed way to calculate one solar cycle from an explicit UTC instant anchor, without pretending that the anchor is a local date or fixed UTC offset. Accept validated Coordinates, UtcInstant, MethodProfile, and AsrCriterion. Return the selected upper-transit UTC instant and the seven existing event kinds (Fajr, sunrise, Dhuhr, Asr, sunset, Maghrib, Isha), preserving each EventRule and UnavailableReason. Add a distinct result/record type containing truthful input/method/model/selection provenance and an explicit versioned UTC-anchor selection-policy ID. Specify and enforce the supported anchor range with a typed error; handle overflow rather than panic/saturate. Keep the astronomy model ID unchanged if astronomy is unchanged.

IMPLEMENTATION BOUNDARY: reuse the existing transit/event calculation internally; do not copy the prayer formulas. Preserve the existing CalculationInput, PrayerTimes, CalculationRecord, calculate_prayer_times API and all its historical results/records. Preserve normal CLI output byte-for-byte. Keep salah-core dependency-free. Do not change salah-time, IANA data, tzdb fixtures, method IDs/revisions, formulas, event adjustment/rounding, high-latitude behavior, or historical contract v0.3. Do not map civil dates to transits in this packet. Preserve unrelated files, including .idea/.

SPECIFICATION: create specification/utc-anchor-contract-v0.1.md describing the exact UTC-anchor selection algorithm and its boundary behavior, input/output units and range, selected-transit provenance, and the fact that a UTC anchor alone does not prove that transit belongs to a requested local date. Add only a short status/pointer in docs/roadmap.md and docs/civil-time-spike.md after implementation. Do not mark either phase complete or make a consumer accuracy claim.

EVIDENCE: add focused regression coverage for equality of old and new paths when the anchor is the legacy local-noon UTC estimate; include the existing reference locations, a pre-epoch case, and a date-line/offset extreme. Check anchors on both sides of a transit-selection boundary and report the actual selected transits. Check unavailable-event preservation. Run cargo fmt --all -- --check, cargo clippy --locked --offline --workspace --all-targets -- -D warnings, cargo test --locked --offline --workspace, and git diff --check. If evidence finds that the stated selector cannot be specified or old results cannot be preserved, stop and report the counterexample without silently changing behavior.

RETURN: changed files and diff stat; public API and policy ID; anchor-range rule; old/new equivalence evidence; boundary observations; command results; remaining limitations. Stop for architect review. Do not commit, push, start F2-TZ2, or implement a UI.
```

## Following packet

[F2-TZ2](civil-time-date-selector.md) will use the pinned `salah-time` zone rules to identify **all** solar transits whose converted local date matches the requested date. It will return explicit outcomes for zero, one, or multiple matching cycles, including skipped dates and date-line transitions. It will not choose one silently. Only after that selector is reviewed can a zone-aware daily prayer calculation be assembled and assessed for consumer use.
