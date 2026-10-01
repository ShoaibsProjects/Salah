# Salah: what we are building now

**Updated:** 1 October 2026. The [roadmap](roadmap.md) owns phase gates; this page is the plain-language map. “Implemented” means code exists and has the stated checks, not that it is approved for consumer use.

## Current piece

**Phase 4 → F4-A2: Offline Places and iPhone 12 Compatibility.** Built on F3-A1 Apple Native Foundation and F4-A1 One-Shot Apple Location. The earlier offline web setup and saved browser package remain available. These are research adapters; no accuracy, civil-time or religious gate is passed.

The first native SwiftUI app now compiles for iPhone/iPad and Simulator with the Rust engine statically bundled. A shared `salah-bridge` preserves the previous strict JSON operations; `salah-ffi` carries them across a versioned C boundary. Swift copies Rust-owned outputs and never calculates prayer times or converts event timezones. The installed app needs no server for manual calculation. It includes searchable IANA zones, a native date picker, selected-zone today, explicit method/Asr choices, unavailable-event explanations and the calculation record. See [Apple build instructions/evidence](../apps/apple/README.md) and the [Apple research/specification](../specification/apple-foundation-v0.1.md).

Core Location is opt-in with When-in-Use permission, a 30-second normal / 90-second precise deadline, reported fix age/accuracy, cancellation and city/manual fallback. The precise action can offer temporary full accuracy with a declared purpose. Apple chooses location sources; neither a native request nor a Simulator result proves GNSS-only radio-off positioning. The Apple app now searches 34,152 bundled GeoNames city points and saves up to 20 places only on explicit request. Saved startup reuses confirmed settings only while relevant rule/map/method identities match; today is recalculated by Rust. Records are app-private, backup-excluded and request complete iOS Data Protection. Simulator checks cannot establish physical encryption. See [F4-A2](../specification/apple-offline-places-v0.1.md). **Apple evidence:** unsigned device and Simulator builds pass; the app launches and renders its initial form on iOS 26 Simulator. Place search, persistence, startup/edit races, identity invalidation, corrupt/oversized records and deletion pass on iPhone 12 and iPhone 17 Simulator profiles. Swift warning-as-error device and Simulator builds pass. Six complete CLI/WASM/iOS Simulator schedule records match, covering DST, quarter-hour/date-line offsets, skipped dates and polar unavailable events. Host-model checks cover late lookup/manual-edit races and selected-zone today. The Rust suite passes with 106 tests, 1 intentionally ignored, plus 1 documentation test; format, Clippy and web-package build pass. Native control/accessibility and physical-device acceptance remain open. Linked Rust panic-symbolication retains a file-metadata API without an established Apple required reason, so Store privacy review is blocked and the manifest remains a research draft.

The whole operating idea is recorded in the [offline product contract](offline-product-contract.md). Valid coordinates now feed the embedded Rust boundary map automatically. A single timezone suggestion fills the field, with one confirmation because the current map is approximate; several/no suggestions require a choice. The device supplies a UTC epoch instant, and Rust fills today's date in that selected location's zone. The device's displayed timezone is not used. The clock source stays visible, and manual coordinates, zone, and date remain available.

The earlier one-shot device-location flow remains optional. It displays reported accuracy and now preserves the provider's fix timestamp in exported acquisition provenance. A manual coordinate edit rejects a late sensor reply. Browser Geolocation does not reveal whether GNSS or a network source was used; strict offline sensor acquisition is future native/embedded work. The research in the [offline product contract](offline-product-contract.md) distinguishes no-cost standalone GNSS from platform location services, which may use assisted or network-derived fixes. A compass cannot provide coordinates, and software cannot prove a plausible device wall clock is correct without another source.

The same Rust engine builds into WebAssembly. Additive bounded lookup/clock/selection exports preserve manual versus confirmed-map choice; the original strict v1 request remains available. The screen runs Rust in a worker, displays events and their rules, and exports a local provenance record. It performs no prayer mathematics or timezone/date conversion in JavaScript. Method and Asr choices remain explicit, with the existing plain-language explanations. See the [bridge contract](../specification/wasm-bridge-contract-v0.1.md) and [offline setup contract](../specification/offline-setup-contract-v0.1.md).

“Save this app for offline use” checks and caches the complete public app/engine/data inventory, including notices. It needs the files available initially, which can be a local archive and server without internet. Later loading uses only that saved version; applying a new complete version explicitly reloads the page. Browser-cleared/evicted storage still needs restoration. This cache stores no coordinates, preferences, or schedule. One browser cache-only restart and calculation check now passes with its local server stopped; physical radio-off, browser-matrix, eviction, and update acceptance remain open. Bluetooth is not used.

The earlier named-zone CLI remains available. Direct manual zone choice bypasses polygon lookup, and skipped dates, zero/multiple cycles, and unavailable events remain visible in the bridge. No formula, method profile, solver threshold, or historical reference result was changed.

**Evidence so far:** cache-only browser use calculated Minneapolis automatically and London on the 2026 spring DST transition day; the real timezone select and native typed/calendar date control were exercised. Minneapolis prayer labels matched AlAdhan's minute labels. London Hanafi Asr differs from its minute-only AlAdhan label (Salah 17:27:18, AlAdhan 17:26); the cause is unresolved and recorded in the [offline setup evidence](../specification/offline-setup-contract-v0.1.md). The suite passes with 100 tests, 1 intentionally ignored, and 1 documentation test; format, warning-free Clippy, WASM/package build, JavaScript syntax, and diff checks pass. The full reference comparison is a spot-check, not certification. Reaching the embedded boundary map increases the module size; the generated inventory reports exact sizes. The earlier F3-W1 approximately 726 KB WASM and direct Node.js operation are historical evidence. Browser visual/interaction beyond the tested flows, offline eviction/update lifecycle, accessibility, real-device acquisition, physical airplane-mode and cross-target acceptance remain open. Phase 1 and Phase 2 remain open.

**Next pieces:** native form/permission/accessibility acceptance, resolution of the Apple metadata/privacy blocker before distribution, then physical iPhone manual calculation and one-shot location with airplane mode plus Wi-Fi/Bluetooth off. Record device/OS, permission, indoors/outdoors, wait, fix timestamp and accuracy. The Apple client now exists; physical offline positioning has not been tested. A separate Android `GPS_PROVIDER` adapter remains future work. Investigate the unresolved Asr comparison with sourced vectors before formula changes; broaden browser lifecycle acceptance. Physical locked-device saved-place protection, next-prayer/reminders and signed-pack platform integration remain separate pieces. The sourced offline directory and Apple saved places are implemented; browser/Android equivalents remain future work. Independent scientific and methodology review continues.

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
| Shared Engine Bridge | `salah-bridge` | Strict bounded JSON operations used unchanged by native and browser adapters. |
| Browser Engine Bridge | `salah-wasm` | Thin WASM exports of shared schedule, lookup and selected-zone clock operations. |
| Apple Engine Bridge | `salah-ffi` | C ABI 1: borrowed bounded UTF-8 input, opaque Rust-owned responses, exact release function. |
| Apple Schedule Screen | `apps/apple` | Native SwiftUI research app with embedded Rust/data and optional one-shot Core Location. Simulator evidence recorded; physical/Store acceptance open. |
| First Schedule Screen | `apps/web` | Local worker calculation, automatic setup, explicit choice, provenance download, and opt-in saved public app package; browser acceptance pending. |

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
| F3-W2 | Browser Reliability and Offline Loading | Hash-checked saved public package implemented; browser restart/update/eviction, accessibility, and interaction acceptance open. |
| F3-A1 | Apple Native Foundation | Device/Simulator builds and six complete cross-target records pass; global portability gate remains open. |
| F4-A2 | Offline Places and iPhone 12 Compatibility | Bundled city search, opt-in private saved places and identity checks implemented; iPhone 12/17 Simulator model checks pass, physical protection/positioning and interaction remain open. |
| F4-A1 | One-Shot Apple Location | Foreground adapter and native form implemented; physical radio-off and native accessibility/interaction acceptance open. Draft privacy manifest has a linked-Rust metadata blocker. |
| F4-C1 | Location and Choice Clarity | Implemented; JavaScript syntax, diff, and contract wording checked. Browser interaction/accessibility and real-device location review remain open. No phase gate is advanced. |
| F4-C2 | Offline Setup: Timezone Suggestion and Device Date | Implemented additive Rust operations and automatic filling; confirmation/manual correction retained. Native/WASM static build evidence, with browser/device acceptance open. |

Phase 1 (accuracy and method integrity) and Phase 2 remain open. High-latitude alternatives, Qibla, broader independent comparisons, Android and consumer app acceptance remain on the roadmap. A working update subsystem or native shell does not settle those other gates.

## Source discipline

The project may consult [quran.ai](https://mcp.quran.ai/) for verified Qur'an text, named translations, and attributed tafsir during religious-methodology research. Record the exact verse, edition, retrieved content, and date when using it; fetch tafsir for interpretation and preserve scholarly differences. Numerical solar algorithms, timezone law, and physical error budgets require independently checkable scientific/civil sources. Neither an AI inference nor an unreviewed interpretation may silently become a calculation parameter or religious ruling. Research tools remain optional; daily calculation stays offline.
