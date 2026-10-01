# Third-party notices for the local web preview

The generated `pkg/licenses/` directory contains the exact bundled IANA notice,
the timezone-boundary attribution and data licenses, and license texts found
at the roots of the locked third-party Rust packages. `pkg/license-inventory.json`
records package versions, declared SPDX expressions, and copied filenames.
That inventory covers the workspace dependency graph, including packages that
the WebAssembly linker may omit; it is not an audited minimal binary inventory.
Preserve these notices when copying the preview folder.

- IANA timezone snapshot: 2026d; notice at `pkg/licenses/IANA-LICENSE`.
- Community timezone boundaries: timezone-boundary-builder / OpenStreetMap
  contributors, distributed by tzf-dist 0.0.2026-d-fix1. Direct manual zone choice
  does not query these polygons. Source attribution and ODbL text are retained
  at `pkg/licenses/TIMEZONE-BOUNDARY-ATTRIBUTION.md` and
  `pkg/licenses/TIMEZONE-BOUNDARY-ODBL-LICENSE`.
- wasm-bindgen 0.2.129: MIT OR Apache-2.0; it generates the JavaScript boundary.
  It is a library/build tool, not a remote calculation service.

The project source-code license remains undecided. Generated notices do not
grant a project license or complete a legal/release review. See the repository's
README and stewardship roadmap before public distribution.
