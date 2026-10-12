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


## Prayer learning companion

Main reading reference: *A Simple Guide to the Prophet’s Prayer*, Abu Khadeejah Abdul-Wāhid Alam, Salafi Publications, third edition (November 2021), ISBN 9781902727585. Original explanatory notes cite printed page numbers. The publisher’s PDF and illustrations are not redistributed. See the in-app Sources section for copyright conditions and the original author link. No endorsement is claimed.

Qur’an 1:1–7 and 112:1–4: ar-uthmani-hafs text and Muhammad A. S. Abdel Haleem’s English translation (en-abdel-haleem), retrieved through quran.ai on 2026-10-11. Short passages are attributed study excerpts. Transliteration and non-Qur’an English meanings are editorial reading aids.

Offline city search reuses the native app’s adapted GeoNames snapshot, retrieved 2026-10-01, licensed [CC BY 4.0](https://creativecommons.org/licenses/by/4.0/). Source: [GeoNames](https://www.geonames.org/), cities15000 and countryInfo. Selection, alias limits, and ordering are performed by tools/build_city_directory.py. Coordinates are approximate reference points, not device locations. GeoNames does not endorse Salah.
