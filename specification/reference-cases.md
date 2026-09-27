# Reference cases for kernel v0.3

These cases are independent checks of the first offline core. They are not a certification of any religious method. The built-in `research-15` profile uses 15° solar depression for both Fajr and Isha, no Dhuhr/Maghrib adjustment, and no high-latitude fallback. It is not attributed to ISNA or another institution.

## Solar events: U.S. Naval Observatory

The [USNO Complete Sun and Moon Data API](https://aa.usno.navy.mil/data/api#rstt) returned these civil clock times for the listed dates. The API reports to the minute. Tests convert the listed fixed offset to UTC and allow ±90 seconds to account for reporting precision and model/horizon differences.

| Place and date | Coordinates | Fixed offset | Rise | Upper transit | Set | Source |
| --- | --- | --- | --- | --- | --- | --- |
| Minneapolis, 2026-09-27 | 44.9778, -93.2650 | UTC−05:00 | 07:06 | 13:04 | 19:01 | [USNO response](https://aa.usno.navy.mil/api/rstt/oneday?date=2026-09-27&coords=44.9778,-93.2650&tz=-5) |
| Makkah, 2026-03-20 | 21.4225, 39.8262 | UTC+03:00 | 06:25 | 12:28 | 18:32 | [USNO response](https://aa.usno.navy.mil/api/rstt/oneday?date=2026-03-20&coords=21.4225,39.8262&tz=3) |
| Sydney, 2026-12-21 | -33.8688, 151.2093 | UTC+11:00 | 05:41 | 12:53 | 20:05 | [USNO response](https://aa.usno.navy.mil/api/rstt/oneday?date=2026-12-21&coords=-33.8688,151.2093&tz=11) |
| Tromsø, 2026-06-21 | 69.6492, 18.9553 | UTC+02:00 | none | 12:46 | none | [USNO response](https://aa.usno.navy.mil/api/rstt/oneday?date=2026-06-21&coords=69.6492,18.9553&tz=2) |
| Kiritimati, 2026-09-27 | 1.8721, -157.4278 | UTC+14:00 | 06:18 | 12:21 | 18:24 | [USNO response](https://aa.usno.navy.mil/api/rstt/oneday?date=2026-09-27&coords=1.8721,-157.4278&tz=14) |
| Quito, 2050-09-27 | 0.1807, -78.4678 | UTC−05:00 | 06:02 | 12:05 | 18:08 | [USNO response](https://aa.usno.navy.mil/api/rstt/oneday?date=2050-09-27&coords=0.1807,-78.4678&tz=-5) |
| Makkah, 2028-02-29 | 21.4225, 39.8262 | UTC+03:00 | 06:42 | 12:33 | 18:25 | [USNO response](https://aa.usno.navy.mil/api/rstt/oneday?date=2028-02-29&coords=21.4225,39.8262&tz=3) |
| 65.72° N, 2026-06-21 | 65.72, 0 | UTC+00:00 | 00:10 | 12:02 | 23:53 | [USNO response](https://aa.usno.navy.mil/api/rstt/oneday?date=2026-06-21&coords=65.72,0&tz=0) |
| 65.74° N, 2026-06-21 | 65.74, 0 | UTC+00:00 | none | 12:02 | none | [USNO response](https://aa.usno.navy.mil/api/rstt/oneday?date=2026-06-21&coords=65.74,0&tz=0) |
| Sydney, 1900-12-21 | -33.8688, 151.2093 | UTC+10:00 | 04:41 | 11:53 | 19:06 | [USNO response](https://aa.usno.navy.mil/api/rstt/oneday?date=1900-12-21&coords=-33.8688,151.2093&tz=10) |
| London, 2100-06-21 | 51.5072, -0.1276 | UTC+00:00 | 03:43 | 12:03 | 20:22 | [USNO response](https://aa.usno.navy.mil/api/rstt/oneday?date=2100-06-21&coords=51.5072,-0.1276&tz=0) |

The USNO API's `isdst` field is not used as the civil-time authority here; the test supplies the shown explicit offset. A fixed offset is not a time-zone database.

Near a grazing horizon, small differences in mean refraction or solar-radius assumptions can change a result from a few minutes of night to continuous daylight. At 65.735° N on 2026-06-21, [USNO reports brief rise/set events](https://aa.usno.navy.mil/api/rstt/oneday?date=2026-06-21&coords=65.735,0&tz=0), while `NOAA-MEEUS-SOLAR-2` with its fixed −0.833° horizon reports no crossing. This is a documented model-boundary difference, not evidence that one output is universally authoritative. The kernel should expose its horizon assumption wherever it explains such a result.

## Prayer-specific comparison: Adhan JS

[Adhan JS](https://github.com/batoulapps/adhan-js), npm package `adhan` version **4.4.6**, was run with `CalculationParameters('Other', 15, 15, 0, 0)`, `Rounding.None`, zero adjustments, and the named Asr criterion. Its outputs are UTC instants. For these ordinary locations, no high-latitude substitution is expected. Tests allow ±60 seconds because the solar model and Asr declination treatment differ.

| Place/date | Asr | Fajr UTC | Asr UTC | Isha UTC |
| --- | --- | --- | --- | --- |
| Makkah, 2026-03-20 | Standard | 02:24:01 | 12:53:21 | 16:32:43 |
| Minneapolis, 2026-09-27 | Hanafi | 10:45:34 | 22:10:46 | 2026-09-28 01:21:16 |
| London, 2026-06-21 | Standard | 00:15:18 | 16:25:13 | 23:49:15 |

The Minneapolis sample times in the original planning conversation were illustrative and do not match the USNO solar events. They are not reference cases.

## Named angle profile: PrayTimes MWL table

The [PrayTimes calculation-method table](https://praytimes.org/docs/methods) lists **18° Fajr** and **17° Isha** for MWL. Its [v2 JavaScript reference code](https://praytimes.org/code/v2/js/PrayTimes.js) defines the same angles and a default Dhuhr adjustment of zero minutes. The locally retrieved reference file had SHA-256 `f0e8442410446b1bf2db613bdebd3aa3aef5cdf5483068fbd9212d5acaa180fd`. The Salah profile is named `mwl-angles-18-17` to describe those published parameters. It does not claim endorsement by the Muslim World League, replicate every PrayTimes high-latitude rule, or resolve local mosque practice.

PrayTimes v2 was called with method `MWL`, explicit coordinates, fixed offset, DST `0`, and `Float` output. The floating local hours were converted to the UTC instants below. Tests allow ±60 seconds because the astronomical models and numerical iterations differ.

| Place/date | Fajr UTC | Dhuhr UTC | Standard Asr UTC | Isha UTC |
| --- | --- | --- | --- | --- |
| Minneapolis, 2026-09-27 | 10:28:02 | 18:03:57 | 21:22:11 | 2026-09-28 01:33:06 |
| Makkah, 2026-03-20 | 02:11:04 | 09:28:12 | 12:52:57 | 16:41:21 |

Adhan JS v4.4.6 with a custom 18°/17° profile agrees closely on Fajr and Isha. Its standard Asr in Minneapolis is about one minute earlier than this kernel and PrayTimes. Adhan fixes its Asr shadow-angle calculation using the day's solar coordinates; Salah fixes the noon shadow at calculated upper transit. The different Asr model is recorded rather than hidden by rounding.

## Numerical basis and limits

The solar position follows the equations described by [NOAA's solar calculation details](https://gml.noaa.gov/grad/solcalc/calcdetails.html), which are based on Jean Meeus, *Astronomical Algorithms*. Julian day is derived from UTC as a practical approximation to UT1. Model `NOAA-MEEUS-SOLAR-2` uses a mean apparent horizon of −0.833° for sunrise/sunset, sea-level observation, and no local terrain or weather correction. These comparisons establish minute-level agreement for the listed cases; they do not validate every date, latitude, atmosphere, Islamic method, or civil-time rule. The implementation should keep widening its independent case set before consumer release.
