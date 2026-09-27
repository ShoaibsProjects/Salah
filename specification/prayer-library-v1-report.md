# Prayer-rule library matrix v1

**Status:** Phase 1 P1.3 comparison for `salah-core` v0.3.0, model `NOAA-MEEUS-SOLAR-2`, observed 2026-09-27. This is an engineering comparison, not a religious certification or a global accuracy bound. The [source snapshot](../data/reference/prayer-library-v1.tsv) and [offline Rust audit](../crates/salah-core/tests/prayer_library_matrix.rs) contain all signed UTC comparisons.

## Sources and matching

Two independently implemented JavaScript libraries were executed with the same numeric coordinates, local Gregorian date, Fajr/Isha angles, zero adjustments, and Standard/Hanafi shadow factor. The two parameter sets are `research-15` (15°/15°) and `mwl-angles-18-17` (18°/17°). The latter is a parameter label, not an institutional endorsement. Each row declares a fixed offset to select a local solar cycle. Both source programs ran with `TZ=UTC`; PrayTimes received the offset explicitly, and Adhan returned UTC `Date` values. The comparison is of UTC instants. No device time-zone lookup was used. For Kiritimati, Adhan must receive 2026-09-26 UTC to select the local 2026-09-27 solar cycle; PrayTimes returns unwrapped local hours 29–43, so the manifest records a −1-day cycle normalization before converting to UTC. These source-specific choices are explicit in the last two columns of each row.

| Source | Pinned identity | Calculation options | Precision and high-latitude behavior |
| --- | --- | --- | --- |
| PrayTimes JavaScript v2.5 | [Source file](https://praytimes.org/code/v2/js/PrayTimes.js), SHA-256 `f0e8442410446b1bf2db613bdebd3aa3aef5cdf5483068fbd9212d5acaa180fd` | `MWL` or `ISNA` only as a constructor base; angles explicitly overwritten, `dhuhr='0 min'`, `maghrib='0 min'`, Asr `Standard`/`Hanafi`, `highLats='None'`, zero tuning, elevation 0 m, DST 0, `Float` output. | Floating local hours converted through the stated offset to UTC and stored to milliseconds. No high-latitude adjustment is applied. A non-existent angle may return `NaN` (`no_event` in the snapshot). |
| Adhan JS npm `adhan` 4.4.6 | [Package](https://www.npmjs.com/package/adhan/v/4.4.6), npm tarball integrity `sha512-cpLLcr6qxx+mLc6jgSXJwAAHxDFrhXhVLuzRbZPrpQKIEpJw+0LcfG6u9WEjq66kcTX/2sCkI3VztgRoUXBnFg==`; package.json SHA-256 `cde0971d595046294f0090cc12ee19566bda96083a5acf6071e4b45ff2496de6`; sorted `lib/cjs/*.js` content SHA-256 `b8825c79bec06c175956761425fc9c3d9326a95b9b341298d6c15fdccef1e6b7` | `CalculationParameters('Other', fajrAngle, ishaAngle, 0, 0)`, `Rounding.None`, Shafi/Standard or Hanafi, zero manual and method adjustments, `polarCircleResolution=Unresolved`. | `Date` results have second precision. The package still applies its `MiddleOfTheNight` safety bound to Fajr/Isha; there is no `None` value for that rule. The snapshot labels `angle` versus `fallback` based on its raw solar angle result. Fallback outputs are not a matched no-substitution comparison. |

The source manifest was generated with [`tools/generate_prayer_matrix.cjs`](../tools/generate_prayer_matrix.cjs), which checks source hashes. The original third-party source code is not a runtime or repository dependency. Source coordinates are treated as numerical geodetic input; neither library output supplies a coordinate datum, atmospheric observation, terrain horizon, or institutional method review. The exact upstream source algorithms are therefore comparison evidence, not an independent physical oracle.

## Coverage and signed discrepancies

The manifest has **28 rows**: seven locations/dates × two angle sets × two Asr criteria. Regimes include ordinary northern/southern locations, equinox and solstice, London winter and summer, Tromsø polar night, and the UTC+14 date line. Values below are the maximum absolute **Salah raw UTC seconds − source UTC seconds** among rows where both return an instant. They are observations for this selected set; duplicated angle/Asr combinations do not add independent solar cases.

| Event | PrayTimes maximum absolute difference | Adhan maximum absolute difference | Status caveat |
| --- | ---: | ---: | --- |
| Fajr | 20.155 s | 2.516 s | London summer 18° has no angle crossing in Salah/PrayTimes; Adhan supplies a labeled fallback. |
| Dhuhr | 22.285 s | 1.573 s | Both sources use solar transit with zero added minutes in this run; the PrayTimes maximum is at Kiritimati. |
| Standard/Hanafi Asr | 19.203 s | 63.817 s | Both libraries return an apparent Asr in Tromsø polar night where Salah marks it unavailable; those four rows per source are status disagreements, excluded from these numeric maxima. |
| Maghrib | 25.177 s | 1.859 s | All sources return no sunset/Maghrib in Tromsø winter; zero added minutes. |
| Isha | 23.630 s | 8.496 s | London summer 17° has the same no-crossing versus Adhan fallback distinction as Fajr. |

Selected signed examples, in seconds (`Salah − source`):

| Case and rule | PrayTimes | Adhan |
| --- | ---: | ---: |
| Minneapolis 2026-09-27, 18° Fajr | −0.070 | −0.314 |
| Minneapolis 2026-09-27, Standard Asr | −14.766 | +63.817 |
| Minneapolis 2026-09-27, Hanafi Asr | −16.023 | +29.361 |
| Makkah 2026-03-20, Standard Asr | +2.846 | −21.116 |
| London 2026-06-21, 15° Fajr | −18.875 | −0.223 |
| London 2026-06-21, 17° Isha | `both_no_event` | `source_only_event` (fallback) |
| Tromsø 2026-12-21, Standard Asr | `source_only_event` | `source_only_event` |
| Kiritimati 2026-09-27, 18° Fajr | +20.155 | +2.516 |
| Kiritimati 2026-09-27, Maghrib | +25.177 | +1.625 |

The offline test prints every row/event signed difference or status. No threshold was tuned to make this matrix pass; the test verifies source metadata and records differences. The earlier ±60-second allowance in selected reference tests did **not** cover Minneapolis Standard Asr against Adhan: this matrix makes its +63.817-second discrepancy visible rather than widening that allowance.

## Discrepancy ledger

| ID | Evidence and cause | Impact and status |
| --- | --- | --- |
| P1.3-A1, Minneapolis Asr | Adhan 4.4.6 `lib/cjs/SolarTime.js` (`afternoon`) fixes the noon-shadow target using solar declination at **00:00 UTC** for its date. Salah uses declination at **upper transit**. In a controlled Adhan calculation for Minneapolis Standard Asr, its target altitude changes from 25.9486° (00:00 UTC declination −1.5548°) to 25.8299° (transit declination −1.8478°); recomputing Adhan's hour angle moves Asr **+47.815 s**. Salah − Adhan is +63.817 s in the selected case, leaving roughly 16 s from other model/solver differences. For Hanafi the same substitution moves Adhan +21.999 s. PrayTimes evaluates its declination near its computed Asr estimate and is 14–16 s later than Salah in Minneapolis. | The main direction and most of the Standard Asr gap are explained by the shadow target epoch. The remaining component is **not yet apportioned** to specific ephemeris/solver terms. No formula changed; an astronomy reviewer should assess the noon-shadow definition and broader seasonal cases before a strong Asr claim. |
| P1.3-H1, London missing twilight | At 51.5072° N on 2026-06-21, 18° Fajr and 17° Isha have no crossing in Salah and PrayTimes with high-latitude substitution disabled. Adhan's raw `SolarTime.hourAngle` is nonfinite and its `PrayerTimes` implementation supplies middle-of-night safety values at 00:02:13 UTC for Fajr and 00:02:27 UTC on the next day for Isha. | **Different high-latitude rule**, not an ordinary clock-time discrepancy. Adhan values are excluded from the matched-angle numerical comparison. Salah requires a separately named reviewed fallback before supplying times here. |
| P1.3-P1, Tromsø polar-night Asr | On 2026-12-21 the noon Sun remains below the horizon; the USNO source matrix marks rise/set unavailable. The absolute latitude–declination difference is greater than 90°, so Salah's positive-altitude Asr shadow target cannot occur. Both source libraries' Asr formulas continue through the tangent/arc-cotangent expression and yield UTC times despite the absent above-horizon shadow. | **Event-definition/status disagreement** in this polar regime. Do not use these generated source times to override Salah. The source APIs do not establish a local religious fallback. Keep polar-night Asr unsupported pending astronomical and religious review. |
| P1.3-D1, Kiritimati source cycle | PrayTimes `Float` output is unwrapped (29–43 local hours), and Adhan's input date selects a UTC rather than the intended UTC+14 local date. The generator explicitly normalizes the former by −24 hours and supplies the preceding UTC date to the latter. After this, Salah − PrayTimes is +22.285 s at transit and +25.177 s at Maghrib, while Salah − Adhan is +1.450 s and +1.625 s respectively. | **Civil-cycle mapping is resolved for this specific case**, with all normalization recorded in the manifest. The residual seconds differ by solar model and source iteration; no global date-line mapping claim follows from one island. |

| ID | Responsible review role | State |
| --- | --- | --- |
| P1.3-A1 | Astronomy reviewer; person unassigned | Shadow-target epoch explanation reproduced; residual model difference open. |
| P1.3-H1 | Islamic-methodology reviewer; person unassigned | Source fallback classified; Salah policy and wording open. |
| P1.3-P1 | Astronomy and Islamic-methodology reviewers; people unassigned | Source status disagreement reproduced; polar-night policy open. |
| P1.3-D1 | Technical maintainer; person unassigned | Source-cycle normalization reproduced for this case; wider date-line coverage open. |

## Reproduction and gate effect

Run `cargo test --locked --offline -p salah-core --test prayer_library_matrix -- --nocapture` for the checked-in snapshot. To regenerate the source values, obtain the two pinned source packages and run `TZ=UTC node tools/generate_prayer_matrix.cjs /absolute/path/to/PrayTimes.js /absolute/path/to/adhan > /tmp/prayer-library-v1.tsv`, then compare bytes with the manifest. Node and the source packages are **audit-only**; the Rust core and its offline test have no third-party runtime dependency.

P1.3 has a reproducible comparison matrix and a bounded explanation of the known Asr gap. It does **not** close Phase 1 or establish a global tolerance. Independent ephemeris review, the polar Asr question, primary method-source review, rounding contract, and the near-grazing USNO disagreement remain open. The next bounded task is **P1.4 method provenance and user-facing review**.
