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
