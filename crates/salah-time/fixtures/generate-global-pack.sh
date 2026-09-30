#!/bin/sh
# Build and verify the complete named-zone pack from pinned IANA tzdata2026d.
# Network access is used only by this maintenance script, never at runtime.
set -eu

MODE=${1:-verify}
case "$MODE" in
  verify|--write) ;;
  *) echo "usage: $0 [--write]" >&2; exit 2 ;;
esac

HERE=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
CRATE=$(CDPATH= cd -- "$HERE/.." && pwd)
WORK=$(mktemp -d "${TMPDIR:-/tmp}/salah-tzglobal.XXXXXX")
trap 'rm -rf "$WORK"' 0

DATA_URL=https://data.iana.org/time-zones/releases/tzdata2026d.tar.gz
DATA_SHA256=0cb2aa8e333c3dc049badc42a0c61f21987b8cd44e107fa900bad764aacc7767
CODE_URL=https://data.iana.org/time-zones/releases/tzcode2026d.tar.gz
CODE_SHA256=2f5c9f7fe29e6b8cb863583667884b8ce17b0a485355a054b591c6bdfcd81791

curl -fsSL -o "$WORK/tzdata2026d.tar.gz" "$DATA_URL"
curl -fsSL -o "$WORK/tzcode2026d.tar.gz" "$CODE_URL"
python3 - "$WORK" "$DATA_SHA256" "$CODE_SHA256" <<'PY'
import hashlib
import pathlib
import sys

work = pathlib.Path(sys.argv[1])
for name, expected in [
    ("tzdata2026d.tar.gz", sys.argv[2]),
    ("tzcode2026d.tar.gz", sys.argv[3]),
]:
    actual = hashlib.sha256((work / name).read_bytes()).hexdigest()
    if actual != expected:
        raise SystemExit(f"{name}: SHA-256 mismatch: {actual}")
print("Pinned IANA archive hashes verified")
PY

mkdir "$WORK/build" "$WORK/out" "$WORK/generated"
tar -xzf "$WORK/tzdata2026d.tar.gz" -C "$WORK/build"
tar -xzf "$WORK/tzcode2026d.tar.gz" -C "$WORK/build"
if [ "$(cat "$WORK/build/version")" != '2026d' ]; then
  echo "Unexpected IANA data version" >&2
  exit 1
fi

make -C "$WORK/build" zic
ZIC_VERSION=$("$WORK/build/zic" --version)
if [ "$ZIC_VERSION" != 'zic (tzcode) 2026d' ]; then
  echo "Unexpected zic version: $ZIC_VERSION" >&2
  exit 1
fi
(cd "$WORK/build" && ./zic -b slim -d "$WORK/out" \
  africa antarctica asia australasia etcetera europe \
  northamerica southamerica factory backward)

CC_CMD=${CC:-cc}
CC_ID=$("$CC_CMD" --version | head -n 1)
HOST_ID=$(uname -a)
python3 - "$WORK/out" "$WORK/build/LICENSE" "$WORK/generated" \
  "$ZIC_VERSION" "$CC_CMD" "$CC_ID" "$HOST_ID" "$DATA_URL" "$DATA_SHA256" \
  "$CODE_URL" "$CODE_SHA256" <<'PY'
import hashlib
import json
import pathlib
import sys

out = pathlib.Path(sys.argv[1])
license_path = pathlib.Path(sys.argv[2])
generated = pathlib.Path(sys.argv[3])
zic_version, compiler_command, compiler, host = sys.argv[4:8]
data_url, data_hash, code_url, code_hash = sys.argv[8:12]

paths = sorted(path for path in out.rglob("*") if path.is_file())
if not paths:
    raise SystemExit("zic produced no zone files")

pack = bytearray()
unique = {}
zones = []
for path in paths:
    relative = path.relative_to(out)
    zone_id = relative.as_posix()
    if relative.is_absolute() or ".." in relative.parts or "\\" in zone_id:
        raise SystemExit(f"unsafe generated zone path: {zone_id!r}")
    data = path.read_bytes()
    if not data.startswith(b"TZif"):
        raise SystemExit(f"generated file is not TZif: {zone_id}")
    digest = hashlib.sha256(data).hexdigest()
    key = (len(data), digest)
    if key in unique:
        offset, existing = unique[key]
        if existing != data:
            raise SystemExit(f"SHA-256 collision while packing {zone_id}")
    else:
        offset = len(pack)
        pack.extend(data)
        unique[key] = (offset, data)
    zones.append({
        "id": zone_id,
        "offset": offset,
        "bytes": len(data),
        "sha256": digest,
    })

if zones != sorted(zones, key=lambda zone: zone["id"]):
    raise SystemExit("zone inventory is not sorted")

pack_bytes = bytes(pack)
pack_hash = hashlib.sha256(pack_bytes).hexdigest()
inventory_digest = hashlib.sha256()
inventory_digest.update(b"SALAH-TZIF-ZONE-INVENTORY-V1\0")
for zone in zones:
    name = zone["id"].encode("ascii")
    inventory_digest.update(len(name).to_bytes(4, "big"))
    inventory_digest.update(name)
    inventory_digest.update(zone["offset"].to_bytes(8, "big"))
    inventory_digest.update(zone["bytes"].to_bytes(8, "big"))
    inventory_digest.update(bytes.fromhex(zone["sha256"]))
inventory_hash = inventory_digest.hexdigest()
license_bytes = license_path.read_bytes()
manifest = {
    "data_version": "2026d",
    "release_year": 2026,
    "schema_version": 1,
    "source_archive": {"url": data_url, "sha256": data_hash},
    "generator": {
        "tzcode_archive_url": code_url,
        "tzcode_archive_sha256": code_hash,
        "zic_version": zic_version,
        "compiler_command": compiler_command,
        "compiler": compiler,
        "host": host,
        "command": "./zic -b slim -d <OUT> africa antarctica asia australasia etcetera europe northamerica southamerica factory backward",
        "posix_footer": "Slim TZif files rely on RFC 8536 POSIX footer rules after explicit transitions; Jiff evaluates the footer dynamically.",
    },
    "license": {
        "file": "IANA-LICENSE",
        "sha256": hashlib.sha256(license_bytes).hexdigest(),
        "note": "Exact LICENSE file from tzdata2026d.tar.gz. All redistributed payload entries are compiled TZif data from the named IANA zone sources; optional C source files are not redistributed. Review licensing before public distribution.",
    },
    "pack": {
        "file": "tzif-pack.bin",
        "bytes": len(pack_bytes),
        "sha256": pack_hash,
        "inventory_sha256": inventory_hash,
        "unique_tzif_images": len(unique),
        "zone_ids": len(zones),
    },
    "zones": zones,
}

manifest_path = generated / "manifest.json"
manifest_path.write_text(json.dumps(manifest, indent=2, sort_keys=True) + "\n")
(generated / "tzif-pack.bin").write_bytes(pack_bytes)
(generated / "IANA-LICENSE").write_bytes(license_bytes)

def rust_string(value):
    # IANA zone IDs are ASCII; JSON string quoting is valid Rust quoting here.
    return json.dumps(value, ensure_ascii=True)

index = [
    "// Generated by fixtures/generate-global-pack.sh; do not edit by hand.",
    'pub(crate) const DATA_VERSION: &str = "2026d";',
    "pub(crate) const RELEASE_YEAR: i32 = 2026;",
    "pub(crate) const PACK_SCHEMA_VERSION: u32 = 1;",
    "pub(crate) const PACK_SHA256: &str =",
    f'    "{pack_hash}";',
    "pub(crate) const INVENTORY_SHA256: &str =",
    f'    "{inventory_hash}";',
    "#[rustfmt::skip]",
    "#[derive(Clone, Copy)]",
    "pub(crate) struct ZoneEntry { pub id: &'static str, pub offset: usize, pub len: usize }",
    "#[rustfmt::skip]",
    "pub(crate) const SUPPORTED_ZONE_IDS: &[&str] = &[",
]
index.extend(f"    {rust_string(zone['id'])}," for zone in zones)
index.extend([
    "];",
    "#[rustfmt::skip]",
    "pub(crate) const ZONE_ENTRIES: &[ZoneEntry] = &[",
])
index.extend(
    f"    ZoneEntry {{ id: {rust_string(zone['id'])}, offset: {zone['offset']}, len: {zone['bytes']} }},"
    for zone in zones
)
index.extend([
    "];",
    'pub(crate) static TZIF_PACK: &[u8] = include_bytes!("../fixtures/global/tzif-pack.bin");',
    "#[rustfmt::skip]",
    "pub(crate) fn bytes(zone_id: &str) -> Option<&'static [u8]> {",
    "    let index = ZONE_ENTRIES.binary_search_by(|entry| entry.id.cmp(zone_id)).ok()?;",
    "    let entry = ZONE_ENTRIES.get(index)?;",
    "    let end = entry.offset.checked_add(entry.len)?;",
    "    TZIF_PACK.get(entry.offset..end)",
    "}",
    "",
])
(generated / "global_zone_index.rs").write_text("\n".join(index))
PY

TARGET="$HERE/global"
if [ "$MODE" = "--write" ]; then
  mkdir -p "$TARGET"
  cp "$WORK/generated/manifest.json" "$TARGET/manifest.json"
  cp "$WORK/generated/tzif-pack.bin" "$TARGET/tzif-pack.bin"
  cp "$WORK/generated/IANA-LICENSE" "$TARGET/IANA-LICENSE"
  cp "$WORK/generated/global_zone_index.rs" "$CRATE/src/global_zone_index.rs"
  echo "Generated full IANA 2026d zone pack; inspect manifest and binary before review."
else
  python3 - "$WORK/generated/manifest.json" "$TARGET/manifest.json" <<'PY'
import json
import pathlib
import sys

generated = json.loads(pathlib.Path(sys.argv[1]).read_text())
recorded = json.loads(pathlib.Path(sys.argv[2]).read_text())
# Compiler and host values describe the original pack build. Current host
# identity may differ; byte equality of the TZif pack is the reproducibility
# check for a new host.
for manifest in (generated, recorded):
    for key in ("compiler_command", "compiler", "host"):
        manifest["generator"].pop(key, None)
if generated != recorded:
    raise SystemExit("Generated manifest data differs from the recorded pack")
PY
  for file in tzif-pack.bin IANA-LICENSE; do
    if ! cmp -s "$WORK/generated/$file" "$TARGET/$file"; then
      echo "Generated $file differs; review tool/source output before any pack update." >&2
      exit 1
    fi
  done
  if ! cmp -s "$WORK/generated/global_zone_index.rs" "$CRATE/src/global_zone_index.rs"; then
    echo "Generated zone index differs; inspect the pack manifest and index." >&2
    exit 1
  fi
  echo "Full IANA 2026d zone pack matches the reproducible generator."
fi

python3 "$CRATE/../../tools/validate_tzif_pack.py" "$TARGET/manifest.json"
