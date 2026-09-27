# USNO solar matrix v1 — signed comparison report

**Kernel:** `salah-core` v0.3.0, `NOAA-MEEUS-SOLAR-2`, `research-15` with zero Dhuhr adjustment. **Source:** USNO Complete Sun and Moon Data API 4.0.1, retrieved 2026-09-27 UTC. **Input:** each row's coordinates, local date, and explicitly supplied fixed UTC offset in the [source manifest](../data/reference/solar-usno-v1.tsv). This report records evidence for selected cases, not global accuracy or observed-sunrise precision.

The signed number is **Salah raw UTC instant minus USNO's minute-formatted UTC value**, in seconds. A positive sign means the kernel is later. The offline [comparison test](../crates/salah-core/tests/usno_solar_matrix.rs) prints every value with `--nocapture`; the [source verifier](../tools/verify_usno_matrix.py) re-fetches URLs and checks response hashes and selected events. All 19 primary source responses and Reykjavík's secondary response matched their recorded hashes and event selections when re-fetched on 2026-09-27 UTC.

## Results by case

| Case ID | Cohort | Regime | Rise Δ | Upper transit Δ | Set Δ |
| --- | --- | --- | ---: | ---: | ---: |
| `usno-2026-minneapolis-autumn` | development | ordinary | +21.0 s | -1.4 s | -10.1 s |
| `usno-2026-makkah-equinox` | development | ordinary | -3.3 s | +9.6 s | -18.8 s |
| `usno-2026-sydney-solstice` | development | ordinary | -21.1 s | +1.9 s | +25.6 s |
| `usno-2026-tromso-summer` | development | polar-no-event | No event ✓ | -0.5 s | No event ✓ |
| `usno-2026-kiritimati-dateline` | development | date-line | -9.2 s | -5.6 s | -3.4 s |
| `usno-2050-quito` | development | boundary-year | -28.5 s | -15.0 s | -1.6 s |
| `usno-2028-makkah-leap` | development | leap-year | -8.8 s | +7.1 s | -19.3 s |
| `usno-2026-6572-grazing` | development | near-grazing | -94.3 s | -10.5 s | +111.0 s |
| `usno-2026-65735-grazing-disagreement` | development | near-grazing | Disagrees | -10.5 s | Disagrees |
| `usno-2026-6574-continuous` | development | polar-no-event | No event ✓ | -10.5 s | No event ✓ |
| `usno-1900-sydney` | development | boundary-year | -19.0 s | +5.8 s | -28.2 s |
| `usno-2100-london` | development | boundary-year | +20.7 s | -28.6 s | -18.4 s |
| `usno-2026-nairobi-equinox` | holdout | ordinary | -4.9 s | +11.6 s | +27.0 s |
| `usno-2026-buenos-aires-solstice` | holdout | ordinary | +16.9 s | -19.0 s | +5.3 s |
| `usno-2026-anchorage-summer` | holdout | high-lat-existing | +15.2 s | -29.0 s | -16.1 s |
| `usno-2026-tromso-winter` | holdout | polar-no-event | No event ✓ | Unreported | No event ✓ |
| `usno-2026-apia-dateline` | holdout | date-line | +24.8 s | +11.0 s | +8.8 s |
| `usno-2026-reykjavik-summer` | holdout | high-lat-existing | +8.9 s | -23.5 s | +2.0 s |
| `usno-2026-pago-pago-dateline` | holdout | date-line | +25.9 s | -20.9 s | +4.4 s |

`No event ✓` means the source explicitly reports continuous daylight/darkness at the horizon and the kernel also returns unavailable. `Unreported` means the API did not supply that event; it is not a no-event claim. `Disagrees` is the known 65.735° N event-existence mismatch and is **open**, not a passed tolerance.

## Summary by regime

| Regime | Numeric UTC comparisons | Largest absolute difference among those | Matching no-event cells | Known disagreements | Source unreported cells |
| --- | ---: | ---: | ---: | ---: | ---: |
| ordinary | 15 | 27.0 s | 0 | 0 | 0 |
| polar-no-event | 2 | 10.5 s | 6 | 0 | 1 |
| date-line | 9 | 25.9 s | 0 | 0 | 0 |
| boundary-year | 9 | 28.6 s | 0 | 0 | 0 |
| leap-year | 3 | 19.3 s | 0 | 0 | 0 |
| near-grazing | 4 | 111.0 s | 0 | 2 | 0 |
| high-lat-existing | 6 | 29.0 s | 0 | 0 | 0 |

Across the 19 cases (12 development, seven holdout), **48** reported UTC instants fall within the previously documented case allowances, **six** explicit source no-event cells match, **two** near-grazing event-existence cells disagree, and **one** winter upper transit is not reported by USNO. The largest signed magnitude among ordinary cases is 27.0 seconds; the largest among the two high-latitude existing-event holdouts is 29.0 seconds. The 65.72° N near-grazing rise and set differ by −94.3 and +111.0 seconds respectively. These are sample observations against a minute-formatted source, not uncertainty bounds for all Earth locations.

## Open discrepancy and source interpretation

At 65.735° N on 2026-06-21, USNO reports a 00:04 rise and 23:59 set, while Salah's fixed −0.833° apparent horizon has no crossing. The source also lists a 00:00 set from the preceding solar cycle. The event-existence disagreement is sensitive to the physical horizon/refraction model; the USNO response does not itself specify all matching model assumptions. It needs a separate ephemeris comparison under explicit, matched definitions before changing the kernel or assigning a universal answer.

Reykjavík's one-day response lists a 00:04 set from the preceding cycle. The manifest uses the **following day's** 00:04 set for the selected 21 June evening solar cycle and records both source hashes. Tromsø's winter response explicitly says continuously below the horizon but omits an upper-transit clock value; that transit cell is `unreported`.

The source's coordinate datum, observer elevation, horizon/refraction rule, and underlying ephemeris time scale are not specified in the retrieved response and are marked unknown in the manifest. A formatted-minute source can differ from an otherwise equivalent raw-second result due to quantization alone. The current ±90-second ordinary and ±180-second near-grazing comparison allowances are investigation thresholds for this matrix, not a release precision target. Time-zone selection, future DST law, terrain, weather, and Islamic method review remain outside this comparison.

## Reproduction

```bash
cargo test --locked --offline -p salah-core --test usno_solar_matrix -- --nocapture
python3 tools/verify_usno_matrix.py --all
```

The first command runs offline and has no third-party crate dependency. The second is optional, uses Python's standard library and network access, and reports source drift without rewriting the manifest.
