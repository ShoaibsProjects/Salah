# Offline setup and browser package contract v0.1

**Pieces:** Phase 4 → F4-C2: Offline Setup → Timezone Suggestion and Device Date; Phase 3 → F3-W2: Browser Reliability → Saved App Package.

**Status:** implemented research adapters, 1 October 2026. Static/native/WASM build evidence is recorded below. Browser/device acceptance and all existing phase gates remain open.

Read the [offline product contract](../docs/offline-product-contract.md) for the operating promise, hardware limits, and direction for future agents. This specification adds interfaces to the [original bridge](wasm-bridge-contract-v0.1.md); its v1 calculation request and shared schedule remain available.

## 1. Shared boundaries

Rust owns coordinate validation, embedded-map lookup, zone selection, UTC-to-local conversion, and prayer calculation. The browser reads optional device inputs and renders returned labels. Neither Rust nor this page calls a prayer, clock, geocoding, or IP-location service.

The browser's optional Geolocation provider is outside this control: it does not expose a portable GNSS-only/no-network switch. Record `unknown_browser_provider`, reported accuracy, and fix timestamp; do not call it certified offline positioning. A compass supplies direction, not coordinates. Native GNSS adapters, saved places, and an offline place directory remain separate work.

Each JSON operation rejects input over 8192 UTF-8 bytes before parsing, unknown/duplicate/missing fields, wrong types, trailing data, and unsupported schema. The wasm-bindgen string copy has already occurred; this is a parsing bound, not a preallocation cap. Typed errors have no fabricated result. Existing kernel formulas, methods, unavailable statuses, and historical reference data are unchanged.

## 2. Coordinate-to-zone lookup

`lookup_timezone_json(&str) -> String` accepts:

```json
{
  "schema": "salah-zone-lookup-request-v1",
  "latitude_degrees": 44.9778,
  "longitude_degrees": -93.2650
}
```

It returns schema `salah-zone-lookup-response-v1`, `status: ok`, and `candidates` containing:

- `scope: research_preview`, the exact checked coordinates, `zone_ids`, and `cardinality` (`one_suggestion`, `multiple_suggestions`, or `no_coverage`).
- `requires_confirmation: true` and `accuracy_footprint_checked: false`.
- `boundary_version`, `boundary_sha256`, and `tzdb_version` from the existing bundled lookup.

Errors use the same envelope shape with `status: error` and `error {code, message}`. Codes are `request_too_large`, `invalid_request`, `unsupported_schema`, `invalid_coordinates`, and `zone_lookup_failed`. Messages are diagnostic, not stable identifiers.

The map is approximate community geometry and checks a point, not the receiver's entire error region. A unique result may fill the zone field; the person must still confirm it. Multiple/no-coverage results never select an arbitrary zone. Manual named-zone choice remains available. Civil timezone law is data, not a prediction from longitude or solar position.

## 3. Selected-zone clock reading

`local_clock_json(&str) -> String` accepts:

```json
{
  "schema": "salah-local-clock-request-v1",
  "zone_id": "America/Chicago",
  "utc_unix_seconds": 1790856000
}
```

The timestamp is an explicit input, not a claimed current or independently verified instant. Rust creates a bundled `RuntimeZone` and converts that instant. A successful schema `salah-local-clock-response-v1` contains `clock`:

- `source: caller_clock_unverified`, echoed `utc_unix_seconds` and selected `zone_id`.
- `local_date` (`YYYY-MM-DD`), `local_time` (`HH:MM:SS`), and `offset_seconds_east`.
- `rule_pack {tzdb_version, sha256, inventory_sha256}`.

Errors use `request_too_large`, `invalid_request`, `unsupported_schema`, `unsupported_zone`, or `invalid_clock_instant`. Calendar conversion retains the existing 1900–2100 supported local-date range. No host timezone or host clock is read by Rust.

In the browser, `Math.floor(Date.now() / 1000)` supplies the epoch instant. JavaScript performs no local-date/timezone conversion. The selected location's date can differ from the device's displayed date, including at the date line. A plausible but incorrect device wall clock cannot be proved correct without an independent time source; display its source and allow manual correction. Gregorian date conversion makes no Hijri or moon-sighting claim.

## 4. Provenance-preserving calculation

`calculate_schedule_with_selection_json(&str) -> String` accepts all v1 schedule fields with schema `salah-schedule-request-v2` and one additional field:

```text
zone_choice: "manual" | "confirmed_suggestion"
```

`manual` uses existing direct manual selection, without a polygon lookup. `confirmed_suggestion` re-runs the bundled lookup in Rust at the exact submitted coordinates and confirms the submitted zone against that result. The bridge never accepts a client-provided candidate array as evidence. Additional errors are `zone_lookup_failed` and `invalid_zone_confirmation`.

The response remains `salah-schedule-response-v1` containing the existing `salah-local-schedule-v1` document. Its zone-selection record identifies the actual choice. The original `calculate_schedule_json` remains manual-only with the original strict v1 shape. No method or Asr preference is inferred from country, timezone, sensors, or clock.

## 5. Browser setup state

- Location permission is requested only after an explicit tap. Existing bounded one-shot acquisition/retry remains; no continuous location watch is introduced.
- A valid coordinate edit starts one local map operation after a 400 ms idle period. It clears the previous zone confirmation and automatic date. Editing the zone cancels acceptance of a pending map suggestion. Engine readiness does not overwrite an explicit manual/example selection.
- One candidate is filled automatically; multiple/no coverage requires a supported choice. Confirmation is always visible. Checking it requests the selected-zone date when device-today mode is enabled.
- Device-today mode starts enabled; the date is read-only in this mode. Turning it off enables a manual date. The labeled research example explicitly selects its own date/zone.
- Each submission obtains a fresh clock instant in device-today mode. Returning to the visible page refreshes the preview. There is no background date polling. A date change invalidates the old schedule; next-prayer state and clock-jump monitoring are future work.
- Revision/request IDs reject stale clock/map/calculation responses. Editing coordinates cancels acceptance of any pending location reply, including when the prior source was already manual. A later sensor reply cannot overwrite that manual choice.
- Setup requests have a 15-second application timeout; the existing 30-second worker health timeout remains. These limits are not scientific tolerances. Failure requests correction and never invents a schedule.
- Export schema `salah-web-schedule-export-v2` retains the exact engine `schedule` and `location_input`, adding a submission snapshot under `setup_input`: location acquisition, candidate lookup (or null), and device/manual/example date source. These records are local to the explicit download; public asset caching never stores them.

## 6. Offline browser package

`tools/build_web.py` generates `pkg/offline-manifest.json` and `offline-worker.js`. The manifest schema is `salah-offline-package-v1`; each sorted entry has `path`, `sha256`, and `bytes`. The build ID is SHA-256 of the UTF-8 compact JSON serialization of the ordered entry array, with those key orders. Paths are ASCII. The worker template itself is an entry; the generated worker is installed through the browser's service-worker mechanism, avoiding a recursive hash definition.

The public inventory includes HTML, styles, UI/worker/setup/offline scripts, WASM/bindings, build metadata, and third-party notices/license texts. It contains no coordinates, form state, religious preferences, or schedules. Generated files remain Git-ignored; CI's existing web artifact includes them.

Saving is an explicit action. Initial installation requires the files to be available from localhost or HTTPS, possibly served from a transferred local archive without internet. No remote endpoint/CDN is built into the page. File-URL loading is not supported.

Before installation succeeds, the worker checks its exact manifest identity, at most 512 distinct relative paths, declared total size at most 64 MiB, same-origin/no-redirect responses, and each asset's exact size/hash. A mismatch fails installation. These checks detect package mismatch; they do not authenticate a publisher or replace signed-rule-pack verification.

Caches are scoped by app path and build ID. Activation removes older caches of that app scope. A new complete package waits while a prior page is open; the explicit apply action activates it and reloads the page. Updating public assets does not change method/data rules inside an already running WASM instance.

After activation, same-origin GETs within the app scope use only exact saved assets (root navigation maps to `index.html`); an absent file returns an explicit 503 response. There is no network fallback for a partial saved app. The completeness indicator checks every inventoried asset exists locally; it does not recalculate all hashes on every startup. Explicit refresh can repair the same saved version when the exact source files are available.

Browser storage may be cleared or evicted. A missing app cannot reopen itself without a transferred/restored copy. Native packaging remains the stronger planned distribution path for strict installed offline use. Browser offline restart, eviction recovery, update lifecycle, and interaction/accessibility acceptance remain open.

## 7. Evidence and remaining limits

Local checks: formatting, warning-free all-target Clippy, native build, pinned offline WASM release build/binding generation, JavaScript syntax, and clean diff whitespace. No tests are added or run locally, following the user's instruction. Build success is not browser/device acceptance or independent accuracy evidence.

The newly reachable bundled boundary map increases the WASM size. The 1 October build observed 4943813 WASM bytes and 5676852 total inventoried bytes across 144 public files, before compression; the generated inventory records exact identity. These are build observations, not fixed size contracts. The earlier approximately 726 KB bridge observation is historical. Performance on low-memory/embedded targets remains unmeasured.

Scientific/methodology/civil-time gates, boundary-error footprint handling, high-latitude policy, native GNSS acquisition, saved places, mobile bindings, Qibla, reminders, signed browser update activation, production licensing/stewardship, and broader device evidence remain open. Automatic setup does not pass any of those gates.
