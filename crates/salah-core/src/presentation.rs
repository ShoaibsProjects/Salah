//! Research-preview prayer-start minute display adapter.
//!
//! Policy `prayer-start-ceil-minute` revision `0.1` (see
//! `specification/presentation-contract-v0.1.md`). This module performs no
//! astronomy and changes no calculation result. It starts from the kernel's
//! already adjusted [`Event::Occurs::utc`] second, converts it with the
//! record's explicit [`FixedUtcOffset`], and labels the first whole local
//! minute at or after that second using Euclidean integer arithmetic.
//!
//! The adapter covers the five prayer beginnings only (Fajr, Dhuhr, Asr,
//! Maghrib, Isha). Sunrise and sunset receive no minute receipt. An
//! [`Event::Unavailable`] yields its original reason and no minute. The
//! displayed label never replaces [`Event::utc`] and is not a notification
//! instant, fasting cutoff, or endorsed timetable value.

use core::fmt;

use crate::prayer::EventRule;
use crate::{CivilDate, Event, FixedUtcOffset, PrayerTimes, UnavailableReason, UtcInstant};

/// Display-policy identity for the research-preview adapter.
pub const DISPLAY_POLICY_ID: &str = "prayer-start-ceil-minute";
/// Display-policy revision for the research-preview adapter.
pub const DISPLAY_POLICY_REVISION: &str = "0.1";

/// The five prayer beginnings covered by the display policy.
///
/// Sunrise and sunset are intentionally absent: they retain second-precision
/// presentation and receive no whole-minute prayer-start receipt.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrayerStart {
    Fajr,
    Dhuhr,
    Asr,
    Maghrib,
    Isha,
}

impl PrayerStart {
    /// Stable prayer-start name used in receipts and CLI labels.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Fajr => "Fajr",
            Self::Dhuhr => "Dhuhr",
            Self::Asr => "Asr",
            Self::Maghrib => "Maghrib",
            Self::Isha => "Isha",
        }
    }

    /// All five prayer starts in display order.
    pub const fn all() -> [Self; 5] {
        [
            Self::Fajr,
            Self::Dhuhr,
            Self::Asr,
            Self::Maghrib,
            Self::Isha,
        ]
    }
}

impl fmt::Display for PrayerStart {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

/// A whole local minute label with its Gregorian date.
///
/// The date travels with the minute: a post-midnight label (for example a
/// London summer Isha) carries the converted local date, never the requested
/// record date by assumption.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LocalDisplayMinute {
    date: CivilDate,
    hour: u8,
    minute: u8,
}

impl LocalDisplayMinute {
    /// Hour of day, 00–23.
    pub const fn hour(self) -> u8 {
        self.hour
    }

    /// Minute of hour, 00–59.
    pub const fn minute(self) -> u8 {
        self.minute
    }

    /// Local Gregorian date carrying the minute.
    pub const fn date(self) -> CivilDate {
        self.date
    }
}

impl fmt::Display for LocalDisplayMinute {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {:02}:{:02}", self.date, self.hour, self.minute)
    }
}

/// Occurring/unavailable outcome of one prayer-start receipt.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PrayerStartStatus {
    /// The solar condition occurred. `utc` is the kernel's already adjusted
    /// beginning; `display` is the whole-minute label at or after it; `rule`
    /// is the kernel's original [`EventRule`].
    Occurs {
        utc: UtcInstant,
        display: LocalDisplayMinute,
        rule: EventRule,
    },
    /// The solar condition did not occur. Carries the kernel's original
    /// reason and no minute value.
    Unavailable { reason: UnavailableReason },
}

/// Typed receipt for one prayer-start display label.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PrayerStartReceipt {
    /// Which of the five prayer beginnings this receipt describes.
    pub prayer: PrayerStart,
    /// Occurring (with UTC, minute, rule) or unavailable (with reason).
    pub status: PrayerStartStatus,
    /// Method ID from the calculation record.
    pub method_id: &'static str,
    /// Method revision from the calculation record.
    pub method_revision: &'static str,
    /// Display-policy ID (`prayer-start-ceil-minute`).
    pub display_policy_id: &'static str,
    /// Display-policy revision (`0.1`).
    pub display_policy_revision: &'static str,
}

/// Build the display receipt for one prayer beginning.
///
/// The caller cannot pair an [`Event`] with a different record: the event is
/// selected internally from `times` via `prayer`, and the offset, method
/// identity, and rule come from the same result.
pub fn prayer_start_minute(times: &PrayerTimes, prayer: PrayerStart) -> PrayerStartReceipt {
    let event = match prayer {
        PrayerStart::Fajr => times.fajr,
        PrayerStart::Dhuhr => times.dhuhr,
        PrayerStart::Asr => times.asr,
        PrayerStart::Maghrib => times.maghrib,
        PrayerStart::Isha => times.isha,
    };
    let status = match event {
        Event::Occurs { utc, rule, .. } => PrayerStartStatus::Occurs {
            utc,
            display: ceil_to_local_minute(utc, times.record.utc_offset),
            rule,
        },
        Event::Unavailable { reason } => PrayerStartStatus::Unavailable { reason },
    };
    PrayerStartReceipt {
        prayer,
        status,
        method_id: times.record.method.id,
        method_revision: times.record.method.revision,
        display_policy_id: DISPLAY_POLICY_ID,
        display_policy_revision: DISPLAY_POLICY_REVISION,
    }
}

/// First whole local minute at or after the adjusted UTC second.
///
/// `L` is the adjusted second expressed in local seconds; `M` is the ceiling
/// of `L` to a 60-second multiple via Euclidean division, including pre-1970
/// negative values. The
/// Gregorian date, hour, and minute are then derived from `M`, preserving any
/// midnight or date-line rollover.
fn ceil_to_local_minute(utc: UtcInstant, offset: FixedUtcOffset) -> LocalDisplayMinute {
    let local_seconds = utc
        .unix_seconds
        .checked_add(i64::from(offset.minutes_east()) * 60)
        .expect("local second is outside the supported calculation range");
    let minute_start = local_seconds.div_euclid(60);
    let ceil_minutes = if local_seconds.rem_euclid(60) == 0 {
        minute_start
    } else {
        minute_start + 1
    };
    let label_seconds = ceil_minutes
        .checked_mul(60)
        .expect("local minute is outside the supported calculation range");
    let civil = UtcInstant {
        unix_seconds: label_seconds,
    }
    .to_utc();
    LocalDisplayMinute {
        date: civil.date,
        hour: civil.hour,
        minute: civil.minute,
    }
}
