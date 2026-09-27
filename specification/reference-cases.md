# Reference cases for kernel v0.1

These cases are independent checks of the first offline core. They are not a certification of any religious method. The built-in `research-15` profile uses 15° solar depression for both Fajr and Isha, no Dhuhr/Maghrib adjustment, and no high-latitude fallback. It is not attributed to ISNA or another institution.

## Solar events: U.S. Naval Observatory

The [USNO Complete Sun and Moon Data API](https://aa.usno.navy.mil/data/api#rstt) returned these civil clock times on 27 September 2026. The API reports to the minute. Tests convert the listed fixed offset to UTC and allow ±90 seconds to account for reporting precision and model/horizon differences.

| Place and date | Coordinates | Fixed offset | Rise | Upper transit | Set | Source |
| --- | --- | --- | --- | --- | --- | --- |
| Minneapolis, 2026-09-27 | 44.9778, -93.2650 | UTC−05:00 | 07:06 | 13:04 | 19:01 | [USNO response](https://aa.usno.navy.mil/api/rstt/oneday?date=2026-09-27&coords=44.9778,-93.2650&tz=-5) |
| Makkah, 2026-03-20 | 21.4225, 39.8262 | UTC+03:00 | 06:25 | 12:28 | 18:32 | [USNO response](https://aa.usno.navy.mil/api/rstt/oneday?date=2026-03-20&coords=21.4225,39.8262&tz=3) |
| Sydney, 2026-12-21 | -33.8688, 151.2093 | UTC+11:00 | 05:41 | 12:53 | 20:05 | [USNO response](https://aa.usno.navy.mil/api/rstt/oneday?date=2026-12-21&coords=-33.8688,151.2093&tz=11) |
| Tromsø, 2026-06-21 | 69.6492, 18.9553 | UTC+02:00 | none | 12:46 | none | [USNO response](https://aa.usno.navy.mil/api/rstt/oneday?date=2026-06-21&coords=69.6492,18.9553&tz=2) |
| Kiritimati, 2026-09-27 | 1.8721, -157.4278 | UTC+14:00 | 06:18 | 12:21 | 18:24 | [USNO response](https://aa.usno.navy.mil/api/rstt/oneday?date=2026-09-27&coords=1.8721,-157.4278&tz=14) |

The USNO API's `isdst` field is not used as the civil-time authority here; the test supplies the shown explicit offset. A fixed offset is not a time-zone database.

## Prayer-specific comparison: Adhan JS

[Adhan JS](https://github.com/batoulapps/adhan-js), npm package `adhan` version **4.4.6**, was run with `CalculationParameters('Other', 15, 15, 0, 0)`, `Rounding.None`, zero adjustments, and the named Asr criterion. Its outputs are UTC instants. For these ordinary locations, no high-latitude substitution is expected. Tests allow ±60 seconds because the solar model and Asr declination treatment differ.

| Place/date | Asr | Fajr UTC | Asr UTC | Isha UTC |
| --- | --- | --- | --- | --- |
| Makkah, 2026-03-20 | Standard | 02:24:01 | 12:53:21 | 16:32:43 |
| Minneapolis, 2026-09-27 | Hanafi | 10:45:34 | 22:10:46 | 2026-09-28 01:21:16 |
| London, 2026-06-21 | Standard | 00:15:18 | 16:25:13 | 23:49:15 |

The Minneapolis sample times in the original planning conversation were illustrative and do not match the USNO solar events. They are not reference cases.

## Numerical basis and limits

The solar position follows the equations described by [NOAA's solar calculation details](https://gml.noaa.gov/grad/solcalc/calcdetails.html), which are based on Jean Meeus, *Astronomical Algorithms*. Julian day is derived from UTC as a practical approximation to UT1. The model uses a mean apparent horizon of −0.833° for sunrise/sunset, sea-level observation, and no local terrain or weather correction. These comparisons establish minute-level agreement for the listed cases; they do not validate every date, latitude, atmosphere, Islamic method, or civil-time rule. The implementation should keep widening its independent case set before consumer release.
