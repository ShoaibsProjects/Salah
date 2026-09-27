# Method register v1

**Status:** Phase 1 P1.4 provenance evidence for `salah-core` v0.3.0. This register traces each named profile to its source and records review state. It is not a religious certification, a regional default, or a global accuracy claim. The machine-readable companion is [`method-sources-v1.tsv`](../data/reference/method-sources-v1.tsv). Behavior is defined by the [calculation contract v0.3](calculation-contract-v0.3.md); kernel parameters were read from `crates/salah-core/src/method.rs` and are repeated here, not changed. No qualified Islamic-methodology review has occurred; every user-facing wording below is **Pending** such review.

## 1. Profile parameters and per-parameter sources

### 1.1 `research-15` revision 0.1 — non-institutional research set

| Parameter | Kernel value | Source | Edition / URL | Date | Hash |
| --- | --- | --- | --- | --- | --- |
| Fajr depression | 15.0° | Salah calculation contract v0.1; `MethodProfile::research_15()` | Repo file `specification/calculation-contract-v0.1.md`; `crates/salah-core/src/method.rs` | `unknown`: contract v0.1 carries no publication date | `not_applicable`: no external file hashed |
| Isha depression | 15.0° | Same as Fajr | Same as Fajr | `unknown`: same reason | `not_applicable`: same reason |
| Dhuhr adjustment | 0 s | Same as Fajr | Same as Fajr | `unknown`: same reason | `not_applicable`: same reason |
| Maghrib adjustment | 0 s | Same as Fajr | Same as Fajr | `unknown`: same reason | `not_applicable`: same reason |
| Asr applicability | Standard and Hanafi supported; shadow factor 1 or 2 with noon baseline fixed at upper transit | Calculation contract v0.3; `AsrCriterion` in `method.rs` | `specification/calculation-contract-v0.3.md` | Contract v0.3 is the current research implementation record; no separate publication date | `not_applicable`: no external file hashed |
| Missing-event behavior | `Unavailable` typed reason; no substituted time | Calculation contract v0.3 | `specification/calculation-contract-v0.3.md` | Same as Asr row | `not_applicable`: same reason |
| High-latitude rule | `none`; no fallback is generated | Calculation contract v0.3 | `specification/calculation-contract-v0.3.md` | Same as Asr row | `not_applicable`: same reason |
| Region / community | `not_applicable`: research profile has no region or community | — | — | — | — |
| Ramadan / interval behavior | `not_applicable`: set defines Fajr/Isha angles only; no Ramadan or fixed-interval rule | — | — | — | — |
| Kernel implements Ramadan / interval | No: kernel v0.3.0 supports angle-based Fajr/Isha plus second adjustments only | Calculation contract v0.3 | `specification/calculation-contract-v0.3.md` | Same as Asr row | `not_applicable`: same reason |
| Intended use | Non-institutional engineering comparison and regression baseline only | This register | — | 2026-09-27 (register creation) | — |

An institutional primary source is `not_applicable` to `research-15` by design: it is a non-institutional research set, not offered to consumers. Neither PrayTimes artifact in §2 is a source for this profile.

### 1.2 `mwl-angles-18-17` revision 0.1 — published parameter set, not an endorsement

| Parameter | Kernel value | Source | Edition / URL | Date | Hash |
| --- | --- | --- | --- | --- | --- |
| Fajr depression | 18.0° | SECONDARY: PrayTimes methods-table webpage (§2a) and v2 JS file (§2b) | `https://praytimes.org/docs/methods`; `https://praytimes.org/code/v2/js/PrayTimes.js` | Webpage: retrieved 2026-09-27 (architect retrieval recorded; independently verified by a second fetch the same day confirming the MWL 18/17 row). JS file: retrieved 2026-09-27 per [prayer-library report](prayer-library-v1-report.md); live bytes re-fetched 2026-09-27 match the pinned hash | Webpage: hash does not apply. JS file: SHA-256 `f0e8442410446b1bf2db613bdebd3aa3aef5cdf5483068fbd9212d5acaa180fd` (file only) |
| Isha depression | 17.0° | Same secondary pair | Same URLs | Same dates | Same hashes |
| Dhuhr adjustment | 0 s | Kernel zero adjustment per `mwl_angles_18_17()`; P1.3 comparison ran `dhuhr='0 min'` | `crates/salah-core/src/method.rs`; comparison options in [prayer-library report](prayer-library-v1-report.md) | JS retrieval 2026-09-27 | JS file hash as above (comparison-run evidence, not a kernel source) |
| Maghrib adjustment | 0 s | Kernel zero adjustment; P1.3 comparison ran `maghrib='0 min'` | Same as Dhuhr row | Same as Dhuhr row | Same as Dhuhr row |
| Asr applicability | Standard and Hanafi supported; shadow factor 1 or 2 with noon baseline fixed at upper transit | Calculation contract v0.3; `AsrCriterion` in `method.rs` | `specification/calculation-contract-v0.3.md` | Current research implementation record | `not_applicable`: no external file hashed |
| Missing-event behavior (Salah) | `Unavailable` typed reason; no substituted time | Calculation contract v0.3 | `specification/calculation-contract-v0.3.md` | Same as Asr row | `not_applicable`: same reason |
| High-latitude rule (Salah) | `none`; no fallback is generated | Calculation contract v0.3 | `specification/calculation-contract-v0.3.md` | Same as Asr row | `not_applicable`: same reason |
| External high-latitude behavior (separate) | The pinned file's default is `highLats: 'NightMiddle'`; P1.3 explicitly overrode it to `'None'`, under which PrayTimes returns `NaN`, recorded as `no_event`. The MWL entry itself carries no high-latitude rule. Adhan JS 4.4.6 applies its `MiddleOfTheNight` safety bound and is labeled `angle` vs `fallback`. Institutional high-latitude practice is not inferred from either library. | [Prayer-library report](prayer-library-v1-report.md); `tools/generate_prayer_matrix.cjs`; pinned JS bytes (hash-matched live re-fetch 2026-09-27) | Adhan `adhan` 4.4.6; PrayTimes v2 JS file above | Source retrieval and calculation 2026-09-27 | PrayTimes hash as above; Adhan tarball integrity `sha512-cpLLcr6qxx+mLc6jgSXJwAAHxDFrhXhVLuzRbZPrpQKIEpJw+0LcfG6u9WEjq66kcTX/2sCkI3VztgRoUXBnFg==` |
| Region / community | SECONDARY-source description only: “Europe, Far East, parts of US” per the methods-table webpage row for MWL. Primary institutional applicability is unverified. | Methods-table webpage (§2a) | `https://praytimes.org/docs/methods` | Retrieved 2026-09-27 (architect + independent verification) | Webpage: hash does not apply |
| Ramadan / interval behavior | The pinned MWL entry is angle-based (`fajr: 18, isha: 17`) with no fixed Isha interval or Ramadan branch in that entry — verified 2026-09-27 against hash-matched JS bytes. Institutional Ramadan practice is not inferred. | Pinned v2 JS source file (§2b) | `https://praytimes.org/code/v2/js/PrayTimes.js` | Verified 2026-09-27 | JS file hash as above (file only) |
| Kernel implements Ramadan / interval | No: kernel v0.3.0 supports angle-based Fajr/Isha plus second adjustments only | Calculation contract v0.3 | `specification/calculation-contract-v0.3.md` | Current research implementation record | `not_applicable`: same reason |
| Intended use | Published-parameter-set comparison label only; not an endorsement by the Muslim World League, not a regional default, not a complete timetable | This register; [decision record](../docs/decisions.md) | — | 2026-09-27 (register creation) | — |

## 2. The two PrayTimes artifacts, recorded separately

These are both **SECONDARY** sources. They must never be conflated.

- (a) **Methods-table webpage** — `https://praytimes.org/docs/methods`. Human-readable table listing 18° Fajr and 17° Isha for MWL, with the MWL region described as “Europe, Far East, parts of US” (SECONDARY description only; primary institutional applicability unverified). Retrieval: architect fetch recorded 2026-09-27; an independent verification fetch by this agent the same day confirmed the MWL 18/17 row and region text. The JS-file SHA-256 below does **not** apply to this webpage.
- (b) **v2 JS source file** — `https://praytimes.org/code/v2/js/PrayTimes.js`, SHA-256 `f0e8442410446b1bf2db613bdebd3aa3aef5cdf5483068fbd9212d5acaa180fd`. This hash identifies **the JS file only**, never the webpage. Retrieved 2026-09-27 per the [prayer-library report](prayer-library-v1-report.md) and [`data/reference/README.md`](../data/reference/README.md); live bytes re-fetched 2026-09-27 matched the pinned hash exactly. The file serves both as the audited P1.3 comparison implementation and as corroborating secondary evidence for the 18/17 angles: its MWL entry reads `params: { fajr: 18, isha: 17 }` (angle-based, no fixed Isha interval or Ramadan branch in that entry), its default setting is `highLats: 'NightMiddle'`, and P1.3 explicitly overrode that default to `'None'` with angles overwritten to 18/17, `dhuhr='0 min'`, `maghrib='0 min'`.

## 3. Primary-source gaps and search scope

- Primary institutional confirmation for the 18°/17° angles has **not been established**. Search scope in P1.4-M1 was limited to the two secondary PrayTimes artifacts above plus existing repo evidence (checked 2026-09-27). No systematic primary-literature search was performed, so the absence of a primary document anywhere is **not** established. The `mwl-angles-18-17` name therefore describes a twice-corroborated secondary parameter set only.
- Upstream default high-latitude rules beyond the pinned file's `NightMiddle` default, Ramadan behavior beyond the angle-based MWL entry, and other fixed-interval variants were not verified and remain `unknown` where they exceed the P1.3 run configuration. Institutional practice is not inferred from either library.
- If a primary source is found later, or if it differs from the secondary table, this register must gain a new version; the existing rows must not be silently overwritten.

## 4. Short user-facing explanations (all Pending qualified Islamic-methodology review)

(a) **Method choice** — *Pending.* Where the app offers a choice of twilight-angle conventions, each option states its angles and source. The `research-15` baseline is an engineering-only reference with no religious attribution and is not offered as a consumer method. The 18°/17° option reproduces two twilight angles shown in a published secondary table; it is not an endorsement, a regional default, or a complete timetable. Different angles give different calculated beginnings. Sources: [calculation contract v0.3](calculation-contract-v0.3.md) (repo record checked 2026-09-27) and [PrayTimes methods table](https://praytimes.org/docs/methods) (retrieved 2026-09-27); consumer wording remains a proposal pending review.

(b) **Calculated beginning vs mosque timetable and iqamah** — *Pending.* A calculated beginning is the result of the selected solar condition (for example, a twilight depression, the apparent horizon, solar transit, or an Asr shadow target) combined with the selected method's rules and adjustments. A mosque timetable and its scheduled iqamah are separate local information and may differ from the calculated beginning. Confirm congregation arrangements with the local mosque. Source: [calculation contract v0.3](calculation-contract-v0.3.md) (repo record checked 2026-09-27); wording remains a proposal pending review.

(c) **Unavailable events** — *Pending.* When the selected solar condition has no crossing in the local solar cycle on a given day, the app reports the event as unavailable instead of inventing a time. No substitute time is generated. A separately reviewed high-latitude rule would be needed before showing an adjusted time. Source: [calculation contract v0.3](calculation-contract-v0.3.md) (repo record checked 2026-09-27); wording remains a proposal pending review.

## 5. Review-state table

| Item | State | Reviewer role | Person |
| --- | --- | --- | --- |
| `research-15` parameters and non-institutional label | `unreviewed` | Islamic-methodology reviewer | Vacant |
| `mwl-angles-18-17` parameters and non-endorsement label | `unreviewed` | Islamic-methodology reviewer | Vacant |
| Wording (a) method choice | Pending | Islamic-methodology reviewer | Vacant |
| Wording (b) calculated beginning vs timetable/iqamah | Pending | Islamic-methodology reviewer | Vacant |
| Wording (c) unavailable high-latitude events | Pending | Islamic-methodology reviewer | Vacant |
| Shadow-target and solar-model treatment (P1.3-A1 residual) | Open | Astronomy reviewer | Vacant |
| Primary-source confirmation for the 18/17 angles | Open | Islamic-methodology reviewer | Vacant |

## 6. Assumptions, discrepancies, and limitations

**Assumptions.** Kernel values were read directly from `crates/salah-core/src/method.rs` (`research_15()` and `mwl_angles_18_17()`, both revision 0.1) together with the [calculation contract v0.3](calculation-contract-v0.3.md); the register repeats them without change. The JS-file retrieval date of 2026-09-27 is taken from the [prayer-library report](prayer-library-v1-report.md) and [`data/reference/README.md`](../data/reference/README.md), with live bytes re-fetched 2026-09-27 matching the pinned SHA-256 exactly. The methods-table webpage was retrieved 2026-09-27 (architect retrieval recorded; independent verification fetch the same day confirmed the MWL 18/17 row and the “Europe, Far East, parts of US” region text). No systematic primary-literature search was performed.

**Notes, not discrepancies.** The contract v0.3 cites the methods-table webpage URL for the profile source; that citation stands and is now corroborated by the audited JS file as a second secondary artifact — webpage-alone citation is not inherently a discrepancy. The MWL-associated label rests on two corroborating secondary artifacts with primary institutional confirmation not established. No code, tolerance, or contract-behavior change was made; everything above is recorded here and in the TSV for later review.

**Limitations.** The Islamic-methodology reviewer, astronomy reviewer, civil-time/data maintainer, and product/accessibility reviewer roles are all vacant; nothing in this register is reviewed religious wording. The register does not support claims that `mwl-angles-18-17` is an endorsed, default, or complete timetable; that either profile covers Ramadan rules, fixed-interval variants, high-latitude substitution, elevation, time-zone mapping, or global accuracy; or that library agreement constitutes physical or religious proof. Phase 1 remains open until the gate review in P1.6.
