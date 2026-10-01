# Salah: the offline product contract

**Accepted direction:** 1 October 2026, from the maintainer's instruction that essential use must work offline, fill device inputs when available, and ask for manual input when automation cannot be trusted.

This is the canonical operating vision for every future agent and builder. Read it with the [architecture](vision-and-architecture.md), [roadmap](roadmap.md), and [current work](current-work.md). The roadmap still owns release gates; this document does not certify the research engine or mark those gates complete.

Salah should be a calm, free companion that can calculate prayer times on the device, explain its choices, and remain useful when internet access disappears. Its center is the same Rust engine on every platform. Its essential data travels with the installation. The person should not have to type coordinates, timezone names, and today's date repeatedly when a usable local input is already available.

## 1. The promise

- Essential calculation needs no account, subscription, remote prayer service, cloud clock, online geocoder, IP-location service, or paid API.
- Installed astronomy, method definitions, timezone rules, and boundary maps are sufficient for the supported calculation scope. Each result records their identities.
- Device readings are inputs to checked adapters. The engine validates and combines them; it cannot create a sensor reading when no receiver or trustworthy input exists.
- Automatic filling reduces typing. It exposes source, uncertainty, and correction controls instead of presenting a guess as a fact.
- Manual entry, previously saved places, and a future sourced offline place directory are complete alternatives to live positioning. Core use remains available when permission is declined or the device lacks location hardware.
- No method or Asr preference is inferred from a sensor, country, name, or nationality. The person chooses a practice once and can change it.
- Daily essentials are free to use. Maintenance, data stewardship, licensing, platform compatibility, and future civil-time changes still require people and updates.

An app can be shipped and installed from local files without internet. A browser needs the complete files transferred or cached before it can reopen offline. A never-installed website cannot produce a missing engine or data pack. Native distribution, locally served files, and a saved browser package are separate delivery mechanisms for the same engine.

## 2. What hardware can supply

| Input | Local source | What is and is not established |
| --- | --- | --- |
| Latitude/longitude | GNSS receiver, a documented native location provider, explicitly saved coordinates, or manual/offline place choice | A receiver can obtain a satellite fix without an internet prayer/location API. Hardware, permission, visibility of satellites, receiver startup, and platform access determine availability. A compass measures heading and supplies no latitude/longitude. |
| Current instant | Device wall clock/RTC; possibly independently recorded GNSS time in a future native adapter | Read an epoch timestamp without a clock-server request. A device clock can be wrong; its presence alone does not prove accuracy. GNSS time scales and validity flags require reviewed conversion before use as UTC. |
| Civil timezone | Bundled boundary map, then bundled IANA rules | A civil timezone is a jurisdiction/operator choice encoded in data. Longitude divided by 15, a UTC offset, and the device's displayed timezone cannot replace this mapping. |
| Local Gregorian date | Rust conversion of the current UTC instant under the selected zone's rules | “Today” belongs to the selected location. Device home-zone date and UTC date may differ from it, especially across the date line. |
| Qibla bearing | Offline mathematical bearing from accepted coordinates | Planned. The magnetometer/compass supplies an observed device heading for the optional orientation view. Calibration and magnetic-to-true-north handling require separate work. |

“Universal” means a shared input contract and engine, with adapters that report what the platform can provide. It does not mean all phones, tablets, laptops, browsers, watches, or embedded boards contain the same sensors.

## 3. Offline positioning policy

There are two different capabilities:

1. **Offline calculation available:** the installed engine and data can calculate from accepted coordinates and time inputs with no network service.
2. **Offline live positioning available:** an adapter has documented receiver/provider evidence that it can obtain a fix while network paths are unavailable.

The second must not be inferred from the first. The Web Geolocation API does not identify its physical source or expose a portable GNSS-only switch. `enableHighAccuracy` is a preference, not proof that GPS supplied the result or that no network was used. The browser feature therefore remains optional, with `unknown_browser_provider` provenance. Strict sensor-only operation cannot be promised for the current web adapter.

A future Android adapter will investigate the native GNSS provider rather than depend on Google Play location services. A future Apple adapter will audit Core Location's actual capabilities; browser/OS source uncertainty remains visible wherever the API cannot enforce the intended policy. Embedded devices may connect directly to a GNSS receiver. Neither native adapter is implemented by this document.

The native contract must record coordinates, units, horizontal accuracy estimate, reported fix time, age relative to the observation clock, provider/source evidence, and permission/capability status. Old fixes are explicitly saved/previous fixes, not silently relabeled as live. Device precision settings and location error remain visible. The app requests a bounded fix when needed and stops active acquisition afterward. Background travel tracking is a separate opt-in feature decision.

If positioning fails, explain the missing capability once and offer a saved place or manual/offline place selection. Repeatedly pressing a button cannot create a GPS receiver or independent coordinates. Do not silently substitute IP position, a nearby timezone's center, the last city visited, or the developer's sample location.

## 4. Timezone selection without a service

The standard pipeline is:

```text
accepted coordinates and location uncertainty
  → embedded coordinate-to-zone map
  → all candidate IANA identifiers, with map version
  → explicit resolution/confirmation where needed
  → selected IANA zone
  → pinned civil-time rules for each UTC instant
```

The current map is approximate community geometry. Its point lookup does not check the complete device-error footprint. It can fill one candidate automatically, but the research preview still requires a confirmation tap. This is a check of the selection, not a demand to type an IANA name from memory.

For multiple candidates, no coverage, sea/air travel, a disagreement with local practice, or a manual correction, offer supported choices and preserve their origin. The device timezone can be a future diagnostic hint; it is never automatically substituted for the location's timezone.

Future automatic acceptance needs a reviewed policy covering location uncertainty, border geometry, data applicability, and the person's saved confirmation. Sampling a few nearby points is not a proof that an entire uncertainty region belongs to one zone. A movement, changed coordinates, relevant new boundary pack, or conflicting evidence invalidates old confirmation.

Installed rules handle DST and unusual offsets locally. Future political decisions cannot be predicted from sensors or solar geometry. Packages remain usable offline, show their date/version, and support compatible authenticated updates imported from a file or through optional distribution. An internet connection must not become necessary for calculating with the installed snapshot.

## 5. Clock and date policy

The universal time input is an explicit UTC epoch instant, not the device's formatted local date string. In the web adapter, `Date.now()` is used only to obtain epoch milliseconds, converted to whole seconds; Rust performs every timezone/date conversion with the bundled snapshot.

After a timezone is confirmed, today's Gregorian date is filled automatically. Refresh the instant when calculating, when returning to the app, and later through a documented local-midnight scheduling policy. Manual date selection is always available for a past/future schedule or a clock error. A midnight change clears stale results before another calculation is shown.

The current device-clock reading is labeled unverified. If its date is outside the supported range or it cannot be converted, return an error and request a manual correction. Compare wall-clock progress with a monotonic clock and expose significant clock changes in future next-prayer/reminder work; a monotonic clock measures elapsed time and cannot establish a correct calendar date by itself.

Without an independent time source, offline software cannot prove that a plausible but mis-set wall clock is correct. The truthful outcome is automatic use with visible source and a correction path. Hijri calculations and observed religious dates are a separate future module; a Gregorian clock does not settle moon-sighting practice.

## 6. Everyday flow

1. Start from an explicitly saved place and settings, or ask the person to choose a location source. Request permission when they select a device fix; no unsolicited prompt on first visit.
2. Accept a usable reading or offer manual/offline place choice. Show the current location and its source.
3. Resolve timezone candidates locally. Fill an unambiguous suggestion; ask for a confirmation/correction under the current research policy.
4. Fill today from the device instant converted in the selected zone. Show the source and allow another date.
5. Use the person's explicitly selected prayer method and Asr criterion. Save preferences only through a clear local-storage feature, with deletion/correction controls.
6. Compute UTC events in Rust, localize using the same rule snapshot, and expose unavailable conditions and separately reviewed alternatives.
7. Show a calm daily schedule with explanations. Add current/next prayer, Qibla, and reminders only with their own specified rules and platform evidence.

Routine use should then require few taps. Confirmation is repeated when relevant evidence changes, not merely to make a person re-enter the same data every day. Saved-place persistence and its invalidation policy are future work; the present preview keeps setup in page memory.

## 7. System boundaries

```text
hardware / RTC / GNSS / explicitly saved place
        ↓ thin platform adapter, permission and source record
checked setup + offline timezone choice + local-date derivation
        ↓
salah-engine
  ├─ salah-core       astronomy, method rules, UTC events
  ├─ salah-location   embedded candidate map and explicit resolution
  └─ salah-time       pinned IANA rules, DST, local dates
        ↓
versioned result + input/data provenance
        ↓ thin web / native / embedded presentation
```

`salah-update` verifies optional packages; storage/recovery stays in `salah-update-store` and platform adapters. Updates do not secretly change a displayed schedule. App shell, translations, methods, place directory, and civil-time data are packaged locally. Existing library/build dependencies are pinned and inventoried; “no online dependencies” is a runtime service policy, not a claim that operating systems and all compiled libraries can be removed.

## 8. Reliability, size, and privacy

- Perform coordinate lookup on accepted changes, not every animation frame. Reuse immutable parsed rules and data where useful. One-shot positioning and on-demand work keep idle usage low.
- Load the boundary data only when needed. Its current 4.1 MB source artifact increases the browser module when lookup is exposed; report actual packaged sizes. Do not claim that global boundary data can be both absent and fully usable offline.
- Bundle all essential app files. Browser offline preparation checks an exact build inventory and file hashes before declaring its copy ready. Cache state is checked locally; browser eviction or user-cleared storage still requires reinstalling files. Native installations provide a stronger packaging boundary, subject to their OS.
- Preserve permission denial, no fix, stale coordinates, unsupported clocks/zones, date-line changes, missing solar events, and data/update failure as explicit states. Never display a fabricated successful time.
- Async replies are accepted only for the setup that requested them. Manual edits cancel acceptance of pending device fixes and prior computations.
- Keep coordinates, practice settings, and calculated schedules on device. Cache only public app/data files. Explicit exports contain the person's chosen calculation data; no location history is collected by default.

## 9. What exists and what remains

| Capability | Present state |
| --- | --- |
| Rust prayer/solar events, civil-time conversion, bounded UTC/status output | Implemented research code; independent accuracy/methodology gates remain open. |
| Embedded timezone rules and approximate boundary lookup | Implemented. Broader independent civil/boundary validation remains open. |
| Web automatic timezone suggestion and selected-zone device-date conversion | Implemented in F4-C2; one timezone confirmation remains under the current map policy. |
| Hash-checked browser package and cached startup | Implemented in F3-W2; browser offline restart, update, eviction, and accessibility acceptance remain open. |
| Browser device fix | Optional platform provider with unknown source/network behavior; not certified offline sensor acquisition. |
| Android/Apple native GNSS/location adapters and embedded receiver integration | Planned capability audits and build slices. |
| Saved places/settings and sourced offline place directory | Planned. Manual coordinates already work without a lookup service. |
| Qibla, reviewed high-latitude alternatives, next-prayer state, notifications, broader method registry | Still require implementation/source review and their gates. |
| Independent science, Islamic methodology, privacy/security, licenses, production keys and succession | Open release/stewardship work. |

No whole-product accuracy or universal sensor claim follows from automatic filling or a successful build. The first supported release must define its devices, input sources, model assumptions, geography, dates, and evidence.

## 10. Direction for future agents

Follow this contract and one bounded packet from the roadmap. Preserve the kernel and its known discrepancy ledger. Keep client logic about acquisition/display only; timezone and date derivation use the shared Rust rules. Implement hardware capability evidence before promising strict offline live location. Preserve user confirmations and reproducibility when convenience is added. Expand source-backed data and device acceptance before broadening product claims.

The next steps after this slice are real browser offline/permission/clock acceptance, a native GNSS capability probe, and saved-place/offline directory design. Scientific and methodology review continue in parallel. The long-lived asset is a reviewable engine and maintained, archived data, with a gentle interface around them.

## Technical sources

Consulted 1 October 2026 for this operating contract; no source is a runtime service dependency:

- [W3C Geolocation](https://www.w3.org/TR/geolocation/): source-agnostic provider, permission, timestamp/accuracy, and high-accuracy preference. It provides no portable offline/GNSS-only guarantee.
- [Android LocationManager](https://developer.android.com/reference/android/location/LocationManager): GNSS, network, and fused providers; the native adapter still needs device/build evidence.
- [ECMAScript Date specification](https://tc39.es/ecma262/multipage/numbers-and-dates.html#sec-date.now): epoch timestamp from the host clock.
- [IANA timezone database](https://www.iana.org/time-zones): political rule changes and periodic data updates.
- [Existing boundary/data provenance](../specification/coordinate-zone-selection-contract-v0.1.md): pinned local artifact, approximate geometry, license, confirmation rule, and current footprint limitation.
