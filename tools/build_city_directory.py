#!/usr/bin/env python3
"""Generate the bundled Apple city catalogue from pinned local GeoNames files.

No network operation. City reference points are approximate, not device fixes.
"""
import hashlib
import io
import json
from pathlib import Path
import zipfile

ROOT = Path(__file__).resolve().parent.parent
SOURCE = ROOT / "data/third-party/geonames-2026-10-01"
OUTPUT = ROOT / "apps/apple/Salah"
PINS = {
    "cities15000.zip": "f00213685cfe4ee1deeb7a4e52ce6258ac99d7b27209ffca0e8590ccab433763",
    "countryInfo.txt": "93bafc525813f22e4711ff9ed6d626343094ce48c26388dc7c49189b3d7d5512",
    "readme.txt": "b1957379b6c1242c700c98ac9a8aa0a09f56c3c0a50ee72175527005f48ef2c5",
}


def main():
    sources = {}
    for name, expected in PINS.items():
        data = (SOURCE / name).read_bytes()
        if hashlib.sha256(data).hexdigest() != expected:
            raise ValueError(f"Pinned GeoNames source differs: {name}")
        sources[name] = data
    countries = {}
    for line in sources["countryInfo.txt"].decode().splitlines():
        if not line or line.startswith("#"):
            continue
        row = line.split("\t")
        countries[row[0]] = row[4]
    cities = []
    ids = set()
    with zipfile.ZipFile(io.BytesIO(sources["cities15000.zip"])) as archive:
        member = archive.getinfo("cities15000.txt")
        if member.file_size > 30_000_000:
            raise ValueError("Unexpectedly large source table")
        for line in archive.read(member).decode().splitlines():
            row = line.split("\t")
            if len(row) != 19:
                raise ValueError("Unexpected GeoNames row shape")
            identifier, latitude, longitude = int(row[0]), float(row[4]), float(row[5])
            if identifier in ids or not (-90 <= latitude <= 90 and -180 <= longitude <= 180):
                raise ValueError("Invalid GeoNames point/identifier")
            if row[8] not in countries:
                raise ValueError("Missing country identity")
            ids.add(identifier)
            # Keep bounded source aliases for common alternate spellings.
            aliases = list(dict.fromkeys(n for n in row[3].split(",") if 0 < len(n) <= 80))[:24]
            cities.append([identifier, row[1], row[2], row[8], latitude, longitude, int(row[14]), aliases])
    cities.sort(key=lambda city: (-city[6], city[0]))
    catalogue = {"schema": "salah-offline-cities-v1", "version": "geonames-2026-10-01",
                 "countries": countries, "cities": cities}
    data = (json.dumps(catalogue, ensure_ascii=False, separators=(",", ":")) + "\n").encode()
    if len(data) > 8_000_000 or not 20_000 <= len(cities) <= 50_000:
        raise ValueError("Unexpected catalogue size/count")
    (OUTPUT / "cities-v1.json").write_bytes(data)
    manifest = {"schema": "salah-offline-cities-manifest-v1", "version": catalogue["version"],
                "file": "cities-v1.json", "rows": len(cities), "bytes": len(data),
                "sha256": hashlib.sha256(data).hexdigest(), "retrieved": "2026-10-01",
                "license": "CC-BY-4.0", "point_semantics": "approximate_city_reference_point",
                "sources": [{"url": "https://download.geonames.org/export/dump/" + name,
                             "sha256": digest} for name, digest in PINS.items()]}
    (OUTPUT / "cities-manifest-v1.json").write_text(json.dumps(manifest, indent=2) + "\n")
    print(f"Bundled {len(cities)} GeoNames city points; {len(data)} bytes; {manifest['sha256']}")


if __name__ == "__main__":
    main()
