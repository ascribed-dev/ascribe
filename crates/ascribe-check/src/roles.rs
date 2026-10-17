//! What `page-description-missing` and `review-overdue` decide about a page,
//! read from the fields its content type gives those roles (SPEC §7.2).
//!
//! The checks report from these, and the language server's This page rows
//! say the same from them, so the two can't disagree.

use ascribe_core::{Date, diagnostics};
use ascribe_model::{Checks, Field};
use serde_yaml_ng::Value;

/// Whether `page-description-missing` reports a page whose description
/// field is `field` and whose frontmatter is `value`.
pub fn description_missing(checks: &Checks, field: &Field, value: &Value) -> bool {
    if checks.is_off(diagnostics::PAGE_DESCRIPTION_MISSING)
        // A required field's absence is already `frontmatter-missing-field`,
        // and a default is a description.
        || field.required
        || field.default.is_some()
    {
        return false;
    }
    match value.get(field.name.as_str()) {
        None | Some(Value::Null) => true,
        Some(Value::String(s)) => s.trim().is_empty(),
        // Another type is `frontmatter-type-mismatch`.
        Some(_) => false,
    }
}

/// The description a page has in its field `field`, as written: its own
/// string, or the field's default when it leaves the field out. `None`
/// when it has neither, or the value isn't a string.
pub fn description<'v>(field: &'v Field, value: &'v Value) -> Option<&'v str> {
    match value.get(field.name.as_str()) {
        None | Some(Value::Null) => field.default.as_ref()?.as_str(),
        Some(v) => v.as_str(),
    }
}

/// The date a page's review-date field `field` holds, when it holds one.
pub fn review_date(field: &Field, value: &Value) -> Option<Date> {
    match value.get(field.name.as_str()) {
        Some(Value::String(text)) => Date::parse(text),
        _ => None,
    }
}

/// Whether `review-overdue` reports a page due for review on `due`, on the
/// day `today`: the date has passed, and the check is on.
pub fn review_overdue(checks: &Checks, today: Option<Date>, due: Date) -> bool {
    today.is_some_and(|today| due < today) && !checks.is_off(diagnostics::REVIEW_OVERDUE)
}
