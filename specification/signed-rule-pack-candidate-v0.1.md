# Signed rule-pack candidate v0.1

**Status:** F2-TZ9-P2a verifier implemented in `salah-update`. It authenticates and validates a candidate in memory. It does not install, activate, persist, fetch, or use candidate bytes for prayer calculations. Phase 2 remains open.

## Trust boundary

The application must provide a nonempty set of Ed25519 public keys pinned by a trusted application release. A package must never supply its own trust root. There is no production key in this repository; the deterministic private key under `#[cfg(test)]` is only a test fixture and is not compiled into a release. Key custody, rotation, revocation, and distribution require a release policy before an update channel is enabled.

The verifier accepts a detached `SignedPackCandidate`: key ID, monotonically increasing sequence, required boundary-artifact SHA-256, raw manifest bytes, raw TZif pack bytes, raw license bytes, and a 64-byte Ed25519 signature. It uses `ed25519-dalek` 2.2.0 `verify_strict`. The signature authenticates this exact candidate under an application-pinned key; it does not by itself prove IANA or governmental accuracy.

## Exact signed message

`candidate_signing_bytes` is the shared publisher/verifier encoder. The signature is over these bytes in this exact order:

```text
ASCII "SALAH-TZIF-UPDATE-V1" followed by one zero byte
sequence as unsigned 64-bit big-endian
for each of key_id, boundary_sha256, manifest, tzif_pack, license:
    byte length as unsigned 64-bit big-endian
    raw bytes
```

The signature field itself is excluded. Length prefixes and the domain string prevent field-boundary and cross-protocol ambiguity. The manifest is signed as its original bytes; no JSON reserialization or canonicalization can change the authenticated message.

## Rejection order and limits

Before signature verification, the verifier checks key-ID syntax and limits candidate sizes (manifest 4 MiB, TZif pack 32 MiB, license 256 KiB), rejects a sequence at or below the trusted `last_accepted_sequence`, requires the currently bundled boundary SHA-256, and selects a pinned key by ID. It then checks the strict signature **before** parsing JSON or TZif. The caller must persist the sequence high-water mark in trusted durable state; passing zero after each restart would lose downgrade protection.

After signature verification, schema 1 JSON is deserialized into exact typed fields with unknown/duplicate fields rejected. The verifier checks version/year syntax, source-hash formats, filenames, blob and license hashes, the separate zone-inventory hash, sorted unique safe zone IDs, per-zone hashes, slice bounds, contiguous deduplicated image coverage, declared counts, and every unique image with Jiff's caller-byte TZif parser. A candidate must contain all 598 identifiers currently supported by the bundled engine. It may add IDs up to the schema's 2048-ID limit. An older IANA release is rejected; the same release may only have the identical bundled blob **and inventory** hashes. The verifier does not fetch source archives, so their manifest hashes remain publisher provenance that must be independently checked in the release pipeline.

`VerifiedRulePack` has private fields and can only be constructed by this verifier. It retains the signed raw bytes, signature metadata, exact identity, and indexed zone slices for a future activation adapter. It cannot be passed to today's `salah-time` calculation path, which still accepts only exact embedded 2026d bytes. Any validation error returns a typed `UpdateError`; it never falls back to activating the candidate.

## Boundaries before activation

This revision targets the current boundary artifact hash. A later boundary release needs an explicitly signed compatibility record and identifier migration review; equal release labels are insufficient. An application must not expose a candidate as active until it can install the package atomically, archive the old active package, keep a bundled recovery path, persist the accepted sequence, and roll back after a failed startup. Saved results must keep the rule-pack identity that produced them. Mobile and WASM target builds of the verifier remain unverified.

The verifier adds pinned build/runtime dependencies: `ed25519-dalek` 2.2.0 (BSD-3-Clause), `sha2` 0.10.9 (MIT OR Apache-2.0), `serde` 1.0.229 and `serde_json` 1.0.151 (MIT OR Apache-2.0), plus the existing Jiff parser. No new dependency enters `salah-core`. No network service is needed for verification or daily calculation.

The separate [P2b.1 repository prototype](rule-pack-repository-v0.1.md) now supplies experimental Unix persistence and trial recovery using this verifier. That storage selection still cannot change a calculation in `salah-time`; production keys, portable activation, and runtime integration remain open.

## Evidence

Focused tests sign the pinned 2026d fixture with a test-only key, verify all zone inventory and a selected byte slice, and reject changed bytes, unknown keys, a nonincreasing sequence, incompatible boundary hash, bad pack checksum despite a valid signature, unsupported schema, duplicate JSON fields, non-ASCII release metadata, and a signed inventory-hash mismatch. The existing civil-time test confirms unpinned bytes are rejected before the TZif parser runs. These checks establish the verifier's behavior on the selected fixture; they are not an external cryptographic audit or a production update-channel approval.
