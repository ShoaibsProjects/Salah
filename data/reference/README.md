# Solar reference data

`solar-usno-v1.tsv` is the first machine-readable, offline reference matrix for Salah's solar model. It contains **19 stable cases**: 12 development cases already known during kernel work and seven later holdout cases. The current [signed-difference report](../../specification/usno-matrix-v1-report.md) explains the results. The [accuracy budget](../../specification/accuracy-budget.md) explains why a selected comparison allowance is not a global precision claim.

## Source and preservation

The source is the [U.S. Naval Observatory Complete Sun and Moon Data API](https://aa.usno.navy.mil/data/api#rstt), response `apiversion` 4.0.1 when retrieved on 2026-09-27 UTC. Each row stores its exact request URL, the SHA-256 of the raw response bytes, and the reported local clock values converted to UTC using the **explicit offset supplied in that request**. A response can be checked with the optional standard-library-only tool:

```bash
python3 tools/verify_usno_matrix.py --case usno-2026-reykjavik-summer
python3 tools/verify_usno_matrix.py --all
```

The tool retrieves source responses over the network for audit only. Runtime calculation and the Rust comparison test need no network or third-party package. Full USNO response bodies are not committed here; the URL, API version, hash, and selected factual event values are preserved. If the remote response changes or disappears, its hash identifies the change but cannot reconstruct the old body. Long-term archival rights and a durable mirror remain an operations decision.

## TSV schema

Every row has a stable `case_id`, `cohort` (`development` or `holdout`), `regime`, local Gregorian date, latitude/longitude in degrees, and the supplied UTC offset in minutes east. The source fields record name/version, primary URL/hash, optional secondary URL/hash, and UTC retrieval date. Each event field is a UTC ISO timestamp, `no_event`, or `unreported`.

- `no_event` means the primary response explicitly says the Sun is continuously above or below the **horizon** and therefore has no rise/set that day.
- `unreported` means the selected source response did not report that phenomenon. It **does not** assert that the astronomical event is impossible.
- `source_precision=minute_formatted` describes the `HH:MM` clock strings returned by this API. It does not assert a particular rounding algorithm or physical error.
- `coordinate_datum`, observer elevation, ephemeris time scale, and horizon assumption are marked `unreported` where the response did not specify them. The engine's own assumptions are separately recorded in the [calculation contract](../../specification/calculation-contract-v0.3.md).
- `source_event_definition` preserves the API's `Rise`, `Upper Transit`, and `Set` labels. This is not an assertion that every physical assumption matches Salah's model.

The date-line holdouts include Samoa at UTC+13 and nearby American Samoa at UTC−11, emphasizing that longitude alone does not supply the civil date. The USNO one-day response lists phenomena occurring within a **civil date**, which may span different solar cycles. At 65.735° N on 2026-06-21, the response contains a `Set` at 00:00 from the preceding cycle and a `Set` at 23:59 after that day's transit; the manifest selects the latter. Reykjavík's 2026-06-21 response lists a 00:04 `Set` from the preceding cycle, so the selected evening sunset is taken from the **2026-06-22** response at 00:04. Both request URLs and hashes are stored. Tromsø in winter has an explicit continuously-below-horizon marker for rise/set, but its response omits `Upper Transit`; the manifest records transit as `unreported` rather than `no_event`.

## Comparison

Run the offline, dependency-free Rust comparison with:

```bash
cargo test --locked --offline -p salah-core --test usno_solar_matrix -- --nocapture
```

It reports the signed difference `Salah raw UTC seconds − USNO UTC minute value` for each reported event. It checks event existence, stable IDs, source metadata shape, the development/holdout split, and the current case-specific allowances. The known 65.735° N event-existence disagreement is printed explicitly and does not silently become an allowed time error.

The seven holdouts were selected after the earlier hard-coded regression cases. Passing them is additional evidence for those conditions only; the sample is neither random nor representative of all dates, atmospheres, coordinates, or Islamic methods.
