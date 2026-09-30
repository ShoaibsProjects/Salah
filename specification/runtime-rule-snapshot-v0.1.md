# Runtime rule snapshot v0.1

**Piece:** Phase 2 → F2-TZ9 → P2b.2a: Verified Snapshot in Calculations. 30 September 2026. Research scope; both phase gates remain open.

## Decision and trust boundary

Separate payload integrity from package authentication. `salah-time` owns a bounded, immutable `ValidatedRulePack`: schema, exact blob/license/inventory hashes, zone slices, and TZif parsing. The existing `salah-update` verifier authenticates the exact signed message **before** asking that shared loader to parse the payload. A checksum-valid payload alone is not a trusted update.

The engine accepts a `RuntimeRuleSnapshot` built from either the compiled pack or a `VerifiedRulePack` returned by the pinned-key verifier (including a repository load). It never accepts an arbitrary manifest, TZif slice, or caller-written identity as an authenticated runtime. The runtime owns its snapshot, so later repository changes cannot alter a calculation in flight.

## Runtime rule

1. Select a bundled or verified snapshot explicitly; the existing engine entry point continues to select bundled rules.
2. Retain explicit coordinate/zone selection, date, method, and Asr inputs. Boundary compatibility follows the exact artifact authenticated by the verifier; a later IANA release need not have the old boundary release label.
3. Resolve one zone from that snapshot. Use it for date existence, transit membership, transit conversion, and every occurring event. Missing/invalid data is an error, never a silent switch to bundled bytes.
4. Preserve the same exact pack identity (schema, IANA version/year, blob SHA-256, inventory SHA-256) in the schedule record, date classification, local transit, and all local event readings. The engine also records bundled versus signed origin, signing-key ID, signed package sequence, and boundary artifact hash.
5. Keep UTC calculation, event rules, missing events, and zero/one/multiple cycle outcomes unchanged. A different civil-time snapshot can change local-date membership and local clock labels; such a recomputation is a new result.

## Ownership and compatibility

Runtime metadata owns strings rather than borrowing permanent static labels. `RulePackIdentity` and version fields use `Cow<'static, str>` so compiled constants stay available while verified releases retain owned labels. This is a research API change: identity is `Clone`, not `Copy`; callers must clone when retaining it. No string is leaked to manufacture a static lifetime.

The shared integrity loader retains the verifier's current schema/release/name-coverage rules and size limits. It introduces the already pinned Serde/JSON and SHA-256 libraries into the civil-time crate; no new package or mandatory service enters the workspace or astronomical core. Runtime zone handles have private fields and are constructed only from bundled rules or the validated inventory.

## Storage and confirmation

`RulePackRepository::selected_pack` reauthenticates the stored archive. The application can move that verified value into a runtime and calculate before calling the existing sequence-checked `confirm_trial`. Constructing a runtime or calculating a schedule does not confirm a trial. If calculation fails, report the error and explicitly roll back through the repository; an interrupted trial still follows the repository recovery protocol.

One successful schedule is a limited runtime exercise, not a health certificate for every zone/date or a production release approval. The API does not automate app startup, trial confirmation, key custody, fetches, retention, or platform storage. Existing snapshot handles remain immutable if the repository later rolls back; applications must replace their runtime explicitly and retain saved-result provenance.

## Remaining evidence and boundaries

Acceptance requires the bundled path to preserve existing results, a signed alternative to change the intended civil rules and every identity consistently, invalid signatures/payloads to fail before use, stored selection/reopen behavior, date-line/DST/polar status preservation, and no host database/network access. Implementation build/static checks alone do not settle these acceptance cases. A synthetic later-release fixture must be labeled synthetic and is never evidence of actual future law.

Mobile/WASM target builds, target persistence, hardware durability, production key stewardship, broader independent civil-time comparisons, and consumer release remain open. Historical v0.1 contracts describe their original scopes; this contract records the additive runtime path and ownership change.

## Usage and progress

The application first opens its repository (which performs restart recovery), then loads its selection under the current trusted keys:

```rust
let snapshot = match repository.selected_pack()? {
    Some(pack) => salah_engine::RuntimeRuleSnapshot::from_verified(pack),
    None => salah_engine::RuntimeRuleSnapshot::bundled(),
};
let result = salah_engine::calculate_selected_local_day_schedule_with_snapshot(
    &snapshot, &zone_selection, requested_date, method, asr_criterion,
)?;
```

The application supplies the repository and explicit inputs. This sequence does not confirm a trial or catch an IO/authentication error as bundled fallback. A separate application policy must inspect trial status and exercise the runtime before calling `confirm_trial` for the expected selected sequence. A confirmed repository selection and a directly verified candidate carry the same signed-package source label; that label does not claim durable installation or a completed health check.

This implementation was built and linted locally across the workspace and existing targets. No new runtime tests were added or executed in this slice. Required focused acceptance evidence above remains pending. The shared loader's validation body retains the previous verifier's decisions; its error categories are mapped back to the existing `UpdateError` variants. No third-party package is added or upgraded in the lockfile; dependency edges move to the shared loader. Astronomical formulas, method profiles, source data, tolerances, and CLI output code are unchanged.
