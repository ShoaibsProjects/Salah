# Salah for iPhone and iPad — native research preview

A small SwiftUI app around the existing Rust engine. The installed app contains
the calculation model, explicit method profiles, 598 named IANA zones, the
approximate timezone boundary map, and 34,152 sourced city points. City search,
saved-place reuse and calculation need no internet,
account, API key, cloud backend, or browser server.

**Pieces:** Phase 3 → **F3-A1: Apple Native Foundation**; Phase 4 →
**F4-A1: One-Shot Apple Location** and **F4-A2: Offline Places and iPhone 12 Compatibility**.
Target: **iPhone 12 family and newer on iOS 17+**, alongside compatible iPads.
No Apple Intelligence, LiDAR or newer-model-only hardware is required. The [Apple research/specification](../../specification/apple-foundation-v0.1.md)
records the primary sources and limitations. The [roadmap](../../docs/roadmap.md)
still owns release gates. This is a development app, not an App Store release
or an independently approved prayer timetable.

## What works

- Search installed city names/aliases and choose a labeled approximate reference point offline.
- Opt in to save up to 20 named places, with an optional startup choice and deletion.
- Recompute today locally on reopening; changed rule/map/method identities require confirmation.
- Native manual latitude/longitude, searchable timezone list, and Gregorian date picker.
- Embedded Rust timezone suggestions with explicit confirmation or manual override.
- Today's date from the device UTC clock converted by Rust in the selected location's zone.
- Explicit calculation profile and Standard/Hanafi Asr choice, with plain-language explanations.
- Rust calculations off the UI actor; seven solar/prayer events, second-precision local readings,
  unavailable reasons, skipped dates, multiple cycles, and calculation/data provenance.
- Optional foreground Apple location request, cancellation, and a user-selected precise retry.

Swift never calculates prayer times or converts event timezone offsets.
`salah-bridge` contains the shared strict JSON operations used by native and
WebAssembly clients. `salah-ffi` provides C ABI 1 with Rust-owned responses;
Swift copies the bytes and frees each response once. Requests are limited to
8192 UTF-8 bytes. Ordinary invalid input produces an error, never a replacement
schedule. Rust unwind capture does not recover process abort, allocation
failure, or invalid foreign pointers.

## Build locally

Requires a Mac, full Xcode with an iOS SDK, Python 3.11+, and the pinned Rust
toolchain. First-time tooling/dependency installation may require internet;
the installed application's essential calculation does not. Install the Rust
targets using the channel in [rust-toolchain.toml](../../rust-toolchain.toml):

```sh
rustup target add --toolchain 1.98.1 aarch64-apple-ios aarch64-apple-ios-sim x86_64-apple-ios
cargo fetch --locked
python3 tools/build_apple.py
```

Run these from the repository root. The builder uses locked offline Cargo,
packages separate device and universal Simulator libraries as
`apps/apple/Generated/SalahEngine.xcframework`, and builds the unsigned app in
`target/apple-xcode/Build/Products/Debug-iphonesimulator/Salah.app`.
Generated libraries, third-party notices, and a build/hash inventory stay out
of Git. No Swift Package Manager dependencies are fetched.

Open [Salah.xcodeproj](Salah.xcodeproj) in Xcode, choose the **Salah** scheme,
then an iOS 17+ simulator, and Run. No developer account is needed for the
simulator. The builder also supports:

```sh
python3 tools/build_apple.py --framework-only
python3 tools/build_apple.py --device-app
```

The second command checks the unsigned iPhone/iPad compile and link. To install
on a physical device, select your own signing team in Xcode; the project has
no embedded signing identity. Keep the bundle identifier local to your build.
Distribution signing, an archive, icons, privacy approval, and Store review
remain separate work.

## Reproduce the bridge check

Build the web package, boot an iOS simulator, and supply its UUID:

```sh
python3 tools/build_web.py
xcrun simctl list devices available
python3 tools/check_apple.py --simulator YOUR_BOOTED_SIMULATOR_UUID
```

This compiles a developer-only Swift executable using the actual Rust C ABI,
runs it in the simulator, and compares complete schedule records with the
native CLI and generated WebAssembly module. Node.js is a developer dependency
for this comparison, not an app dependency. No location is requested.

After installing the built app in that simulator, check offline places as well:

```sh
xcrun simctl install YOUR_BOOTED_SIMULATOR_UUID target/apple-xcode/Build/Products/Debug-iphonesimulator/Salah.app
python3 tools/check_apple.py --simulator YOUR_BOOTED_SIMULATOR_UUID --places
```

The place check compiles the production model/storage/search code into a
developer-only executable and uses a disposable app-cache folder. It never
replaces the actual saved-place file. It covers aliases/diacritics/country labels,
city-to-Rust scheduling, save/reopen/startup, edits during startup, changed-data
invalidation, deletion, malformed/oversized data preservation and explicit reset.
This is model acceptance; actual native control taps remain to be reviewed.

## Evidence recorded on 1 October 2026

- Xcode 27.0 (27A266a), iOS SDK 27.0, pinned Rust 1.98.1.
- Rust static libraries built for arm64 iOS, arm64 Simulator, and x86_64 Simulator.
- Unsigned Debug iOS/device and universal Simulator apps compiled and linked;
  Release Simulator compilation/linking also passed.
- App installed/launched in iPhone 12 and iPhone 17 simulator profiles running iOS 26.0 (23A343).
  Device and universal Simulator builds also pass with Swift warnings treated as errors.
  The six-record comparison and new place acceptance pass on both profiles.
  A screenshot confirms initial native form/engine startup. Native control interaction,
  VoiceOver and physical-device behavior are not established by that screenshot.
- `check_apple.py` passed complete CLI/WASM/iOS Simulator record equality for
  Minneapolis, London spring DST, Apia's skipped 2011-12-30, Tromsø polar day,
  Kathmandu's quarter-hour offset, and Kiritimati's date-line offset.
  Lookup, selected-zone date carriage, unavailable status, malformed input and
  invalid UTF-8 rejection passed through Swift and the native ABI.
- A separate temporary host-model check passed lookup/edit races, timezone
  confirmation, manual override, skipped date, stale calculation rejection,
  selected-zone today, and invalid coordinates. This is host evidence, not GPS evidence.
- Workspace tests, formatting and warning-free Clippy passed. Shared operation
  bodies preserve the previous web implementation; no prayer formulas or method profiles changed.

## Location, privacy and remaining work

Apple Core Location chooses its sources. iPhones and cellular iPads may obtain
satellite fixes without internet; Wi-Fi-only iPads may lack satellite hardware.
This app cannot enforce GNSS-only or prove that Apple used no network. A compass
does not supply coordinates. Manual inputs always remain available.

The normal request is bounded to 30 seconds and targets 100-metre accuracy;
the explicit precise request allows 90 seconds and asks for best available accuracy.
Each accepts a finite valid fix no more than 60 seconds old or 5 seconds ahead
of the device clock. With approximate permission, the precise action offers
Apple's temporary full-accuracy prompt using the declared prayer-location purpose.
Declining preserves a labeled approximate estimate; improvement is never promised. Accuracy is an estimate, not a
guarantee. A satellite cold start may take longer than this app's deadline.
Manual edits, cancellation, timeout and backgrounding reject old callbacks.

Salah sends no coordinates to a server. Setup stays in memory unless the person
chooses **Save this place**. Saved records use an app-private, backup-excluded
file with complete iOS Data Protection requested and checked on physical devices.
No analytics, iCloud sync, background location, notification scheduler, Qibla
feature or signed-pack activation is added. Simulator checks do not prove
hardware encryption or locked-device behavior. See the
[offline-place contract](../../specification/apple-offline-places-v0.1.md) for
source pins, storage rules and acceptance limits. The bundled GeoNames directory
adds 6,806,396 bytes and needs no online geocoder. Regenerate it entirely offline
with `python3 tools/build_city_directory.py`.
The device wall clock remains unverified and timezone laws can require updated data.

**Store readiness is blocked:** the bundled Rust panic-symbolication code
retains a file-metadata API (`fstat`), and a valid Apple required-reason declaration
for its full access scope has not been established. Ordinary `panic=abort`
did not remove it. `C617.1` covers the new app-container/bundle metadata reads;
it does not establish an approved reason for Rust symbolication of system images.
The included privacy manifest is a research draft, not
distribution approval. See the specification before archiving or publishing.

Next: exercise the native controls and permission flows, then measure manual
calculation and location outdoors/indoors on a named physical iPhone with
airplane mode enabled and Wi-Fi/Bluetooth separately disabled. Record fix age,
wait and reported accuracy; failure must preserve manual use. Resolve the privacy
blocker before distribution; test complete protection while locking a physical phone.
Accessibility, Qibla, reminders,
Android and independent calculation/methodology review have their own gates.
