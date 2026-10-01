#!/usr/bin/env python3
"""Build a bundled Rust XCFramework and unsigned iOS research app, offline.

Requires Xcode and the pinned Rust iOS targets installed beforehand. No Swift
package manager, generated bindings dependency, or runtime service is needed.
"""

import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import sys
import tomllib

ROOT = Path(__file__).resolve().parent.parent
APPLE = ROOT / "apps/apple"
GENERATED = APPLE / "Generated"
TARGETS = ("aarch64-apple-ios", "aarch64-apple-ios-sim", "x86_64-apple-ios")


def run(command, **kwargs):
    subprocess.run(command, cwd=ROOT, check=True, **kwargs)


def licenses(channel, environment):
    metadata = json.loads(subprocess.check_output(
        ["rustup", "run", channel, "cargo", "metadata", "--locked", "--offline",
         "--format-version", "1", "--filter-platform", TARGETS[0]],
        cwd=ROOT, env=environment, text=True,
    ))
    packages = {p["id"]: p for p in metadata["packages"]}
    # Workspace metadata may contain optional edges not enabled by the Apple
    # build. Use Cargo's active normal/build tree for each compiled target.
    enabled = set()
    for target in TARGETS:
        tree = subprocess.check_output(
            ["rustup", "run", channel, "cargo", "tree", "--locked", "--offline",
             "-p", "salah-ffi", "--target", target, "--edges", "normal,build",
             "--prefix", "none", "--format", "{p}"], cwd=ROOT, env=environment, text=True,
        )
        for line in tree.splitlines():
            match = re.match(r"^(\S+) v(\S+)", line)
            if match:
                enabled.add(match.groups())
    reachable = {i for i, p in packages.items() if (p["name"], p["version"]) in enabled}
    notice_root = GENERATED / "Notices"
    if notice_root.exists():
        shutil.rmtree(notice_root)
    notice_root.mkdir(parents=True)
    inventory = []
    for package in sorted((packages[i] for i in reachable), key=lambda p: p["name"]):
        if package.get("source") is None:
            continue
        source_root = Path(package["manifest_path"]).parent
        names = []
        destination = notice_root / f'{package["name"]}-{package["version"]}'
        destination.mkdir()
        for source in sorted(source_root.iterdir()):
            if source.is_file() and source.name.startswith(("LICENSE", "COPYING", "UNLICENSE")):
                shutil.copyfile(source, destination / source.name)
                names.append(source.name)
        if not names:
            raise ValueError(f'No license text found for compiled dependency {package["name"]}')
        inventory.append({"package": package["name"], "version": package["version"],
                          "declared_license": package.get("license"), "copied_texts": names})
    notices = {
        "IANA-LICENSE": ROOT / "crates/salah-time/fixtures/global/IANA-LICENSE",
        "TIMEZONE-BOUNDARY-ATTRIBUTION.md": ROOT / "data/third-party/tzf-2026d/ATTRIBUTION.md",
        "TIMEZONE-BOUNDARY-ODBL-LICENSE": ROOT / "data/third-party/tzf-2026d/LICENSE_DATA",
        "TZF-DIST-LICENSE": ROOT / "data/third-party/tzf-2026d/LICENSE",
        "TZF-RS-LICENSE": ROOT / "data/third-party/tzf-2026d/TZF-RS-LICENSE",
        "THIRD-PARTY.md": APPLE / "THIRD-PARTY.md",
        "GEONAMES-ATTRIBUTION.md": ROOT / "data/third-party/geonames-2026-10-01/ATTRIBUTION.md",
        "GEONAMES-README.txt": ROOT / "data/third-party/geonames-2026-10-01/readme.txt",
    }
    for name, source in notices.items():
        shutil.copyfile(source, notice_root / name)
    (notice_root / "license-inventory.json").write_text(json.dumps(inventory, indent=2) + "\n")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--framework-only", action="store_true", help="Do not build the Swift app")
    parser.add_argument("--device-app", action="store_true", help="Build unsigned device app instead of simulator")
    args = parser.parse_args()
    catalogue_manifest = json.loads((APPLE / "Salah/cities-manifest-v1.json").read_text())
    catalogue = (APPLE / "Salah/cities-v1.json").read_bytes()
    if len(catalogue) != catalogue_manifest["bytes"] or hashlib.sha256(catalogue).hexdigest() != catalogue_manifest["sha256"]:
        raise ValueError("Bundled offline city directory differs from its manifest")
    channel = tomllib.loads((ROOT / "rust-toolchain.toml").read_text())["toolchain"]["channel"]
    compiler = subprocess.check_output(["rustup", "which", "--toolchain", channel, "rustc"], text=True).strip()
    environment = os.environ.copy()
    environment["RUSTC"] = compiler
    environment["IPHONEOS_DEPLOYMENT_TARGET"] = "17.0"
    GENERATED.mkdir(parents=True, exist_ok=True)
    libraries = []
    for target in TARGETS:
        run(["rustup", "run", channel, "cargo", "build", "--locked", "--offline",
             "--release", "--target", target, "-p", "salah-ffi"], env=environment)
        library = ROOT / "target" / target / "release/libsalah_ffi.a"
        libraries.append(library)
    framework = GENERATED / "SalahEngine.xcframework"
    if framework.exists():
        shutil.rmtree(framework)
    simulator = GENERATED / "libsalah_ffi_sim.a"
    run(["xcrun", "lipo", "-create", str(libraries[1]), str(libraries[2]), "-output", str(simulator)])
    command = ["xcodebuild", "-create-xcframework"]
    for library in (libraries[0], simulator):
        command.extend(["-library", str(library), "-headers", str(ROOT / "crates/salah-ffi/include")])
    run(command + ["-output", str(framework)])
    licenses(channel, environment)
    manifest = {
        "schema": "salah-apple-engine-build-v1", "abi": 1,
        "rust_toolchain": channel, "minimum_ios": "17.0",
        "cargo_lock_sha256": hashlib.sha256((ROOT / "Cargo.lock").read_bytes()).hexdigest(),
        "city_catalogue_sha256": catalogue_manifest["sha256"],
        "libraries": [
            {"target": target, "bytes": library.stat().st_size,
             "sha256": hashlib.sha256(library.read_bytes()).hexdigest()}
            for target, library in zip(TARGETS, libraries)
        ],
    }
    (GENERATED / "engine-build-manifest.json").write_text(json.dumps(manifest, indent=2) + "\n")
    if not args.framework_only:
        sdk = "iphoneos" if args.device_app else "iphonesimulator"
        destination = "generic/platform=iOS" if args.device_app else "generic/platform=iOS Simulator"
        run(["xcodebuild", "-quiet", "-project", str(APPLE / "Salah.xcodeproj"), "-scheme", "Salah",
             "-configuration", "Debug", "-sdk", sdk, "-destination", destination,
             "-derivedDataPath", str(ROOT / "target/apple-xcode"),
             "CODE_SIGNING_ALLOWED=NO", "SWIFT_TREAT_WARNINGS_AS_ERRORS=YES", "build"])
        product = ROOT / f"target/apple-xcode/Build/Products/Debug-{sdk}/Salah.app"
        print(f"Built unsigned research app: {product}")
    print(f"Bundled engine built: {framework}")
    return 0


if __name__ == "__main__":
    try:
        sys.exit(main())
    except (OSError, ValueError, subprocess.CalledProcessError) as error:
        print(f"Apple build failed: {error}", file=sys.stderr)
        sys.exit(1)
