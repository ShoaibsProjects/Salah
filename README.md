# Salah

**Project status:** Rust calculation-kernel research preview, September 2026. The repository has no consumer mobile or web application yet. Phase 1 validation remains open. F2-TZ0 and F2-TZ1 are reviewed; F2-TZ2 local-date selection is implemented in the current working tree and awaits review. No production timetable is authorized.

Salah is building a free, private, offline-capable prayer-time system for Earth. Its durable center is a documented, versioned Rust calculation core with reproducible reference data. Future iOS, Android, web, and other clients are intended to consume that core rather than reimplement prayer mathematics in UI code.

> **Important:** The current implementation is research infrastructure, not a religious ruling, institutionally endorsed timetable, or global civil-time solution. Read the calculation record and method metadata before interpreting a result.

## What exists today

- `salah-core` **v0.3.0** calculates one location and one Gregorian local date offline.
- The core returns UTC instants, raw unrounded seconds, typed `Occurs`/`Unavailable` events, event rules, and a reproducibility record.
- The implemented solar model is `NOAA-MEEUS-SOLAR-2`, using NOAA equations derived from Meeus, a sea-level apparent horizon of `-0.833°`, and UTC as a practical approximation to UT1.
- Checked constructors reject invalid coordinates, dates, offsets, and method parameters. Supported dates are 1900–2100.
- Built-in parameter profiles are `research-15` (15° Fajr/Isha, explicitly unattributed) and `mwl-angles-18-17` (18° Fajr/17° Isha, sourced to the PrayTimes parameter table; not an MWL endorsement).
- Asr supports `standard` and `hanafi` shadow criteria. Dhuhr and Maghrib adjustments are represented in the method profile and are zero in both built-in profiles.
- The solver handles upper transit, direction-aware sunrise/sunset/twilight crossings, near-grazing interior extrema, and explicit unavailable events. It does **not** silently apply a high-latitude fallback.
- The optional research-preview presentation adapter `prayer-start-ceil-minute` v0.1 produces a separate whole-minute label for the five prayer beginnings. It never replaces the calculation's UTC instant and is not a notification instant, fasting cutoff, or mosque timetable.
- `salah-cli` exposes the kernel with explicit coordinates, date, fixed UTC offset, method, Asr criterion, and optional `--display-minute` output.
- `salah-time` **v0.1.0** converts UTC instants with pinned `jiff` 0.2.37 and caller-supplied TZif bytes from a six-zone IANA 2026d fixture pack. The F2-TZ2 research selector uses the F2-TZ1 UTC-anchor API to find every solar cycle whose transit maps to a requested local date; it returns zero, one, or multiple matches explicitly. It does not infer a zone from coordinates or read the host zone database.
- Reference and validation assets include a 19-case USNO solar matrix, grazing-horizon evidence, a 28-row prayer-library comparison, an Asr residual audit, method-source manifests, and presentation/rounding tests.
- CI runs the Rust workspace checks through the repository workflow; the project is dependency-light and has no mandatory runtime service.

## Current boundaries

The current kernel is intentionally narrower than the product vision:

- `--utc-offset` is a manually supplied fixed offset, **not** an IANA time zone and not a DST rule set.
- There is no coordinate-to-time-zone lookup, global bundled zone pack, consumer location experience, Qibla implementation, notifications, Flutter/mobile client, WASM client, or web app.
- Civil-time coverage is limited to the six pinned probe zones: `Etc/UTC`, `America/Chicago`, `Europe/London`, `Asia/Kathmandu`, `Pacific/Kiritimati`, and `Pacific/Apia`. F2-TZ2 is under architect review; its selector is not a consumer daily schedule.
- Elevation, terrain, weather, and local atmospheric conditions are not modeled; elevation is fixed at sea level.
- No high-latitude alternative is selected. If the chosen solar condition does not cross during the cycle, the result is explicitly unavailable.
- Phase 1 has not passed. Existing allowances and comparisons are case-specific investigation triggers, not global accuracy claims. Institutional method review, broader astronomy evidence, civil-time integration, and release thresholds remain open.

## Repository map

```text
crates/
  salah-core/       Offline solar/prayer kernel, typed events, records, tests
  salah-cli/        Minimal research command-line interface
  salah-time/       Pinned TZif conversion and local-date transit selector

data/reference/    Versioned USNO, prayer-library, Asr, grazing, and method data

docs/               Vision, roadmap, validation plan, decisions, handoff, civil-time probe
specification/      Calculation contracts, accuracy protocol, reports, method and display contracts
tools/              Offline/reference-audit helpers (Python and Node)
.github/workflows/  Rust CI
```

The existing CLI flow sends validated coordinates/date/fixed offset and a versioned method profile to `salah-core`, which computes UTC events and statuses. The experimental IANA flow sends coordinates, method, Asr criterion, a requested local date, selected zone ID, and pinned TZif bytes to `salah-time`; it uses `salah-core` to enumerate candidate solar cycles, then verifies each transit's local date with those same zone bytes and records the zone-data version.

## Quick start

### Requirements

- Rust toolchain from `rust-toolchain.toml` (`1.98.1`, with `clippy` and `rustfmt`).
- Python 3 for the USNO audit helper.
- Node.js is needed only for the optional prayer-matrix regeneration tool.

### Run a calculation

```bash
cargo run -p salah-cli -- \
  --lat 44.9778 --lon -93.2650 --date 2026-09-27 \
  --utc-offset -05:00 --method research-15 --asr hanafi
```

To additionally show the research-preview prayer-start minute labels:

```bash
cargo run -p salah-cli -- \
  --lat 44.9778 --lon -93.2650 --date 2026-09-27 \
  --utc-offset -05:00 --method research-15 --asr hanafi \
  --display-minute
```

Supported methods are `research-15` and `mwl-angles-18-17`; supported Asr values are `standard` and `hanafi`. The CLI prints seconds and UTC alongside the fixed-offset local text, and reports unavailable events instead of inventing a time.

### Test offline

```bash
cargo test --locked --offline
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

Focused reference checks:

```bash
cargo test --locked --offline -p salah-core --test usno_solar_matrix -- --nocapture
cargo test --locked --offline -p salah-core --test prayer_library_matrix -- --nocapture
cargo test --locked --offline -p salah-core --test presentation_minute -- --nocapture
python3 tools/verify_usno_matrix.py --all
```

The Python command performs an online source audit when it needs to retrieve USNO responses; the Rust reference tests themselves use committed data and do not require network access. The optional `tools/generate_prayer_matrix.cjs` tool requires separately obtained, hash-checked Adhan JS and PrayTimes sources.

## Documentation and evidence

Start with these documents:

- [Vision and architecture](docs/vision-and-architecture.md) — product promise and separation of concerns.
- [Delivery roadmap](docs/roadmap.md) — authoritative phase gates and current position.
- [Phase 1 validation plan](docs/phase-1-validation.md) — active evidence tasks and open limitations.
- [Calculation contract v0.3](specification/calculation-contract-v0.3.md) — current kernel interface and assumptions.
- [Accuracy budget](specification/accuracy-budget.md) — comparison protocol and what is not yet measured.
- [Reference data guide](data/reference/README.md) — provenance and offline matrix commands.
- [Presentation contract v0.1](specification/presentation-contract-v0.1.md) — separate research-preview minute-display policy.
- [Civil-time spike](docs/civil-time-spike.md) — reviewed six-zone TZif conversion boundary.
- [Civil-date transit selector v0.1](specification/civil-date-transit-selector-v0.1.md) — F2-TZ2 API and completeness argument.
- [Decision record](docs/decisions.md) — accepted choices and unresolved decisions.
- [Contributor and agent guidance](AGENTS.md) — rules for calculation changes, evidence, and review.

When changing calculation behavior, preserve the prior contract/model/profile identity, add provenance and a regression case, compare UTC instants before rounded local display, and document unresolved astronomical or scholarly questions. Do not treat a passing sample or a phase document as permission to claim global accuracy or production readiness.

## Planned next steps

The active roadmap keeps Phase 1 validation and independent review open while the F2-TZ2 six-zone selector is reviewed. Next, the civil-time layer must localize every event and then grow toward a versioned global zone pack and coordinate-to-zone data. Portable bindings, an accessible experience, an offline consumer beta, and public release stewardship remain later milestones.

The repository intentionally defers accounts, ads, cloud-calculation dependencies, social features, blockchain, Mars support, and AI-generated prayer times.
