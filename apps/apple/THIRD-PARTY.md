# Bundled research-app notices

Salah's Apple app links the same offline Rust calculation engine as the web
preview. The kernel has no third-party crate dependencies. The bridge,
timezone rules, and approximate boundary lookup use the dependencies pinned
by the repository's Cargo.lock.

The build bundles `Notices/license-inventory.json` and available package
license texts for the Apple bridge's active Cargo normal/build dependency trees
across its three targets (including build tools, not a claim that every package
is linked). Missing license text stops this development builder.
It also includes the IANA license and timezone-boundary attribution/ODbL
texts. This folder contains public notices only, never personal location.

The timezone-boundary data is approximate community geometry, not an
authoritative legal boundary map. A suggestion requires confirmation.
Calculation-method provenance and unresolved scientific/methodology review
remain documented in the repository. No App Store or consumer accuracy
approval follows from this development package.

The project's own source license and release stewardship remain undecided;
copied third-party notices do not grant a project license.
