//! Calendar dates, as a `date` frontmatter field holds them (`YYYY-MM-DD`).
//!
//! No library reads the clock: a check that compares with today's date is
//! given it by its caller, which makes one with [`Date::from_unix_seconds`].

use std::fmt;

/// A day of the proleptic Gregorian calendar. Dates compare in calendar
/// order.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Date {
    year: u32,
    month: u32,
    day: u32,
}

impl Date {
    /// The date written `YYYY-MM-DD`, when it's a real calendar date.
    pub fn parse(s: &str) -> Option<Date> {
        let b = s.as_bytes();
        if b.len() != 10 || b[4] != b'-' || b[7] != b'-' {
            return None;
        }
        let digits = |r: std::ops::Range<usize>| b[r].iter().all(u8::is_ascii_digit);
        if !(digits(0..4) && digits(5..7) && digits(8..10)) {
            return None;
        }
        let year = s[0..4].parse().ok()?;
        let month = s[5..7].parse().ok()?;
        let day = s[8..10].parse().ok()?;
        Date::new(year, month, day)
    }

    /// The date, when there's such a day.
    pub fn new(year: u32, month: u32, day: u32) -> Option<Date> {
        let leap =
            (year.is_multiple_of(4) && !year.is_multiple_of(100)) || year.is_multiple_of(400);
        let days = match month {
            1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
            4 | 6 | 9 | 11 => 30,
            2 if leap => 29,
            2 => 28,
            _ => return None,
        };
        (1..=days)
            .contains(&day)
            .then_some(Date { year, month, day })
    }

    /// The day, in UTC, that a time counted in seconds since the Unix epoch
    /// falls on.
    pub fn from_unix_seconds(seconds: u64) -> Date {
        // Howard Hinnant's `civil_from_days`, for days since 1970-01-01.
        let z = seconds / 86_400 + 719_468;
        let era = z / 146_097;
        let doe = z - era * 146_097;
        let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
        let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
        let mp = (5 * doy + 2) / 153;
        let day = doy - (153 * mp + 2) / 5 + 1;
        let month = if mp < 10 { mp + 3 } else { mp - 9 };
        let year = yoe + era * 400 + u64::from(month <= 2);
        Date {
            year: u32::try_from(year).unwrap_or(u32::MAX),
            month: month as u32,
            day: day as u32,
        }
    }
    /// The days from `earlier` to this date: negative when `earlier` is
    /// later.
    pub fn days_since(self, earlier: Date) -> i64 {
        self.day_number() - earlier.day_number()
    }

    /// The date `days` days after this one. Before the Unix epoch, which
    /// no date Ascribe is given is, it's 1970-01-01.
    pub fn plus_days(self, days: u32) -> Date {
        let number = self.day_number() + i64::from(days);
        Date::from_unix_seconds(u64::try_from(number).unwrap_or_default() * 86_400)
    }

    /// Its month's name in English, `September`, for a person to read.
    pub fn month_name(self) -> &'static str {
        const NAMES: [&str; 12] = [
            "January",
            "February",
            "March",
            "April",
            "May",
            "June",
            "July",
            "August",
            "September",
            "October",
            "November",
            "December",
        ];
        NAMES[(self.month as usize - 1) % 12]
    }

    /// Its year.
    pub fn year(self) -> u32 {
        self.year
    }

    /// Its day of the month.
    pub fn day(self) -> u32 {
        self.day
    }

    /// Days since 1970-01-01: Howard Hinnant's `days_from_civil`.
    fn day_number(self) -> i64 {
        let (month, day) = (i64::from(self.month), i64::from(self.day));
        let year = i64::from(self.year) - i64::from(month <= 2);
        let era = year.div_euclid(400);
        let yoe = year - era * 400;
        let mp = (month + 9) % 12;
        let doy = (153 * mp + 2) / 5 + day - 1;
        let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
        era * 146_097 + doe - 719_468
    }
}

impl fmt::Display for Date {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:04}-{:02}-{:02}", self.year, self.month, self.day)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_real_dates_only() {
        assert!(Date::parse("2024-02-29").is_some() && Date::parse("2023-02-29").is_none());
        assert!(Date::parse("2024-13-01").is_none() && Date::parse("2024-1-01").is_none());
        assert_eq!(
            Date::parse("2026-10-10").map(|d| d.to_string()),
            Some("2026-10-10".to_owned())
        );
    }

    #[test]
    fn dates_compare_in_calendar_order() {
        let a = Date::parse("2025-12-31");
        let b = Date::parse("2026-01-01");
        assert!(a < b);
    }

    #[test]
    fn days_between_dates() {
        let day = |s| Date::parse(s).unwrap_or(Date::from_unix_seconds(0));
        assert_eq!(day("2026-10-11").days_since(day("2026-09-28")), 13);
        assert_eq!(day("2024-03-01").days_since(day("2024-02-28")), 2);
        assert_eq!(day("2026-01-01").days_since(day("2026-01-02")), -1);
        assert_eq!(day("1970-01-01").day_number(), 0);
        assert_eq!(day("2026-10-10").day_number(), 1_791_604_315 / 86_400);
        assert_eq!(day("2026-09-28").month_name(), "September");
        assert_eq!(day("2026-10-10").plus_days(3), day("2026-10-13"));
        assert_eq!(day("2026-12-30").plus_days(3), day("2027-01-02"));
        assert_eq!(day("2024-02-28").plus_days(0), day("2024-02-28"));
    }

    #[test]
    fn days_since_the_epoch() {
        assert_eq!(Date::from_unix_seconds(0).to_string(), "1970-01-01");
        assert_eq!(
            Date::from_unix_seconds(951_782_400).to_string(),
            "2000-02-29"
        );
        // 2026-10-10T03:51:55Z.
        assert_eq!(
            Date::from_unix_seconds(1_791_604_315).to_string(),
            "2026-10-10"
        );
    }
}
