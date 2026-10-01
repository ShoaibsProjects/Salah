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

**As of 1 October 2026: the Rust research engine, local web preview, and first native iPhone/iPad development app are built.** They use the same engine and bundled data. The Apple app compiles for device and Simulator; six complete schedule records match the CLI and WebAssembly. Physical iPhone offline-location testing, native usability and consumer releases remain ahead. Phase 1 accuracy/methodology and Phase 2 civil-time gates remain open.

**Latest pieces:** Phase 3 → **F3-A1: Apple Native Foundation**; Phase 4 → **F4-A2: Offline Places and iPhone 12 Compatibility**. [Build the Apple preview](apps/apple/README.md). The web preview's offline setup and saved package remain available.

The engine is already here: it calculates solar and prayer events offline and converts them into local clock readings. It keeps the choices and data behind each result, so someone can later understand why a time was shown.

You can now use that local schedule from the terminal: choose a location, date, timezone, method, and Asr setting. The command prints the times with their local dates and UTC instants. It can also return a shared JSON document for the apps we build next. Your explicit timezone choice works without consulting the boundary map.

That same engine builds into WebAssembly. The local browser screen is a quiet place to read a schedule and see why each time was calculated. Coordinates feed the embedded timezone map; a single suggestion fills automatically, with one confirmation because the map is approximate. Rust converts the device's UTC clock into today's date for that location. Manual correction is always available, and method/Asr choices remain yours.

If you choose device location, the browser asks once and displays the provider's uncertainty. A compass supplies direction, not coordinates. A GNSS receiver can work without internet, but hardware and platform access differ; this browser API cannot certify which source was used or that no network was involved. Salah calls no remote prayer or location service. Browser/device review remains ahead.

The whole operating idea is kept in the [offline product contract](docs/offline-product-contract.md): one local engine, bundled essential data, automatic setup where readings are usable, and clear correction when they are missing or uncertain. A saved browser copy can reopen with its engine files offline; the new Apple binary bundles them at installation. On iPhone 12 and newer with iOS 17+, choose a city offline or save a place for a quieter next visit. Optional device location is bounded and shows uncertainty; manual inputs remain available. City points are approximate and travel needs a new location choice. We want daily use to stay available when a server disappears.

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
| **Offline Setup and Saved Package** | Embedded coordinate-to-zone suggestions, selected-zone date from the device UTC clock, and opt-in hash-checked public app caching. Browser restart/update/eviction acceptance remains pending. |
| **Apple App and Native Bridge** | SwiftUI iPhone/iPad research app, static Rust C ABI, bundled engine/data, searchable zones, native date control, optional one-shot location, offline city search and opt-in saved places. Six cross-target records match; physical-device/Store acceptance remains open. |

Published checkpoint [1be80bc](https://github.com/ShoaibsProjects/Salah/commit/1be80bc) passed **100 Rust tests**, six pack-validator tests, formatting/lint checks, and GitHub CI. One subprocess helper is intentionally ignored by the ordinary runner and invoked by its parent recovery test.

The runtime and new interface slices have local build and static checks. No new tests were added or run locally for the interface, as requested for this build. Focused acceptance cases for the interface and alternative snapshots remain open; the earlier checkpoint's counts are prior evidence. See the [runtime contract](specification/runtime-rule-snapshot-v0.1.md) and [interface contract](specification/named-zone-interface-contract-v0.1.md).

These checks are engineering evidence. Independent scientific and Islamic-methodology review remain necessary.

The Apple foundation adds four shared-boundary checks and two C ABI checks;
the current suite passes 106 Rust tests plus one documentation test, with the
same intentionally ignored subprocess helper. Unsigned iPhone/Simulator builds
and six CLI/WASM/iOS Simulator record comparisons pass. The linked Rust panic
diagnostics leave an Apple metadata/privacy declaration unresolved, so the
manifest is a development draft and Store distribution is blocked. See the
[Apple evidence and next steps](apps/apple/README.md).

The original WebAssembly slice built with the pinned compiler/binding generator and had one direct module use in Node.js. Its earlier approximately 726 KB WASM did not expose the boundary lookup. The new offline setup build includes that global geometry: WASM is about 4.9 MB and the public saved inventory about 5.7 MB before compression. Native builds/lints, WASM generation, and JavaScript syntax checks are recorded. No new tests were added or run locally. Build evidence does not complete browser/device or offline lifecycle acceptance. See the [bridge](specification/wasm-bridge-contract-v0.1.md) and [offline setup](specification/offline-setup-contract-v0.1.md) contracts.

</details>

## Our intention

- **Free to use.** Everyday prayer calculations should require no subscription, paid API, or account.
- **Private by design.** Location can stay on the device; manual coordinates and zone choice remain available.
- **Offline essentials.** Daily calculation should work without a server or internet connection.
- **Gentle setup.** Use available local coordinates, embedded timezone data, and the device clock; show their source and make correction simple.
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
- High-latitude alternatives, Qibla, Islamic calendar features, Android, notifications, and consumer releases remain ahead. The Apple and web clients are research previews; physical radio-off Apple positioning and Store/privacy acceptance are open.
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

Open **http://127.0.0.1:8080** in a current browser. Enter coordinates or request an available device fix, confirm the embedded map's timezone suggestion, and choose your method/Asr preference. Today is derived locally by Rust in that zone; turn off device-today mode to choose another date. The labeled research example remains available. No coordinates are sent to Salah and no remote prayer/geocoding API is called. The optional browser/OS location provider may itself need connectivity; its source is unknown. Every event displayed comes from the Rust document.

Choose **Save this app for offline use** while the complete files are available. The worker verifies the build inventory and asset hashes before reporting them saved. Its activated copy uses local cached assets without a server fallback; a complete new version waits for your explicit apply/reload. Browser storage can be cleared or evicted, so keep a local copy for restoration. This behavior is implemented with browser lifecycle acceptance still open. GitHub Actions builds a downloadable `salah-web-research-preview` archive; preserve its data/library notices. The source license and production release review remain open.

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

Start with [AGENTS.md](AGENTS.md), the [offline product contract](docs/offline-product-contract.md), the [decision record](docs/decisions.md), and the [accuracy budget](specification/accuracy-budget.md). Preserve historical contracts and reference evidence when changing behavior.

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
- [Offline setup and saved browser package](specification/offline-setup-contract-v0.1.md)

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
