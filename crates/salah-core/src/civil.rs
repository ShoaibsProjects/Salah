use core::fmt;

use crate::CalculationError;

/// Proleptic Gregorian civil date. Public construction is restricted to the
/// solar model's supported input range, 1900–2100.
///
/// The fields are private so a caller cannot bypass calendar validation.
///
/// ```compile_fail
/// use salah_core::CivilDate;
/// let _ = CivilDate { year: 2026, month: 2, day: 31 };
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CivilDate {
    year: i32,
    month: u8,
    day: u8,
}

impl CivilDate {
    pub const fn year(self) -> i32 {
        self.year
    }

    pub const fn month(self) -> u8 {
        self.month
    }

    pub const fn day(self) -> u8 {
        self.day
    }

    pub fn new(year: i32, month: u8, day: u8) -> Result<Self, CalculationError> {
        if !(1900..=2100).contains(&year) || !(1..=12).contains(&month) {
            return Err(CalculationError::InvalidDate);
        }
        let days = match month {
            2 if is_leap_year(year) => 29,
            2 => 28,
            4 | 6 | 9 | 11 => 30,
            _ => 31,
        };
        if day == 0 || day > days {
            return Err(CalculationError::InvalidDate);
        }
        Ok(Self { year, month, day })
    }

    /// Days from the Unix epoch. Calendar conversion follows Howard Hinnant's
    /// `days_from_civil` algorithm for the proleptic Gregorian calendar.
    pub fn days_since_unix_epoch(self) -> i64 {
        let mut year = i64::from(self.year);
        let month = i64::from(self.month);
        let day = i64::from(self.day);
        if month <= 2 {
            year -= 1;
        }
        let era = year.div_euclid(400);
        let year_of_era = year - era * 400;
        let shifted_month = month + if month > 2 { -3 } else { 9 };
        let day_of_year = (153 * shifted_month + 2) / 5 + day - 1;
        let year_of_era_day = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
        era * 146_097 + year_of_era_day - 719_468
    }

    fn from_days_since_unix_epoch(days: i64) -> Self {
        let z = days + 719_468;
        let era = z.div_euclid(146_097);
        let day_of_era = z - era * 146_097;
        let year_of_era =
            (day_of_era - day_of_era / 1460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
        let mut year = year_of_era + era * 400;
        let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
        let month_prime = (5 * day_of_year + 2) / 153;
        let day = day_of_year - (153 * month_prime + 2) / 5 + 1;
        let month = month_prime + if month_prime < 10 { 3 } else { -9 };
        if month <= 2 {
            year += 1;
        }
        Self {
            year: year as i32,
            month: month as u8,
            day: day as u8,
        }
    }
}

fn is_leap_year(year: i32) -> bool {
    year % 4 == 0 && (year % 100 != 0 || year % 400 == 0)
}

impl fmt::Display for CivilDate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:04}-{:02}-{:02}", self.year, self.month, self.day)
    }
}

/// A fixed offset is a first-slice input, not an IANA time zone or DST rule.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FixedUtcOffset {
    minutes_east: i16,
}

impl FixedUtcOffset {
    pub fn from_minutes_east(minutes_east: i16) -> Result<Self, CalculationError> {
        if !(-840..=840).contains(&minutes_east) {
            return Err(CalculationError::InvalidUtcOffset);
        }
        Ok(Self { minutes_east })
    }

    pub fn minutes_east(self) -> i16 {
        self.minutes_east
    }
}

impl fmt::Display for FixedUtcOffset {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let sign = if self.minutes_east < 0 { '-' } else { '+' };
        let total = i32::from(self.minutes_east).abs();
        write!(f, "{sign}{:02}:{:02}", total / 60, total % 60)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct UtcInstant {
    pub unix_seconds: i64,
}

impl UtcInstant {
    pub fn to_local(self, offset: FixedUtcOffset) -> CivilDateTime {
        let local_seconds = self.unix_seconds + i64::from(offset.minutes_east) * 60;
        let day_number = local_seconds.div_euclid(86_400);
        let seconds_in_day = local_seconds.rem_euclid(86_400);
        CivilDateTime {
            date: CivilDate::from_days_since_unix_epoch(day_number),
            hour: (seconds_in_day / 3600) as u8,
            minute: ((seconds_in_day % 3600) / 60) as u8,
            second: (seconds_in_day % 60) as u8,
        }
    }

    pub fn to_utc(self) -> CivilDateTime {
        self.to_local(FixedUtcOffset { minutes_east: 0 })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CivilDateTime {
    pub date: CivilDate,
    pub hour: u8,
    pub minute: u8,
    pub second: u8,
}

impl fmt::Display for CivilDateTime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {:02}:{:02}:{:02}",
            self.date, self.hour, self.minute, self.second
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gregorian_dates_cross_epoch_and_leap_days() {
        assert_eq!(
            CivilDate::new(1970, 1, 1).unwrap().days_since_unix_epoch(),
            0
        );
        let leap = CivilDate::new(2000, 2, 29).unwrap();
        assert_eq!(
            CivilDate::from_days_since_unix_epoch(leap.days_since_unix_epoch()),
            leap
        );
        assert!(CivilDate::new(2100, 2, 29).is_err());
        let before_epoch = CivilDate::new(1969, 12, 31).unwrap();
        assert_eq!(before_epoch.days_since_unix_epoch(), -1);
    }

    #[test]
    fn every_supported_gregorian_day_round_trips() {
        let first = CivilDate::new(1900, 1, 1).unwrap().days_since_unix_epoch();
        let last = CivilDate::new(2100, 12, 31)
            .unwrap()
            .days_since_unix_epoch();
        for day_number in first..=last {
            let date = CivilDate::from_days_since_unix_epoch(day_number);
            assert_eq!(date.days_since_unix_epoch(), day_number);
            assert!(CivilDate::new(date.year(), date.month(), date.day()).is_ok());
        }
    }

    #[test]
    fn offset_conversion_can_cross_local_date() {
        let instant = UtcInstant { unix_seconds: 0 };
        let west = FixedUtcOffset::from_minutes_east(-300).unwrap();
        assert_eq!(instant.to_local(west).to_string(), "1969-12-31 19:00:00");
    }
}
