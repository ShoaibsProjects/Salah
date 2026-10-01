<div align="center">

# Salah

*Prayer, with clarity and care.*

Assalamu alaikum — you are welcome here.

[Our intention](#our-intention) · [Our progress](#where-we-are-today) · [For builders](#try-the-engine)

</div>

## Welcome

Thank you for stopping by. Salah is a small effort to make prayer-time information clear, private, and available wherever life takes you.

At home or on a journey, we hope it becomes a quiet, useful companion. You do not need to understand the technology to be part of this work. Your questions, experience, and care for your community are welcome too.

> I am God; there is no god but Me. So worship Me and keep up the prayer so that you remember Me.
>
> — Qur’an [20:14](https://quran.com/20/14), translation by Muhammad A. S. Abdel Haleem

Our hope is to offer something useful with care: prayer times people can understand, privacy they can keep, and room for the different scholarly practices of their communities.

## Where we are today

**As of 1 October 2026: the Rust research engine and first local web preview are being built and checked.** The preview now has an optional one-time device-location request and plain-language calculation choices. Mobile applications and a consumer web release are still ahead. Phase 1 accuracy and methodology review and Phase 2 civil-time gates remain open.

**Latest interface work:** Phase 4 exploratory experience → **F4-C1: Location and Choice Clarity**. The Phase 3 WebAssembly bridge remains a research preview with browser acceptance pending.

The engine is already here: it calculates solar and prayer events offline and converts them into local clock readings. It keeps the choices and data behind each result, so someone can later understand why a time was shown.

You can now use that local schedule from the terminal: choose a location, date, timezone, method, and Asr setting. The command prints the times with their local dates and UTC instants. It can also return a shared JSON document for the apps we build next. Your explicit timezone choice works without consulting the boundary map.

That same engine now builds into WebAssembly. The first local browser screen is a quiet place to make your choices, read the returned schedule, and open an event to see why that time was calculated. If you choose, one tap asks the browser for a single location estimate; the app shows the device-reported uncertainty and never watches location in the background. A compass shows direction, not coordinates. Your device may need connectivity for its location provider, and the timezone remains your explicit choice. The page itself makes no lookup or sends coordinates to Salah. Browser visual/interaction review, real-device location review, and durable offline installation remain ahead.

We have built signed timezone-update storage with restart recovery. The new runtime slice connects a verified snapshot to calculations and records its exact identity. Its focused acceptance evidence is still ahead; production keys and automatic update activation remain unfinished.

[See what we are building now](docs/current-work.md) · [Follow the roadmap](docs/roadmap.md)

<details>
<summary>The engine pieces and recorded checks</summary>


| Part | What exists |
| --- | --- |
| **Prayer Kernel** | Dependency-free Rust calculations for Fajr, sunrise, Dhuhr, Asr, Maghrib, Isha, and sunset; UTC results and explicit unavailable events. |
| **Civil Clock** | An embedded IANA 2026d pack with 598 named zones; DST, date changes, skipped dates, and local schedule conversion. |
| **Location and Zone Choice** | Approximate offline timezone suggestions, with explicit confirmation or manual choice. |
| **Schedule Composer** | A Rust API combining an explicit location, selected zone, date, method, and Asr criterion. |
| **Update Authentication** | Signature, integrity, inventory, and compatibility checks for candidate timezone packs under a pinned public key. |
| **Update Storage and Recovery** | A Unix prototype for signed archives, trial selection, confirmation, restart recovery, and update replay/downgrade checks. |
| **Calculation Lab Command** | Named-zone local schedules, optional versioned JSON, supported-zone listing, and the earlier fixed-offset research interface. |
| **Browser Bridge and First Screen** | A strict JSON-to-Rust WebAssembly boundary, worker-based local schedule preview, event explanations, and JSON download. Browser acceptance remains pending. |

Published checkpoint [1be80bc](https://github.com/ShoaibsProjects/Salah/commit/1be80bc) passed **100 Rust tests**, six pack-validator tests, formatting/lint checks, and GitHub CI. One subprocess helper is intentionally ignored by the ordinary runner and invoked by its parent recovery test.

The runtime and new interface slices have local build and static checks. No new tests were added or run locally for the interface, as requested for this build. Focused acceptance cases for the interface and alternative snapshots remain open; the earlier checkpoint's counts are prior evidence. See the [runtime contract](specification/runtime-rule-snapshot-v0.1.md) and [interface contract](specification/named-zone-interface-contract-v0.1.md).

These checks are engineering evidence. Independent scientific and Islamic-methodology review remain necessary.

The new WebAssembly slice builds with the pinned compiler and binding generator; its generated WASM was about 726 KB before compression. One direct module use in Node.js produced a schedule. Native builds/lints and JavaScript syntax checks are recorded. No new tests were added or run locally. The in-app browser timed out while opening the local server, so this is not a completed browser or cross-target acceptance review. See the [bridge contract](specification/wasm-bridge-contract-v0.1.md).

</details>

## Our intention

- **Free to use.** Everyday prayer calculations should require no subscription, paid API, or account.
- **Private by design.** Location can stay on the device; manual coordinates and zone choice remain available.
- **Offline essentials.** Daily calculation should work without a server or internet connection.
- **Explainable times.** A result should carry its method, astronomical model, adjustments, timezone data, and any unavailable condition.
- **Respect for communities.** Calculated prayer beginnings, mosque timetables, and iqamah are different things. Method choices deserve clear explanations and qualified review.
- **A durable center.** iOS, Android, and web clients will use the same documented engine. The interface can evolve while the calculation foundation remains reviewable.

Free use is our product intention. Long-term maintenance still needs people, governance, and a sustainable funding plan. Timezone laws can change, so offline snapshots also need a responsible update path.

## What still needs care

This repository is a **research preview**, with no institutionally endorsed or production-ready timetable. We will keep its limits visible as it grows.

<details>
<summary>Read the current limits and review questions</summary>

- Known questions remain around near-grazing horizon comparisons, Asr model/solver differences, and polar-night Asr policy. [The Phase 1 gate report](specification/phase-1-gate-report-v0.1.md) records the evidence and open questions.
- The kernel currently uses a sea-level horizon model, UTC as a practical approximation to UT1, and dates from 1900–2100. Terrain, elevation, and local atmospheric conditions are not modeled.
- High-latitude alternatives, Qibla, Islamic calendar features, mobile interfaces/bindings, notifications, and a consumer web release remain ahead. The first WASM bridge/local screen is a research preview.
- Available profiles are `research-15` and `mwl-angles-18-17`. The first is an engineering profile; the second records a published secondary parameter set. Neither is a universal default or institutional endorsement. See the [method register](specification/method-register-v1.md).
- Timezone boundary data is approximate. Independent transition checks cover selected locations; a large zone inventory does not establish accuracy everywhere.
- Signed update storage is a Unix prototype. The explicit runtime path is implemented with acceptance evidence pending. Production signing stewardship, platform integration, and automatic activation remain unfinished.

When the chosen solar condition does not occur, the engine says so. Any later alternative must be separately named, documented, and reviewed.

</details>

## Try the engine

Use the Rust toolchain pinned in [rust-toolchain.toml](rust-toolchain.toml). An initial build needs the locked build dependencies; subsequent calculations and checks can run offline once those are present.

```bash
cargo fetch --locked
cargo run --locked --offline -p salah-cli -- schedule \
  --lat 44.9778 --lon -93.2650 --date 2026-09-30 \
  --zone America/Chicago --method mwl-angles-18-17 --asr hanafi
```

This is a **research example**, with an explicit zone and method; it is not a recommended regional default. Named-zone schedules use bundled IANA rules for DST and date changes. The program reads no device timezone and needs no server for calculations.

Add `--json` for the versioned schedule document. List available timezone names with:

```bash
cargo run --locked --offline -p salah-cli -- zones
```

Skipped dates, multiple solar cycles, and unavailable events stay visible. No alternative time is silently inserted. The [interface contract](specification/named-zone-interface-contract-v0.1.md) describes the input and output precisely.

<details>
<summary>The earlier fixed-offset research command</summary>

```bash
cargo run --locked --offline -p salah-cli -- \
  --lat 44.9778 --lon -93.2650 --date 2026-09-27 \
  --utc-offset -05:00 --method research-15 --asr hanafi
```

`--utc-offset` is a fixed offset; it does not perform IANA/DST selection. This older command remains available for reproducing earlier research outputs.

Supported Asr criteria are `standard` and `hanafi`. Add `--display-minute` for the separately labeled prayer-start minute preview; it does not change the underlying UTC instant.

</details>

### Open the local browser preview

Build tools are needed once; calculation then runs in the local Rust/WASM module. Use Python 3.11+:

```bash
rustup target add wasm32-unknown-unknown --toolchain 1.98.1
cargo fetch --locked
cargo install --locked wasm-bindgen-cli --version 0.2.129
python3 tools/build_web.py
python3 -m http.server 8080 --bind 127.0.0.1 --directory apps/web
```

Open **http://127.0.0.1:8080** in a current browser. Choose your settings, or explicitly fill the labeled research example, then calculate. The page makes no location lookup, sends no coordinates to Salah, and calls no remote prayer API. If you choose device location, the browser/operating-system provider may use its own available sources and may need connectivity; the page displays its reported accuracy estimate. Every time displayed comes from the Rust document; local dates are preserved.

This preview needs its local HTTP server for initial asset loading. Installable offline startup and cache updates are future work. GitHub Actions also builds a downloadable `salah-web-research-preview` archive; preserve its data and library notices. The project source license and production release review remain open.

### Run the checks

```bash
cargo fmt --all -- --check
cargo clippy --locked --offline --workspace --all-targets -- -D warnings
cargo test --locked --offline --workspace
python3 tools/test_validate_tzif_pack.py
```

Committed Rust reference cases run offline. Optional source-audit and matrix-regeneration tools have their own research dependencies; those tools are not required for daily calculation.

## For builders and reviewers

<details>
<summary>Architecture, evidence, and contributor notes</summary>

```text
salah-core          solar events, prayer rules, typed UTC results
salah-time          recorded timezone rules and local schedules
salah-location      offline zone suggestions and explicit selection
salah-engine        schedule composition, data notices, shared JSON
salah-update        signed candidate verification
salah-update-store  local archives, trial selection, and recovery
salah-cli           named-zone schedules, JSON, fixed-offset lab
salah-wasm          bounded explicit JSON bridge to the same engine
apps/web            local worker-based schedule preview
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
- [Verified runtime snapshot](specification/runtime-rule-snapshot-v0.1.md)
- [Named-zone command and schedule document](specification/named-zone-interface-contract-v0.1.md)
- [WebAssembly bridge and first browser screen](specification/wasm-bridge-contract-v0.1.md)

</details>

Astronomers, qualified Islamic-methodology reviewers, engineers, accessibility specialists, translators, and thoughtful users are welcome. A careful question or a clearly documented discrepancy can help as much as a new feature. We ask contributors to treat one another with patience, explain uncertainty honestly, and keep the person relying on the result in mind.

### Licensing and stewardship

A project source-code license has not yet been selected; free use is the intended product direction, not a license grant. Third-party materials retain their own licenses and attribution: [timezone boundary data](data/third-party/tzf-2026d/ATTRIBUTION.md) and [IANA timezone data](crates/salah-time/fixtures/global/IANA-LICENSE). Maintainer responsibilities and production release stewardship remain open roadmap work.

</details>

## You are welcome to stay

We hope Salah becomes a small, dependable part of a person’s day: clear when they need information, quiet when they need space, and welcoming when they return.

> those who have faith and whose hearts find peace in the remembrance of God- truly it is in the remembrance of God that hearts find peace-
>
> — Qur’an [13:28](https://quran.com/13/28), translation by Muhammad A. S. Abdel Haleem

May Allah place barakah in this work, keep our intentions sincere, and make it beneficial to those who use it.

[Translation sources and verification](docs/readme-quran-sources.md), retrieved 30 September 2026. The welcome and project intentions above are our own words.

_Grounded with quran.ai: fetch_translation(20:14, 13:28, en-abdel-haleem)._
