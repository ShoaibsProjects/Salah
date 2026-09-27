# Phase 1 — validation and method integrity

**Status:** active plan. P1.1 has an initial [accuracy-budget protocol](../specification/accuracy-budget.md); P1.2 has a [19-case USNO manifest](../data/reference/solar-usno-v1.tsv) and [signed report](../specification/usno-matrix-v1-report.md); P1.3 has a [28-row prayer-library manifest](../data/reference/prayer-library-v1.tsv) and [discrepancy report](../specification/prayer-library-v1-report.md). Near-grazing, polar Asr, and independent review remain open. The [delivery roadmap](roadmap.md) owns the phase gate. This document does not declare the kernel accurate for every Earth location or religious convention.

## Starting evidence and known limits

`salah-core` v0.3.0 uses the `NOAA-MEEUS-SOLAR-2` solar model, an assumed sea-level apparent horizon of −0.833°, UTC as a practical approximation to UT1, explicit fixed offset, and no high-latitude substitution. The [current reference cases](../specification/reference-cases.md) include selected USNO rise/transit/set values, Adhan JS prayer comparisons, PrayTimes angle-profile comparisons, date-line and polar cases, and 1900/2100 samples. They are useful regression evidence, not a measured global error distribution. Their ±90-second USNO and ±60-second library tolerances are **case comparison allowances**, not a product-wide accuracy guarantee.

The current MWL-named profile is only an 18°/17° parameter set sourced to a PrayTimes table. It is not an institutionally reviewed regional default. The 15°/15° profile is explicitly for research.

## P1.1 — Numerical accuracy budget and comparison protocol

**Objective:** define what “accurate” means for each output and prevent a single clock-time difference from mixing unrelated causes.

**Deliverable:** the draft [accuracy budget](../specification/accuracy-budget.md), subject to wider source evidence and independent review before any global accuracy claim. It must define:

- Event definitions and units: upper transit, solar-center altitude at the apparent horizon, twilight depression, and Asr shadow target. Specify whether a comparison uses raw UTC seconds, rounded seconds, or displayed minutes.
- Separate budgets or measured discrepancies for numerical root solving, solar ephemeris, horizon/refraction/elevation assumptions, prayer-method parameters, civil-time conversion, and display rounding. Do not combine them into one unexplained score.
- Comparison regimes: ordinary sunrise/sunset, high-latitude but existing events, grazing/tangent events, polar no-event days, and dates near the supported 1900/2100 boundaries. A tolerance for one regime does not silently apply to another.
- Source precision and assumptions. USNO API values are minute formatted; independent implementations may use different ephemerides, atmospheric models, rounding, and Asr definitions. Record those before setting a pass threshold.
- A predeclared triage threshold and an evidence-based release threshold. A result outside the triage threshold triggers investigation; it is not automatically “wrong.” Do not choose thresholds after seeing only cases that pass.
- A discrepancy ledger template with cause, impact, supported-scope decision, owner, and status.

**Acceptance:** a reviewer can take one case and explain which differences are numerical, physical, methodological, civil-time, or display-related. Any proposed numeric threshold is labeled **target**, **current observation**, or **release criterion**. No unsupported precision claim appears in the CLI or product copy.

## P1.2 — Versioned source manifest and solar matrix

**Objective:** make every external expected value traceable and rerunnable.

Build a compact source manifest with case ID, coordinates and datum, local date, source URL/API and version, retrieval date, response hash or permitted archived response, source precision/time scale, horizon/elevation assumptions, fixed offset used for presentation, expected UTC events, and reason for inclusion. Preserve the original response where license and terms allow; otherwise preserve an exact retrieval recipe and checksum. Keep reference tooling outside the runtime core.

Choose cases deliberately across both hemispheres; equinoxes and solstices; equatorial, mid-, high-, and polar latitudes; longitude/date-line extremes; leap years; and 1900, contemporary, 2050, and 2100 dates. Use a separate set of cases for model tuning and independent holdout review. Record missing-event and near-tangent cases without forcing a clock-time tolerance where the underlying event definition differs.

**Acceptance:** a reviewer can retrieve or verify each reference and recompute its UTC value; the matrix includes normal, boundary, and unavailable events; failures report case ID and discrepancy. Matrix breadth is documented, so a few examples cannot be described as global coverage.

## P1.3 — Prayer-rule comparison

Compare Fajr, Isha, Dhuhr, Maghrib, and Standard/Hanafi Asr against at least two independently implemented libraries under **matched explicit parameters**. Record library version, configuration, rounding, high-latitude behavior, and UTC output. Investigate the already observed approximately one-minute Asr difference between Adhan JS and Salah/PrayTimes before any accuracy claim for that rule. Treat source-to-source disagreement as evidence to explain, not a vote.

**Acceptance:** each prayer event has ordinary and edge cases; parameter or model differences are distinguished from bugs; unexplained discrepancies remain visible in the ledger and block claims for the affected regime.

## P1.4 — Method provenance and religious review

Create a versioned method-source register. For every named profile, record its parameter values, source edition/URL, retrieval or publication date, applicable region/community, adjustment and Ramadan behavior, high-latitude behavior, and review state. Prefer a primary institution document when available. A secondary implementation table may support a parameter set but cannot by itself justify institutional endorsement or a universal default. Ask qualified reviewers to check user-facing descriptions and disputed practices. Do not generate a fatwa.

**Acceptance:** research parameters stay clearly labeled; a named default is not shipped until its source and review status support that label; changes to a profile are versioned rather than silently replacing historical behavior.

## P1.5 — Event and rounding contract

Specify raw event, adjusted prayer beginning, displayed timetable minute, and notification instant separately. Decide how minute rounding works per selected method, including whether an adjustment occurs before rounding. Verify that Maghrib/sunset, missing events, and Isha crossing local midnight are represented correctly. Do not round away a discrepancy to make comparisons appear to agree.

**Acceptance:** the contract contains exact examples derived from cited vectors, and the CLI/result schema reveals which rule produced a displayed time.

## P1.6 — Gate report

Summarize the matrix and discrepancy ledger by regime and event; state what was verified, what remains excluded, and who reviewed method wording. Record whether the numerical and source criteria are met. If a criterion is not met, keep Phase 1 open or narrow the explicitly supported scope. A consumer UI may be explored, but it must not present the research engine as a globally validated timetable.

## Order and next work item

P1.1, P1.2, and the initial P1.3 prayer-rule comparison are written. P1.3 records the approximately one-minute Asr disagreement and the polar/high-latitude status differences without altering the kernel. P1.5 rounding definitions can proceed alongside method-source work. P1.6 is a gate review, not a date-driven ceremony.

The next bounded task is **P1.4: method provenance and user-facing review**. Trace every named profile to its primary source where possible, mark the research profile clearly, and record what has or has not received qualified religious review. The [accuracy budget](../specification/accuracy-budget.md), [solar report](../specification/usno-matrix-v1-report.md), and [prayer-rule report](../specification/prayer-library-v1-report.md) show which release claims remain unsupported.

Each later calculation change needs a linked discrepancy, a source or invariant that demonstrates it, a contract/model/profile version decision, and a regression case. Keep historical contracts and reference provenance intact.
