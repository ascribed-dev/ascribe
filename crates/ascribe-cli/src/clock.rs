//! The day the checks run on: `review-overdue` compares a page's review date
//! with it. The libraries don't read the clock, so the binary does, here,
//! and hands the day to each project it loads.

use std::sync::OnceLock;
use std::time::{SystemTime, UNIX_EPOCH};

use ascribe_core::Date;

/// The variable that sets the day instead of the clock, `YYYY-MM-DD`, so a
/// test or a CI job gets the same answer every day.
pub const TODAY_VARIABLE: &str = "ASCRIBE_TODAY";

/// Today's date in UTC, or the one `ASCRIBE_TODAY` gives. A value that isn't
/// a date is reported once, on standard error, and the clock is used.
pub fn today() -> Option<Date> {
    static WARNED: OnceLock<()> = OnceLock::new();
    if let Ok(value) = std::env::var(TODAY_VARIABLE) {
        if let Some(day) = Date::parse(value.trim()) {
            return Some(day);
        }
        WARNED.get_or_init(|| {
            eprintln!(
                "warning: {TODAY_VARIABLE} is \"{value}\", which isn't a date written YYYY-MM-DD; using today's"
            );
        });
    }
    let seconds = SystemTime::now().duration_since(UNIX_EPOCH).ok()?.as_secs();
    Some(Date::from_unix_seconds(seconds))
}
