# Asr residual audit v1 (P1.3-A1-R1)

**Status:** Phase 1 P1.3 evidence only, observed 2026-09-28 UTC. This document
decomposes the Minneapolis 2026-09-27 Asr gap against pinned Adhan JS 4.4.6 and
PrayTimes JS v2.5 under matched explicit parameters. It changes no formula,
solver tolerance, method ID/revision, profile, contract, or gate. The existing
[28-row matrix](../data/reference/prayer-library-v1.tsv) and its
[p1.3 report](prayer-library-v1-report.md) are preserved unchanged, including
the recorded +63.817-second Minneapolis Standard Asr gap as historical evidence.
All signed differences below are **Salah raw unrounded UTC minus the stated
row instant** unless a column says otherwise. Cases here are **selected**, not
holdout validation. No numerical match below proves a religious Asr definition.

Evidence manifest: [`asr-residual-v1.tsv`](../data/reference/asr-residual-v1.tsv)
(30 rows, 28 columns). Every row of that file is cited in §9.

## 1. Question and scope

The prayer-library report traces most of the Minneapolis Standard Asr gap to
Adhan's shadow-target declination epoch (00:00 UTC versus transit), moving
Adhan +47.815 s and leaving roughly 16 s unapportioned. This audit reproduces
that substitution exactly, separates Standard from Hanafi, extends the same
controls to two contrasting pinned-matrix rows (Makkah 2026-03-20 equinox,
Cape Town 2026-12-21 southern-hemisphere solstice), holds the shadow target
fixed while solar-position/root methods differ, and distinguishes Adhan
`Date` integer-second quantization from model differences.

Out of scope: any `crates/**` change, high-latitude rule, global Asr error
bound, fiqh ruling, and any Phase 2 behavior.

## 2. Pinned source identities (verified before use)

Both third-party programs were obtained outside this repository on 2026-09-28
UTC, hash-checked against the identities pinned in the prayer-library report,
and used audit-only. Neither was added as a runtime dependency nor committed.

| Source | Identity | Hash verification (2026-09-28 UTC) |
| --- | --- | --- |
| PrayTimes JS v2.5 | `https://praytimes.org/code/v2/js/PrayTimes.js` | SHA-256 `f0e8442410446b1bf2db613bdebd3aa3aef5cdf5483068fbd9212d5acaa180fd` — matched |
| Adhan JS npm `adhan` 4.4.6 | `https://www.npmjs.com/package/adhan/v/4.4.6` | tarball integrity `sha512-cpLLcr6qxx+mLc6jgSXJwAAHxDFrhXhVLuzRbZPrpQKIEpJw+0LcfG6u9WEjq66kcTX/2sCkI3VztgRoUXBnFg==` — matched; `package.json` SHA-256 `cde0971d595046294f0090cc12ee19566bda96083a5acf6071e4b45ff2496de6` — matched; sorted `lib/cjs/*.js` content SHA-256 `b8825c79bec06c175956761425fc9c3d9326a95b9b341298d6c15fdccef1e6b7` — matched; `package.json` version `4.4.6` — matched |
| Salah core under audit | `salah-core` 0.3.0, model `NOAA-MEEUS-SOLAR-2`, tree at `4d41e61` | local build only; authoritative Salah instants taken from the unmodified Rust core via a throwaway path-dependency probe (not committed) |

If either pinned source had been unobtainable, this audit would have reported
the blockage instead of substituting an unpinned replacement. Both were
obtained, so no blockage applies.

## 3. Frozen inputs

All rows use profile `mwl-angles-18-17` (18° Fajr, 17° Isha, zero
Dhuhr/Maghrib adjustment, `high_latitude_rule="none"`, elevation 0 m), fixed
offsets as listed, and `Rounding.None` / `Float` / raw-seconds outputs as
applicable. Asr depends only on the shadow factor here, so Fajr/Isha angles
do not move any Asr instant in this audit.

| Case | Local date | Latitude | Longitude | Fixed offset | Regimes |
| --- | --- | --- | --- | --- | --- |
| minneapolis-autumn | 2026-09-27 | 44.9778° | −93.265° | −300 min (−05:00) | ordinary northern autumn |
| makkah-equinox | 2026-03-20 | 21.4225° | 39.8262° | +180 min (+03:00) | ordinary equinox, lower latitude |
| cape-town-solstice | 2026-12-21 | −33.9249° | 18.4241° | +120 min (+02:00) | ordinary southern solstice |

Shadow factors: Standard f = 1, Hanafi f = 2.

## 4. Equations and units

Angles in degrees, internal library times in fractional hours, outputs as UTC
instants (ISO-8601, millisecond-exact where the implementation allows),
signed differences in seconds. Let φ = latitude (degrees), δ = solar
declination at the implementation's chosen epoch (degrees), f ∈ {1, 2}.

All three implementations use the same geometric shadow-target form. With
noon shadow length s₀ = tan|φ − δ|:

```text
h* = arctan( 1 / ( f + tan|φ − δ| ) )        (target altitude, degrees)
Asr = afternoon time t with solar altitude(t) = h*
```

PrayTimes writes the same target as `angle = −arccot(f + tan|φ − δ|)`; its
`sunAngleTime` takes a *depression* argument, so a negative depression is a
positive altitude target. The condition solved is identical; only the
declination epoch, ephemeris, and root method differ.

## 5. Code findings per implementation

### 5.1 Salah (`crates/salah-core/src/prayer.rs`, `solar.rs` — read only)

- Declination epoch: **upper transit**. `prayer.rs:130` evaluates
  `position(transit).declination_radians`; the noon-shadow baseline
  `noon_zenith_distance = |φ − δ_transit|` is fixed there (lines 130–137).
- Target: `target_altitude = atan(1 / (f + tan(noon_zenith))` in degrees
  (line 137). A transit at or beyond the pole
  (`noon_zenith_distance ≥ π/2`) yields `Unavailable`, not a time.
- Root: direction-aware falling `crossing` on `[transit, transit + 43200]`
  of `altitude_degrees(t) − target`, with an 80-step golden-section interior
  extremum check and 50 bisections (`solar.rs:104–170`). `altitude_degrees`
  re-evaluates the NOAA declination at each candidate t (time-varying
  ephemeris along the search).
- Rounding: nearest integer second (`utc.round()`), raw `f64` preserved
  alongside (`prayer.rs:212–224`). Salah rows therefore carry both a rounded
  reported instant and an exact fractional instant; their difference (≤ 0.5 s)
  is ordinary rounding, not model error.
- Not exposed (marked `unknown` where needed): the 8-iteration transit
  residual (`solar.rs:84–87` returns only the `f64`); no convergence bound is
  established (see accuracy budget).

### 5.2 Adhan 4.4.6 (audited copy, `lib/cjs/*.js`)

- Declination epoch for the shadow target: **00:00 UTC of the input date**.
  `SolarTime.js` constructs `SolarCoordinates(julianDay(y, m, d, 0))`, and
  `afternoon(shadowLength)` computes
  `tangent = |latitude − this.solar.declination|`,
  `angle = atan(1 / (shadowLength + tan(tangent)))`. The `TODO source shadow
  angle calculation` comment is in the audited file and changes nothing.
- Root: `correctedHourAngle` (`Astronomical.js`): closed-form H₀ from the
  00:00-epoch declination, then **one** correction `dm = (h − h₀) / term4`
  using Meeus-interpolated declination/RA at fractional time m. Ephemeris is
  full Meeus (mean longitude/anomaly, equation of center, nutation,
  apparent sidereal time) per `Astronomical.js` / `SolarCoordinates.js`.
- Output quantization: `TimeComponents.js` applies `Math.floor` to
  hours/minutes/seconds, and `DateUtils.roundedMinute` with `Rounding.None`
  adds zero offset. Adhan `Date` outputs are therefore **truncations** (floor)
  of the fractional-hour result, not nearest-second rounding. Truncation
  magnitudes measured here: 0.096–0.891 s (§7). This is kept separate from
  model residuals that are an order of magnitude larger.
- Not returned by the API (`unknown` in the manifest): the interpolated
  declination at the afternoon root epoch inside `correctedHourAngle`.

### 5.3 PrayTimes v2.5 (audited copy, `PrayTimes.js`)

- Declination epoch: **event-proximate**. `asrTime(factor, time)` evaluates
  `sunPosition(jDate + time)` with `jDate = julian(y, m, d) − lng/360`; the
  single main iteration (`numIterations = 1`) updates the estimate once from
  the default seed (`times.asr = 13`, i.e. 13/24 day). Measured seed-epoch
  versus final-epoch declinations: Minneapolis −1.8638° → −1.9184°/−1.9318°;
  Makkah −0.0713° → −0.0238°/−0.0081°; Cape Town −23.4349° → −23.4352°/−23.4353°.
- Target: `angle = −arccot(f + tan|lat − decl|)` fed to `sunAngleTime`,
  whose closed-form `arccos` root uses the declination at the same epoch.
- Ephemeris: low-precision USNO-approximation `sunPosition` (single
  sinusoidal terms), distinct from both NOAA (Salah) and Meeus (Adhan).
- Output: `Float` local hours → UTC milliseconds in the manifest generator;
  no second quantization. Internal `dayPortion` seed values are not logged.

## 6. Observed raw UTC results (measurement)

Salah reference instants (unrounded, Rust core) and source outputs:

| Case × criterion | Salah unrounded (reference) | Adhan `Date` (floored) | PrayTimes Float→UTC (ms) | Salah−Adhan (s) | Salah−PrayTimes (s) |
| --- | --- | --- | --- | ---: | ---: |
| Minneapolis Standard | 21:21:55.817 | 21:20:52 | 21:22:10.583 | **+63.817** | −14.766 |
| Minneapolis Hanafi | 22:11:15.361 | 22:10:46 | 22:11:31.384 | +29.361 | −16.023 |
| Makkah Standard | 12:52:59.884 | 12:53:21 | 12:52:57.038 | −21.116 | +2.846 |
| Makkah Hanafi | 13:50:14.968 | 13:50:24 | 13:50:12.775 | −9.032 | +2.193 |
| Cape Town Standard | 14:29:27.109 | 14:29:32 | 14:29:22.824 | −4.891 | +4.285 |
| Cape Town Hanafi | 15:45:21.574 | 15:45:23 | 15:45:15.507 | −1.426 | +6.067 |

The Minneapolis Standard +63.817 s reproduces the pinned matrix value
byte-for-byte; the allowance is unchanged. Note the sign flip for Adhan at
Makkah (Salah *earlier* than Adhan near equinox) and the small Cape Town gaps
near solstice, where the declination-epoch term nearly vanishes (§8).

A transit-only cross-check (Dhuhr, no shadow target involved) measures
Salah−Adhan at +1.573 s (Minneapolis), −1.436 s (Makkah), and +1.276 s
(Cape Town). These values do not bound the solar-position or root-method
difference at Asr, because the event is evaluated at another altitude and
time. They show only that the observed transit instants are close in these
three selected cases.

## 7. Controlled counterfactual C1: Adhan target at transit declination

Only the shadow-target declination was changed (00:00 UTC → transit epoch,
via Adhan's own `interpolate` at m = transit/24); Adhan's
`correctedHourAngle` machinery is untouched. The transit-epoch declinations
are −1.8478° (Minneapolis), −0.0850° (Makkah), −23.4371° (Cape Town) —
identical to Salah's NOAA transit declinations to 4 dp, so the C1 targets
match Salah's displayed targets (25.8299°/18.0650°, 35.6530°/22.6703°,
40.1576°/24.5908°) to 4 dp. **C1 is therefore a near-fixed-target
control**: the remaining residual is predominantly ephemeris + root method
(+ sub-second quantization handling), with any sub-display-precision target
difference still unmeasured. It is not the original 00:00 target-epoch term.

| Case × criterion | 00:00 target → transit target | Exact shift (s) | Exact residual¹ (s) | Floored residual² (s) |
| --- | --- | ---: | ---: | ---: |
| Minneapolis Standard | 25.9486° → 25.8299° | +47.814 | **+15.112** | +15.817 |
| Minneapolis Hanafi | 18.1250° → 18.0650° | +21.998 | +7.198 | +7.361 |
| Makkah Standard | 35.5918° → 35.6530° | −16.430 | −5.224 | −5.116 |
| Makkah Hanafi | 22.6435° → 22.6703° | −6.995 | −2.594 | −2.032 |
| Cape Town Standard | 40.1566° → 40.1576° | −0.281 | −5.070 | −4.891 |
| Cape Town Hanafi | 24.5904° → 24.5908° | −0.120 | −1.402 | −0.426 |

¹ Salah unrounded minus C1 exact fractional instant (quantization-free).
² Salah unrounded minus C1 floored `Date` (test-comparable convention).

The Minneapolis exact shifts (+47.814 s / +21.998 s) agree with the
previously reported +47.815 s / +21.999 s to 1 ms. The floored Standard
residual +15.817 s ≈ 16 s is the residual named in the packet. Adhan floor
truncations at the observed instants (0.891, 0.165, 0.538, 0.557, 0.461,
0.096 s) and at C1 instants (0.705, 0.163, 0.108, 0.562, 0.179, 0.976 s) are
an order of magnitude smaller than the model residuals (except Cape Town
Hanafi, where the residual itself is ~1 s — see §10).

## 8. PrayTimes target analysis (hypothesis, labeled as such)

PrayTimes native targets (event-proximate declination) versus transit targets:

| Case × criterion | Native target | Transit target | Δ (deg) | Observed Salah−Pray (s) |
| --- | --- | --- | ---: | ---: |
| Minneapolis Standard | 25.8013° | 25.8299° | −0.0286 | −14.766 |
| Minneapolis Hanafi | 18.0477° | 18.0650° | −0.0173 | −16.023 |
| Makkah Standard | 35.6770° | 35.6530° | +0.0240 | +2.846 |
| Makkah Hanafi | 22.6835° | 22.6703° | +0.0132 | +2.193 |
| Cape Town Standard | 40.1568° | 40.1576° | −0.0008 | +4.285 |
| Cape Town Hanafi | 24.5905° | 24.5908° | −0.0003 | +6.067 |

To apportion these, the local Salah altitude fall rate at each Asr was
measured by finite difference (±30 s): −401.1, −365.5, −268.8, −261.4,
−289.9, −296.6 s/deg respectively. Multiplying Δ by that rate gives a
**hypothetical** target-epoch term: −11.5, −6.3, +6.5, +3.5, −0.2, −0.1 s,
leaving remainders of −3.3, −9.7, −3.6, −1.3, +4.5, +6.2 s to
ephemeris (USNO-approx vs NOAA) + single-pass closed-form root. This split
is explicitly a hypothesis: it transfers Salah's local slope to another
ephemeris and does not separate PrayTimes' seed/iteration effect. The Cape
Town rows are the cleanest near-fixed-target evidence (Δ < 0.001°), so their
+4.285/+6.067 s gaps predominantly reflect ephemeris/root differences under
this hypothesis; a small target-epoch contribution remains possible.

## 9. Row index — every manifest row cited

Manifest [`asr-residual-v1.tsv`](../data/reference/asr-residual-v1.tsv), 28
columns; `unknown`/`not_applicable` markers carry reasons in-cell. Signed
columns are Salah-unrounded-minus-row-instant; `quantization_exact_minus_reported_s`
is exact-minus-reported (Adhan floor ≥ 0, Salah rounding ±, PrayTimes 0.000).

| Row ID | Kind | Section |
| --- | --- | --- |
| `asr-msp-std-salah-observed` | measurement | §6 (reference 21:21:55.817Z) |
| `asr-msp-std-adhan-observed` | measurement | §6 (+63.817 s) |
| `asr-msp-std-praytimes-observed` | measurement | §6 (−14.766 s) |
| `asr-msp-std-adhan-transit-target` | computation | §7 (shift +47.814 s; residual +15.112/+15.817 s) |
| `asr-msp-std-praytimes-target-analysis` | hypothesis | §8 (Δ −0.0286°; implied −11.5 s; remainder −3.3 s) |
| `asr-msp-han-salah-observed` | measurement | §6 (reference 22:11:15.361Z) |
| `asr-msp-han-adhan-observed` | measurement | §6 (+29.361 s) |
| `asr-msp-han-praytimes-observed` | measurement | §6 (−16.023 s) |
| `asr-msp-han-adhan-transit-target` | computation | §7 (shift +21.998 s; residual +7.198/+7.361 s) |
| `asr-msp-han-praytimes-target-analysis` | hypothesis | §8 (Δ −0.0173°; implied −6.3 s; remainder −9.7 s) |
| `asr-mak-std-salah-observed` | measurement | §6 (reference 12:52:59.884Z) |
| `asr-mak-std-adhan-observed` | measurement | §6 (−21.116 s; sign flip vs Minneapolis) |
| `asr-mak-std-praytimes-observed` | measurement | §6 (+2.846 s) |
| `asr-mak-std-adhan-transit-target` | computation | §7 (shift −16.430 s; residual −5.224/−5.116 s) |
| `asr-mak-std-praytimes-target-analysis` | hypothesis | §8 (Δ +0.0240°; implied +6.5 s; remainder −3.6 s) |
| `asr-mak-han-salah-observed` | measurement | §6 (reference 13:50:14.968Z) |
| `asr-mak-han-adhan-observed` | measurement | §6 (−9.032 s) |
| `asr-mak-han-praytimes-observed` | measurement | §6 (+2.193 s) |
| `asr-mak-han-adhan-transit-target` | computation | §7 (shift −6.995 s; residual −2.594/−2.032 s) |
| `asr-mak-han-praytimes-target-analysis` | hypothesis | §8 (Δ +0.0132°; implied +3.5 s; remainder −1.3 s) |
| `asr-ct-std-salah-observed` | measurement | §6 (reference 14:29:27.109Z) |
| `asr-ct-std-adhan-observed` | measurement | §6 (−4.891 s) |
| `asr-ct-std-praytimes-observed` | measurement | §6 (+4.285 s) |
| `asr-ct-std-adhan-transit-target` | computation | §7 (shift −0.281 s; residual −5.070/−4.891 s) |
| `asr-ct-std-praytimes-target-analysis` | hypothesis | §8 (Δ −0.0008°; remainder +4.5 s ephemeris/root) |
| `asr-ct-han-salah-observed` | measurement | §6 (reference 15:45:21.574Z) |
| `asr-ct-han-adhan-observed` | measurement | §6 (−1.426 s) |
| `asr-ct-han-praytimes-observed` | measurement | §6 (+6.067 s) |
| `asr-ct-han-adhan-transit-target` | computation | §7 (shift −0.120 s; residual −1.402/−0.426 s) |
| `asr-ct-han-praytimes-target-analysis` | hypothesis | §8 (Δ −0.0003°; remainder +6.2 s ephemeris/root) |

## 10. Minneapolis compared with the contrasting cases

- **Direction of the target-epoch term is seasonal, not constant.**
  Minneapolis (late September, declination falling through −1.6° → −1.8°)
  gains +47.8/+22.0 s from the substitution; Makkah (equinox, declination
  rising −0.24° → −0.09°) *loses* −16.4/−7.0 s; Cape Town (solstice,
  declination static at −23.44°) moves ≤ 0.3 s. A single Minneapolis-derived
  correction must not be generalized.
- **The near-fixed-target residual does not vanish anywhere.** With C1 targets
  matching Salah's displayed targets to 4 dp, quantization-free
  residuals of +15.1/+7.2 s (Minneapolis), −5.2/−2.6 s (Makkah),
  −5.1/−1.4 s (Cape Town) remain. These are ephemeris (NOAA vs Meeus) plus
  root-method (time-varying bisection vs single-step interpolated hour angle)
  differences and any remaining sub-display-precision target mismatch, with
  Salah's transit-iteration residual an unmeasured contributor. They are not
  apportioned further in this packet.
- **PrayTimes sits on the opposite side at Minneapolis** (−14.8/−16.0 s:
  PrayTimes later) while Adhan sits earlier (+63.8/+29.4 s). Source-to-source
  disagreement exceeds either source's gap to Salah in that case; agreement
  must not be treated as a vote.
- **Quantization is bounded and separate.** Adhan floor truncation (≤ 0.891 s
  observed here) cannot explain residuals of 5–15 s, but it is material at
  Cape Town Hanafi, where the floored residual (−0.426 s) differs from the
  exact residual (−1.402 s) by the 0.976 s C1 truncation. Comparisons must
  state which convention they use.

## 11. Explained versus open ledger

| Term | State |
| --- | --- |
| Adhan 00:00-UTC versus transit declination epoch | **Explained and reproduced**: +47.814 s Standard / +21.998 s Hanafi at Minneapolis; sign and magnitude vary by season (§7) |
| Adhan `Date` floor quantization | **Bounded**: 0.096–0.891 s observed; 0.108–0.976 s at C1 instants; convention stated per residual |
| Salah nearest-second rounding | **Bounded**: ≤ 0.5 s by construction; exact column removes it |
| Near-fixed-target residual (Adhan) | **Measured but unapportioned**: +15.112/+7.198 s (Msp), −5.224/−2.594 s (Mak), −5.070/−1.402 s (CT), quantization-free convention; predominantly ephemeris/root, with sub-display-precision target difference unmeasured |
| PrayTimes target-epoch vs ephemeris/root split | **Hypothesis only** (§8); Cape Town near-fixed-target residuals (+4.285/+6.067 s) are the firmest ephemeris/root evidence |
| Salah transit-iteration residual | **Unknown**: not exposed by the API; candidate contributor to every residual here |
| Religious Asr definition | **Explicitly not decided**: geometry agreement is not fiqh agreement |

## 12. Reproduction

1. Check out this tree; run the offline matrix test
   `cargo test --locked --offline -p salah-core --test prayer_library_matrix -- --nocapture`
   and confirm the Minneapolis Standard Asr `adhan=+63.817s` line.
2. Fetch the two pinned sources (§2), verify hashes, and with `TZ=UTC` run
   `tools/generate_prayer_matrix.cjs` per the prayer-library report to
   regenerate the baseline instants.
3. Recompute C1 with Adhan's own `Astronomical.correctedHourAngle`,
   substituting only the `afternoon` target angle computed from the
   transit-interpolated declination; confirm the exact shifts in §7 to 1 ms.
4. Validate the manifest schema/arithmetic: 30 data rows × 28 columns, unique
   `row_id`, no empty cells, signed columns equal Salah-reference minus row
   instant to 1 ms.

## 13. Limitations and reviewer gaps

- Selected cases only (3 locations × 2 criteria); no seasonal sweep, no
  high-latitude or polar Asr, no holdout set. Nothing here supports a global
  Asr error bound.
- The NOAA replica used to cross-check Salah transit declinations is an
  audit computation ported from `solar.rs`, not a second implementation; its
  agreement (altitude at Salah Asr equals target to ~1e-6°) validates the
  arithmetic, not the physics.
- Adhan's internal root-epoch declination and Salah's transit residual are
  `unknown` (API limits), so the fixed-target residual cannot be split
  further without deeper instrumentation.
- The PrayTimes slope split (§8) transfers Salah's local dAlt/dt across
  ephemerides and is labeled hypothesis for that reason.
- Astronomy reviewer: vacant. Islamic-methodology reviewer: vacant. No
  qualified review of the noon-shadow definition or user-facing Asr wording
  has occurred.

## 14. Gate effect

None. P1.3-A1's residual is now decomposed per case, but the near-fixed-target
remainder is open and unreviewed; Phase 1 remains open per the
[interim gate report](phase-1-gate-report-v0.1.md).
