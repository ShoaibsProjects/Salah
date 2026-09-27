# Salah 2050: product vision and architecture

**Status:** proposal for review  
**Date:** 27 September 2026  
**Horizon:** useful through 2050 with active maintenance  
**Initial domain:** prayer times on Earth

**Execution:** the active [delivery roadmap](roadmap.md) governs phases and release gates

## 1. Executive summary

Salah 2050 should make daily prayer times understandable, available offline, and respectful of established differences in calculation and practice. The first product serves Muslims on iOS, Android, and the web. It shows Fajr, sunrise, Dhuhr, Asr, Maghrib, and Isha; the next prayer; a Qibla bearing; and the settings that produced each time.

The lasting asset is a **documented Rust calculation core**, a versioned calculation specification, reference data, and independent validation. The app is a simple interface to that core. A backend can help distribute updates or mosque schedules, but daily calculation must work locally. The system must never require a commercial prayer-time API, account, subscription, or paid cloud service to perform its core function.

“Works until 2050 for free” is a product and maintenance goal: core calculation remains free to users and free of mandatory external runtime services. It is not a promise that an unchanged 2026 binary, future civil-time rules, app-store distribution, hosting, or maintenance will cost nothing or remain valid without updates. The project needs named maintainers, a funding plan, data updates, and a migration path across platforms.

The product's distinctive feature is **“Why this time?”** A person can see the location, selected method, relevant solar parameter, time-zone interpretation, adjustments, and any fallback behind a displayed time. That information should be understandable at a glance, with full technical details available when wanted.

## 2. Product promise and boundaries

### People and situations

- A person at home wants today's times with little setup and reliable local reminders.
- A traveler wants times for the current or manually selected location, including when offline.
- Someone comparing a mosque timetable with a calculated time wants a clear explanation of the difference.
- An older user wants large, legible type, simple controls, and predictable notifications.
- A mosque, researcher, or developer may later use the documented engine to create a timetable or another interface.

### Non-negotiable principles

1. **Local calculation.** Given coordinates, date, and a defined method, the core can calculate without internet access.
2. **Method transparency.** Show the selected convention and adjustments. Do not claim that one configuration resolves all fiqh differences.
3. **Explicit uncertainty.** If an astronomical event does not occur, time-zone data is stale, or an adjustment was used, show that status.
4. **Privacy.** Location stays on the device for core functions. No account, tracking, or behavioral advertising is needed.
5. **Portability.** Keep calculation logic independent of Flutter, Apple, Google, and web presentation code.
6. **Reproducibility.** Record the calculation inputs and exact versions of the algorithm and data used.
7. **Accessibility and restraint.** Fast, calm screens; no engagement loops or aggressive ads.

The first release supports Earth. Watch, vehicle, embedded, and mosque-display clients are future uses of the same engine. Space or Mars prayer-time rules are outside the first version; their jurisprudential basis would require separate work. Avoid building a generic planetary framework before Earth calculations are proven.

## 3. What the first product does

### Launch scope

| Capability | Launch behavior |
| --- | --- |
| Daily times | Fajr, sunrise, Dhuhr, Asr, Maghrib, and Isha for the selected local date. Distinguish sunset from Maghrib internally, even when the selected method makes their displayed times equal. |
| Location | Permission-based device coordinates, manual coordinates, and an offline city search where the bundled data supports it. Always show which location is in use. |
| Time zone | Resolve the location's civil-time zone locally when possible; show the zone and warn if device settings conflict or data may be outdated. |
| Methods | Documented method profiles for major regional conventions, plus standard and Hanafi Asr settings where applicable. Preserve each profile's actual parameters and source. |
| High latitudes | Detect unavailable twilight events and apply only a visibly selected, documented rule. |
| Notifications | Optional local reminders generated from local results, with clear permission and scheduling status. |
| Qibla | Offline great-circle bearing to the Kaaba. The compass view is clearly distinguished from the calculated bearing and gives calibration guidance. |
| Today screen | Current/next prayer highlighted, countdown, daily list, location, and visible method name. |
| Explanation | “Why this time?” for each result; short human explanation plus detailed calculation record. |
| Platforms | iOS and Android applications, plus a responsive web application. The browser version may have different notification and background limits. |
| Privacy | No account, location upload, advertising identifier, or mandatory backend. |

The initial interface should remain usable when location permission is declined. It should ask for notification permission only when the user enables reminders.

### Later capabilities

Monthly and annual timetables; local PDF, CSV, JSON, and calendar export; Ramadan views; a Hijri calendar with clear regional/date uncertainty; mosque timetable and iqamah data; Jumu'ah schedules; widgets and watches; English, Arabic, Urdu, Turkish, and Malay; screen-reader and right-to-left refinement; selectable adhan audio; educational Quran or hadith reminders with verified sourcing; travel features; and a developer-facing calculation lab. The lab can later become an interactive prayer-time simulator that shows the Sun's path and explains how results change by date, location, and method. Each is a separate release decision, not a reason to delay a trustworthy daily schedule.

## 4. Islamic methodology and governance

Prayer-time calculations combine astronomy with selected interpretive conventions. The software should document the distinction among:

1. **Astronomical quantities:** solar position, transit, altitude, sunrise, and sunset under stated physical assumptions.
2. **Calculation conventions:** Fajr and Isha twilight angles or intervals, rounding, offsets, and high-latitude rules.
3. **Juristic choices:** Asr shadow criterion, and other method-specific interpretations.
4. **Local practice:** mosque prayer-start timetables and iqamah times.
5. **Personal settings:** selected method, display format, and user adjustments.

The method registry should include profiles such as Muslim World League, ISNA, Egyptian, Umm al-Qura/Makkah, Karachi, Tehran, and Jafari where their parameters and usage can be documented. A method's name alone is insufficient: definitions can vary by edition, implementation, region, and date. Each profile needs a source, parameter values, version, geographic/community notes, and review date. Some profiles use a fixed interval for Isha rather than a twilight angle, including special Ramadan behavior; encode that as data and document its source. Do not assume that all listed profiles share the same Maghrib, midnight, or Asr rules.

The system must support standard and Hanafi Asr configurations, twilight-angle differences for Fajr and Isha, and documented high-latitude strategies such as middle of night, one-seventh, and angle-based adjustment. These are selectable methods, not interchangeable claims of universal religious authority. A scholarly advisory process should review user-facing descriptions and defaults for the communities served. Qualified scholars and local authorities should be consulted where practice is disputed or the normal astronomical event is absent. The application must not issue a fatwa or generate one with AI.

Keep three displayed concepts separate: **calculated prayer beginning**, **mosque timetable**, and **iqamah**. A mosque may deliberately set a congregation time after the prayer begins. If both sources are available, label and show them separately, with a plain-language explanation of any difference. Users who follow a trusted local mosque should be able to use its timetable, particularly where local practice or high-latitude conditions require guidance.

The Hijri date needs its own provenance. A calculated calendar date, a published civil calendar, and an observed local moon-sighting decision can differ. Do not silently use a calculated date as a universal religious determination.

## 5. System architecture

```text
Versioned specification + method registry + reference vectors
                         |
                  Rust Salah Core
     astronomy | prayer rules | high latitudes | Qibla
     time interpretation | calendar interfaces | status
                         |
                stable typed API / FFI / WASM
              /              |              \
       iOS/Android           Web         future clients
              \              |              /
       platform location, storage, notifications, UI
                         |
           optional, signed data and app updates
```

**Core:** Rust is the project decision. It provides one computational implementation that can be compiled for mobile and WebAssembly. Use explicit domain types, stable public interfaces, minimal unsafe code, documented numerical assumptions, and a small audited dependency set. Rust does not make a formula correct by itself; correctness comes from specification, reference comparisons, review, and maintenance.

**Mobile UI:** Evaluate Flutter as the initial shared iOS/Android interface. Its build and binding complexity with Rust must be prototyped early. If the result is cumbersome or inaccessible, native SwiftUI and Jetpack Compose clients remain viable because the core API is independent. Platform code owns permissions, location acquisition, local notifications, and accessibility integration.

**Web UI:** A responsive web client can call the Rust core through WebAssembly. A PWA can cache the application and data for offline use after installation or first load. Browser location permissions, background execution, and notification behavior differ by browser and device; communicate those limits in the web product.

**Backend:** None is required for calculating times. Optional hosting can serve app releases, signed data packs, method documentation, and eventually mosque schedules. Normal operation must continue if that hosting disappears.

**Dependency policy:** “No dependencies” means no mandatory external runtime services. The app still depends on operating systems, build tools, libraries, and device facilities. Pin and inventory build dependencies; prefer small maintained components; vendor or mirror critical sources when licenses permit; document how another organization can rebuild and maintain the core. Use conventional hashes and signatures for update integrity. Blockchain adds no useful capability here.

## 6. Calculation contract

Publish a **Salah Calculation Specification** before treating implementation results as authoritative. It must define input validation, coordinate conventions, units, time scales, astronomical assumptions, prayer-method semantics, numerical precision, rounding, and result status. The core API should return instants in UTC and the relevant local representation; the UI must not calculate prayer mathematics.

### Minimum input model

- Geographic latitude and longitude, and optional elevation with provenance.
- Civil date and an IANA time-zone identifier or an explicit manual offset when no zone can be determined.
- Astronomy model version, method profile and version, Asr choice, high-latitude rule, and all adjustments.
- The exact time-zone data version used for local conversion.

### Minimum output model

- Fajr, sunrise, Dhuhr, Asr, sunset, Maghrib, and Isha as UTC instants and local display times where defined. Optional derived outputs such as Islamic midnight and the last third of night require separately specified definitions and method provenance.
- Per-event status: direct astronomical event, rule-based adjustment, manually supplied timetable, or unavailable.
- Intermediate solar quantities needed for explanation and validation.
- A calculation record with all inputs, versions, offsets, and applied rules.

Results should be reproducible from archived code **and** archived method/time-zone data. A version label alone is insufficient if the corresponding data later disappears. Preserve immutable release artifacts and reference vectors. Historical reproduction and the best current estimate for a future date are different tasks: future civil-time law can change after a schedule is generated.

### Astronomy and precision

Define a documented solar model for declination, equation of time, transit, apparent solar altitude, sunrise/sunset, and twilight. State how refraction, the solar disk, observer elevation, pressure, and temperature are handled or deliberately omitted. Compute Asr from the selected shadow relationship, with its assumptions explicit. Do not invent a time when a required event has no solution. Store precision internally, round only for presentation according to a specified rule, and apply user/mosque offsets as visible transformations after the base result.

The Qibla module computes a great-circle initial bearing from coordinates to the Kaaba using a documented reference coordinate. The phone's compass supplies an observed device heading, which can be inaccurate due to magnetic interference or calibration. Display the bearing separately from sensor guidance.

## 7. Time zones, location, and travel

Coordinates determine solar events; they do **not** by themselves provide an unchanging civil-time zone. The application needs a local mapping from coordinates to time-zone identifiers, a versioned IANA time-zone database, and a clear manual override. Borders, ships, aircraft, and politically changed rules can make mapping ambiguous. Show the chosen zone and how it was obtained.

Compute the event as a UTC instant first, then convert with the chosen zone's rules for that instant. Handle daylight-saving transitions, unusual offsets, the date line, and local dates with 23 or 25 hours. A mismatch between device time zone and location time zone should produce a visible warning, not a silent change to the calculation. Manual city selection must use that city's coordinates and zone, not the user's device zone. GPS may be unavailable indoors or without permission; manual coordinates and saved places remain available.

Bundle a usable time-zone data snapshot and an offline location-to-zone mapping. Allow compatible data packs to be updated independently of the Rust engine when platforms permit, with signatures, version metadata, rollback, and compatibility rules. App-store platform policies may still require a full app release for some update mechanisms. If rules change while a device stays offline, local civil-time labels can become wrong; communicate the last update date and allow a manual correction.

For ships and aircraft, location and civil-time conventions can change rapidly. A later travel mode can recalculate as location changes and make the chosen time reference explicit. Do not silently present a fixed city schedule as the current local schedule in transit.

## 8. High latitudes and unavailable events

At high latitudes, a selected Fajr or Isha twilight angle may not occur on a given date. In polar regions, sunrise or sunset can also be absent. The engine must report that condition explicitly and preserve the raw astronomical finding. A selected, documented fallback can then produce a separate adjusted time. Show which rule was used and why.

Support established approaches only after documenting their definitions and reviewing their user-facing descriptions. Do not silently apply one worldwide default or imply that a mathematically generated fallback settles the relevant fiqh question. Local mosque or qualified scholarly guidance may be especially important in these locations.

## 9. User experience

The home screen should show the place, date, current or next prayer, a legible countdown, and the six daily items. Settings and details are one tap away. Use calm typography, generous spacing, high contrast, large text, screen-reader labels, right-to-left layout, and careful Arabic typography. Avoid streaks, feeds, promotional banners, excessive animation, and notification pressure. The design can borrow clear information hierarchy and progressive disclosure from strong learning and productivity interfaces without copying another brand.

“Why this time?” should have two levels:

- **Simple:** “Fajr uses your selected method's twilight parameter at this location. A high-latitude adjustment was applied.”
- **Detailed:** coordinates, location source and accuracy, UTC instant, local time zone and data version, method and parameter values, astronomical model, adjustment chain, engine version, and result status.

Do not show a made-up percentage of “religious accuracy.” Use concrete statuses such as location approximate, zone manually selected, twilight unavailable, adjustment applied, or mosque timetable selected. Method comparison can be an educational view, but it must label each result and avoid presenting a winner.

## 10. Privacy, ethics, and cost

Core calculations occur on the device. Request location only when useful; offer manual entry first. Ask for notifications only when the user opts in. Store settings and saved places locally. Make any future synchronization an explicit opt-in with a separate privacy explanation. Avoid collecting location history or religious behavior data. Do not monetize prayer-time access through ads, paywalls, or dark patterns.

The core product can be free because each calculation has no per-request API cost. Keeping it available through 2050 still requires labor, security fixes, compatibility work, time-zone and method updates, device testing, signing, store fees, and web hosting. A sustainable model could use donations, grants, or optional institutional services, provided the core remains free and independently operable. Funding and ownership need an explicit governance decision before a “free forever” public pledge.

## 11. Validation and release criteria

The first implementation should be validated against independent astronomical references and established prayer-time implementations such as Adhan and PrayTimes, while documenting why differences occur. A comparison library is evidence, not an unquestioned oracle. Obtain review of method definitions from appropriate subject-matter experts.

Reference vectors should cover global latitudes and longitudes, seasonal extremes, leap years, date-line crossings, time-zone changes, daylight-saving transitions, elevation assumptions, high-latitude missing events, polar day/night, method variants, Asr variants, and rounding. Reuse the same vectors for native and WebAssembly builds. Document acceptable numerical error separately from differences caused by method choice.

Before launch, a user must be able to: calculate offline from manual coordinates; see which location, zone, and method produced a time; recognize an adjusted or unavailable event; receive correctly scheduled local notifications under supported OS conditions; and understand that a future mosque timetable cannot replace the labeled base calculation. Review accessibility, privacy disclosures, and app-store rules. Notifications are best-effort OS facilities: permissions, focus modes, battery policy, scheduling limits, and device clock errors can delay or suppress them, so the app must not promise guaranteed delivery.

## 12. Build order and roadmap

The active, phase-gated plan is [Delivery roadmap](roadmap.md). Its seven phases begin with the existing research foundation, then validation, civil time and difficult geography, portable bindings, trustworthy experience, offline consumer beta, and public release with stewardship. Governance and 2050 reproducibility run through every phase. The earlier stage list was a planning sketch and is superseded by that roadmap.

After launch, add the monthly timetable and export, mosque and iqamah schedules, Hijri and Ramadan features, more languages, widgets, watch/vehicle clients, and a graphical calculation lab according to user need. Keep optional content and AI assistance separate from deterministic calculation. AI may explain a documented result or help with navigation; it must not generate the prayer time or issue religious rulings.

## 13. Principal risks and responses

| Risk | Response |
| --- | --- |
| Wrong time from formula, rounding, or implementation | Formal specification, independent reference comparisons, reviewed edge cases, and visible versioning. |
| Incorrect or outdated civil time | Bundled and updateable time-zone data, zone display, freshness indicator, manual override, and explicit travel behavior. |
| Location error | Show location source/accuracy and allow manual correction; avoid silently switching places. |
| Scholarly disagreement | Document method provenance, allow selection, separate mosque practice, and obtain qualified review. |
| No twilight or sunrise/sunset event | Explicit unavailable status and documented, user-visible fallback. |
| Notification failure | Local scheduling, rescheduling on relevant changes, platform-specific checks, and honest delivery limits. |
| Maintenance cost or abandoned project | Open specification, archived data and vectors, portable core, simple dependencies, funded maintainership, and successor documentation. |
| App-store or browser changes | Stable core interface, native platform adapters, and a web distribution path. |
| Scope growth | Ship daily times and trust features first; defer content, social, AR, and experimental travel modes. |

## 14. Decisions still required

1. Software license, maintainers, and ownership policy for the existing public Salah repository.
2. Which communities and regions receive reviewed default method profiles at launch. No global default should be chosen merely because it is convenient for implementation.
3. Release precision targets, broader model validation, and presentation rounding policy; the research kernel already records its current astronomy model.
4. Time-zone lookup dataset, license, update channel, and archive policy.
5. Advisory reviewers and the process for resolving a documented method dispute.
6. Flutter-versus-native mobile prototype result and web offline support targets.
7. Funding and stewardship for updates through 2050.

## 15. Reference starting points

These are starting points for technical and methodology review, not automatic endorsements of every parameter or implementation decision:

- [Adhan project and shared implementations](https://github.com/batoulapps/Adhan)
- [PrayTimes methods](https://praytimes.org/docs/methods)
- [PrayTimes manual, including high-latitude approaches](https://praytimes.org/manual)
- [U.S. Naval Observatory: rise, set, and twilight calculations](https://aa.usno.navy.mil/faq/rs_algor)
- [IANA time-zone database](https://www.iana.org/time-zones)
- [Rust documentation](https://doc.rust-lang.org/)
- [Flutter platform support](https://docs.flutter.dev/platform-integration/)

The first Rust CLI, versioned contract, and selected reference cases now exist. The active next deliverable is the Phase 1 numerical accuracy budget and wider independent reference matrix described in the [validation plan](phase-1-validation.md).
