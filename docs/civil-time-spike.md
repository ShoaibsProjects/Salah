# F2-TZ0 — offline civil-time build slice

**Status:** implemented and architect-reviewed as a pre-gate Phase 2 probe in `9f0c97e`, 2026-09-28 UTC. It does not pass Phase 1 or authorize a consumer timetable. The [roadmap](roadmap.md) permits this bounded data-source probe while Phase 1 review continues.

## Why this slice

The Rust kernel already returns UTC instants and accepts a manually supplied fixed offset. A person choosing `America/Chicago` needs each UTC event converted under that zone's rules, including daylight-saving transitions. The first code slice should prove that conversion **offline from an explicitly bundled, versioned IANA data pack**. It does not yet infer a zone from coordinates or use a zone to select the kernel's local solar cycle. Those are separate problems and need separate evidence.

The seam is `UtcInstant` → `(local date, clock, UTC offset in seconds, zone ID, data version)`. The prayer engine's UTC event and method record stay unchanged. Historical IANA offsets can contain seconds, so the current minute-only `FixedUtcOffset` must not represent resolved zone offsets. UTC-to-local conversion is unique for a valid instant; the reverse local-to-UTC operation can be ambiguous or nonexistent around transitions and is outside this packet. A skipped local calendar day, such as Samoa's 2011 change, must be recognized before a later zone-aware prayer-date selector is added.

## Source and implementation facts checked

| Item | Checked fact | Limit |
| --- | --- | --- |
| [IANA Time Zone Database](https://www.iana.org/time-zones) | Latest release shown on 2026-09-28 UTC: **2026d**, released 2026-09-11. [Official source archive](https://data.iana.org/time-zones/releases/tzdata2026d.tar.gz) SHA-256 `0cb2aa8e333c3dc049badc42a0c61f21987b8cd44e107fa900bad764aacc7767`. Its internal `version` file says `2026d`. | Future political decisions can invalidate a previously bundled rule. Version 2026d is a snapshot, not a guarantee through 2050. |
| `tzdata2026d` `LICENSE` | The archive says its code/data files are public domain unless specified there; three named optional C files are BSD 3-clause when present. The included `LICENSE` SHA-256 is `0613408568889f5739e5ae252b722a2659c02002839ad970a63dc5e9174b27cf`. | Record which files are redistributed and preserve the license text. A broader distribution/legal review is still needed. |
| [RFC 8536](https://www.rfc-editor.org/rfc/rfc8536) | Specifies the TZif binary format, including transition data and a POSIX-style footer for future rules. | A parser that ignores the footer can return wrong future offsets. |
| [Jiff `TimeZone::tzif`](https://docs.rs/jiff/0.2.37/jiff/tz/struct.TimeZone.html#method.tzif) | Jiff 0.2.37 can construct rules from caller-supplied TZif bytes, so Salah can avoid the host OS zone database. Jiff is licensed Unlicense OR MIT and is an optional candidate for a new civil-time adapter crate. | Library choice still needs a small portability/dependency review. Do not use `TimeZone::get` or another implicit system lookup for this slice. |
| [Jiff bundled database](https://docs.rs/jiff-tzdb/0.1.8/jiff_tzdb/) | The inspected `jiff-tzdb` 0.1.8 package identifies its embedded data as **2026c**. | It does not match the checked 2026d official release. Do not silently report it as 2026d. |

The official IANA source and a TZif parser are build/runtime **libraries and data**, not mandatory network services. This fits the [project dependency policy](vision-and-architecture.md#5-system-architecture): core use remains offline and free, while build dependencies are pinned and inventoried. `salah-core` should remain independent of any civil-time parser.

## Boundary for F2-TZ0

Build a small `salah-time` adapter crate, separate from `salah-core`. Its initial public operation accepts a caller-supplied canonical zone ID and TZif bytes from a pinned pack, plus `UtcInstant`, and returns a typed local timestamp containing the resolved offset and data version. Invalid zone IDs, malformed TZif bytes, missing zones, or an unsupported instant must return explicit errors. No host time-zone fallback, network call, guessed offset, or device-zone assumption is allowed.

Use a small **audited fixture pack** from the official 2026d source for the first implementation; include at least Etc/UTC, America/Chicago, Europe/London, Asia/Kathmandu, Pacific/Kiritimati, and Pacific/Apia. Record the exact source archive hash, compiler/tool version and command, TZif file hashes, data version, and license in a manifest. If a reproducible compiler path cannot be established, stop and report the blocker rather than checking in unexplained binary bytes. The eventual global pack and coordinate-to-zone map are separate decisions.

The first result must distinguish:

- the **zone identifier** chosen by the caller;
- the **IANA data version** used;
- the **UTC instant** supplied by the kernel;
- the **resolved offset in seconds at that instant**;
- the **local date and clock** obtained under that offset.

Do not feed the resolved offset back into `CalculationInput` in this packet. The existing fixed-offset calculation interface and historical contract v0.3 remain unchanged. A later zone-aware daily calculation must define how the requested local date selects a solar cycle, including skipped dates and rare ambiguous local noons, before it can replace the manual selector.

## Completed delegation prompt — archive; do not rerun

```text
You are implementing F2-TZ0, a BOUNDED OFFLINE CIVIL-TIME CODE SLICE, in /Users/shoaibakthar/Documents/Salah. Start from current main. Read AGENTS.md; docs/roadmap.md Phase 2 and parallel-work rules; docs/vision-and-architecture.md dependency and time-zone sections; docs/civil-time-spike.md in full; specification/calculation-contract-v0.3.md; crates/salah-core/src/civil.rs and lib.rs READ ONLY. Phase 1 remains open; this is an allowed pre-gate probe, not a consumer release.

TASK: add a separate Rust adapter crate for UTC-instant-to-local conversion using caller-supplied IANA TZif bytes and a caller-supplied canonical zone ID. Use the official IANA tzdata2026d source archive (URL and SHA-256 pinned in docs/civil-time-spike.md) to produce a small reproducible fixture pack for Etc/UTC, America/Chicago, Europe/London, Asia/Kathmandu, Pacific/Kiritimati, and Pacific/Apia. Include a manifest with archive identity/hash, exact compiler/tool identity and generation command, each generated TZif hash, data version, and redistributed license. If official-source generation is not reproducible, stop with a precise blocker; never substitute the host OS database without labeling it.

Use an audited TZif parser rather than writing a partial format parser. Jiff 0.2.37 TimeZone::tzif is the current candidate, but confirm its feature/dependency set, offline/portable behavior, and treatment of post-transition POSIX footer before selecting it. Pin the dependency and explain why it is acceptable under the project's no-mandatory-service policy. The new crate must accept bytes supplied by the caller; do not call TimeZone::get or read /usr/share/zoneinfo implicitly. Keep salah-core and its outputs unchanged.

Expose a small typed API that converts a salah_core::UtcInstant to a local civil date/time, resolved offset **in seconds**, canonical zone ID, and tzdb version. Do not coerce a historical second offset into the minute-only FixedUtcOffset. The data version must come from the identified pack, not a host setting. Reject invalid IDs/bytes/missing zone/unsupported instants explicitly. Do not infer a zone from coordinates. Do not yet map a requested local date to the prayer engine's fixed-offset selector, implement inverse local-to-UTC, change CalculationInput/Record, add Qibla or high-latitude rules, or make consumer accuracy claims.

Show evidence for UTC instants immediately before/after Chicago DST start and end in 2026, a London transition, Kathmandu's fractional-hour offset, Kiritimati UTC+14 date carriage, and Apia's 2011 skipped local date. Derive expected values from the pinned 2026d pack and preserve exact UTC inputs, local outputs, and offset provenance; explain that this selected fixture set is not global validation. Confirm the API has no network use and no host-zone fallback. Preserve .idea/ and unrelated files. Return the exact diff, source/fixture hashes, API and dependency summary, observed conversions, limitations, and any blockers. Stop for architect review; do not commit, push, or start coordinate mapping or prayer-date integration.
```

## Review gate after the code slice

Accept F2-TZ0 only if every result names its zone/data version, transition behavior follows the bundled bytes, and changing host time-zone settings cannot change the output. This proves the civil-time seam for selected zones. It does not prove global zone coverage, future law, coordinate lookup, or correct prayer-date selection. The next code packet can then define the zone-aware date selector against DST gaps, overlaps, skipped civil dates, and date-line cases.

**Review result:** accepted for the six-zone probe. `salah-time` 0.1.0 uses Jiff 0.2.37 with only its `std` feature and exact 2026d fixture-byte matching before a result receives that version label. `salah-core` remains dependency-free. The generator verifies both official archive hashes before extraction and every TZif/LICENSE hash before replacing checked-in fixtures; it reproduced the manifest on this host. Formatting, linting, and the full 41-test offline workspace suite passed. The seven civil evidence cases passed under two different host `TZ` settings in separate processes. An unsafe process-wide environment mutation in the submitted test was removed. CI now fetches locked build dependencies once before its offline checks; the same fetch-then-offline sequence passed with a fresh Cargo cache. The conversion runtime needs no network. Mobile and WASM targets are not installed on this host, so portability is **unverified**. The adapter reparses bytes per call and is limited to six named zones; parsed-rule reuse, full IANA coverage, authenticated update/rollback, and zone-aware prayer-date selection remain later work.
