#!/usr/bin/env python3
"""Validate a local Salah TZif pack manifest and its exact payload bytes.

This is an offline integrity and format check, not signature verification.
The caller must obtain the manifest from a trusted source before it can be
used to authenticate an update.
"""

from __future__ import annotations

import argparse
import hashlib
import io
import json
import re
import sys
from pathlib import Path
from zoneinfo import ZoneInfo

HEX_SHA256 = re.compile(r"[0-9a-f]{64}\Z")
IANA_VERSION = re.compile(r"[0-9]{4}[a-z]\Z")
ZONE_ID = re.compile(r"[A-Za-z0-9._+/-]+\Z")
MAX_MANIFEST_BYTES = 4 * 1024 * 1024
MAX_PACK_BYTES = 32 * 1024 * 1024
MAX_LICENSE_BYTES = 256 * 1024
MAX_ZONE_IDS = 2048


class PackValidationError(ValueError):
    pass


def unique_object_pairs(pairs: list[tuple[str, object]]) -> dict:
    result = {}
    for key, value in pairs:
        require(key not in result, f"duplicate manifest key: {key}")
        result[key] = value
    return result


def require(condition: bool, message: str) -> None:
    if not condition:
        raise PackValidationError(message)


def object_at(value: object, label: str) -> dict:
    require(isinstance(value, dict), f"{label} must be an object")
    return value


def integer_at(value: object, label: str, minimum: int = 0,
               maximum: int = (1 << 64) - 1) -> int:
    require(type(value) is int and minimum <= value <= maximum,
            f"{label} must be an integer from {minimum} through {maximum}")
    return value


def sha256_at(value: object, label: str) -> str:
    require(isinstance(value, str) and HEX_SHA256.fullmatch(value) is not None,
            f"{label} must be a lowercase SHA-256 hex string")
    return value


def local_file(base: Path, value: object, label: str) -> Path:
    require(isinstance(value, str) and value not in ("", ".", ".."),
            f"{label} must be a local file name")
    require(Path(value).name == value and "/" not in value and "\\" not in value,
            f"{label} must not contain path components")
    path = base / value
    require(path.is_file(), f"{label} is missing: {path}")
    require(not path.is_symlink(), f"{label} must not be a symlink")
    return path


def validate(manifest_path: Path, boundary_version: str | None = None) -> tuple[str, int, int]:
    require(not manifest_path.is_symlink(), "manifest must not be a symlink")
    require(manifest_path.stat().st_size <= MAX_MANIFEST_BYTES,
            "manifest exceeds the format size limit")
    manifest = object_at(json.loads(manifest_path.read_text(encoding="utf-8"),
                                    object_pairs_hook=unique_object_pairs), "manifest")
    require(integer_at(manifest.get("schema_version"), "schema_version") == 1,
            "unsupported pack schema")
    version = manifest.get("data_version")
    require(isinstance(version, str) and IANA_VERSION.fullmatch(version) is not None,
            "data_version must be an IANA release such as 2026d")
    require(integer_at(manifest.get("release_year"), "release_year") == int(version[:4]),
            "release_year differs from data_version")
    if boundary_version is not None:
        require(boundary_version == version,
                f"boundary version {boundary_version} does not match TZif version {version}")

    source = object_at(manifest.get("source_archive"), "source_archive")
    sha256_at(source.get("sha256"), "source_archive.sha256")
    require(isinstance(source.get("url"), str) and source["url"].startswith("https://"),
            "source_archive.url must be an HTTPS URL")
    generator = object_at(manifest.get("generator"), "generator")
    sha256_at(generator.get("tzcode_archive_sha256"), "generator.tzcode_archive_sha256")

    pack_info = object_at(manifest.get("pack"), "pack")
    pack_path = local_file(manifest_path.parent, pack_info.get("file"), "pack.file")
    require(pack_path.stat().st_size <= MAX_PACK_BYTES,
            "pack exceeds the format size limit")
    pack = pack_path.read_bytes()
    require(len(pack) == integer_at(pack_info.get("bytes"), "pack.bytes", 1),
            "pack byte count differs from manifest")
    require(hashlib.sha256(pack).hexdigest() == sha256_at(pack_info.get("sha256"), "pack.sha256"),
            "pack SHA-256 differs from manifest")

    license_info = object_at(manifest.get("license"), "license")
    license_path = local_file(manifest_path.parent, license_info.get("file"), "license.file")
    require(license_path.stat().st_size <= MAX_LICENSE_BYTES,
            "license exceeds the format size limit")
    require(hashlib.sha256(license_path.read_bytes()).hexdigest() ==
            sha256_at(license_info.get("sha256"), "license.sha256"),
            "license SHA-256 differs from manifest")

    zones = manifest.get("zones")
    require(isinstance(zones, list) and 0 < len(zones) <= MAX_ZONE_IDS,
            "zones must be a nonempty bounded list")
    require(len(zones) == integer_at(pack_info.get("zone_ids"), "pack.zone_ids", 1),
            "zone count differs from manifest")
    previous_id = ""
    unique_ranges: set[tuple[int, int]] = set()
    parsed_images: set[tuple[int, int]] = set()
    inventory_digest = hashlib.sha256()
    inventory_digest.update(b"SALAH-TZIF-ZONE-INVENTORY-V1\0")
    for number, raw_zone in enumerate(zones):
        zone = object_at(raw_zone, f"zones[{number}]")
        zone_id = zone.get("id")
        require(isinstance(zone_id, str) and ZONE_ID.fullmatch(zone_id) is not None
                and len(zone_id) <= 255
                and all(part not in ("", ".", "..") for part in zone_id.split("/")),
                f"zones[{number}].id is unsafe")
        require(previous_id < zone_id, f"zones must be sorted and unique: {zone_id}")
        previous_id = zone_id
        offset = integer_at(zone.get("offset"), f"{zone_id}.offset")
        length = integer_at(zone.get("bytes"), f"{zone_id}.bytes", 1)
        end = offset + length
        require(end <= len(pack), f"{zone_id} slice exceeds the pack")
        image = pack[offset:end]
        zone_hash = sha256_at(zone.get("sha256"), f"{zone_id}.sha256")
        require(hashlib.sha256(image).hexdigest() == zone_hash,
                f"{zone_id} SHA-256 differs from manifest")
        encoded_id = zone_id.encode("ascii")
        inventory_digest.update(len(encoded_id).to_bytes(4, "big"))
        inventory_digest.update(encoded_id)
        inventory_digest.update(offset.to_bytes(8, "big"))
        inventory_digest.update(length.to_bytes(8, "big"))
        inventory_digest.update(bytes.fromhex(zone_hash))
        require(image.startswith(b"TZif"), f"{zone_id} is not TZif data")
        key = (offset, end)
        if key not in parsed_images:
            try:
                ZoneInfo.from_file(io.BytesIO(image), key=zone_id)
            except Exception as error:
                raise PackValidationError(f"{zone_id} TZif parse failed: {error}") from error
            parsed_images.add(key)
        unique_ranges.add(key)

    require(len(unique_ranges) == integer_at(pack_info.get("unique_tzif_images"),
            "pack.unique_tzif_images", 1), "unique image count differs from manifest")
    require(inventory_digest.hexdigest() == sha256_at(pack_info.get("inventory_sha256"),
            "pack.inventory_sha256"), "zone inventory SHA-256 differs from manifest")
    cursor = 0
    for start, end in sorted(unique_ranges):
        require(start == cursor, "unique TZif images overlap or leave unreferenced pack bytes")
        cursor = end
    require(cursor == len(pack), "pack has trailing unreferenced bytes")
    return version, len(zones), len(unique_ranges)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("manifest", type=Path, help="local pack manifest.json")
    parser.add_argument("--boundary-version", help="required matching boundary release")
    args = parser.parse_args()
    try:
        version, zone_count, unique_count = validate(args.manifest, args.boundary_version)
    except (OSError, UnicodeError, json.JSONDecodeError, PackValidationError) as error:
        print(f"TZif pack validation failed: {error}", file=sys.stderr)
        return 1
    print(f"TZif pack {version}: {zone_count} identifiers, {unique_count} unique images verified")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
