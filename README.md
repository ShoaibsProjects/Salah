<p align="center">
  <img src="docs/assets/salah-welcome.svg" alt="Salah — prayer, with clarity and care. Assalamu alaikum." width="1000">
</p>

<p align="center">
  <a href="#a-place-for-salah-in-everyday-life">Everyday life</a> ·
  <a href="#the-care-we-want-to-offer">Our intention</a> ·
  <a href="#being-built-with-care">Our progress</a> ·
  <a href="#technical-details">Technical details</a>
</p>

## You are welcome here

Assalamu alaikum wa rahmatullah.

Perhaps you are at home, planning the day ahead. Perhaps you have just arrived in a city where everything feels unfamiliar. Perhaps you are somewhere peaceful, with no signal, and would simply like to know the prayer times for the place you have chosen.

Salah is being built with these ordinary moments in mind: a gentle way to find prayer-time information, understand the choices behind it, and carry it with you.

We hope to make the practical part a little easier, leaving you more space for what matters to you in prayer. You are welcome whether you know your local timetable well or are still learning how to read one.

> I am God; there is no god but Me. So worship Me and keep up the prayer so that you remember Me.
>
> — Qur’an [20:14](https://quran.com/20/14), translation by Muhammad A. S. Abdel Haleem

<p align="center"><img src="docs/assets/salah-divider.svg" alt="" width="1000"></p>

## A place for salah in everyday life

These are the situations we are designing for. **Salah is currently a development preview, with independent review still ahead.** The sections below describe the experience we hope to offer; they are not a claim that every feature or situation is ready today.

### At home, before the day becomes busy

A morning with family. A day of study. A shift at work. Knowing the day's prayer times can help you make room for prayer among your responsibilities.

In the Apple preview, you can save a place and your chosen settings after calculating a schedule. When you return, it uses that saved place to calculate the new day. There is no need to repeat the location search each time. If you have travelled, choose your new place so the displayed schedule follows your intended location.

### Arriving somewhere unfamiliar

After a long journey, even a simple task can feel like one more thing to figure out. We want choosing your destination to be straightforward.

The Apple preview includes city search that works without internet. You can choose a city by name and check its timezone. A city point is approximate; where a usable device location is available, it can help refine your position. You can also correct the location yourself.

A destination's daily schedule is useful for planning your stay. Prayer while aboard an aircraft or ship can involve different circumstances; this preview does not yet offer an in-transit prayer mode or determine the religious rulings for your journey.

### When there is no connection

A rural visit, a walk away from town, a weak signal, or simply a day with mobile data switched off: access should remain possible.

Once the complete Apple app is installed, its city search, saved places and calculations work without an internet connection. Prepare your place before leaving when you can. A new device location may still be unavailable, particularly indoors; a saved place is useful when it still matches where you are.

Offline availability does not remove the need for a correct device date, an appropriate location or occasional updates. We will keep those limits visible and make correction as simple as we can.

### Learning what the times mean

It is reasonable to ask why two timetables differ. A calculation setting, a different Asr practice, or a mosque's congregation time can change what you see.

Salah explains the selected calculation and Asr choices in ordinary language. We want you to be able to ask, **“Why this time?”**, and find an understandable answer, with more detail available if you want it.

The choice belongs to you and the guidance you follow. We welcome questions without assuming that everyone begins with the same knowledge.

### Alongside your mosque and community

A mosque may publish a prayer timetable and separate congregation times. Those serve a different purpose from a calculated prayer beginning.

Salah aims to make that distinction clear and respect the established practices of different communities. If you follow a trusted local mosque, its guidance remains important—especially where local conditions or differing scholarly practices need explanation. Mosque timetable integration is planned; it is not available in the present preview.

### Helping someone you love

Perhaps you are helping a parent set up a phone, showing a friend how to choose a city, or making the screen easier to read together.

We want a small number of clear choices, legible words and explanations that can be read patiently. Older users, people using screen readers and people reading in different languages deserve the same care. Accessibility and language work remain part of the journey ahead, and your experience can help us improve them.

## The care we want to offer

**A quiet experience.** We want the useful information to be easy to find, with room to read and without pressure to keep opening the app.

**Privacy you can choose.** You can choose a place without requesting device location. Salah sends no coordinates to a prayer or location server. In the Apple preview, saving a place is your choice, and saved copies can be deleted.

**Everyday access without a subscription.** Free core use is our intention. No account or paid prayer-time service is required by the current calculation. Keeping the project useful for years will still need people, responsible maintenance and sustainable support.

**Respect for your practice.** A place does not decide your calculation method or Asr choice. Different communities deserve clear explanations and qualified review, without one setting being presented as the only religious truth.

**Honesty when something is uncertain.** If a selected solar condition does not occur, the preview explains that a time is unavailable. It does not quietly substitute a guess. Some regions need qualified local guidance and a separately reviewed approach.

**Care that can continue.** Our ambition reaches toward 2050 and beyond: useful prayer-time information that people can keep using and maintaining. That is a commitment to ongoing work, not a promise that today's unchanged application will remain correct forever.

<p align="center"><img src="docs/assets/salah-divider.svg" alt="" width="1000"></p>

## Being built with care

**As of 1 October 2026, Salah has working development previews for Apple devices and the browser.** The Apple preview includes offline city search, optional device location, saved places, date selection, prayer calculations and explanations of the choices behind them.

It is **not yet an App Store release or an independently approved prayer timetable**. Real-phone checks, accessibility work and independent astronomical and Islamic-methodology review remain open. Please continue using trusted prayer-time guidance while this work is being reviewed.

Qibla guidance, reminders and adhan, mosque timetables, an Islamic calendar, more languages and Android are still ahead. We will describe them as available when they are actually built and reviewed.

If you would like to help, you do not need to arrive with a perfect answer. Tell us where an explanation was unclear, what was difficult to read, or what a person in your community would need. Careful questions are valuable. Qualified reviewers, translators, accessibility specialists and engineers are welcome too.

[Share a question or experience](https://github.com/ShoaibsProjects/Salah/issues)

## Technical details

The engineering material lives here so the welcome above can stay focused on people. **Current piece: Phase 4 → F4-A2, Offline Places and iPhone 12 Compatibility.** The [roadmap](docs/roadmap.md) owns phase gates; the [current-work record](docs/current-work.md) explains implemented scope and next steps.

<details>
<summary><strong>Explore the engineering, evidence and build instructions</strong></summary>

### Architecture and supported targets

| Component | Implemented research scope |
| --- | --- |
| `salah-core` | Dependency-free Rust solar/prayer kernel; typed UTC events and explicit unavailable states. No AI-generated times. |
| `salah-time` | Bundled IANA 2026d rules for 598 named zones; local-date cycles, skipped dates, DST and event conversion. |
| `salah-location` / `salah-engine` | Approximate offline coordinate-to-zone suggestions, explicit confirmation/manual override and schedule composition. |
| `salah-bridge` / `salah-wasm` / `salah-ffi` | Shared strict JSON operations, WebAssembly and bounded native C ABI 1; clients delegate calculation to Rust. |
| `apps/apple` | Swift 6/SwiftUI preview targeting iPhone 12 family and newer on iOS 17+, alongside compatible iPads. Embedded engine/data, optional foreground Core Location and opt-in saved places. |
| Offline city directory | 34,152 pinned GeoNames reference points; 6,806,396-byte catalogue with size/hash verification and bundled CC BY 4.0 attribution. City points are approximate; GeoNames timezone fields are not used. |
| Saved places | At most 20 app-private records. Atomic writes, backup exclusion and complete iOS Data Protection requested/checked on physical devices; rule/map/method identities checked before startup reuse. |
| `apps/web` / `salah-cli` | Local worker-based browser preview, opt-in hash-checked public-asset caching, named-zone CLI schedules and versioned JSON. |
| `salah-update` / `salah-update-store` | Signed candidate checks and an experimental Unix archive/trial/recovery path. Production keys, platform activation and broader runtime acceptance remain open. |

The installed app requires no remote calculation service, geocoder, account or API key. First-time development tooling may need internet. A browser needs the complete files transferred or cached before offline reopening; cleared storage needs restoration.

Core Location chooses its own sources. A compass supplies heading, not coordinates; a native location result cannot prove GNSS-only or radio-off acquisition. Normal requests allow 30 seconds and precise requests 90 seconds, with optional temporary full-accuracy authorization. Saved places are labeled stored, never presented as fresh fixes. Device UTC is explicitly unverified; Rust derives today in the selected location's zone.

### Recorded evidence and release limits

- [Apple checkpoint `6aeb537`](https://github.com/ShoaibsProjects/Salah/commit/6aeb537): unsigned device/universal Simulator builds pass with Swift warnings treated as errors. City search, save/reopen, startup/edit races, identity invalidation, deletion and corrupt/oversized storage checks pass on iPhone 12 and iPhone 17 profiles running iOS 26.0.
- Six complete schedule records match CLI, WebAssembly and iOS Simulator, covering DST, unusual offsets, the date line, a skipped date and polar unavailable events. These are selected comparisons, not a global accuracy certificate.
- The prior [Apple foundation checkpoint](https://github.com/ShoaibsProjects/Salah/commit/14743b5) records 106 passing Rust tests plus one documentation test, with one intentionally ignored subprocess helper invoked by its parent. F4-A2 changed no prayer formulas or Rust code.
- Phase 1 accuracy/methodology and Phase 2 civil-time gates remain open. The kernel uses a sea-level horizon model, UTC as a practical UT1 approximation and dates from 1900–2100; terrain, elevation and local atmospheric conditions are not modeled.
- Available profiles are `research-15` (engineering comparison only) and `mwl-angles-18-17` (secondary published parameters, without institutional endorsement or a worldwide default). Near-grazing horizon, Asr residual and polar-night Asr questions remain documented.
- Physical radio-off positioning, locked-device storage protection, actual native interaction, accessibility and broader browser lifecycle acceptance remain open. Simulator results do not establish hardware performance or encryption.
- **App Store distribution is blocked by an unresolved linked-Rust file-metadata privacy declaration.** The manifest's `C617.1` covers new app-container/bundle reads; it does not resolve system-image symbolication scope. The manifest remains a development draft.

See [Apple build instructions and evidence](apps/apple/README.md), the [offline-place contract](specification/apple-offline-places-v0.1.md) and the [Phase 1 gate report](specification/phase-1-gate-report-v0.1.md).

<details>
<summary><strong>Build and run the research previews</strong></summary>

Use the toolchain in [rust-toolchain.toml](rust-toolchain.toml). The example below makes explicit choices for research; it is not a recommended regional default.

```sh
cargo fetch --locked
cargo run --locked --offline -p salah-cli -- schedule \
  --lat 44.9778 --lon -93.2650 --date 2026-10-01 \
  --zone America/Chicago --method mwl-angles-18-17 --asr hanafi
```

Add `--json` for the schedule record. List supported zones with:

```sh
cargo run --locked --offline -p salah-cli -- zones
```

The earlier fixed-offset research command remains available for historical reproduction:

```sh
cargo run --locked --offline -p salah-cli -- \
  --lat 44.9778 --lon -93.2650 --date 2026-09-27 \
  --utc-offset -05:00 --method research-15 --asr hanafi
```

This fixed-offset command performs no IANA/DST selection. `--display-minute` adds a separately labeled research minute preview and preserves underlying UTC instants.

For the local web preview, use Python 3.11+, the pinned Rust toolchain and binding generator:

```sh
rustup target add wasm32-unknown-unknown --toolchain 1.98.1
cargo fetch --locked
cargo install --locked wasm-bindgen-cli --version 0.2.129
python3 tools/build_web.py
python3 -m http.server 8080 --bind 127.0.0.1 --directory apps/web
```

Open **http://127.0.0.1:8080**. Browser caching stores public app/data files, not saved personal places. Preserve bundled notices and keep a local archive for recovery from storage eviction.

On a Mac with full Xcode, build the native preview:

```sh
rustup target add --toolchain 1.98.1 aarch64-apple-ios aarch64-apple-ios-sim x86_64-apple-ios
python3 tools/build_apple.py
```

Physical installation requires your own signing team; see the [Apple guide](apps/apple/README.md). A build is not distribution approval.

</details>

<details>
<summary><strong>Checks, specifications and contributor guidance</strong></summary>

```sh
cargo fmt --all -- --check
cargo clippy --locked --offline --workspace --all-targets -- -D warnings
cargo test --locked --offline --workspace
python3 tools/test_validate_tzif_pack.py
```

For the Apple bridge and places checks, build/install the preview and pass a booted simulator UUID as described in the Apple guide:

```sh
python3 tools/check_apple.py --simulator YOUR_BOOTED_SIMULATOR_UUID --places
```

Reference cases run offline. Optional source audits and data-regeneration tools have separate research dependencies; they are not daily runtime services.

Start with [AGENTS.md](AGENTS.md). Preserve historical evidence and use the [director handoff](docs/director-handoff.md) for delegated Phase 1 packets. Core documentation:

- [Vision and architecture](docs/vision-and-architecture.md) · [Decision record](docs/decisions.md)
- [Offline operating contract](docs/offline-product-contract.md) · [Accuracy budget](specification/accuracy-budget.md)
- [Calculation contract](specification/calculation-contract-v0.3.md) · [Method register](specification/method-register-v1.md) · [Reference provenance](data/reference/README.md)
- [Presentation policy](specification/presentation-contract-v0.1.md)
- [Local-date cycle selection](specification/civil-date-transit-selector-v0.2.md) · [Local schedules](specification/local-prayer-schedule-contract-v0.1.md) · [Civil-date existence](specification/civil-date-existence-contract-v0.1.md)
- [Global timezone pack](specification/global-timezone-pack-v0.1.md) · [Zone selection](specification/coordinate-zone-selection-contract-v0.1.md) · [Selected-day engine](specification/selected-local-day-engine-contract-v0.1.md)
- [Timezone data lifecycle](docs/civil-time-data-lifecycle.md) · [Pack validation](specification/offline-tzif-pack-interface-v0.1.md)
- [Signed candidates](specification/signed-rule-pack-candidate-v0.1.md) · [Repository/recovery](specification/rule-pack-repository-v0.1.md) · [Runtime snapshots](specification/runtime-rule-snapshot-v0.1.md)
- [Named-zone interface](specification/named-zone-interface-contract-v0.1.md) · [WASM bridge](specification/wasm-bridge-contract-v0.1.md) · [Offline setup](specification/offline-setup-contract-v0.1.md)
- [Apple foundation](specification/apple-foundation-v0.1.md) · [Offline places](specification/apple-offline-places-v0.1.md)

**Licensing and stewardship:** a project source-code license has not yet been selected; the free-use intention is not a license grant. Third-party material retains its own terms: [GeoNames](data/third-party/geonames-2026-10-01/ATTRIBUTION.md), [timezone boundaries](data/third-party/tzf-2026d/ATTRIBUTION.md) and [IANA](crates/salah-time/fixtures/global/IANA-LICENSE). Maintainers, funding and production release stewardship remain open.

The decorative SVGs are local, static project artwork with no external fonts, scripts or tracking. The Qur’an quotations use the verified translation recorded in [the source note](docs/readme-quran-sources.md); the surrounding welcome is original project writing, not a fatwa or attributed scholarly statement.

</details>

</details>

<p align="center"><img src="docs/assets/salah-divider.svg" alt="" width="1000"></p>

## May this work be of benefit

Thank you for spending a little of your time here. Whether you return with a question, help us improve an explanation, or simply wish this work well, you are welcome.

> those who have faith and whose hearts find peace in the remembrance of God- truly it is in the remembrance of God that hearts find peace-
>
> — Qur’an [13:28](https://quran.com/13/28), translation by Muhammad A. S. Abdel Haleem

May Allah keep our intentions sincere, place barakah in this work, and make it useful to the people it reaches. May we build with patience, listen with humility, and treat one another with kindness.

*Prayer, with clarity and care.*

_Grounded with quran.ai: fetch_translation(20:14, 13:28, en-abdel-haleem)._
