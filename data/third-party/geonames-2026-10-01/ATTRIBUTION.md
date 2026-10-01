# GeoNames offline city snapshot

City reference points, population/order fields, source spellings/aliases and
country names are from [GeoNames](https://www.geonames.org/), retrieved
**2026-10-01** from its [daily export directory](https://download.geonames.org/export/dump/).
The pinned provider readme licenses this work under
[Creative Commons Attribution 4.0](https://creativecommons.org/licenses/by/4.0/).
Keep this credit and license link with redistributed data. GeoNames does not
endorse Salah or its calculated times.

| Pinned source | SHA-256 |
| --- | --- |
| `cities15000.zip` | `f00213685cfe4ee1deeb7a4e52ce6258ac99d7b27209ffca0e8590ccab433763` |
| `countryInfo.txt` | `93bafc525813f22e4711ff9ed6d626343094ce48c26388dc7c49189b3d7d5512` |
| `readme.txt` | `b1957379b6c1242c700c98ac9a8aa0a09f56c3c0a50ee72175527005f48ef2c5` |

`tools/build_city_directory.py` performs a documented transformation using
only these local pinned files: retain identifiers, canonical/ASCII names,
country identity, WGS84 coordinates, population and up to 24 bounded source
aliases; sort by population then identifier; encode compact JSON. The exact
generated artifact hash/count/size are in the bundled city manifest.

The source selection covers larger towns and capitals. It is not a complete
street/village directory. Coordinates are approximate city reference points,
not a person's device position, city centroid guarantee, or legal timezone
boundary. GeoNames timezone columns are not used: Rust's existing map/rules
and explicit confirmation determine the application's timezone choice.

The provider readme and country table are stored verbatim. Full CC legal text
was not copied because its server rejected the research fetch; the provider's
license statement and official license link are retained. The generated JSON
is an adapted subset; it does not combine or redefine the kernel's astronomy.
