# Rule-pack repository v0.1

**Piece:** Phase 2 → F2-TZ9 (Timezone Data Updates) → P2b.1 (Local Repository and Crash Recovery). 30 September 2026. Experimental Unix filesystem adapter in `salah-update-store`; neither release gate has passed.

## Purpose and boundary

Archive exact authenticated rule packages, preserve an accepted-update high-water mark across restarts, and recover from an interrupted trial. The application supplies a private local directory and the [trusted-key verifier](signed-rule-pack-candidate-v0.1.md). Stored selection is not calculation activation: today's `salah-time` still accepts only its embedded rule bytes. A later runtime adapter must demonstrate compatibility and a health check before a real application confirms a trial.

`salah-update-store` is separate from the portable verifier and dependency-free astronomical kernel. It uses the existing Serde/JSON and SHA-256 dependencies and standard-library filesystem operations; no new third-party package is added. Its Rust minimum is 1.89 because it uses standard-library advisory file locking. The repository API is compiled only on Unix; other targets expose `STORAGE_PLATFORM_SUPPORTED = false`. This revision was exercised on macOS. Linux, iOS packaging, Android filesystem behavior, Windows, and browser storage remain unverified.

## Storage and trust

The caller chooses a trusted app-private directory whose parent already exists. New directories are created with mode `0700`, and new files with `0600`; the caller's umask may restrict them further. Existing group/other permissions are rejected. Directory/file symlinks and file hard links are rejected at checked entry points. Archive names are internally formatted from the unsigned sequence as 20 decimal digits; signed names or URLs never become filesystem paths.

The OS advisory `repository.lock` is held for the repository handle's lifetime. All cooperating callers must use this API. A second open fails with `Busy`; the lock file is never deleted, and the OS releases the lock on process exit. The directory must live on a local filesystem with supported atomic rename, file/directory sync, and advisory-lock semantics. The caller must guarantee ownership and ACL isolation through its app container; mode checks alone do not establish those properties. This prototype does not defend against a privileged actor or a process with the application's credentials replacing files outside the API. Its path checks are not an adversarial filesystem sandbox. Moving or deleting the entire private directory is a privileged reset outside this protocol; stronger local antirollback requirements need an OS-protected anchor.

Layout:

```text
root/
  repository.lock
  state.json
  packs/
    <20-digit sequence>/
      signed.json         # schema, sequence, key ID, boundary SHA
      manifest.json       # original signed manifest bytes
      tzif-pack.bin       # original signed blob
      IANA-LICENSE        # original signed license bytes
      signature.ed25519   # exact 64-byte signature
```

The state envelope has schema `1`, a revision, the accepted sequence high-water, highest accepted pack identity, selected/prior pack references, trial status, and a SHA-256 over the typed state's JSON serialization. The state checksum detects accidental corruption; it is not a signature or protection against a same-credential attacker editing/recomputing it. The authoritative package authentication is Ed25519 under the application's trusted keys. Archives are reverified on every load, including recovery, and their recorded identity must match exactly. Unknown or revoked keys cannot load an archive.

State reads are capped at 16 KiB, archive metadata at 4 KiB, signatures at 64 bytes, and manifest/blob/license reads use the verifier's shared limits. Unknown/duplicate typed JSON fields are rejected. Missing state beside existing archives is an error and never initializes a zero counter. Invalid state is also an error; an application can still calculate with its compiled pack but must not silently reset update state.

## Transaction and recovery rule

1. `begin_trial` authenticates and validates the entire candidate before writing files. The sequence must exceed the stored high-water. Its IANA release must not precede the highest accepted release, even after a recovery or rollback. Reusing a release label requires identical blob and inventory hashes.
2. Write the original signed bytes into a newly reserved private staging directory. Sync each file and the staging directory, rename it to its final archive name, then sync `packs/`. Archived directories are never overwritten by this API. An orphan final archive from an interruption before state commit can be reused only if all signed bytes and metadata match exactly; otherwise the occupied sequence cannot be replaced. Old archives remain to support reproduction.
3. Write a new small state envelope into an exclusively created temporary file, sync it, atomically rename it over `state.json`, and sync the root directory. The state selects the candidate as an **unconfirmed trial**, retains the prior selection, and advances the sequence and release high-water. Success is returned only after the required sync calls succeed.
4. The consuming runtime may inspect `selected_pack()` and perform its explicit health check. `confirm_trial(expected_sequence)` reverifies it and confirms only the matching trial. Confirmation keeps the prior archive available for rollback. This API cannot certify the runtime health check on the caller's behalf.
5. On reopening, an unconfirmed trial is reverted. A confirmed selection that fails authentication, identity, file validation, or has missing files is also reverted. Restore the prior pack only if it verifies under the current trust store; if that archive is invalid/missing, return a `None` selection directing the caller to its compiled recovery pack. Other IO failures propagate without rewriting the selection, so temporary read errors cannot permanently demote valid data. Return a typed recovery notice when a recovery commits. Both accepted high-water records remain unchanged, so recovery does not re-enable replay or release downgrades.
6. Explicit `rollback()` follows the same prior-or-bundled rule and preserves both high-water records. A failed state replacement/sync is `CommitUncertain`; the session refuses further reads/mutations until closed and reopened because disk and memory may differ. An unchanged previous state or a completed next state is resolved on reopen.

If the process stops after archive persistence but before state commit, the old selection remains authoritative. If the trial state persisted, reopening reverts it. If confirmation persisted, reopening retains the confirmed selection. Partial unpublished temporary files/directories are ignored when valid state exists; this revision does not automatically garbage-collect them or old archives. Disk exhaustion is a typed IO failure. Pruning requires a future retention policy for saved-result reproduction.

## Evidence and limits

Tests exercise exact signed-byte preservation across reopen, first/prior-pack trial recovery, replay rejection after recovery, orphan-archive reuse, corrupted archive recovery, missing/truncated/checksum-invalid state, exclusive locking, symlink rejection, a synthetic later-release downgrade case, bad signatures before writes, an actual failed state rename and poisoned session, and abrupt subprocess exit without destructors. Injected archive-read failures exercise the real recovery/rollback paths and verify byte-for-byte unchanged state until a successful retry. The synthetic release metadata is a state-policy fixture, not evidence of future IANA data. The subprocess test checks OS lock release and trial recovery after process termination.

These tests exercise process interruption and error paths on the available host. They do not reproduce hardware power loss, filesystem/controller failures, every crash instruction boundary, or behavior on other targets. No production signing key, key-rotation policy, remote update channel, downloaded-pack calculation path, or consumer timetable is enabled. Next pieces are target-specific persistence review, portable runtime integration with exact result provenance, and production signing stewardship.
