# Decision record

**Status:** living record. Change a decision deliberately and explain why; do not infer a settled choice from an illustrative example.

## Accepted project direction

| Decision | Reason |
| --- | --- |
| Support Earth first. | The prayer-time domain and validation work are substantial already. Future celestial settings would require separate physical and Islamic research. |
| Use Rust for the calculation core. | One portable implementation can serve native clients and WebAssembly while remaining independent of a UI framework. |
| Make core calculations work offline. | A person must be able to calculate times and Qibla without an account, remote API, or subscription. |
| Publish a versioned calculation specification and reference cases. | Results need to be explainable and reproducible, including after the original maintainers or interface change. |
| Keep Islamic methods configurable and sourced. | Astronomy alone does not settle all jurisprudential and regional choices. |
| Keep mosque prayer-start and iqamah times separate from calculated events. | They answer different questions and may differ intentionally. |
| Keep the core free to users and private by default. | Access to daily prayer information must not depend on data collection or payment. |
| Exclude blockchain, social feeds, ads, and AI-generated times from the initial product. | They add cost or risk without improving the core calculation. |

## Working implementation choices

| Choice | Current position | Revisit when |
| --- | --- | --- |
| Mobile interface | Flutter is a candidate, not a dependency of the Rust engine. | After a small Rust binding probe and accessibility review. |
| Web interface | Rust-to-WASM with a responsive client is the leading approach. | After checking offline, browser, and deployment behavior. |
| First time input | Explicit fixed UTC offset for the initial research CLI; IANA zone support is required before claiming global civil-time accuracy. | At the F3 civil-time milestone. |
| First method profile | `research-15` uses 15° Fajr and Isha angles with zero adjustments. It carries no institutional attribution. | Replace or supplement only after source and scholarly review. |
| Crate layout | Begin with one core crate and one CLI crate. | Split only when real module boundaries and independent reuse are clear. |
| Update system | Versioned, authenticated data packs are a long-term target. | When method or time-zone update requirements and platform limits are known. |

## Unresolved decisions

- First calculation profile and authoritative parameter sources.
- Solar model, physical assumptions, rounding, and numerical tolerance.
- High-latitude defaults by region and the scholarly review process.
- Time-zone boundary dataset, licensing, and update mechanism.
- Product license, maintainers, funding, and governance through 2050.
- Launch languages, platform minimums, and notification behavior by OS.

The repository name is **Salah**. “2050” describes the maintenance horizon, not a guarantee that software or civil-time data can remain unchanged until that year.
