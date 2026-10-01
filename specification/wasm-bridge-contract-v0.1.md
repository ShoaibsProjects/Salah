# Browser WebAssembly bridge contract v0.1

**Piece:** Phase 3 → F3-W1: Portable Calculation Platform → WebAssembly Bridge and First Schedule Screen.

**Status:** research implementation. Native lint/build checks and a generated WebAssembly artifact exist. No phase gate is passed.

## Architecture

`apps/web` → module Web Worker → generated wasm-bindgen boundary → `salah-wasm` → existing `salah-engine`, `salah-location`, `salah-time`, and `salah-core`.

The browser sends explicit inputs and displays the [shared schedule document](named-zone-interface-contract-v0.1.md). Rust owns input validation, solar calculations, local-date selection, timezone conversion, event rules, and unavailable statuses. JavaScript contains no prayer mathematics and performs no `Date` conversion of the returned local labels. The bridge uses the bundled IANA 2026d snapshot and direct manual zone selection only. Signed-pack activation and storage remain outside this bridge.

## Exported interface

`calculate_schedule_json(request_json: &str) -> String` accepts this exact request shape:

```json
{
  "schema": "salah-schedule-request-v1",
  "latitude_degrees": 44.9778,
  "longitude_degrees": -93.2650,
  "local_date": "2026-09-30",
  "zone_id": "America/Chicago",
  "method_id": "mwl-angles-18-17",
  "asr": "hanafi"
}
```

These are example research choices, not a regional recommendation. Every field is required. The bridge rejects unknown/duplicate fields, trailing data, wrong JSON types, unsupported schema/profile/Asr/zone, invalid coordinates, and noncanonical or invalid dates. Dates are Gregorian 1900–2100. Method IDs and Asr values have the same explicit meanings as the named-zone CLI. There is no date, location, timezone, method, or Asr inference.

Requests over 8192 UTF-8 bytes are rejected before JSON parsing. This bounds parsing work; wasm-bindgen has already copied the JavaScript string into module memory, so it is not a preallocation memory cap. Default serde_json recursion limits remain enabled. This adapter accepts a small request, not arbitrary imported update payloads.

Successful envelope:

```text
{ "schema": "salah-schedule-response-v1", "status": "ok", "schedule": <salah-local-schedule-v1 document> }
```

Handled-error envelope:

```text
{ "schema": "salah-schedule-response-v1", "status": "error", "error": { "code": <code>, "message": <explanation> } }
```

Error codes are `request_too_large`, `invalid_request`, `unsupported_schema`, `invalid_coordinates`, `invalid_date`, `unsupported_method`, `unsupported_asr`, `unsupported_zone`, `calculation_failed`, and `encoding_failed`. Messages are diagnostic text, not stable machine identifiers. Error envelopes contain no schedule. A missing astronomical event remains a successful schedule containing an unavailable event. Traps/out-of-memory/platform failures are not caught and renamed as astronomical outcomes.

`zone_inventory_json() -> String` returns schema `salah-zone-inventory-v1`, scope `research_preview`, the bundled `zone_ids` array, and exact bundled `rule_pack` identity. These are possible manual choices, not coordinate-derived suggestions. The generated module exposes only these two application operations plus wasm-bindgen memory/initialization support.

## Browser behavior

- All calculation controls begin unselected; placeholders do not become inputs. The clearly labeled Minneapolis example fills an explicit configuration only when requested, and does not auto-calculate or select a regional default.
- Worker initialization loads the adjacent generated JavaScript/WASM files. All assets and fonts are local; no CDN, account, analytics, or remote prayer API is used.
- Device coordinates are requested only after a user taps “Fill coordinates from device location.” This calls `navigator.geolocation.getCurrentPosition` once with the energy-conscious `enableHighAccuracy: false` hint, `maximumAge: 30_000` and a 20-second timeout. A 25-second UI limit reports a stalled browser request. It never registers continuous `watchPosition`, runs in the background, or auto-prompts on page load. The page checks for a secure context and API support first. The browser requests permission and delegates the one-shot estimate to the device's available location provider. The operating system/provider may use GPS, Wi-Fi, cellular, a cached fix, or other sources; some sources may need connectivity. The page makes no location lookup and sends no coordinates to Salah, but it cannot control the browser/OS provider or guarantee an offline fix. Failure, denial, or missing hardware leaves manual entry available.
- Successful coordinates are copied into editable fields to seven decimal places. The provider-reported horizontal accuracy radius in metres is rounded upward for display and retained in page memory; this is an estimate, not a promise of exact position. The time-zone field remains a separate user choice. No device coordinates or accuracy are stored by this page between visits. Editing either coordinate changes its recorded source to manual and clears the old accuracy estimate. Pressing the explicit example button identifies its coordinates as an example.
- The Fajr/Isha profile selector explains that its angles describe the Sun's position below the horizon and that the published 18°/17° set is labeled MWL in a secondary table. It identifies `research-15` as an engineering comparison. Neither is set as a default or presented as a universally correct practice. Asr help explains that the Standard and Hanafi criteria use one and two extra object-heights of shadow beyond the noon shadow, then directs users to follow their mosque or trusted scholar; it issues no ruling.
- Calculation happens in a worker. Responses carry request IDs. Editing a choice clears the prior schedule and cancels acceptance of its in-flight response; older replies are ignored.
- Loading/calculation watchdogs stop the worker after 30 seconds with an explicit failure. Worker failure removes old results and generates no substitute time. This timeout is an application health limit, not an astronomical threshold.
- Each row uses the returned full local date and `HH:MM:SS`; opening it shows the returned rule, UTC, second-precision offset, and zone/data version. No display-minute or notification policy is applied. Sunrise/sunset remain distinct observations, including a separate Maghrib row.
- Skipped dates, zero cycles, multiple cycles, and missing events remain explicit. Every matching cycle is displayed; none is selected silently.
- User and engine strings are rendered through `textContent`; they are not interpolated into HTML. The page declares a same-origin CSP with `wasm-unsafe-eval` for module compilation, no object embedding or form submissions, and no referrer. A production host should supply appropriate headers as part of its separate deployment review.
- The download uses envelope schema `salah-web-schedule-export-v1`, containing the exact shared `salah-local-schedule-v1` document under `schedule` and a `location_input` object with source (`device_geolocation`, `manual_coordinates`, or `example_coordinates`) and `reported_accuracy_radius_meters` (or `null` when no device estimate applies). The envelope adds acquisition provenance; the nested calculation document remains exactly the engine's result. This user-requested download stays on device. No schedule request or coordinates are sent to the static server. The server sees normal asset requests.
- This is a development preview served over HTTP/localhost. Calculation is local after assets load. Durable offline installation, service-worker cache updates, restart persistence, notification scheduling, and production distribution are not implemented. File-URL module/worker loading is not the supported path.
- The welcome and project intentions are original copy. No new Qur'an translation, quotation, or religious ruling is introduced by this screen.

## Build and package

The new `salah-wasm` crate is an `rlib`/`cdylib`; it leaves the dependency-free prayer kernel unchanged. `serde` 1.0.229 and `serde_json` 1.0.151 already exist in the workspace. `wasm-bindgen` is pinned to 0.2.129 (MIT OR Apache-2.0; published MSRV 1.81). Seven new binding-support packages are checksummed in Cargo.lock; no existing third-party version is upgraded. The bridge declares Rust 1.88 to match orchestration dependencies.

The matching `wasm-bindgen-cli` 0.2.129 is a build-only tool, with its own locked dependencies outside the application lockfile. Installation fetches build tooling; daily calculation does not require that CLI or a registry connection.

```bash
rustup target add wasm32-unknown-unknown --toolchain 1.98.1
cargo fetch --locked
cargo install --locked wasm-bindgen-cli --version 0.2.129
python3 tools/build_web.py
python3 -m http.server 8080 --bind 127.0.0.1 --directory apps/web
```

The build helper requires Python 3.11+ and selects the channel from rust-toolchain.toml. It resolves the exact rustup compiler into `RUSTC` so a Homebrew compiler earlier in PATH cannot accidentally bypass the installed WebAssembly target. After initial tool/dependency installation, compilation uses `--locked --offline`.

Generated `apps/web/pkg/` is ignored by Git. Its manifest records binding/toolchain versions, Cargo.lock hash, sizes, and SHA-256 for the generated binding files. These are diagnostics and reproduction inputs, not a signed release certificate. The helper copies bundled data notices and license texts found at locked package roots, with a license inventory; see [web notices](../apps/web/THIRD-PARTY.md). Missing/package-specific notices and the undecided project source license still need release review.

CI retains the existing workspace job and adds a WebAssembly build job with a downloadable `salah-web-research-preview` artifact, retained for 14 days. No new test job or suite is added. The archive contains the static preview and generated package; it is a research artifact, not a public hosted app or approved consumer release.

## Recorded evidence and limits

- Pinned-compiler release build for `wasm32-unknown-unknown` and wasm-bindgen web generation succeeded locally. The observed generated WASM was 726400 bytes before compression; this is a build observation, not a fixed size contract.
- Native all-target Clippy with warnings denied, formatting, JavaScript syntax checks, and a locked native workspace build are the local static checks. No tests are added or run locally, following the user's instruction.
- The generated WASM was operated directly through its generated binding in Node.js for the explicitly shown Minneapolis configuration; it returned the shared schedule with the bundled identity. This single use is not a cross-platform equivalence matrix or independent accuracy evidence.
- The local HTTP server served the preview. The in-app browser connection timed out and the app reported a locked Mac; browser visual/interaction review remains pending. Build success and Node module execution do not establish browser, mobile, screen-reader, or offline-restart acceptance.
- Phase 1/2 scientific/methodology/civil-time gates and Phase 3 cross-target equivalence remain open. Core discrepancies, broader sources, reviewed high-latitude rules, Qibla, portable mobile bindings, and update stewardship retain their existing owners/limitations.
- Next client work: browser reliability/visual review, durable offline loading, and an explicitly specified next-prayer state before reminders. Mobile binding work must reuse the same engine and versioned inputs/results.
