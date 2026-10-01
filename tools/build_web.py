#!/usr/bin/env python3
"""Build the local browser artifact; no package manager or CDN is required."""

import hashlib
import json
import os
import shutil
from pathlib import Path
import subprocess
import sys
import tomllib

ROOT = Path(__file__).resolve().parent.parent
BINDGEN_VERSION = "0.2.129"


def main():
    try:
        version = subprocess.check_output(
            ["wasm-bindgen", "--version"], text=True
        ).strip()
        if version != f"wasm-bindgen {BINDGEN_VERSION}":
            raise RuntimeError(
                f"Expected wasm-bindgen {BINDGEN_VERSION}, got {version!r}. "
                f"Install with: cargo install --locked wasm-bindgen-cli --version {BINDGEN_VERSION}"
            )
        channel = tomllib.loads((ROOT / "rust-toolchain.toml").read_text(encoding="utf-8"))["toolchain"]["channel"]
        compiler = subprocess.check_output(["rustup", "which", "--toolchain", channel, "rustc"], text=True).strip()
        build_env = os.environ.copy()
        build_env["RUSTC"] = compiler
        subprocess.run(
            ["rustup", "run", channel,
             "cargo", "build", "--locked", "--offline", "--release",
             "--target", "wasm32-unknown-unknown", "-p", "salah-wasm"],
            cwd=ROOT, check=True, env=build_env,
        )
        output = ROOT / "apps/web/pkg"
        output.mkdir(parents=True, exist_ok=True)
        subprocess.run(
            ["wasm-bindgen", str(ROOT / "target/wasm32-unknown-unknown/release/salah_wasm.wasm"),
             "--target", "web", "--out-dir", str(output), "--out-name", "salah_wasm"],
            cwd=ROOT, check=True,
        )
        metadata = json.loads(subprocess.check_output(
            ["rustup", "run", channel, "cargo", "metadata", "--locked", "--offline", "--format-version", "1"],
            cwd=ROOT, env=build_env, text=True,
        ))
        license_inventory = []
        for package in metadata["packages"]:
            if package.get("source") is None:
                continue
            package_root = Path(package["manifest_path"]).parent
            destination = output / "licenses" / f'{package["name"]}-{package["version"]}'
            destination.mkdir(parents=True, exist_ok=True)
            names = set()
            for pattern in ("LICENSE*", "COPYING*", "UNLICENSE*"):
                for source in package_root.glob(pattern):
                    if source.is_file():
                        shutil.copyfile(source, destination / source.name)
                        names.add(source.name)
            license_inventory.append({"package": package["name"], "version": package["version"],
                                      "declared_license": package.get("license"), "copied_texts": sorted(names)})
        data_notices = {
            "IANA-LICENSE": ROOT / "crates/salah-time/fixtures/global/IANA-LICENSE",
            "TIMEZONE-BOUNDARY-ATTRIBUTION.md": ROOT / "data/third-party/tzf-2026d/ATTRIBUTION.md",
            "TIMEZONE-BOUNDARY-ODBL-LICENSE": ROOT / "data/third-party/tzf-2026d/LICENSE_DATA",
            "TZF-DIST-LICENSE": ROOT / "data/third-party/tzf-2026d/LICENSE",
            "TZF-RS-LICENSE": ROOT / "data/third-party/tzf-2026d/TZF-RS-LICENSE",
        }
        notice_directory = output / "licenses"
        notice_directory.mkdir(parents=True, exist_ok=True)
        for name, source in data_notices.items():
            shutil.copyfile(source, notice_directory / name)
        (output / "license-inventory.json").write_text(json.dumps(license_inventory, indent=2) + "\n", encoding="utf-8")
        artifacts = {}
        for name in ("salah_wasm.js", "salah_wasm_bg.wasm", "salah_wasm.d.ts", "salah_wasm_bg.wasm.d.ts"):
            data = (output / name).read_bytes()
            artifacts[name] = {"sha256": hashlib.sha256(data).hexdigest(), "bytes": len(data)}
        manifest = {"schema": "salah-web-build-v1", "wasm_bindgen_version": BINDGEN_VERSION,
                    "rust_toolchain": channel,
                    "cargo_lock_sha256": hashlib.sha256((ROOT / "Cargo.lock").read_bytes()).hexdigest(),
                    "artifacts": artifacts}
        (output / "build-manifest.json").write_text(json.dumps(manifest, indent=2) + "\n", encoding="utf-8")
        print("Browser artifact built in apps/web/pkg.")
        print("Serve locally: python3 -m http.server 8080 --bind 127.0.0.1 --directory apps/web")
    except (OSError, RuntimeError, subprocess.CalledProcessError) as exc:
        print(f"Web build failed: {exc}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
