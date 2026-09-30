# Offline location-to-time-zone mapping

The experimental `salah-location` crate turns validated coordinates into a sorted set of suggested IANA time-zone names using a bundled community boundary map. It performs the lookup offline. The caller must explicitly confirm a suggestion or choose a zone manually; the mapping never silently chooses among matches.

The data is approximate and is not a legal boundary authority. The 2026d map includes ocean polygons and may assign an `Etc/GMT` zone on open water; this is not a shipboard or aircraft timekeeping policy. Simplified geometry can place points close to borders in the neighboring zone. Keep manual time-zone selection available even when the dataset returns one match. The IANA `2026d` rule pack must match the boundary release; mismatch fails closed.

The intended calling pattern keeps suggestion and choice separate:

```rust
use salah_core::Coordinates;
use salah_location::lookup_timezone_candidates;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let coordinates = Coordinates::new(44.9778, -93.2650)?;
    let candidates = lookup_timezone_candidates(coordinates)?;
    // Show candidates to the user. Even a single result is not selected implicitly.
    let selected_zone = candidates.confirm_suggestion("America/Chicago")?;
    let zone_id = selected_zone.zone_id();
    println!("Selected time zone: {zone_id}");
    Ok(())
}
```

If the mapped candidate is wrong or the map has no result, use `manual_override` with any identifier from the bundled IANA zone pack. The returned selection keeps the original coordinates, candidates, data versions, and whether the user confirmed a suggestion or overrode it.

See the [versioned contract](../specification/coordinate-zone-selection-contract-v0.1.md) for source, license, data-version, accuracy, manual-override, and maintenance details. These working-tree changes do not pass the Phase 2 gate or authorize a consumer accuracy claim.
