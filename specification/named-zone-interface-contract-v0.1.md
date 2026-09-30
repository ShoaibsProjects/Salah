# Named-zone interface contract v0.1

**Piece:** Phase 3 → F3-C1: Engine Interface → Named-Timezone Command and Schedule Document.

**Status:** implemented research interface; native build and static checks only. No phase gate is passed.

## Purpose and boundary

Expose the existing local-date engine as a usable offline command and one reusable JSON document. The adapter performs no astronomy, timezone inference, new rounding, high-latitude substitution, or update activation. Earlier kernel, local-date, and runtime contracts remain authoritative for calculations.

This piece is an interface probe allowed by the [roadmap](../docs/roadmap.md) while Phase 1 and Phase 2 gates remain open. It does not approve a regional method default or a consumer timetable.

## Explicit manual zone selection

`salah_location::select_manual_zone(checked_coordinates, zone_id)` accepts only identifiers in the bundled supported inventory. It does not initialize or query the polygon finder. Membership in that inventory is not evidence that the user chose the jurisdictionally appropriate zone.

The result records `SelectionOrigin::ManualWithoutLookup`. Its retained `ZoneCandidates` has `lookup_performed() == false`, no candidates, and `CandidateCardinality::NotLookedUp`. This is distinct from a performed lookup returning `NoCoverage`. Installed boundary constants remain available for the existing engine compatibility check; they do not establish that mapping occurred. The JSON document omits boundary artifact provenance for an unperformed lookup.

Existing lookup results record `lookup_performed() == true`; confirmation and overrides after lookup preserve their existing origins and metadata. Overriding a direct manual selection retains `ManualWithoutLookup`. Confirmation remains impossible without a matching mapped candidate. Downstream exhaustive matches must handle the two new enum variants. This is an additive experimental API change, not a claim that the public API is frozen.

This extends [coordinate-to-zone contract v0.1](coordinate-zone-selection-contract-v0.1.md) and the [selected local-day contract](selected-local-day-engine-contract-v0.1.md) without changing their earlier lookup-derived calculations.

## Commands

```bash
cargo run --locked --offline -p salah-cli -- schedule \
  --lat 44.9778 --lon -93.2650 --date 2026-09-30 \
  --zone America/Chicago --method mwl-angles-18-17 --asr hanafi
```

This is an explicit research configuration, not a recommendation of a method for Minneapolis. The same command with `--json` emits only the schedule document plus a final newline. `salah-cli zones` lists every identifier in the bundled pack, one per line. `schedule --help` and `zones --help` describe their interfaces.

| Input | Contract |
| --- | --- |
| `--lat`, `--lon` | Finite degrees validated by `Coordinates::new`; latitude −90…90, longitude −180…180. |
| `--date` | Canonical Gregorian `YYYY-MM-DD`, validated by `CivilDate::new`, 1900–2100. No system-clock inference. |
| `--zone` | Exact case-sensitive supported IANA identifier. Manual user choice, including link names. No ambient `TZ`, device timezone, or map inference. |
| `--method` | Required: `research-15` or `mwl-angles-18-17`. No default. The first is engineering-only; the second is a published secondary parameter set with pending methodology review. |
| `--asr` | Required: `standard` or `hanafi`. No default. |
| `--json` | Optional boolean, at most once; cannot take a value. |

Missing values, duplicate flags, unknown flags, unsupported profiles/zones, and invalid dates/coordinates are errors. `--utc-offset` and `--display-minute` are not named-zone options. The existing fixed-offset invocation and its optional minute preview retain their calculation and ordinary schedule-printing path; top-level help expands to describe both interfaces.

The CLI uses the bundled snapshot only. Signed snapshot selection remains an explicit Rust API operation under the [runtime contract](runtime-rule-snapshot-v0.1.md). No key, repository, network, or trial-confirmation policy is selected by this command.

Text includes all seven events with full local dates, seconds, resolved offsets (`±HH:MM:SS`), and UTC. Local labels are derived separately for each instant; one noon offset is never reused for all events. A skipped date, zero solar cycles, multiple solar cycles, and unavailable events remain separate explicit outcomes. Multiple cycles are all printed; none is chosen silently. A successful command with unavailable events is a valid result, not a fabricated replacement.

The response is built in memory before writing to stdout. Errors go to stderr and return failure. Broken pipes are handled as an ordinary reader close in the new subcommands; other output failures return failure. This does not promise atomic pipe writes. No success banner is prepended to JSON.

## Shared document

`salah_engine::schedule_document(&SelectedLocalDaySchedule)` returns a `serde_json::Value` or `ScheduleDocumentError`. The exported `SCHEDULE_DOCUMENT_SCHEMA` is `salah-local-schedule-v1`. This encoder is available to future clients; the CLI merely formats its returned document. It can encode bundled or signed-snapshot results. It does not authenticate caller-constructed records, certify an interpretation, or activate an update.

Clients must check `schema`, distinguish statuses, accept additive object fields, and never substitute a missing time. Existing field semantics and status values must change only under a new schema identity. JSON object key order and whitespace are not contractual; cycle and event array order is contractual.

| Field/group | Meaning |
| --- | --- |
| `schema`, `scope` | Document identity and `research_preview` status. |
| `requested_local_date`, `coordinates`, `zone_id` | Explicit requested date, checked latitude/longitude in degrees, exact selected zone identifier. |
| `zone_selection` | Origin (`manual_without_lookup`, `manual_override`, `user_confirmed_suggestion`) and boundary lookup status (`not_performed`/`performed`). A performed lookup includes cardinality, all candidates, and exact map provenance. |
| `rule_pack` | Schema number, IANA version, release year, exact pack SHA-256, and exact inventory SHA-256 used by the calculation. |
| `runtime_source` | `bundled`, or `signed_package` with sequence, key ID, and boundary compatibility hash. Authentication metadata is not production approval or freshness. |
| `kernel`, `method` | Core version, astronomy model, method ID/revision/source, Fajr/Isha angles, Dhuhr/Maghrib adjustment seconds, and explicit Asr criterion. |
| `schedule_policy`, `transit_selection_policy` | IDs and revisions for the requested-date selection and localization semantics. |
| `civil_date` | Independent `exists` or `skipped` status, policy, and all constant-offset UTC intervals. Interval ends are **inclusive**. Skipped dates have an empty interval array. |
| `cycle_match_status`, `cycles` | `zero` with an empty array, `one` with one cycle, or `multiple` with all matches in existing engine UTC order. Zero alone does not imply a skipped civil date. |
| Each cycle | UTC anchor, selected unadjusted local transit with exact rule identity, raw transit solver seconds, anchor policy, high-latitude rule, assumed elevation, and seven events. |
| Each occurring event | `status: occurs`, `reading` with rounded UTC Unix seconds, ISO-shaped UTC string, local date/time, offset in **seconds east**, zone/IANA/pack identity; original raw floating Unix seconds and original event rule. |
| Each unavailable event | `status: unavailable`, reason `no_crossing_in_solar_cycle`, **no reading, instant, solver value, or substitute time**. |

Event order is `fajr`, `sunrise`, `dhuhr`, `asr`, `sunset`, `maghrib`, `isha`. Sunset and Maghrib remain separate even when their instants coincide. Rule kinds are `solar_transit`, `apparent_horizon`, `solar_depression` (degrees), `asr_shadow` (factor), `sunset_with_adjustment` (seconds), and `transit_with_adjustment` (seconds).

Dates and times are Gregorian civil labels. Event dates can differ from the requested local date. The raw floating solver number is preserved as a JSON number; clients must use the integer UTC instant for the existing rounded-second result and must not infer greater physical accuracy from displayed fractional digits. Every exported floating value is checked for finiteness; an invalid value fails the whole export rather than being silently encoded as JSON null. This is an encoding guard, not a new numerical solver or consistency validator for arbitrary public records.

## Dependencies, evidence, and remaining work

- `salah-core` remains dependency-free and unchanged. No formulas, thresholds, profiles, reference data, or calculation contracts change.
- Engine and CLI reuse pinned `serde_json` 1.0.151, already present in the lockfile. No third-party package is added or upgraded. This dependency is for serialization, not a service. CLI now declares Rust 1.88, matching its existing orchestration dependencies; the workspace kernel minimum remains unchanged.
- One existing location-unit-test fixture gains the private `lookup_performed: true` field needed to compile its old performed-lookup case. No test is added or executed for this piece, following the user's instruction.
- Local evidence: locked offline workspace build, formatting, all-target Clippy with warnings denied, and diff whitespace inspection. These do not establish runtime regression coverage, cross-target equivalence, or independent accuracy.
- Focused acceptance cases for this interface and the earlier verified-snapshot path remain open. Historical test counts are not new coverage. Phase 1 and Phase 2 gates and Phase 3 cross-platform gate remain open.
- The next implementation piece is a portable binding probe reusing this document and the same engine, followed by a simple daily schedule client. Qibla, reviewed high-latitude alternatives, broader method sources, and update stewardship remain their own roadmap tasks.
