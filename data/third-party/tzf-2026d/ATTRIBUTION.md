# Time-zone boundary data attribution

The bundled coordinate-to-time-zone database is distributed by `tzf-dist` 0.0.2026-d-fix1 from timezone-boundary-builder release 2026d. Most geographic input data comes from OpenStreetMap contributors, assembled and quality-controlled by the timezone-boundary-builder project.

- Data source: <https://github.com/evansiroky/timezone-boundary-builder/releases/tag/2026d>
- Pinned artifact versions and hashes: [`manifest.json`](manifest.json)
- Upstream build statistics and geometry comparison: [`STATS.md`](STATS.md), [`BORDER_CHANGE.md`](BORDER_CHANGE.md)
- OpenStreetMap attribution: <https://www.openstreetmap.org/copyright>
- `tzf-rs` lookup implementation: <https://github.com/ringsaturn/tzf-rs/tree/v2.1.2>
- Data license: see [`LICENSE_DATA`](LICENSE_DATA) (ODbL-1.0)
- Code license: see [`LICENSE`](LICENSE) and [`TZF-RS-LICENSE`](TZF-RS-LICENSE) (MIT)

The data is approximate community-maintained timezone geography. It is not an official boundary register. Shipped applications that embed the database must include suitable source attribution and applicable license notices in an accessible legal/about surface.
