# Offline TZif pack interface v0.1

**Status:** first F2-TZ9 integrity/provenance slice. This is a local pack format and bundled-result identity, not an update activation or authentication scheme. Phase 2 remains open.

## Pack identity

The bundled 2026d rule pack has manifest schema `1`, IANA release `2026d`, release year `2026`, exact `tzif-pack.bin` SHA-256 `e919bf1d69ca9948a559963bc1384576144e1513367aefcd4340c83a79b8f816`, and zone-inventory SHA-256 `17d496af9df7ca6997fd279d3b8f78c4d854c1e02e12300bbb7eb03e8fcc4317`. `salah-time::BUNDLED_RULE_PACK_IDENTITY` carries those fields. Every `LocalCivilTime`, `LocalDateTransitRecord`, and `LocalDateExistence` now returns the same `RulePackIdentity`; the existing `tzdb_version` field remains for compatibility. A selected-day schedule retains the identity through its record, date classification, and occurring local events. The separate civil-time data assessment reads the release year from that identity.

The blob hash identifies the exact compiled bytes. The inventory hash additionally binds each zone ID to its offset, length, and image hash, so a reordered index cannot reuse the same blob identity. Both are reproducibility checksums, **not** digital signatures. The manifest's source archive and `tzcode` hashes are provenance claims; this validator checks their format but cannot verify source archives unless those archives are separately obtained and hashed.

## Schema and local validation

`fixtures/global/manifest.json` has `schema_version: 1`, `data_version`, `release_year`, source/generator hashes, pack filename/length/blob hash/inventory hash/counts, license filename/hash, and sorted zone entries with ID, offset, length, and per-slice hash. The generator writes the corresponding Rust identity constants from the same hashes and release metadata, avoiding separate hand-maintained values. Unique TZif slices form a contiguous, nonoverlapping byte range; several IDs may reference the same exact slice.

Inventory hash encoding v1 starts with ASCII `SALAH-TZIF-ZONE-INVENTORY-V1` and one zero byte. For each sorted zone, append the ID byte length as unsigned 32-bit big-endian, ASCII ID bytes, offset and image length as unsigned 64-bit big-endian, and the 32 raw bytes represented by that zone's SHA-256 hex. SHA-256 of that full stream is `inventory_sha256`.

Run the standard-library maintenance check:

```bash
python3 tools/validate_tzif_pack.py crates/salah-time/fixtures/global/manifest.json --boundary-version 2026d
```

It rejects unsupported schema, version mismatch, unsafe paths/IDs, duplicate JSON keys, missing or symlinked local files, mismatched pack/license/zone hashes, unsorted or duplicate IDs, out-of-range slices, overlapping or unreferenced images, count mismatches, and malformed TZif images as parsed by Python's `zoneinfo` from supplied bytes. The generator runs the same check after writing or comparing its output. No host time-zone database or network is used by the validator. Python 3.10+ is required for this maintenance tool.

The Rust runtime still accepts only the compiled pack's exact per-zone bytes. `parse_pinned_zone` rejects a different image, even if its name matches. This slice therefore cannot activate an external candidate pack. It adds no third-party Rust dependency and does not alter UTC prayer calculations, local offsets, or zone selection.

## Next acceptance boundary

The [signed-candidate verifier](signed-rule-pack-candidate-v0.1.md) now checks signatures under application-supplied trusted keys, the current boundary artifact, and full name inventory. Before any candidate pack can become active, add production key custody/rotation, versioned archive, durable sequence state, and atomic activation with rollback to the bundled pack. Distinguish cryptographic authentication of the package from its format and checksum checks. Preserve the old pack to reproduce saved results; recomputation under a new pack must produce a new result with its new identity. Platform distribution rules and independent transition evidence remain required. No network-supplied bytes enter the current calculation path.
