#!/usr/bin/env python3
"""Run the Apple bridge on a booted simulator and compare CLI/WASM records.

Developer check only. Does not request location, change radios, or prove GPS.
Build the Apple framework and web package first; requires Xcode and Node.js.
"""

import argparse
import json
import os
from pathlib import Path
import platform
import shutil
import subprocess
import tomllib
import uuid

ROOT = Path(__file__).resolve().parent.parent


def run(command, **kwargs):
    return subprocess.run(command, cwd=ROOT, check=True, **kwargs)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--simulator", required=True, help="UUID of an already booted iOS simulator")
    parser.add_argument("--places", action="store_true", help="Check offline cities/private storage in the installed app's disposable test folder")
    args = parser.parse_args()
    arm = platform.machine() == "arm64"
    target = "aarch64-apple-ios-sim" if arm else "x86_64-apple-ios"
    swift_target = "arm64-apple-ios17.0-simulator" if arm else "x86_64-apple-ios17.0-simulator"
    sdk = subprocess.check_output(["xcrun", "--sdk", "iphonesimulator", "--show-sdk-path"], text=True).strip()
    swift_environment = os.environ.copy()
    swift_environment["SDKROOT"] = sdk
    probe = ROOT / "target/check-apple-bridge"
    fixtures_path = ROOT / "target/apple-fixtures.json"
    run(["xcrun", "swiftc", "-swift-version", "6", "-warnings-as-errors", "-sdk", sdk,
         "-target", swift_target, "-import-objc-header", "apps/apple/Salah/NativeBridge.h",
         "-Xcc", "-Icrates/salah-ffi/include", "apps/apple/Salah/RustEngine.swift",
         "apps/apple/Salah/EngineDocuments.swift", "tools/check_apple_bridge.swift",
         str(ROOT / f"target/{target}/release/libsalah_ffi.a"), "-o", str(probe)], env=swift_environment)
    run(["xcrun", "simctl", "spawn", args.simulator, str(probe), str(fixtures_path)])
    fixtures = json.loads(fixtures_path.read_text())
    channel = tomllib.loads((ROOT / "rust-toolchain.toml").read_text())["toolchain"]["channel"]
    environment = os.environ.copy()
    environment["RUSTC"] = subprocess.check_output(
        ["rustup", "which", "--toolchain", channel, "rustc"], text=True).strip()
    for name, record in fixtures.items():
        coordinates = record["coordinates"]
        command = ["rustup", "run", channel, "cargo", "run", "--quiet", "--locked", "--offline",
                   "-p", "salah-cli", "--", "schedule", "--json",
                   "--lat", str(coordinates["latitude_degrees"]),
                   "--lon", str(coordinates["longitude_degrees"]),
                   "--date", record["requested_local_date"], "--zone", record["zone_id"],
                   "--method", record["method"]["id"], "--asr", record["method"]["asr"]]
        cli = json.loads(subprocess.check_output(command, cwd=ROOT, env=environment, text=True))
        if cli != record:
            raise AssertionError(f"CLI / Simulator record differs: {name}")
    run(["node", "--input-type=module", "-"], input=r'''
import {readFileSync} from 'node:fs';
import {pathToFileURL} from 'node:url';
const bridge = await import(pathToFileURL(process.cwd() + '/apps/web/pkg/salah_wasm.js'));
bridge.initSync({module: readFileSync('apps/web/pkg/salah_wasm_bg.wasm')});
const fixtures = JSON.parse(readFileSync('target/apple-fixtures.json', 'utf8'));
const canonical = v => Array.isArray(v) ? v.map(canonical) : v && typeof v === 'object'
    ? Object.fromEntries(Object.keys(v).sort().map(k => [k, canonical(v[k])])) : v;
for (const [name, record] of Object.entries(fixtures)) {
    const request = {schema: 'salah-schedule-request-v2', ...record.coordinates,
        local_date: record.requested_local_date, zone_id: record.zone_id,
        method_id: record.method.id, asr: record.method.asr, zone_choice: 'manual'};
    const result = JSON.parse(bridge.calculate_schedule_with_selection_json(JSON.stringify(request)));
    if (JSON.stringify(canonical(result.schedule)) !== JSON.stringify(canonical(record)))
        throw Error('WASM / Simulator record differs: ' + name);
    console.log(name + ': complete CLI / WASM / iOS Simulator record equality');
}
''', text=True)
    if args.places:
        bundle = subprocess.check_output(["xcrun", "simctl", "get_app_container", args.simulator,
                                         "org.shoaibsprojects.salah.preview", "app"], text=True).strip()
        container = subprocess.check_output(["xcrun", "simctl", "get_app_container", args.simulator,
                                            "org.shoaibsprojects.salah.preview", "data"], text=True).strip()
        scratch = Path(container) / "Library/Caches" / f"SalahAcceptance-{uuid.uuid4()}"
        places_probe = ROOT / "target/check-apple-places"
        run(["xcrun", "swiftc", "-swift-version", "6", "-warnings-as-errors", "-sdk", sdk,
             "-target", swift_target, "-import-objc-header", "apps/apple/Salah/NativeBridge.h",
             "-Xcc", "-Icrates/salah-ffi/include", "apps/apple/Salah/RustEngine.swift",
             "apps/apple/Salah/EngineDocuments.swift", "apps/apple/Salah/SetupModel.swift",
             "apps/apple/Salah/DeviceLocation.swift", "apps/apple/Salah/SavedPlaces.swift",
             "apps/apple/Salah/CityDirectory.swift", "tools/check_apple_places.swift",
             str(ROOT / f"target/{target}/release/libsalah_ffi.a"), "-o", str(places_probe)], env=swift_environment)
        try:
            run(["xcrun", "simctl", "spawn", args.simulator, str(places_probe), bundle, str(scratch)])
        finally:
            if scratch.exists():
                shutil.rmtree(scratch) # Only this newly named disposable acceptance folder.
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
