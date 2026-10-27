//! The registry of `ascribe.toml`'s keys, [`SETTINGS_FILE`]: every key with
//! its table, kind, choices, default, and description, in the content model
//! reference's order. The loader reads its keys and defaults from it, so a
//! key it accepts is a key the registry has, and the default it uses is the
//! one the registry says.
//!
//! A check's own settings aren't here: a check is described by its `when` in
//! `crates/ascribe-core/data/diagnostics.toml`.

use std::sync::OnceLock;

use toml::de::{DeTable, DeValue};

/// The registry, which the binary carries.
pub const SETTINGS_FILE: &str = include_str!("../settings.toml");

/// The table a field's or an attribute's table form is, in [`Setting::table`].
pub const FIELD_TABLE: &str = "<field>";

/// Keys the loader takes in a table though the registry doesn't have them,
/// by table, because each is reported apart, with a message of its own: an
/// old key, or one that belongs somewhere else. The loader reports each, which
/// a test holds.
pub const REPORTED_APART: &[(&str, &str)] = &[
    ("types.<name>", "files"),
    ("types.<name>", "default"),
    ("versions", "current"),
    ("glossary", "terms"),
    ("intended", "term"),
];

/// One key of `ascribe.toml`.
#[derive(Clone, Debug, PartialEq)]
pub struct Setting {
    /// The table the key is in, as the reference names it: empty for the top
    /// level, `<name>` and the like for a part a project names
    /// (`types.<name>`), and [`FIELD_TABLE`] for a field's table form.
    pub table: String,
    /// The key.
    pub key: String,
    /// What its value is: `switch`, `choice`, `number`, `text`, `level`,
    /// `list`, or `table`.
    pub kind: String,
    /// Its type, as the reference's tables write it.
    pub type_text: String,
    /// For a choice or a level, its values, in the order a message lists
    /// them.
    pub choices: Vec<String>,
    /// The value used when the key is absent, when it has one.
    pub default: Option<SettingValue>,
    /// The reference's words for a default that isn't one value.
    pub default_text: Option<String>,
    /// Whether the key must be there.
    pub required: bool,
    /// Whether only a field takes it, and not an attribute.
    pub fields_only: bool,
    /// The release that added it, when the reference marks one.
    pub available: Option<String>,
    /// What it does, in Markdown, as the reference writes it.
    pub description: String,
}

/// A default in the registry.
#[derive(Clone, Debug, PartialEq)]
pub enum SettingValue {
    /// A boolean.
    Switch(bool),
    /// A whole number.
    Number(i64),
    /// A string.
    Text(String),
    /// An array of strings, empty or not.
    List(Vec<String>),
    /// An empty table.
    EmptyTable,
}

/// Every key [`SETTINGS_FILE`] has, in its order. Empty if the file doesn't
/// parse, which a test catches.
pub fn settings() -> &'static [Setting] {
    static SETTINGS: OnceLock<Vec<Setting>> = OnceLock::new();
    SETTINGS.get_or_init(|| parse_settings(SETTINGS_FILE).unwrap_or_default())
}

/// The keys in a data file shaped as [`SETTINGS_FILE`] is, or `None` when an
/// entry lacks a field every entry has, has one the registry doesn't define,
/// or has a field of the wrong type.
pub fn parse_settings(text: &str) -> Option<Vec<Setting>> {
    let table = DeTable::parse(text).ok()?;
    let DeValue::Array(items) = table.get_ref().get("setting")?.get_ref() else {
        return None;
    };
    items
        .iter()
        .map(|item| match item.get_ref() {
            DeValue::Table(t) => entry(t),
            _ => None,
        })
        .collect()
}

fn entry(t: &DeTable<'_>) -> Option<Setting> {
    const FIELDS: [&str; 11] = [
        "table",
        "key",
        "kind",
        "type",
        "choices",
        "default",
        "default-text",
        "required",
        "only",
        "available",
        "description",
    ];
    if t.iter()
        .any(|(k, _)| !FIELDS.contains(&k.get_ref().as_ref()))
    {
        return None;
    }
    let text = |key: &str| match t.get(key).map(|v| v.get_ref()) {
        Some(DeValue::String(s)) => Some(Some(s.to_string())),
        Some(_) => None,
        None => Some(None),
    };
    let choices = match t.get("choices").map(|v| v.get_ref()) {
        Some(DeValue::Array(items)) => items
            .iter()
            .map(|i| match i.get_ref() {
                DeValue::String(s) => Some(s.to_string()),
                _ => None,
            })
            .collect::<Option<Vec<_>>>()?,
        Some(_) => return None,
        None => Vec::new(),
    };
    let default = match t.get("default").map(|v| v.get_ref()) {
        Some(DeValue::Boolean(b)) => Some(SettingValue::Switch(*b)),
        Some(DeValue::Integer(i)) => Some(SettingValue::Number(
            i.as_str().replace('_', "").parse().ok()?,
        )),
        Some(DeValue::String(s)) => Some(SettingValue::Text(s.to_string())),
        Some(DeValue::Array(items)) => Some(SettingValue::List(
            items
                .iter()
                .map(|i| match i.get_ref() {
                    DeValue::String(s) => Some(s.to_string()),
                    _ => None,
                })
                .collect::<Option<Vec<_>>>()?,
        )),
        Some(DeValue::Table(t)) if t.is_empty() => Some(SettingValue::EmptyTable),
        Some(_) => return None,
        None => None,
    };
    let required = match t.get("required").map(|v| v.get_ref()) {
        Some(DeValue::Boolean(b)) => *b,
        Some(_) => return None,
        None => false,
    };
    let fields_only = match text("only")? {
        Some(only) if only == "fields" => true,
        Some(_) => return None,
        None => false,
    };
    Some(Setting {
        table: text("table")??,
        key: text("key")??,
        kind: text("kind")??,
        type_text: text("type")??,
        choices,
        default,
        default_text: text("default-text")?,
        required,
        fields_only,
        available: text("available")?,
        description: text("description")??,
    })
}

/// The keys of `table`, in the registry's order.
pub fn keys(table: &str) -> Vec<&'static str> {
    settings()
        .iter()
        .filter(|s| s.table == table)
        .map(|s| s.key.as_str())
        .collect()
}

/// The keys a field's table form takes, or an attribute's when `attribute`.
pub fn field_keys(attribute: bool) -> Vec<&'static str> {
    settings()
        .iter()
        .filter(|s| s.table == FIELD_TABLE && !(attribute && s.fields_only))
        .map(|s| s.key.as_str())
        .collect()
}

/// The keys the loader takes in `table`: its keys, then the ones
/// [`REPORTED_APART`].
pub(crate) fn taken(table: &str) -> Vec<&'static str> {
    let mut out = keys(table);
    out.extend(
        REPORTED_APART
            .iter()
            .filter(|(t, _)| *t == table)
            .map(|(_, k)| *k),
    );
    out
}

/// The entry for `key` in `table`.
pub fn find(table: &str, key: &str) -> Option<&'static Setting> {
    settings().iter().find(|s| s.table == table && s.key == key)
}

/// The default of `key` in `table` for a project whose `ascribe.toml` sets
/// the keys `written` says it does, by table and key: the registry's,
/// except where a default follows another key. `[consumer] agents` is the
/// registry's `true` only when `[consumer] site` is set (SPEC §9.4), and
/// `false` otherwise. The loader, `ascribe model`, and Project Settings all
/// take a default from here.
pub fn default_for(
    table: &str,
    key: &str,
    written: &dyn Fn(&str, &str) -> bool,
) -> Option<SettingValue> {
    let default = find(table, key)?.default.clone();
    if (table, key) == ("consumer", "agents") {
        let on = matches!(default, Some(SettingValue::Switch(true)));
        return Some(SettingValue::Switch(on && written("consumer", "site")));
    }
    default
}

/// The choices of `key` in `table`.
pub(crate) fn choices(table: &str, key: &str) -> Vec<&'static str> {
    let found = find(table, key);
    debug_assert!(found.is_some(), "the registry has no {table} {key}");
    found.map_or_else(Vec::new, |s| s.choices.iter().map(String::as_str).collect())
}

fn default_of(table: &str, key: &str) -> Option<&'static SettingValue> {
    let found = find(table, key).and_then(|s| s.default.as_ref());
    debug_assert!(
        found.is_some(),
        "the registry has no default for {table} {key}"
    );
    found
}

/// The default of `key` in `table`, a string. Empty if the registry has none,
/// which a test catches.
pub(crate) fn default_text(table: &str, key: &str) -> &'static str {
    match default_of(table, key) {
        Some(SettingValue::Text(s)) => s,
        _ => "",
    }
}

/// The default of `key` in `table`, a boolean.
pub(crate) fn default_switch(table: &str, key: &str) -> bool {
    matches!(default_of(table, key), Some(SettingValue::Switch(true)))
}

/// The default of `key` in `table`, a list of strings. Empty if the
/// registry has none, which a test catches.
pub(crate) fn default_list(table: &str, key: &str) -> &'static [String] {
    match default_of(table, key) {
        Some(SettingValue::List(items)) => items,
        _ => &[],
    }
}

/// The default of `key` in `table`, a whole number. 0 if the registry has
/// none, which a test catches.
pub(crate) fn default_number(table: &str, key: &str) -> u32 {
    match default_of(table, key) {
        Some(SettingValue::Number(n)) => u32::try_from(*n).unwrap_or(0),
        _ => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_registry_parses_and_names_each_key_once() {
        let all = settings();
        assert!(!all.is_empty(), "settings.toml doesn't parse");
        for (i, s) in all.iter().enumerate() {
            assert!(
                !all[..i]
                    .iter()
                    .any(|t| t.table == s.table && t.key == s.key),
                "{} {} is in the registry twice",
                s.table,
                s.key
            );
            assert!(!s.description.trim().is_empty(), "{} {}", s.table, s.key);
        }
    }

    #[test]
    fn a_choice_has_its_default_among_its_choices() {
        for s in settings() {
            if let (false, Some(SettingValue::Text(d))) = (s.choices.is_empty(), &s.default) {
                assert!(s.choices.contains(d), "{} {}: {d}", s.table, s.key);
            }
        }
    }

    /// A key whose values the loader reads through a type of its own lists
    /// the same values here, in the same order.
    #[test]
    fn a_choice_read_through_a_type_has_the_types_values() {
        use crate::inline::InlineMarkup;
        use crate::model::{CheckLevel, Listing, Permission, Purpose};
        use crate::types::FieldRole;
        use crate::vale::Preset;

        let listings: Vec<&str> = Listing::ALL.iter().map(|l| l.name()).collect();
        let permissions: Vec<&str> = Permission::ALL.iter().map(|p| p.name()).collect();
        let pairs: Vec<((&str, &str), Vec<&str>)> = vec![
            (("checks.<check>", "level"), CheckLevel::NAMES.to_vec()),
            (("listing", "default"), listings.clone()),
            (("listing", "older-lines"), listings),
            (("checks.vale", "preset"), Preset::names()),
            (
                (FIELD_TABLE, "role"),
                FieldRole::ALL.iter().map(|r| r.name()).collect(),
            ),
            (
                (FIELD_TABLE, "inline"),
                [InlineMarkup::Code].iter().map(|m| m.name()).collect(),
            ),
        ];
        for ((table, key), names) in pairs {
            assert_eq!(choices(table, key), names, "{table} {key}");
        }
        for purpose in Purpose::ALL {
            assert_eq!(choices("crawlers", purpose.name()), permissions);
        }
        let mut purposes: Vec<&str> = Purpose::ALL.iter().map(|p| p.name()).collect();
        purposes.push("lines");
        assert_eq!(keys("crawlers"), purposes);
    }

    #[test]
    fn an_entry_with_a_field_the_registry_doesnt_define_is_refused() {
        let ok = "[[setting]]\ntable = \"\"\nkey = \"a\"\nkind = \"text\"\ntype = \"string\"\ndescription = \"A.\"\n";
        assert!(parse_settings(ok).is_some());
        assert!(parse_settings(&format!("{ok}defualt = \"x\"\n")).is_none());
    }

    #[test]
    fn every_table_the_registry_names_is_a_key_of_its_parent() {
        for s in settings() {
            if s.table.is_empty() || s.table == FIELD_TABLE {
                continue;
            }
            let (parent, last) = match s.table.rsplit_once('.') {
                Some((p, l)) => (p, l),
                None => ("", s.table.as_str()),
            };
            let named = last.starts_with('<');
            let (parent, last) = if named {
                match parent.rsplit_once('.') {
                    Some((p, l)) => (p, l),
                    None => ("", parent),
                }
            } else {
                (parent, last)
            };
            assert!(
                find(parent, last).is_some(),
                "{}: no `{last}` in `{parent}`",
                s.table
            );
        }
    }
}
