# Apple native foundation v0.1

**Research date:** 1 October 2026. **Pieces:** Phase 3 → **F3-A1: Apple Native Foundation**; Phase 4 → **F4-A1: One-Shot Apple Location**. Read the [offline product contract](../docs/offline-product-contract.md), [current work](../docs/current-work.md), and [roadmap](../docs/roadmap.md). The roadmap owns release gates. This specification is a platform decision and research record, not device acceptance or accuracy certification.

## 1. Decision and boundary

Start an iPhone/iPad client in SwiftUI with minimum iOS 17. This is a project deployment choice, not an Apple requirement for Core Location. Native controls provide accessible forms, timezone choice, and a small date picker without a remote UI service or web wrapper. Other platforms continue to consume the same Rust engine.

Compile Rust as a static library with a small versioned C ABI. Swift exchanges bounded UTF-8 JSON and releases Rust-owned outputs through the matching Rust release function. No panic may unwind across C; malformed requests return explicit errors. Reuse the existing Rust schedule/lookup/clock operations instead of introducing Swift astronomy or host-timezone interpretation. Package device and Simulator variants separately in an XCFramework; Apple explicitly permits static libraries plus public headers in this bundle [A8]. An initial macOS host probe is useful binding evidence but cannot establish an iOS build.

The installed binary contains the essential model, methods, IANA rules, and approximate boundary data. The app has no prayer API, online geocoder, map downloader, account, analytics, Bluetooth scan, or network clock. The current practice choices stay explicit and the existing timezone confirmation rule stays in force. Core Location is an optional Apple system provider, whose internal behavior the app cannot control completely.

## 2. What Apple actually exposes

| Question | Primary-source finding | Product consequence |
| --- | --- | --- |
| Can a compass give coordinates? | Core Location documents compass heading separately from geographic location [A1]. | Compass cannot supply latitude/longitude; Qibla orientation is a later separate adapter. |
| Which Apple hardware has GPS? | Apple Support states GPS is available on iPhone and Wi-Fi + Cellular iPad and distinguishes Wi-Fi-dependent models [A2]. This hardware statement does not mean cellular data is necessary for every satellite fix. | Never assume a Wi-Fi-only iPad or a Mac contains a satellite receiver. Manual/saved coordinates keep calculation usable. |
| Does Core Location require our internet API? | It is an OS framework that selects available GPS, Wi-Fi, cellular and other device components [A1, A2, A3]. | No paid location API is needed. OS permission remains necessary. |
| Can we enforce GNSS-only? | The documented manager exposes desired accuracy, authorization and services; it chooses hardware for the requested precision [A1, A3, A6]. Its source record exposes simulation/accessory flags [A7], not a GNSS/network source selector. | Describe results as Apple system-provided, never GNSS-only or proven network-free. This research establishes no supported GNSS-only switch. |
| Is a requested accuracy guaranteed? | `desiredAccuracy` is best effort; `requestLocation()` can deliver a less accurate fix when the requested precision would take too long [A4, A6]. Reduced authorization overrides precise requests [A5]. | Show reported uncertainty and approximate/full authorization. High accuracy cannot manufacture hardware or permission. |
| Will airplane mode always work? | Apple lists airplane mode among possible service-unavailability reasons [A3]; GPS satellite visibility and hardware also matter [A2]. | Prove behavior on named physical devices with radios disabled; never infer universal offline live positioning from a successful calculation. |

## 3. One-shot acquisition policy

Request **When in Use** permission only after the person chooses “Use device location.” Include `NSLocationWhenInUseUsageDescription` in the app bundle [A3, A9]. Do not prompt on launch, request Always permission, enable background location, or start a continuous tracking loop for a daily schedule.

Check availability and authorization; distinguish not determined, denied, restricted and granted. Permission can change while the app runs. A user action may request one location using `CLLocationManager.requestLocation()`; Apple's method delivers one result or failure and stops the location service [A4]. Choose economical precision first. A person may explicitly request greater precision once. If authorization is reduced, changing `desiredAccuracy` has no effect [A5]; a later temporary full-accuracy flow must have its own declared purpose and consent, rather than silently promising improvement.

The adapter must bound its wait, support cancellation and stop work when no longer needed. `stopUpdatingLocation()` cancels a pending `requestLocation()` [A4]. Guard callbacks against superseded requests: manual coordinate edits, a new request, backgrounding, timeout and cancellation invalidate acceptance of an old reply. No result may overwrite newer manual input.

Validate finite bounded coordinates and finite nonnegative `horizontalAccuracy`; Apple defines a negative accuracy as an invalid latitude/longitude [A7]. Preserve the fix timestamp, clock-relative age, accuracy authorization, and source information when available. `isSimulatedBySoftware` and `isProducedByAccessory` are flags, not proof of a GNSS fix. Simulator/GPX results remain labeled simulated and do not count as physical-device evidence.

The implemented `apps/apple/Salah/DeviceLocation.swift` uses a **30-second deadline**, requests **100-meter desired accuracy** initially and **best desired accuracy** on an explicit precise attempt, and accepts timestamps only when their clock-relative age is **−5 through +60 seconds**. The five-second future allowance handles a small disagreement with the device clock; it does not establish correct UTC. An invalid/old-only response is rejected. Each attempt has a distinct manager identity; completion, manual edits, timeout, cancellation and actual backgrounding cancel it. The temporary permission-prompt inactive state is distinguished from background entry. No automatic repeated acquisition or temporary full-accuracy permission escalation is implemented.

Fix age is checked against the device wall clock, which may itself be wrong; accepting a fresh-looking fix does not verify UTC. Retain unavailable, stale, approximate and invalid outcomes explicitly. Offer manual coordinates for every outcome. Location uncertainty does not prove the approximate timezone boundary map is unambiguous, so its existing explicit confirmation remains.

## 4. Privacy and local persistence

Keep acquired coordinates and schedule setup in memory for this first slice. No coordinate history, iCloud container, shared app group, remote logging or cloud preference sync is needed. Any future “Save this place” feature needs visible opt-in, deletion, data protection and a defined retention policy. Apple recommends encryption for location stored on disk [A3].

App-private Application Support files may be appropriate for a future redundant local saved-place copy. Apple's `isExcludedFromBackup` is suitable for support/cache files not needed in backup, not arbitrary user documents [A12]. Backup exclusion does not encrypt data or prevent all OS-level access. Choose the protection mechanism and fail explicitly if it cannot be applied before promising secure saved location.

Bundle `PrivacyInfo.xcprivacy` as an app resource. Declare tracking false and no data sent off device when that matches the implementation; audit linked dependencies as well as Swift source. Apple's required-reason API policy is about actual calls [A10, A11]. Do not invent reasons for APIs the app never calls. If app-only `UserDefaults` is added, the documented reason is `CA92.1`; if system boot-time APIs measure app elapsed time, the documented timer reason is `35F9.1`; app-container file metadata has `C617.1` [A11]. These are conditional examples, not a declaration that this first slice uses those APIs. A static `.a` file does not carry resources: put the app manifest in the app target and revisit framework-specific requirements before publishing a separate SDK [A10].

The first source review finds no client `UserDefaults`, coordinate persistence, or app network client. The initial manifest contains empty collection and required-reason arrays. That source observation does not finish a linked-binary audit: the Simulator debug dylib imported file-metadata symbols from Rust's standard library, and the optimized Simulator release retained `_fstat`. Local source inspection identifies Rust backtrace symbolication's `File::open(path)` → `file.metadata().len()` for loaded Mach-O image and adjacent dSYM resolution. An isolated pinned-toolchain `panic=abort` build linked with optimized Swift and dead stripping still retained `_fstat` and the backtrace mapping path; its Mach-O platform was iOS Simulator with minimum OS 17.0. Ordinary `panic=abort` therefore does not remove this issue.

**Open distribution blocker:** establish the actual accessible file-path scope and a matching Apple-approved reason, or remove the metadata call path before a Store/archive privacy acceptance claim. The observed purpose is reading binary size for symbolication, not displaying timestamps. Do not add `C617.1` without evidence that all actual metadata accesses fall within its allowed app/group/CloudKit containers; loaded system-image paths make that unproven. Empty manifest arrays in this research preview are not a completed Store audit. Scope the privacy promise to Salah's own handling; Apple controls its OS provider and describes system location data practices separately [A2].

## 5. Build evidence and next pieces

The parent build records concrete paths, supported ABI operations, sizes and successful commands in the app README/current-work entry. Keep these states distinct:

1. **Code exists:** Rust bridge and native source are reviewable.
2. **Host build passes:** verifies a named compiler/platform only.
3. **iOS/Simulator build passes:** requires full Xcode and the SDK/targets, rather than command-line macOS tools alone.
4. **Physical offline positioning passes:** requires a named real device and recorded radio state.

Unsigned device and universal Simulator builds now pass; six complete CLI/WASM/iOS Simulator schedule records match. The app launches and renders its initial native form in the Simulator. See [the app evidence](../apps/apple/README.md) and `tools/check_apple.py` for reproducible commands. Native control/error-flow and accessibility acceptance, physical one-shot positioning, distribution privacy review, then explicit saved places/settings are the next pieces. Qibla, next-prayer state, notifications, signed-pack persistence and consumer release review have separate contracts. Existing astronomy/Asr/methodology discrepancies remain open; a native shell resolves none of them.

The physical gate must record model/OS, installation/build identity, permission and precise-location setting, indoors/outdoors, fix wait, reported accuracy and timestamp, and whether the fix was cached/simulated where observable. Enable airplane mode and separately disable Wi-Fi and Bluetooth; then cold-open, calculate from manual coordinates, request one fix, cancel/fail/retry, reopen, and compare offline schedules with the Rust fixtures. Repeat across a satellite-equipped iPhone and a device without usable offline positioning. Do not turn failure into a guessed coordinate. No physical gate has passed solely from this research.

## 6. Primary sources

All sources retrieved directly from Apple on **2026-10-01**. Developer pages were read through their official DocC JSON counterparts under `https://developer.apple.com/tutorials/data/documentation/`; links below are the human-readable pages. These are research sources and never app runtime dependencies.

| ID | Source | Specific evidence |
| --- | --- | --- |
| A1 | [Core Location](https://developer.apple.com/documentation/corelocation) | Available device components; geographic position and compass heading are separate services. |
| A2 | [About privacy and Location Services](https://support.apple.com/en-us/102515), published 20 May 2026 | GPS hardware scope, several-minute satellite acquisition, obstructions, and Wi-Fi/cellular assistance. |
| A3 | [Configuring your app to use location services](https://developer.apple.com/documentation/corelocation/configuring-your-app-to-use-location-services) and [authorization](https://developer.apple.com/documentation/corelocation/requesting-authorization-to-use-location-services) | Availability, foreground permission, minimum hardware requirements, power-efficient hardware choice, security of stored location. |
| A4 | [`requestLocation()`](https://developer.apple.com/documentation/corelocation/cllocationmanager/requestlocation()) | One fix, failure, less-accurate delivery, stop and cancellation behavior. |
| A5 | [`accuracyAuthorization`](https://developer.apple.com/documentation/corelocation/cllocationmanager/accuracyauthorization) | Reduced permission overrides a more precise desired accuracy. |
| A6 | [`desiredAccuracy`](https://developer.apple.com/documentation/corelocation/cllocationmanager/desiredaccuracy) | Best effort; greater precision can take longer and use more power. |
| A7 | [`horizontalAccuracy`](https://developer.apple.com/documentation/corelocation/cllocation/horizontalaccuracy), [`timestamp`](https://developer.apple.com/documentation/corelocation/cllocation/timestamp), [`CLLocationSourceInformation`](https://developer.apple.com/documentation/corelocation/cllocationsourceinformation), [simulation flag](https://developer.apple.com/documentation/corelocation/cllocationsourceinformation/issimulatedbysoftware), [accessory flag](https://developer.apple.com/documentation/corelocation/cllocationsourceinformation/isproducedbyaccessory) | Uncertainty and invalid negative accuracy; determination time; limited source flags. |
| A8 | [Creating an XCFramework](https://developer.apple.com/documentation/xcode/creating-a-multi-platform-binary-framework-bundle) | Static-library/header packaging; distinct device and Simulator binaries. |
| A9 | [`NSLocationWhenInUseUsageDescription`](https://developer.apple.com/documentation/bundleresources/information-property-list/nslocationwheninuseusagedescription) | Required foreground location-purpose message. |
| A10 | [Privacy manifest files](https://developer.apple.com/documentation/bundleresources/privacy-manifest-files) and [adding a manifest](https://developer.apple.com/documentation/bundleresources/adding-a-privacy-manifest-to-your-app-or-third-party-sdk) | Resource placement and static-library resource boundary. |
| A11 | [Required reason API policy](https://developer.apple.com/documentation/bundleresources/describing-use-of-required-reason-api) and [actual categories/reasons](https://developer.apple.com/documentation/bundleresources/app-privacy-configuration/nsprivacyaccessedapitypes/nsprivacyaccessedapitype) | Declare reasons matching actual calls; conditional reason codes above. |
| A12 | [`isExcludedFromBackup`](https://developer.apple.com/documentation/foundation/urlresourcevalues/isexcludedfrombackup) | Backup exclusion for nonessential support/cache files. |
