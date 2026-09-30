#!/usr/bin/env python3
"""Focused rejection checks for the offline TZif pack validator."""

from __future__ import annotations

import json
import shutil
import tempfile
import unittest
from pathlib import Path

from validate_tzif_pack import PackValidationError, validate

SOURCE = Path(__file__).resolve().parents[1] / "crates/salah-time/fixtures/global"


class PackValidationTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.folder = Path(self.temporary.name)
        for name in ("manifest.json", "tzif-pack.bin", "IANA-LICENSE"):
            shutil.copyfile(SOURCE / name, self.folder / name)
        self.manifest = self.folder / "manifest.json"

    def test_bundled_pack_passes_and_boundary_release_must_match(self) -> None:
        self.assertEqual(validate(self.manifest, "2026d"), ("2026d", 598, 345))
        with self.assertRaisesRegex(PackValidationError, "boundary version"):
            validate(self.manifest, "2026c")

    def test_changed_payload_is_rejected(self) -> None:
        path = self.folder / "tzif-pack.bin"
        payload = bytearray(path.read_bytes())
        payload[-1] ^= 1
        path.write_bytes(payload)
        with self.assertRaisesRegex(PackValidationError, "pack SHA-256"):
            validate(self.manifest)

    def test_duplicate_manifest_key_is_rejected(self) -> None:
        text = self.manifest.read_text()
        self.manifest.write_text(text.replace('"schema_version": 1,',
                                              '"schema_version": 1, "schema_version": 1,', 1))
        with self.assertRaisesRegex(PackValidationError, "duplicate manifest key"):
            validate(self.manifest)

    def test_zone_inventory_must_be_sorted_and_unique(self) -> None:
        manifest = json.loads(self.manifest.read_text())
        manifest["zones"][1]["id"] = manifest["zones"][0]["id"]
        self.manifest.write_text(json.dumps(manifest))
        with self.assertRaisesRegex(PackValidationError, "sorted and unique"):
            validate(self.manifest)

    def test_release_year_must_match_iana_version(self) -> None:
        manifest = json.loads(self.manifest.read_text())
        manifest["release_year"] = 2027
        self.manifest.write_text(json.dumps(manifest))
        with self.assertRaisesRegex(PackValidationError, "release_year differs"):
            validate(self.manifest)

    def test_zone_inventory_hash_is_checked_independently_of_blob_hash(self) -> None:
        manifest = json.loads(self.manifest.read_text())
        manifest["pack"]["inventory_sha256"] = "0" * 64
        self.manifest.write_text(json.dumps(manifest))
        with self.assertRaisesRegex(PackValidationError, "zone inventory SHA-256"):
            validate(self.manifest)


if __name__ == "__main__":
    unittest.main()
