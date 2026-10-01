# Apple offline place setup v0.1

**Piece:** Phase 4 → **F4-A2: Offline Places and iPhone 12 Compatibility**.
**Date:** 1 October 2026. Research implementation; the [roadmap](../docs/roadmap.md)
owns release gates. This extends the [Apple foundation](apple-foundation-v0.1.md)
and [offline operating contract](../docs/offline-product-contract.md).

## Supported platform and acquisition

Target iPhone 12/12 mini/12 Pro/12 Pro Max and newer iPhones on **iOS 17+**;
iPad support remains. This is a deployment/support target, not evidence that
every physical model/OS combination has passed. No Apple Intelligence, dual-band
GNSS, LiDAR, paid API or new-model-only feature is required. Apple's
[iPhone 12 technical specifications](https://support.apple.com/en-us/111876),
retrieved 2026-10-01, list built-in GPS, GLONASS, Galileo, QZSS and BeiDou.
Core Location still chooses sources; this hardware fact does not prove a fix
will succeed indoors or with all radios disabled.

Location remains optional and foreground-only. The normal request retains a
30-second bound and 100-metre desired accuracy. An explicit precise attempt
uses Apple's best desired accuracy and a 90-second bound, giving acquisition
more time without a background loop. These include permission-prompt time.
The same −5…+60-second fix-age policy applies. A satellite cold start can exceed
these limits; a failed fix leaves city and saved-place use available.

If approximate authorization is active, the precise action may request
temporary full accuracy using purpose `PrayerLocation`, declared in Info.plist.
This API is supported on the project's minimum OS. Apple's
[temporary-accuracy documentation](https://developer.apple.com/documentation/corelocation/cllocationmanager/requesttemporaryfullaccuracyauthorization(withpurposekey:completion:)),
retrieved 2026-10-01, says completion occurs for grant, decline or prompt error.
The callback checks a request UUID on the main actor; cancellation or a newer
request rejects it. Decline still permits a visibly approximate estimate.
Permission-sheet inactivity is distinguished from actual backgrounding.
Choosing a city cancels a pending location request. No GNSS-only source or
precision guarantee is introduced.

## Offline city directory

Bundle **34,152** GeoNames city reference points from pinned `cities15000.zip`
and `countryInfo.txt`, retrieved 2026-10-01. The provider describes larger towns
and capitals; this is not every village/address. Names and bounded source
aliases support Unicode/diacritic-insensitive search and country labels to
distinguish places with the same name. Display at most 40 source-ordered matches;
never choose a city from the device timezone or silently infer a user's place.

The person explicitly chooses a city. Its recorded WGS84 reference point fills
coordinates and is labeled **approximate**, not a device fix or mathematical
centroid. Rust supplies timezone candidates; current confirmation/manual
correction remains mandatory. GeoNames timezone columns are not used. A nearby
town's schedule is an approximation for someone elsewhere; device/manual input
can refine it. Neither city selection nor GPS supplies a method/Asr default.

The exact catalogue is 6,806,396 bytes, SHA-256
`085e06b69c8b0e435177680c05d29bdfbc0cba9722b0a2d3f678478426940878`.
Its bundled manifest and [source attribution/pins](../data/third-party/geonames-2026-10-01/ATTRIBUTION.md)
record CC BY 4.0, retrieval and transformation. The offline generator verifies
raw sources; the app verifies the manifest/size/hash before decoding. This is
consistency evidence inside the installed app, not a separate publisher signature.
No online geocoder/search is called. Attribution/terms are bundled; the optional
credit link opens an external page only if chosen.

Load/search on a serial actor after a 250ms typing debounce; cap query length to
160 characters, source rows to 50,000 and bytes to 8 MB. Release the in-memory
index when the city sheet closes. Geographic calculation stays in Rust.

## Explicit saved places

After one successful calculation, the person may save a named place and choose
whether to use it at startup. Store coordinates, original source/fix metadata,
confirmed zone, explicit method ID/revision, Asr, rule-pack hash and relevant
boundary hash. Do not store timetables or a manual date as the new day's answer.
On reopening, use the device UTC instant and Rust's selected-zone rules to
compute today. A saved place is visibly stored; it is never a fresh location fix.
Travel requires selecting another place or requesting location.

Reuse zone confirmation only if the zone still exists and the exact rule pack
matches. If the confirmation relied on map geometry, recheck the pinned boundary
hash and point membership in Rust. Changed/missing data requires confirmation.
A changed method revision rejects the calculated record and requires a new
explicit profile choice. Stored city coordinates remain the user's original
selection; a new catalogue never silently moves them. Startup replies cannot
overwrite newer manual input or acquisition.

Schema `salah-saved-places-v1`, at most 20 distinct UUID records, 80-character
names, bounded/finite coordinates and 64 KiB JSON. A singleton storage actor
reads and atomically replaces app-private `Application Support/SalahPlaces/places-v1.json`.
Malformed/unknown-version/oversized data fails without being overwritten; no
fallback record is invented. The person can delete places or explicitly reset
saved copies. Batch deletion also removes the matching startup choice.

On physical iOS, write with `Data.WritingOptions.completeFileProtection`, apply
0700/0600 directory/file permissions, and verify the protection attribute. Apple
[documents](https://developer.apple.com/documentation/foundation/nsdata/writingoptions/completefileprotection)
that this class encrypts the file and permits access only while unlocked.
Exclude the support directory from backup and verify the flag. Saving fails
explicitly if required protections cannot be confirmed. The Simulator requests
the protection option but cannot establish device encryption; attribute
verification is a physical-device requirement, not a Simulator claim.

No UserDefaults, iCloud container, location history, cloud sync or remote logging
is added. `C617.1` is declared for the app's own container/bundle metadata reads;
this does **not** resolve the separately documented linked-Rust symbolication
privacy question. Store distribution remains blocked pending that review.

## Acceptance and remaining gates

`tools/check_apple.py --simulator UUID --places` uses a fresh disposable folder
in the installed research app's cache container, not its real saved-place file.
It checks source aliases/country distinctions, accent search, city-to-Rust
timezone/schedule, save/reopen/startup, old-source labeling, startup edit races,
rule/map/method invalidation, deletion, corrupt/oversized input preservation and
explicit reset. It also retains the six complete CLI/WASM/Simulator comparisons.
No permission or location request occurs during this check.

On 2026-10-01 these model/storage checks and six complete record comparisons
passed on iPhone 12 and iPhone 17 profiles running iOS 26.0 (23A343). Unsigned
device and universal Simulator builds passed with Swift warnings treated as
errors. This does not establish real A14 performance or hardware acceptance.

Physical iPhone 12 and latest-model acceptance, actual locked-device protection,
permission/temporary-accuracy prompts, radio-off GPS, native control interaction,
VoiceOver/RTL and Store/privacy review remain open. Simulator profiles establish
named software configurations only. Phase 1/2 scientific and methodology gates,
Qibla/reminders and public release are unchanged.
