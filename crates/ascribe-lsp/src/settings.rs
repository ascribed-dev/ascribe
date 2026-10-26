//! The `ascribe/settings` request, and `ascribe/edit`'s `setSetting`: what
//! Project Settings shows and changes. Every setting comes from the registry
//! the loader reads its keys and defaults from (`ascribe_model::settings`),
//! and its value, its place, and its problems from `ascribe.toml` as it is
//! now: the open buffer, unsaved text included, or the file on disk, even
//! while it doesn't load. Each answer names the version of the buffer it
//! read, so the editor draws an answer only for the text it has, and each
//! edit is refused for any other version: the tab and the file can't fall
//! out of step (decision #658).
//!
//! A setting is changed in place with [`ModelFile`], which keeps the file's
//! comments, blank lines, and order: set where the file writes it, added to
//! its table where it doesn't, and reset to its default by taking its line
//! out, with the comment that touches it (decision 8 of the editor's next
//! views). The edit isn't refused for a problem the model then has: the
//! server checks it as it checks typing, and the problem shows under the
//! setting.
//!
//! The checks are a list of their own: every configurable check of the
//! diagnostics registry, with its level and, for the three that take one,
//! its `limit`, the ones the file sets first (decision 7). A check's level
//! and limit are set and reset the same way, as the settings of the table
//! `checks.<check>`, in whichever form the file writes the check: a level
//! (`page-size = "warning"`), an inline table, or a `[checks.<check>]`
//! table.

use std::collections::HashMap;
use std::path::PathBuf;

use ascribe_check::registry::AREAS;
use ascribe_check::{Registry, default_limit, size_text};
use ascribe_core::{DiagnosticSlug, LineIndex, Span, diagnostics};
use ascribe_model::settings::{
    FIELD_TABLE, Setting, SettingValue as Default_, default_for, settings,
};
use lsp_types::{Range, TextDocumentIdentifier, WorkspaceEdit};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value as Json};
use toml_edit::{InlineTable, Item, Value};

use crate::edit::EditResult;
use crate::model_file::{Edit, ModelFile, Seg, path};
use crate::position::Encoding;

/// The request's method name.
pub const METHOD: &str = "ascribe/settings";

/// The kinds of setting Project Settings shows and edits in place; the rest,
/// lists and tables, are the file's.
const SIMPLE: [&str; 5] = ["switch", "choice", "number", "text", "level"];

/// The parameters of `ascribe/settings`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingsParams {
    /// Any of the project's files: a source file, `ascribe.toml`, or another
    /// file in the project's folder.
    pub text_document: TextDocumentIdentifier,
}

/// The answer to `ascribe/settings`. A document that isn't one of the
/// project's files gets no settings.
#[derive(Debug, Default, Serialize, PartialEq)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct SettingsResult {
    /// The `file:` URI of the project's `ascribe.toml`, which the ranges are
    /// in.
    pub model_uri: Option<String>,
    /// The version of `ascribe.toml`'s open document the answer was read
    /// from; `null` when it isn't open, and the answer was read from the
    /// disk.
    pub version: Option<i32>,
    /// Why the file's values can't be read: it isn't valid TOML. `null` when
    /// they can.
    pub unreadable: Option<String>,
    /// Every setting of a simple kind in a table a project doesn't name
    /// (`[project]`, `[consumer]`, `[editor]`, and the rest, not
    /// `[types.<name>]`), in the content model reference's order.
    pub settings: Vec<SettingEntry>,
    /// Every configurable check: the ones the file sets first, then the
    /// rest, each in the diagnostics reference's order.
    pub checks: Vec<CheckEntry>,
    /// The parts of the project with structure the file names: the
    /// frontmatter every type shares, then each content type, dimension,
    /// release target, build, and source, in the content model reference's
    /// order and the file's within each.
    pub parts: Vec<PartEntry>,
}

/// A part of the project with structure, which Project Settings shows as a
/// card that opens the file at its table.
#[derive(Debug, Serialize, PartialEq)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct PartEntry {
    /// The table it's in: `types`, `dimensions`, `releases`, `builds`, or
    /// `sources`; empty for a table at the top level, `[frontmatter]`.
    pub table: String,
    /// Its name in that table: `guide`, or `frontmatter`.
    pub name: String,
    /// The keys the file sets in it, in the file's order.
    pub keys: Vec<String>,
    /// Where the file names it: its key, in its table's header or its
    /// dotted key. `null` when it can't be placed.
    pub set: Option<SettingPlace>,
}

/// One configurable check, as `[checks]` sets it.
#[derive(Debug, Serialize, PartialEq)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct CheckEntry {
    /// Its slug, which `[checks]` names it by: `page-size`.
    pub slug: String,
    /// Its code: `ASC149`.
    pub code: String,
    /// Its name for people: the slug in words, `Page size`.
    pub name: String,
    /// The area the diagnostics reference lists it under: `Pages`.
    pub area: String,
    /// What it reports: the registry's `when`, as plain text and code spans,
    /// as a setting's description is.
    pub when: Vec<SettingPiece>,
    /// The levels `[checks]` takes, in the order a message lists them.
    pub choices: Vec<String>,
    /// Its own level, which it has when the file sets none: `advice`,
    /// `warning`, or `error`.
    pub default: String,
    /// Whether the file names the check in `[checks]` at all.
    pub named: bool,
    /// The level the file sets; `null` when it sets none, or writes one that
    /// isn't a string.
    pub level: Option<String>,
    /// Where the file sets its level; `null` when it doesn't.
    pub set: Option<SettingPlace>,
    /// The server's problems on the lines that set its level.
    pub problems: Vec<SettingProblem>,
    /// Its `limit`, for a check that takes one: `page-size`, `image-large`,
    /// `memo-long`, and `summary-long`.
    pub limit: Option<CheckLimit>,
}

/// A check's `limit`.
#[derive(Debug, Serialize, PartialEq)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct CheckLimit {
    /// `number`, a whole number above 0; or `size`, a number of bytes or a
    /// string with a unit, `"500 KB"`.
    pub kind: String,
    /// The limit used when the file sets none.
    pub default: SettingValue,
    /// The default in words, with its unit: `50,000 characters`, `500 KB`.
    pub default_text: String,
    /// The limit the file sets; `null` when it sets none.
    pub value: Option<SettingValue>,
    /// Where the file sets it; `null` when it doesn't.
    pub set: Option<SettingPlace>,
    /// The server's problems on the lines that set it.
    pub problems: Vec<SettingProblem>,
}

/// One setting.
#[derive(Debug, Serialize, PartialEq)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct SettingEntry {
    /// Its table, as a dotted path: `consumer`, or empty for the top level.
    pub table: String,
    /// Its key.
    pub key: String,
    /// Its name for people: the key in words, `Trailing slash`.
    pub name: String,
    /// What its value is: `switch`, `choice`, `number`, `text`, or `level`.
    pub kind: String,
    /// For a choice or a level, its values, in the order a message lists
    /// them.
    pub choices: Vec<String>,
    /// The value used when the file leaves it out, when it has one.
    pub default: Option<SettingValue>,
    /// The reference's words for a default that isn't one value.
    pub default_text: Option<String>,
    /// What it does, as plain text: the registry's Markdown with a link's
    /// address dropped and its text kept, emphasis taken off, and code spans
    /// as pieces of their own.
    pub description: Vec<SettingPiece>,
    /// Its value in the file; `null` when the file leaves it out, or writes
    /// it as something other than a string, a boolean, or a whole number.
    pub value: Option<SettingValue>,
    /// Where the file sets it; `null` when it leaves it out.
    pub set: Option<SettingPlace>,
    /// The server's problems on the lines that set it, in its words.
    pub problems: Vec<SettingProblem>,
}

/// A setting's value.
#[derive(Clone, Debug, Serialize, PartialEq)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(untagged)]
pub enum SettingValue {
    /// A switch's.
    Switch(bool),
    /// A number's.
    Number(i64),
    /// A string: a choice's, a level's, or a text's.
    Text(String),
}

/// A piece of a description.
#[derive(Debug, Serialize, PartialEq)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct SettingPiece {
    /// Its text.
    pub text: String,
    /// Whether it's a code span.
    pub code: bool,
}

/// Where the file sets a setting.
#[derive(Debug, Serialize, PartialEq)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct SettingPlace {
    /// From its key to the end of its value.
    #[cfg_attr(feature = "json-schema", schemars(with = "crate::schema::LspRange"))]
    pub range: Range,
    /// The line its key is on, counted from 1, as `ascribe.toml:12` names it.
    pub line: u32,
}

/// A problem the server reports on a setting.
#[derive(Debug, Serialize, PartialEq)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct SettingProblem {
    /// The message, as the Problems panel shows it.
    pub message: String,
    /// `error`, `warning`, or `advice`, at the level `[checks]` sets.
    pub severity: String,
}

/// `ascribe.toml` as it is now, and its problems: what both answers read.
pub(crate) struct ModelNow {
    /// The project's `ascribe.toml`.
    pub config: PathBuf,
    /// Its text: the open buffer, else the file.
    pub text: String,
    /// The open buffer's version; `None` when it isn't open.
    pub version: Option<i32>,
    /// Its problems, as they're published.
    pub diagnostics: Vec<ascribe_check::Diagnostic>,
    pub encoding: Encoding,
    /// The files its `[from]` takes from, where a taken setting is set.
    pub taken: Vec<crate::model_file::TakenFile>,
}

/// The settings Project Settings shows: the registry's simple keys of the
/// tables a project doesn't name parts of, and that aren't the items of a
/// list (`[[intended]]`'s).
fn shown() -> impl Iterator<Item = &'static Setting> {
    settings().iter().filter(|s| {
        SIMPLE.contains(&s.kind.as_str())
            && s.table != FIELD_TABLE
            && !s.table.contains('<')
            && !settings()
                .iter()
                .any(|list| list.kind == "list" && joined(&list.table, &list.key) == s.table)
    })
}

/// A table's path and a key, as a dotted path.
fn joined(table: &str, key: &str) -> String {
    if table.is_empty() {
        key.to_owned()
    } else {
        format!("{table}.{key}")
    }
}

/// The configurable checks, in the diagnostics reference's order: by its
/// areas, and in the registry's order within each.
fn configurable() -> impl Iterator<Item = (&'static ascribe_check::Entry, &'static str)> {
    let registry = Registry::global();
    AREAS.iter().flat_map(move |(area, title)| {
        registry
            .entries()
            .filter(move |e| e.configurable && e.area.as_deref() == Some(*area))
            .map(move |e| (e, *title))
    })
}

/// The configurable check `checks.<check>` names, for a table of that form.
fn check_of(table: &str) -> Option<DiagnosticSlug> {
    let slug = DiagnosticSlug::from_name(table.strip_prefix("checks.")?)?;
    diagnostics::CONFIGURABLE.contains(&slug).then_some(slug)
}

/// The problems published on a span of the file.
fn problems_on(now: &ModelNow, span: Span) -> Vec<SettingProblem> {
    now.diagnostics
        .iter()
        .filter(|d| {
            let at = d.location.span;
            at.start() < span.end() && span.start() < at.end().max(at.start() + 1)
        })
        .map(|d| SettingProblem {
            message: d.message.clone(),
            severity: d.severity.as_str().to_owned(),
        })
        .collect()
}

fn setting_place(now: &ModelNow, index: &LineIndex, span: Span) -> SettingPlace {
    SettingPlace {
        range: now.encoding.range(index, span),
        line: index.line_col(span.start()).map_or(0, |p| p.line) + 1,
    }
}

/// The checks as the file sets them.
fn checks_now(now: &ModelNow, file: Option<&ModelFile<'_>>, index: &LineIndex) -> Vec<CheckEntry> {
    let choices = settings()
        .iter()
        .find(|s| s.table == "checks.<check>" && s.key == "level")
        .map(|s| s.choices.clone())
        .unwrap_or_default();
    let (mut named, mut rest): (Vec<CheckEntry>, Vec<CheckEntry>) = configurable()
        .map(|(entry, area)| {
            let slug = entry.slug.as_str();
            let at = path(&["checks", slug]);
            let item = file.and_then(|f| f.get(&at));
            // A level written alone, `page-size = "warning"`, or a table's
            // `level`.
            let level_at = match item {
                Some(Item::Value(Value::String(_))) => at.clone(),
                _ => path(&["checks", slug, "level"]),
            };
            let level_span = file.and_then(|f| place(f, &level_at));
            let limit = DiagnosticSlug::from_name(slug)
                .and_then(default_limit)
                .map(|default| {
                    let limit_at = path(&["checks", slug, "limit"]);
                    let span = file.and_then(|f| place(f, &limit_at));
                    let size = slug == diagnostics::IMAGE_LARGE.as_str();
                    CheckLimit {
                        kind: if size { "size" } else { "number" }.to_owned(),
                        default: SettingValue::Number(i64::try_from(default).unwrap_or(i64::MAX)),
                        default_text: if size {
                            size_text(default)
                        } else {
                            format!("{} characters", ascribe_check::page::thousands(default))
                        },
                        value: file
                            .and_then(|f| f.get(&limit_at))
                            .and_then(Item::as_value)
                            .and_then(value_of),
                        set: span.map(|span| setting_place(now, index, span)),
                        problems: span.map(|span| problems_on(now, span)).unwrap_or_default(),
                    }
                });
            CheckEntry {
                slug: slug.to_owned(),
                code: entry.code.clone(),
                name: name(slug),
                area: area.to_owned(),
                when: pieces(&sentence(entry.when.as_deref().unwrap_or_default())),
                choices: choices.clone(),
                default: if entry.off_by_default {
                    "off"
                } else {
                    entry.severity.as_str()
                }
                .to_owned(),
                named: item.is_some(),
                level: file
                    .and_then(|f| f.get(&level_at))
                    .and_then(Item::as_str)
                    .map(str::to_owned),
                set: level_span.map(|span| setting_place(now, index, span)),
                problems: level_span
                    .map(|span| problems_on(now, span))
                    .unwrap_or_default(),
                limit,
            }
        })
        .partition(|c| c.named);
    named.append(&mut rest);
    named
}

/// A `when` as a sentence: it ends in a period, as the diagnostics reference
/// writes it.
fn sentence(when: &str) -> String {
    if when.is_empty() || when.ends_with('.') {
        when.to_owned()
    } else {
        format!("{when}.")
    }
}

/// The answer to `ascribe/settings`.
pub(crate) fn settings_now(now: &ModelNow) -> SettingsResult {
    let index = LineIndex::new(&now.text);
    let file = ModelFile::parse(&now.text);
    let unreadable = file.as_ref().err().cloned();
    // A value `[from]` takes is shown from the file that holds it; where a
    // value is set is a place in this one.
    let held = ModelFile::parse_taking(&now.text, &now.taken);
    // Whether the file sets a key, for a default that follows another key.
    let written = |t: &str, k: &str| {
        held.as_ref()
            .ok()
            .and_then(|f| f.get(&path(&[t, k])))
            .is_some()
    };
    let settings = shown()
        .map(|setting| {
            let place = file
                .as_ref()
                .ok()
                .and_then(|f| place(f, &key_path(setting)));
            let value = held
                .as_ref()
                .ok()
                .and_then(|f| f.get(&key_path(setting)))
                .and_then(Item::as_value)
                .and_then(value_of);
            let problems = place.map(|span| problems_on(now, span)).unwrap_or_default();
            SettingEntry {
                table: setting.table.clone(),
                key: setting.key.clone(),
                name: name(&setting.key),
                kind: setting.kind.clone(),
                choices: setting.choices.clone(),
                default: default_for(&setting.table, &setting.key, &written).and_then(
                    |d| match d {
                        Default_::Switch(b) => Some(SettingValue::Switch(b)),
                        Default_::Number(n) => Some(SettingValue::Number(n)),
                        Default_::Text(t) => Some(SettingValue::Text(t)),
                        Default_::List(_) | Default_::EmptyTable => None,
                    },
                ),
                default_text: setting.default_text.clone(),
                description: pieces(&setting.description),
                value,
                set: place.map(|span| setting_place(now, &index, span)),
                problems,
            }
        })
        .collect();
    let checks = checks_now(now, file.as_ref().ok(), &index);
    let parts = file
        .as_ref()
        .map(|f| parts_now(now, f, &index))
        .unwrap_or_default();
    SettingsResult {
        model_uri: crate::uri::path_to_uri(&now.config).map(|u| u.as_str().to_owned()),
        version: now.version,
        unreadable,
        settings,
        checks,
        parts,
    }
}

/// The tables whose parts a project names, and Project Settings shows as
/// cards, in the content model reference's order.
const PART_TABLES: [&str; 5] = ["types", "dimensions", "releases", "builds", "sources"];

/// The parts the file names: `[frontmatter]`, whose keys are the fields every
/// type shares, then each table of [`PART_TABLES`]'s entries that is a table
/// itself, which leaves out a key of the table's own, such as `[releases]
/// planned-label`.
fn parts_now(now: &ModelNow, file: &ModelFile<'_>, index: &LineIndex) -> Vec<PartEntry> {
    let mut parts = Vec::new();
    if let Some(root) = file.table_like(&path(&[]))
        && let Some((key, item)) = root.get_key_value("frontmatter")
        && let Some(shared) = item.as_table_like()
    {
        parts.push(PartEntry {
            table: String::new(),
            name: "frontmatter".to_owned(),
            keys: shared.iter().map(|(k, _)| k.to_owned()).collect(),
            set: key
                .span()
                .map(|span| setting_place(now, index, Span::new(span.start, span.end))),
        });
    }
    for table in PART_TABLES {
        let Some(entries) = file.table_like(&path(&[table])) else {
            continue;
        };
        for (name, item) in entries.iter() {
            let Some(part) = item.as_table_like() else {
                continue;
            };
            let key_span = entries.get_key_value(name).and_then(|(key, _)| key.span());
            parts.push(PartEntry {
                table: table.to_owned(),
                name: name.to_owned(),
                keys: part.iter().map(|(k, _)| k.to_owned()).collect(),
                set: key_span
                    .map(|span| setting_place(now, index, Span::new(span.start, span.end))),
            });
        }
    }
    parts
}

/// `setSetting`: `args.table` and `args.key` name a setting, `args.value` is
/// its new value, of its kind, or `null` to reset it to its default, and
/// `args.shown` is the `version` of the answer the setting was shown from: a
/// number, or `null` for the disk. A value the file sets is replaced where it
/// is, one it leaves out is added to its table, and a reset takes the line
/// out with the comment that touches it. A check's level and limit are the
/// table `checks.<check>`'s `level` and `limit`. It's refused for any other
/// version than `ascribe.toml`'s now, a buffer's for the disk's or the
/// disk's for a buffer's included.
pub(crate) fn set_setting(now: &ModelNow, args: &Map<String, Json>) -> EditResult {
    match plan(now, args) {
        Ok(edit) => EditResult::Edit { edit, select: None },
        Err(error) => EditResult::Failed { error },
    }
}

// `Uri` hashes by its text; the map is built once and never mutated through
// a key.
#[allow(clippy::mutable_key_type)]
fn plan(now: &ModelNow, args: &Map<String, Json>) -> Result<WorkspaceEdit, String> {
    let seen = match args.get("shown") {
        Some(Json::Null) => None,
        Some(Json::Number(n)) if n.as_i64().is_some_and(|n| i32::try_from(n).is_ok()) => {
            n.as_i64().and_then(|n| i32::try_from(n).ok())
        }
        _ => {
            return Err(
                "`shown` must be the version the setting was shown from, or null.".to_owned(),
            );
        }
    };
    if seen != now.version {
        return Err(
            "`ascribe.toml` changed after the setting was shown. Choose the value again."
                .to_owned(),
        );
    }
    let text_arg = |name: &str| match args.get(name) {
        Some(Json::String(s)) => Ok(s.as_str()),
        _ => Err(format!("`{name}` must be a string.")),
    };
    let (table, key) = (text_arg("table")?, text_arg("key")?);
    let file = ModelFile::parse_taking(&now.text, &now.taken)?;
    let edits = match check_of(table) {
        Some(slug) => plan_check(&file, slug, key, args.get("value"))?,
        None => {
            let setting = shown()
                .find(|s| s.table == table && s.key == key)
                .ok_or_else(|| {
                    format!(
                        "`{}` isn't a setting Project Settings changes.",
                        named(table, key)
                    )
                })?;
            let value = match (setting.kind.as_str(), args.get("value")) {
                (_, Some(Json::Null)) => None,
                ("switch", Some(Json::Bool(b))) => Some(Value::from(*b)),
                ("number", Some(Json::Number(n))) if n.as_i64().is_some() => {
                    Some(Value::from(n.as_i64().unwrap_or_default()))
                }
                ("choice" | "level" | "text", Some(Json::String(s))) => {
                    Some(Value::from(s.as_str()))
                }
                (kind, _) => {
                    let wants = match kind {
                        "switch" => "true or false",
                        "number" => "a whole number",
                        _ => "a string",
                    };
                    return Err(format!("`{}` takes {wants}.", named(table, key)));
                }
            };
            let keys: Vec<&str> = setting.table.split('.').filter(|k| !k.is_empty()).collect();
            let at = key_path(setting);
            match (file.get(&at), value) {
                (None, None) => Vec::new(),
                (Some(_), None) => vec![file.remove_entry(&keys, &setting.key)?],
                (Some(current), Some(value)) => replaced(&file, &at, current, value)?,
                (None, Some(value)) => vec![file.add_entry(&keys, &setting.key, value)?],
            }
        }
    };
    let uri = crate::uri::path_to_uri(&now.config)
        .ok_or_else(|| "`ascribe.toml` can't be named as a URI.".to_owned())?;
    let mut changes = HashMap::new();
    if edits.is_empty() {
        changes.insert(uri, Vec::new());
    } else {
        let finished = file.finish(edits)?;
        let index = LineIndex::new(&now.text);
        let edits: Vec<lsp_types::TextEdit> = finished
            .edits
            .iter()
            .map(|e| lsp_types::TextEdit {
                range: now.encoding.range(&index, e.span),
                new_text: e.new_text.clone(),
            })
            .collect();
        // A setting `[from]` takes is set in the file that holds it.
        let taken = finished
            .taken_changes(now.encoding)
            .ok_or_else(|| "A file `[from]` takes from can't be named as a URI.".to_owned())?;
        if !edits.is_empty() || taken.is_empty() {
            changes.insert(uri, edits);
        }
        changes.extend(taken);
    }
    Ok(WorkspaceEdit {
        changes: Some(changes),
        document_changes: None,
        change_annotations: None,
    })
}

/// The edit that puts `value` where `current` is, at `at`: none when it's
/// the value there already.
fn replaced(
    file: &ModelFile<'_>,
    at: &[Seg],
    current: &Item,
    value: Value,
) -> Result<Vec<Edit>, String> {
    let same = current
        .as_value()
        .and_then(value_of)
        .is_some_and(|v| Some(v) == value_of(&value));
    Ok(if same {
        Vec::new()
    } else {
        vec![file.replace_value(at, value)?]
    })
}

/// The edits that set a check's `level` or `limit`, or reset it with `null`,
/// in the form the file writes the check in: a level alone, which takes a
/// limit by becoming an inline table; an inline table, rewritten with its
/// other keys as they are; or a `[checks.<check>]` table, whose lines are
/// set, added, and removed as a setting's are. A check the file doesn't name
/// gets `<check> = "level"` in `[checks]`, or a `[checks.<check>]` table of
/// its own for a limit, or when the file has no `[checks]` table to add to.
fn plan_check(
    file: &ModelFile<'_>,
    slug: DiagnosticSlug,
    key: &str,
    value: Option<&Json>,
) -> Result<Vec<Edit>, String> {
    let slug = slug.as_str();
    let named = format!("[checks.{slug}] {key}");
    let limited = DiagnosticSlug::from_name(slug)
        .and_then(default_limit)
        .is_some();
    let size = slug == diagnostics::IMAGE_LARGE.as_str();
    let value = match (key, value) {
        ("level", Some(Json::Null)) => None,
        ("level", Some(Json::String(s))) => Some(Value::from(s.as_str())),
        ("level", _) => return Err(format!("`{named}` takes a string, or null.")),
        ("limit", Some(Json::Null)) if limited => None,
        ("limit", Some(Json::Number(n))) if limited && n.as_i64().is_some() => {
            Some(Value::from(n.as_i64().unwrap_or_default()))
        }
        ("limit", Some(Json::String(s))) if size => Some(Value::from(s.as_str())),
        ("limit", _) if limited => {
            let wants = if size {
                "a whole number of bytes, a size such as \"500 KB\", or null"
            } else {
                "a whole number, or null"
            };
            return Err(format!("`{named}` takes {wants}."));
        }
        _ => {
            return Err(format!(
                "`{named}` isn't a setting Project Settings changes."
            ));
        }
    };
    let at = path(&["checks", slug]);
    let own = path(&["checks", slug, key]);
    let by_hand = || {
        format!(
            "`ascribe.toml` writes `[checks] {slug}` in a form this can't change. Change it by hand."
        )
    };
    Ok(match (file.get(&at), value) {
        (None, None) => Vec::new(),
        (None, Some(value)) => {
            let header = matches!(
                file.get(&path(&["checks"])),
                Some(Item::Table(t)) if !t.is_dotted() && !t.is_implicit()
            );
            if key == "level" && header {
                vec![file.add_entry(&["checks"], slug, value)?]
            } else {
                vec![file.add_table(&["checks", slug], &[(key, value)], &["checks"])?]
            }
        }
        (Some(item @ Item::Value(Value::String(level))), value) => match (key, value) {
            ("level", None) => vec![file.remove_entry(&["checks"], slug)?],
            ("level", Some(value)) => replaced(file, &at, item, value)?,
            (_, None) => Vec::new(),
            (_, Some(value)) => {
                let mut table = InlineTable::new();
                table.insert("level", Value::from(level.value().as_str()));
                table.insert(key, value);
                table.fmt();
                vec![file.replace_value(&at, Value::InlineTable(table))?]
            }
        },
        (Some(Item::Value(Value::InlineTable(current))), value) => {
            match (current.get(key), value) {
                (None, None) => Vec::new(),
                (Some(_), Some(value)) => {
                    let item = file.get(&own).ok_or_else(by_hand)?;
                    replaced(file, &own, item, value)?
                }
                (_, value) => {
                    let mut table = InlineTable::new();
                    for (k, v) in current.iter().filter(|(k, _)| *k != key) {
                        let mut v = v.clone();
                        v.decor_mut().clear();
                        table.insert(k, v);
                    }
                    if let Some(value) = value {
                        table.insert(key, value);
                    }
                    if table.is_empty() {
                        vec![file.remove_entry(&["checks"], slug)?]
                    } else {
                        table.fmt();
                        vec![file.replace_value(&at, Value::InlineTable(table))?]
                    }
                }
            }
        }
        (Some(Item::Table(_)), value) => match (file.get(&own), value) {
            (None, None) => Vec::new(),
            (Some(_), None) => vec![file.remove_entry(&["checks", slug], key)?],
            (Some(current), Some(value)) => replaced(file, &own, current, value)?,
            (None, Some(value)) => vec![file.add_entry(&["checks", slug], key, value)?],
        },
        (Some(_), _) => return Err(by_hand()),
    })
}

/// A setting as a message names it: `[consumer] trailing-slash`.
fn named(table: &str, key: &str) -> String {
    if table.is_empty() {
        key.to_owned()
    } else {
        format!("[{table}] {key}")
    }
}

fn key_path(setting: &Setting) -> Vec<Seg> {
    let mut keys: Vec<&str> = setting.table.split('.').filter(|k| !k.is_empty()).collect();
    keys.push(&setting.key);
    path(&keys)
}

/// Where the file sets the key at `at`: from the key to the end of its
/// value.
fn place(file: &ModelFile<'_>, at: &[Seg]) -> Option<Span> {
    let (Seg::Key(last), parent) = at.split_last()? else {
        return None;
    };
    let (key, item) = file.table_like(parent)?.get_key_value(last)?;
    let (k, v) = (key.span()?, item.span()?);
    Some(Span::new(k.start, v.end.max(k.end)))
}

fn value_of(value: &Value) -> Option<SettingValue> {
    match value {
        Value::String(s) => Some(SettingValue::Text(s.value().clone())),
        Value::Boolean(b) => Some(SettingValue::Switch(*b.value())),
        Value::Integer(i) => Some(SettingValue::Number(*i.value())),
        _ => None,
    }
}

/// A key in words: `trailing-slash` is "Trailing slash".
fn name(key: &str) -> String {
    let words = key.replace(['-', '_'], " ");
    let mut chars = words.chars();
    chars
        .next()
        .map(|c| c.to_uppercase().chain(chars).collect())
        .unwrap_or_default()
}

/// A description's Markdown as plain text and code spans: a link's text is
/// kept and its address dropped, `**` and `*` emphasis is taken off, and a
/// backslash escape is the character it escapes. A registry description
/// uses phrases only in links' addresses, which are dropped.
pub(crate) fn pieces(markdown: &str) -> Vec<SettingPiece> {
    let mut out: Vec<SettingPiece> = Vec::new();
    let mut push = |text: &str, code: bool| {
        if text.is_empty() {
            return;
        }
        match out.last_mut() {
            Some(last) if !last.code && !code => last.text.push_str(text),
            _ => out.push(SettingPiece {
                text: text.to_owned(),
                code,
            }),
        }
    };
    let bytes = markdown.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'\\' if bytes.get(i + 1).is_some_and(u8::is_ascii_punctuation) => {
                push(&markdown[i + 1..i + 2], false);
                i += 2;
            }
            b'`' => {
                let n = bytes[i..].iter().take_while(|b| **b == b'`').count();
                let fence = &markdown[i..i + n];
                match markdown[i + n..].find(fence) {
                    Some(len) => {
                        let inner = &markdown[i + n..i + n + len];
                        let inner =
                            if inner.starts_with(' ') && inner.ends_with(' ') && inner.len() > 1 {
                                &inner[1..inner.len() - 1]
                            } else {
                                inner
                            };
                        push(inner, true);
                        i += 2 * n + len;
                    }
                    None => {
                        push(fence, false);
                        i += n;
                    }
                }
            }
            b'*' => i += 1,
            // A link's text stays; `](address)` goes.
            b'[' => i += 1,
            b']' if bytes.get(i + 1) == Some(&b'(') => {
                let close = markdown[i..].find(')').map_or(bytes.len(), |c| i + c + 1);
                i = close;
            }
            _ => {
                let ch = markdown[i..].chars().next().unwrap_or_default();
                let mut buf = [0; 4];
                push(ch.encode_utf8(&mut buf), false);
                i += ch.len_utf8().max(1);
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn plain(markdown: &str) -> Vec<(String, bool)> {
        pieces(markdown)
            .into_iter()
            .map(|p| (p.text, p.code))
            .collect()
    }

    #[test]
    fn a_description_keeps_its_words_and_its_code_spans() {
        assert_eq!(
            plain(
                "**Routing.** Whether URLs end in `/`: `\"always\"`, as [SPEC §2.2]({repo}/blob/main/SPEC.md#22) says\\*."
            ),
            vec![
                ("Routing. Whether URLs end in ".to_owned(), false),
                ("/".to_owned(), true),
                (": ".to_owned(), false),
                ("\"always\"".to_owned(), true),
                (", as SPEC §2.2 says*.".to_owned(), false),
            ]
        );
        assert_eq!(
            plain("[`ascribe.toml`](x.md) and `` a`b ``"),
            vec![
                ("ascribe.toml".to_owned(), true),
                (" and ".to_owned(), false),
                ("a`b".to_owned(), true),
            ]
        );
    }

    #[test]
    fn no_shown_description_is_left_with_markup() {
        for setting in shown() {
            for piece in pieces(&setting.description).iter().filter(|p| !p.code) {
                assert!(
                    !piece.text.contains(['[', ']', '`', '*']),
                    "{}: {}",
                    setting.key,
                    piece.text
                );
            }
        }
    }

    #[test]
    fn a_key_in_words() {
        assert_eq!(name("trailing-slash"), "Trailing slash");
        assert_eq!(name("spec"), "Spec");
    }
}
