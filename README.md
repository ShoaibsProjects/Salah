# Salah

### A quiet companion for prayer. A careful foundation for trust.

Assalamu alaikum. You are welcome here.

We are building Salah with a simple intention: to help people make room for prayer, with clear information and a calm experience. Wherever someone lives, and however comfortable they are with technology, we want the essentials to be easy to understand and available offline.

> I am God; there is no god but Me. So worship Me and keep up the prayer so that you remember Me.
>
> — Qur’an [20:14](https://quran.com/20/14), translation by Muhammad A. S. Abdel Haleem

Our hope is to offer something useful with care: prayer times people can understand, privacy they can keep, and room for the different scholarly practices of their communities.

## Where we are today

**As of 30 September 2026: the Rust research engine is being built and checked.** Mobile and web applications are still ahead. Phase 1 accuracy and methodology review and Phase 2 civil-time gates remain open.

**Current piece:** Phase 2: Civil Time → F2-TZ9: Timezone Data Updates → **P2b.1: Local Repository and Crash Recovery**.

The engine calculates solar and prayer events, converts them into local clock readings, and preserves the choices and data behind each result. The latest piece stores signed timezone updates and recovers a previous valid selection after an interrupted trial. Stored updates are not yet connected to live prayer calculations.

| Part | What exists |
| --- | --- |
| **Prayer Kernel** | Dependency-free Rust calculations for Fajr, sunrise, Dhuhr, Asr, Maghrib, Isha, and sunset; UTC results and explicit unavailable events. |
| **Civil Clock** | An embedded IANA 2026d pack with 598 named zones; DST, date changes, skipped dates, and local schedule conversion. |
| **Location and Zone Choice** | Approximate offline timezone suggestions, with explicit confirmation or manual choice. |
| **Schedule Composer** | A Rust API combining an explicit location, selected zone, date, method, and Asr criterion. |
| **Update Authentication** | Signature, integrity, inventory, and compatibility checks for candidate timezone packs under a pinned public key. |
| **Update Storage and Recovery** | A Unix prototype for signed archives, trial selection, confirmation, restart recovery, and update replay/downgrade checks. |
| **Calculation Lab Command** | A working command-line research interface using a manually supplied fixed UTC offset. |

The latest local verification on macOS passed **100 Rust tests** and formatting/lint checks. One subprocess helper is intentionally ignored by the ordinary runner and invoked by its parent recovery test. These checks are engineering evidence; independent scientific and Islamic-methodology review remain necessary.

**Next piece:** **P2b.2: Stored-Pack Runtime and Platform Integration** — make the verified selected timezone snapshot the one actually used, and identify it on every civil-time result.

[Understand the current work](docs/current-work.md) · [Read the full roadmap](docs/roadmap.md) · [See the vision](docs/vision-and-architecture.md)

## The promise we are working toward

- **Free to use.** Everyday prayer calculations should require no subscription, paid API, or account.
- **Private by design.** Location can stay on the device; manual coordinates and zone choice remain available.
- **Offline essentials.** Daily calculation should work without a server or internet connection.
- **Explainable times.** A result should carry its method, astronomical model, adjustments, timezone data, and any unavailable condition.
- **Respect for communities.** Calculated prayer beginnings, mosque timetables, and iqamah are different things. Method choices deserve clear explanations and qualified review.
- **A durable center.** iOS, Android, and web clients will use the same documented engine. The interface can evolve while the calculation foundation remains reviewable.

Free use is our product intention. Long-term maintenance still needs people, governance, and a sustainable funding plan. Timezone laws can change, so offline snapshots also need a responsible update path.

## What still needs care

This repository is a **research preview**, with no institutionally endorsed or production-ready timetable.

- Known questions remain around near-grazing horizon comparisons, Asr model/solver differences, and polar-night Asr policy. [The Phase 1 gate report](specification/phase-1-gate-report-v0.1.md) records the evidence and open questions.
- The kernel currently uses a sea-level horizon model, UTC as a practical approximation to UT1, and dates from 1900–2100. Terrain, elevation, and local atmospheric conditions are not modeled.
- High-latitude alternatives, Qibla, Islamic calendar features, mobile/web interfaces, notifications, and portable bindings remain ahead.
- Available profiles are `research-15` and `mwl-angles-18-17`. The first is an engineering profile; the second records a published secondary parameter set. Neither is a universal default or institutional endorsement. See the [method register](specification/method-register-v1.md).
- Timezone boundary data is approximate. Independent transition checks cover selected locations; a large zone inventory does not establish accuracy everywhere.
- Signed update storage is currently a Unix prototype checked on macOS. Production signing stewardship, other platforms, and runtime activation remain unfinished.

When the chosen solar condition does not occur, the engine says so. Any later alternative must be separately named, documented, and reviewed.

## Try the engine

Use the Rust toolchain pinned in [rust-toolchain.toml](rust-toolchain.toml). An initial build needs the locked build dependencies; subsequent calculations and checks can run offline once those are present.

```bash
cargo fetch --locked
cargo run --locked --offline -p salah-cli -- \
  --lat 44.9778 --lon -93.2650 --date 2026-09-27 \
  --utc-offset -05:00 --method research-15 --asr hanafi
```

This is a **research example**, using explicit coordinates and a fixed offset. The CLI’s `--utc-offset` does not perform IANA/DST selection. The Rust schedule API provides the separate named-zone flow.

Supported Asr criteria are `standard` and `hanafi`. Add `--display-minute` for the separately labeled prayer-start minute preview; it does not change the underlying UTC instant.

### Run the checks

```bash
cargo fmt --all -- --check
cargo clippy --locked --offline --workspace --all-targets -- -D warnings
cargo test --locked --offline --workspace
python3 tools/test_validate_tzif_pack.py
```

Committed Rust reference cases run offline. Optional source-audit and matrix-regeneration tools have their own research dependencies; those tools are not required for daily calculation.

## For builders and reviewers

```text
salah-core          solar events, prayer rules, typed UTC results
salah-time          recorded timezone rules and local schedules
salah-location      offline zone suggestions and explicit selection
salah-engine        schedule composition and data notices
salah-update        signed candidate verification
salah-update-store  local archives, trial selection, and recovery
salah-cli           fixed-offset calculation lab
```

Start with [AGENTS.md](AGENTS.md), the [decision record](docs/decisions.md), and the [accuracy budget](specification/accuracy-budget.md). Preserve historical contracts and reference evidence when changing behavior.

<details>
<summary>Calculation, civil-time, and update contracts</summary>

- [Calculation contract v0.3](specification/calculation-contract-v0.3.md)
- [Reference data and provenance](data/reference/README.md)
- [Presentation policy](specification/presentation-contract-v0.1.md)
- [Local-date solar-cycle selection](specification/civil-date-transit-selector-v0.2.md)
- [Local prayer schedule](specification/local-prayer-schedule-contract-v0.1.md)
- [Civil-date existence](specification/civil-date-existence-contract-v0.1.md)
- [Global timezone pack](specification/global-timezone-pack-v0.1.md)
- [Coordinate-to-zone selection](specification/coordinate-zone-selection-contract-v0.1.md)
- [Selected local-day engine](specification/selected-local-day-engine-contract-v0.1.md)
- [Timezone data lifecycle](docs/civil-time-data-lifecycle.md)
- [Offline pack identity and validation](specification/offline-tzif-pack-interface-v0.1.md)
- [Signed candidate verification](specification/signed-rule-pack-candidate-v0.1.md)
- [Local repository and recovery](specification/rule-pack-repository-v0.1.md)

</details>

Astronomers, qualified Islamic-methodology reviewers, engineers, accessibility specialists, translators, and thoughtful users are welcome. A careful question or a clearly documented discrepancy can help as much as a new feature. We ask contributors to treat one another with patience, explain uncertainty honestly, and keep the person relying on the result in mind.

### Licensing and stewardship

A project source-code license has not yet been selected; free use is the intended product direction, not a license grant. Third-party materials retain their own licenses and attribution: [timezone boundary data](data/third-party/tzf-2026d/ATTRIBUTION.md) and [IANA timezone data](crates/salah-time/fixtures/global/IANA-LICENSE). Maintainer responsibilities and production release stewardship remain open roadmap work.

## A gentle intention

We hope Salah becomes a small, dependable part of a person’s day: clear when they need information, quiet when they need space, and welcoming when they return.

> those who have faith and whose hearts find peace in the remembrance of God- truly it is in the remembrance of God that hearts find peace-
>
> — Qur’an [13:28](https://quran.com/13/28), translation by Muhammad A. S. Abdel Haleem

May Allah place barakah in this work, keep our intentions sincere, and make it beneficial to those who use it.

[Translation sources and verification](docs/readme-quran-sources.md), retrieved 30 September 2026. The welcome and project intentions above are our own words.

_Grounded with quran.ai: fetch_translation(20:14, 13:28, en-abdel-haleem)._
