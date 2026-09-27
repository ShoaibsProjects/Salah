# Salah

**Project status:** Rust kernel research preview, September 2026. No consumer application has been implemented yet.

Salah aims to offer a free-to-use prayer-time system for Earth. Its foundation is an offline Rust calculation core with a public calculation specification. iOS, Android, web, and future interfaces can use the same core. The current research CLI calculates prayer times locally; Qibla and the consumer apps are planned next stages. The repository's software license remains undecided, so its code is not yet offered under an open-source license.

Start with the [Vision and architecture](docs/vision-and-architecture.md), then read the [Foundation plan](docs/foundation-plan.md), [Decision record](docs/decisions.md), and [Calculation contract v0.1](specification/calculation-contract-v0.1.md). Future contributors and coding agents should also read [AGENTS.md](AGENTS.md).

The product goal is long-term usefulness through 2050. Calculation methods, default regional settings, scholarly review, open-source license, and the final user interface need decisions before launch.

## Run the first kernel slice

The Rust workspace has no third-party crate dependencies. With Rust installed:

```bash
cargo run -p salah-cli -- \
  --lat 44.9778 --lon -93.2650 --date 2026-09-27 \
  --utc-offset -05:00 --method research-15 --asr hanafi
```

This CLI uses an explicit **fixed UTC offset**, a sea-level solar model, and a **research-only** 15°/15° Fajr/Isha profile. It does not look up the location's time zone, apply DST rules, or use a reviewed institutional method. It reports missing astronomical events without substituting high-latitude times. See the [implemented contract](specification/calculation-contract-v0.1.md) and [reference cases](specification/reference-cases.md) before using a result.
