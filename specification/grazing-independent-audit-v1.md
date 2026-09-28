# Independent near-grazing event-existence audit v1 (P1.2-G1)

**Status:** Phase 1 evidence gathering only, retrieved 2026-09-28 UTC. This audit changes no formula, threshold, tolerance, contract, method, or release claim. It investigates the documented 2026-06-21 near-grazing apparent-horizon event-existence disagreement at longitude 0° and latitude 65.735° N, with 65.72° and 65.74° as bracketing cases, using NASA/JPL Horizons as an independent Sun-center altitude source. Source manifest: [grazing-horizons-v1.tsv](../data/reference/grazing-horizons-v1.tsv). Phase 1 remains open; no consumer accuracy or religious claim follows.

## 1. Question and method

At 65.735° N on 2026-06-21, the pinned USNO row `usno-2026-65735-grazing-disagreement` reports brief rise/set events while Salah's fixed −0.833° apparent-horizon model reports no crossing. The question is which differences are ephemeris geometry, horizon/refraction definition, civil-cycle mapping, or still unknown.

The independent source is NASA/JPL Horizons observer ephemeris for Sun target 10 from a user-defined Earth geodetic site (`coord@399`), quantity 4 (airless apparent azimuth/elevation), `APPARENT=AIRLESS`, CSV output, extra precision, explicit `TIME_TYPE=UT`. The same fixed Salah solar-center threshold h0 = −0.833° is used **only** for a matched-threshold comparison: sampled minimum airless Sun-center elevation minus h0, classified as `dips_below` or `stays_above`. A crossing assertion additionally requires a sign bracket on each relevant side of the minimum; a sampled minimum alone is insufficient. Sensitivity is shown at h0 ± 0.005° without adopting either altered threshold. No exact tangent is declared from samples alone.

For each latitude (65.72°, 65.735°, 65.74°), both adjacent solar-midnight windows were sampled: 2026-06-20/21 (preceding cycle, control) and 2026-06-21/22 (selected cycle for Salah's 2026-06-21 local date, whose transit is near 12:01:49 UT). Each window was first located with a coarse 4-hour, 10-minute-step request, then narrowed to a 40-minute fine request around the minimum with unitless `STEP_SIZE=240` (2400 s / 240 = 10 s output steps). Unitless stepping that divides the requested interval is documented in the [Horizons API documentation](https://ssd-api.jpl.nasa.gov/doc/horizons.html) (`STEP_SIZE` section: "the interval between specified `START_TIME` and `STOP_TIME` is divided into that many evenly spaced output steps"; unitless stepping is "the only way to obtain output with less than 1 minute spacing"). The 10 s output cadence was verified in the returned time tags (e.g. 23:40:00, 23:40:10, …).

## 2. What Horizons actually says (from official documentation and response headers)

All semantics below are taken from the [Horizons API documentation](https://ssd-api.jpl.nasa.gov/doc/horizons.html) and the [Horizons manual](https://ssd.jpl.nasa.gov/horizons/manual.html), plus the headers embedded in the retrieved responses. Where a setting is unclear, it is marked unknown; nothing is inferred from a name.

- **Quantity 4** (manual, "Definition of Observer Table Quantities"): "Apparent azimuth and elevation of target. Adjusted for light-time, the gravitational deflection of light, stellar aberration, precession and nutation. There is an optional (approximate) adjustment for atmospheric refraction (Earth only). … Elevation angle is with respect to plane perpendicular to local zenith direction. TOPOCENTRIC ONLY. Units: DEGREES. Labels: `Azi____(a-app)___Elev` (airless)". The retrieved column footer confirms: "Airless apparent azimuth and elevation of target center. Compensated for light-time, the gravitational deflection of light, stellar aberration, precession and nutation."
- **Refraction:** requested `APPARENT=AIRLESS`; every response header confirms `Atmos refraction: NO (AIRLESS)`. Refraction-corrected output would be a different, explicitly requested setting and was not used.
- **Site:** requested `CENTER='coord@399'`, `COORD_TYPE='GEODETIC'`, `SITE_COORD='0,<lat>,0'` (E-lon deg, Lat deg, Alt km, height 0 km). Every header echoes e.g. `Center geodetic : 0.0, 65.735, 0.0 {E-lon(deg),Lat(deg),Alt(km)}`, `Center-site name: (user defined site below)`, `Center pole/equ : ITRF93`. The geodetic datum name beyond the shown `ITRF93` frame label is unknown.
- **Time scale:** requested `TIME_TYPE='UT'`; headers show `Start time : A.D. 2026-Jun-21 23:30:00.0000 UT`. The embedded column documentation states times after 1962 are in UTC (civil "wall-clock", within 0.9 s of UT1 via leap seconds) converted from internal TDB, and that for future UTC dates the last known leap-second is held constant. Salah's model uses UTC as a practical approximation to UT1 (see calculation contract v0.3); the two conventions are not asserted identical at sub-second level, which does not affect a ~0.002° altitude-margin classification.
- **Ephemeris and Earth orientation:** headers show target/center source `{source: DE441}` and `EOP file : eop.260925.p261222`, `EOP coverage : DATA-BASED 1962-JAN-20 TO 2026-SEP-25. PREDICTS-> 2026-DEC-21`. The 2026-06-21/22 windows fall inside the data-based EOP span.
- **API identity:** live JSON `signature` reports `{"version": "1.2", "source": "NASA/JPL Horizons API"}`; the API documentation page is headed "Version: 1.3 (2025 June)". Both are recorded; the mismatch is a documentation/version-reporting observation, not a data discrepancy.
- **Output precision:** `EXTRA_PREC=YES`, `CSV_FORMAT=YES`, `TIME_DIGITS=FRACSEC`, `CAL_FORMAT=BOTH` (calendar `HR:MN:SC.fff` plus `JDUT`). Elevations display 9 decimal places of a degree. Output decimal places are not physical accuracy; Horizons' physical uncertainty for this use is unknown and marked as such.
- **Horizons' own rise/set markers are a different rule.** The CSV stream carries solar-presence and `r/e/t/s` event markers whose footer states: "Rise and set are with respect to the reference ellipsoid true visual horizon … Horizon dip and yellow-light refraction (Earth only) are considered" (`RTS MARKERS (TVH)`). Those markers use a refracted true-visual-horizon rule, **not** the airless −0.833° center-altitude comparison derived here, and not USNO's minute-formatted Rise/Set labels. The derived `dips_below`/`stays_above` statuses in the TSV are auditor-computed from the airless elevation column and must not be confused with Horizons' own markers.

## 3. Per-latitude, per-window evidence (all 12 TSV rows cited)

h0 = −0.833° throughout. "Margin" = sampled minimum airless elevation minus h0. Positive margin means the sampled arc stays above h0.

| Latitude | Window | TSV rows | Sampled min elevation (UT) | Margin vs h0 | Derived status | ±0.005° sensitivity |
| --- | --- | --- | --- | --- | --- | --- |
| 65.72° | 2026-06-20/21 | `jpl-2026-6572-wA-coarse`, `jpl-2026-6572-wA-fine` | coarse −0.844275900 @00:00; fine −0.844872335 @00:01:40 06-21 | −0.01187° | dips below, bracketed both sides (23:40 −0.74804 above → 91/241 samples below → 00:20 −0.77604 above) | dips at −0.828 and −0.838 |
| 65.72° | 2026-06-21/22 (selected) | `jpl-2026-6572-wB-coarse`, `jpl-2026-6572-wB-fine` | coarse −0.845176601 @00:00; fine −0.845944429 @00:02:00 06-22 | −0.01294° | dips below, bracketed both sides (23:40 −0.74705 above → 95/241 below → 00:20 −0.77883 above) | dips at −0.828 and −0.838 |
| 65.735° | 2026-06-20/21 | `jpl-2026-65735-wA-coarse`, `jpl-2026-65735-wA-fine` | coarse −0.829276260 @00:00; fine −0.829872345 @00:01:40 06-21 | +0.00313° | stays above (all 241 fine samples above h0) | flips to dips at −0.828; stays above at −0.838 |
| 65.735° | 2026-06-21/22 (selected) | `jpl-2026-65735-wB-coarse`, `jpl-2026-65735-wB-fine` | coarse −0.830177057 @00:00; fine −0.830944439 @00:02:00 06-22 | +0.00206° | stays above (all 241 fine samples above h0; neighbours 00:01:50 −0.830943315 and 00:02:10 −0.830934140 both above) | flips to dips at −0.828; stays above at −0.838 |
| 65.74° | 2026-06-20/21 | `jpl-2026-6574-wA-coarse`, `jpl-2026-6574-wA-fine` | coarse −0.824276380 @00:00; fine −0.824872348 @00:01:40 06-21 | +0.00813° | stays above (all 241 above) | stays above at −0.828 and −0.838 |
| 65.74° | 2026-06-21/22 (selected) | `jpl-2026-6574-wB-coarse`, `jpl-2026-6574-wB-fine` | coarse −0.825177208 @00:00; fine −0.825944442 @00:02:00 06-22 | +0.00706° | stays above (all 241 above) | stays above at −0.828 and −0.838 |

Exploratory-lead check: a prior architect query suggested the 65.735° airless minimum near 2026-06-22 00:02 UT lies roughly 0.002° above −0.833°. The independent fine request `jpl-2026-65735-wB-fine` returns −0.830944439 at 2026-06-22 00:02:00 UT, margin +0.002055561°, confirming the lead's order of magnitude from a fresh fetch. The figure was not used as expected truth and no threshold was tuned to it.

## 4. USNO and Salah status mapping (selected 2026-06-21 solar cycle)

Pinned USNO rows (hashes re-verified 2026-09-28 UTC via `tools/verify_usno_matrix.py`: all three "match"):

- `usno-2026-6572-grazing` (hash `a01cc4e8…980d8b`): rise 2026-06-21T00:10:00Z, transit 12:02, set 23:53.
- `usno-2026-65735-grazing-disagreement` (hash `dda5f47c…70239f2`): rise 00:04, transit 12:02, set 23:59 (selected evening Set; the response's additional 00:00 Set belongs to the preceding cycle and is not the selected event).
- `usno-2026-6574-continuous` (hash `f2030605…3a20caac81`): rise `no_event`, transit 12:02, set `no_event`.

Unmodified Salah CLI (`research-15`, Standard Asr, UTC+00:00, local date 2026-06-21), run 2026-09-28:

- 65.72°: sunrise 2026-06-21 00:08:26Z occurs, sunset 23:54:51Z occurs (Dhuhr 12:01:49Z).
- 65.735°: sunrise unavailable, sunset unavailable (Dhuhr 12:01:49Z).
- 65.74°: sunrise unavailable, sunset unavailable (Dhuhr 12:01:49Z).

Comparison (event existence only; no time difference is computed where one side has no event):

| Latitude | USNO (selected cycle) | Salah | Horizons-derived at fixed h0 (selected window B) | Reading |
| --- | --- | --- | --- | --- |
| 65.72° | events | events | dips below (events under h0) | all three agree on existence |
| 65.735° | events (00:04/23:59) | no crossing | sampled arc stays above by +0.00206° | Horizons supports Salah's status under the matched threshold; USNO reports events |
| 65.74° | no events | no crossing | stays above by +0.00706° | all three agree on absence |

## 5. What is established vs unknown

Established:

- Under the matched, fixed airless Sun-center threshold h0 = −0.833°, the independent DE441-based Horizons samples support Salah's event-existence pattern at all three latitudes: a bracketed crossing at 65.72°, and samples above the threshold at 65.735° (minimum sampled margin +0.00206° in the selected window) and 65.74°. This is evidence for that chosen threshold only, not proof of an observed sunrise or a continuous-curve bound between samples.
- The 65.735° classification is fragile in angular units: raising the comparison threshold by 0.005° (to −0.828°) flips the derived status to dipping, while lowering it preserves no-crossing; 65.74° is stable under ±0.005°. A small horizon/refraction assumption difference therefore flips event existence at 65.735° — the sampled arc passes within ~0.002–0.003° of h0.
- Civil-cycle mapping is clean: the 65.735° 00:00 Set from the preceding cycle was excluded from the selected evening event, and both adjacent midnights were sampled so no civil-day label can be mistaken for the selected cycle's minimum.

Unknown / unresolved:

- The cause of the USNO disagreement is **unresolved**. USNO's horizon/refraction rule, ephemeris, and time scale are `unreported_in_API_response`/`unreported` in the pinned manifest; the USNO API returns minute-formatted labels, not an altitude arc. Because the matched-threshold test cannot distinguish "USNO uses a slightly different effective horizon" from "USNO ephemeris geometry differs by ~0.003°", no cause is assigned.
- Horizons' physical accuracy for this comparison is unknown: 9 displayed decimals are output resolution, not an error bound. The sampled minimum is an upper bound on the true minimum (the true minimum lies between 10 s samples near a flat bottom), so the +0.00206° margin could narrow further; no exact tangent is claimed.
- Response hashes cover the full JSON wrapper bytes including the server generation timestamp, so re-fetching the same URL yields a different hash. The pinned hashes reproduce the retrieved bytes of 2026-09-28, not a timeless response body. Full remote response bodies are not committed (rights unknown); URLs, hashes, and selected facts suffice per the packet.

## 6. Source/URL/hash table

Horizons rows (full URLs and hashes in [grazing-horizons-v1.tsv](../data/reference/grazing-horizons-v1.tsv); hash scope: full JSON wrapper bytes):

| Case ID | Window (UT) | Role / step | SHA-256 |
| --- | --- | --- | --- |
| `jpl-2026-6572-wA-coarse` | 06-20 22:00 → 06-21 02:00 | coarse 10 m | `d62347a6…44819213` |
| `jpl-2026-6572-wB-coarse` | 06-21 22:00 → 06-22 02:00 | coarse 10 m | `d62ace8f…9952e4c5` |
| `jpl-2026-65735-wA-coarse` | 06-20 22:00 → 06-21 02:00 | coarse 10 m | `3e5b81c7…2726d782d` |
| `jpl-2026-65735-wB-coarse` | 06-21 22:00 → 06-22 02:00 | coarse 10 m | `65f3c12a…9de95d8363` |
| `jpl-2026-6574-wA-coarse` | 06-20 22:00 → 06-21 02:00 | coarse 10 m | `5fe471a7…68d681a0f` |
| `jpl-2026-6574-wB-coarse` | 06-21 22:00 → 06-22 02:00 | coarse 10 m | `31b87482…c6118f2b` |
| `jpl-2026-6572-wA-fine` | 06-20 23:40 → 06-21 00:20 | fine unitless 240 (10 s) | `8badfa97…42e77a61b5` |
| `jpl-2026-6572-wB-fine` | 06-21 23:40 → 06-22 00:20 | fine unitless 240 (10 s) | `0c1f8337…099751` |
| `jpl-2026-65735-wA-fine` | 06-20 23:40 → 06-21 00:20 | fine unitless 240 (10 s) | `681afcb2…f4cdabe3f9` |
| `jpl-2026-65735-wB-fine` | 06-21 23:40 → 06-22 00:20 | fine unitless 240 (10 s) | `2b2936a2…8baa34` |
| `jpl-2026-6574-wA-fine` | 06-20 23:40 → 06-21 00:20 | fine unitless 240 (10 s) | `f8df561f…384ab5eb2` |
| `jpl-2026-6574-wB-fine` | 06-21 23:40 → 06-22 00:20 | fine unitless 240 (10 s) | `dc471cea…5ee5fe6de` |

USNO rows (from [solar-usno-v1.tsv](../data/reference/solar-usno-v1.tsv), re-verified 2026-09-28): `usno-2026-6572-grazing` (`a01cc4e8…980d8b`), `usno-2026-65735-grazing-disagreement` (`dda5f47c…70239f2`), `usno-2026-6574-continuous` (`f2030605…3a20caac81`).

## 7. Verification and limitations

- `cargo fmt --all -- --check`, `cargo clippy --locked --offline --workspace --all-targets -- -D warnings`, `cargo test --locked --offline --workspace` were run (data/spec-only diff; results reported in the handoff reply).
- TSV schema: stable case IDs, one row per latitude/window/request, no empty fields (`unknown`/`not_applicable` reasoning appears where applicable; source fields are measured values, `derived_*` fields are auditor-computed).
- All 12 Horizons URLs were fetched 2026-09-28 UTC; minima and margins were computed from the returned CSV elevation column, not copied from any prior query.
- Limitations: 10 s sampling bounds but does not pinpoint the true minimum; display precision is not physical accuracy; EOP predictions vs data-based span checked (windows are data-based); USNO-side assumptions remain unknown so the disagreement cause is unresolved; no code, threshold, contract, or claim was changed. Stop for architect review.
