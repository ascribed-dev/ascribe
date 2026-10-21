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
//! comments, blank lines, and order. The edit isn't refused for a problem
//! the model then has: the server checks it as it checks typing, and the
//! problem shows under the setting.

use std::collections::HashMap;
use std::path::PathBuf;

use ascribe_core::{LineIndex, Span};
use ascribe_model::settings::{FIELD_TABLE, Setting, SettingValue as Default_, settings};
use lsp_types::{Range, TextDocumentIdentifier, WorkspaceEdit};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value as Json};
use toml_edit::{Item, Value};

use crate::edit::EditResult;
use crate::model_file::{ModelFile, Seg, path};
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
}

/// The settings Project Settings shows: the registry's simple keys of the
/// tables a project doesn't name parts of.
fn shown() -> impl Iterator<Item = &'static Setting> {
    settings().iter().filter(|s| {
        SIMPLE.contains(&s.kind.as_str()) && s.table != FIELD_TABLE && !s.table.contains('<')
    })
}

/// The answer to `ascribe/settings`.
pub(crate) fn settings_now(now: &ModelNow) -> SettingsResult {
    let index = LineIndex::new(&now.text);
    let file = ModelFile::parse(&now.text);
    let unreadable = file.as_ref().err().cloned();
    let settings = shown()
        .map(|setting| {
            let place = file.as_ref().ok().and_then(|f| place(f, setting));
            let value = file
                .as_ref()
                .ok()
                .and_then(|f| f.get(&key_path(setting)))
                .and_then(Item::as_value)
                .and_then(value_of);
            let problems = place
                .map(|span| {
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
                })
                .unwrap_or_default();
            SettingEntry {
                table: setting.table.clone(),
                key: setting.key.clone(),
                name: name(&setting.key),
                kind: setting.kind.clone(),
                choices: setting.choices.clone(),
                default: setting.default.as_ref().and_then(|d| match d {
                    Default_::Switch(b) => Some(SettingValue::Switch(*b)),
                    Default_::Number(n) => Some(SettingValue::Number(*n)),
                    Default_::Text(t) => Some(SettingValue::Text(t.clone())),
                    Default_::EmptyList | Default_::EmptyTable => None,
                }),
                default_text: setting.default_text.clone(),
                description: pieces(&setting.description),
                value,
                set: place.map(|span| SettingPlace {
                    range: now.encoding.range(&index, span),
                    line: index.line_col(span.start()).map_or(0, |p| p.line) + 1,
                }),
                problems,
            }
        })
        .collect();
    SettingsResult {
        model_uri: crate::uri::path_to_uri(&now.config).map(|u| u.as_str().to_owned()),
        version: now.version,
        unreadable,
        settings,
    }
}

/// `setSetting`: `args.table` and `args.key` name a setting the file sets,
/// `args.value` is its new value, of its kind, and `args.shown` is the
/// `version` of the answer the setting was shown from: a number, or `null`
/// for the disk. It's refused for any other version than `ascribe.toml`'s
/// now, a buffer's for the disk's or the disk's for a buffer's included.
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
    let setting = shown()
        .find(|s| s.table == table && s.key == key)
        .ok_or_else(|| {
            format!(
                "`{}` isn't a setting Project Settings changes.",
                named(table, key)
            )
        })?;
    let value = match (setting.kind.as_str(), args.get("value")) {
        ("switch", Some(Json::Bool(b))) => Value::from(*b),
        ("number", Some(Json::Number(n))) if n.as_i64().is_some() => {
            Value::from(n.as_i64().unwrap_or_default())
        }
        ("choice" | "level" | "text", Some(Json::String(s))) => Value::from(s.as_str()),
        (kind, _) => {
            let wants = match kind {
                "switch" => "true or false",
                "number" => "a whole number",
                _ => "a string",
            };
            return Err(format!("`{}` takes {wants}.", named(table, key)));
        }
    };
    let file = ModelFile::parse(&now.text)?;
    let at = key_path(setting);
    let current = file.get(&at).ok_or_else(|| {
        format!(
            "`ascribe.toml` doesn't set `{}`. Add it to the file by hand.",
            named(table, key)
        )
    })?;
    let mut changes = HashMap::new();
    let uri = crate::uri::path_to_uri(&now.config)
        .ok_or_else(|| "`ascribe.toml` can't be named as a URI.".to_owned())?;
    let same = current
        .as_value()
        .and_then(value_of)
        .is_some_and(|v| Some(v) == value_of(&value));
    if same {
        changes.insert(uri, Vec::new());
    } else {
        let (edits, _) = file.finish(vec![file.replace_value(&at, value)?])?;
        let index = LineIndex::new(&now.text);
        let edits = edits
            .iter()
            .map(|e| lsp_types::TextEdit {
                range: now.encoding.range(&index, e.span),
                new_text: e.new_text.clone(),
            })
            .collect();
        changes.insert(uri, edits);
    }
    Ok(WorkspaceEdit {
        changes: Some(changes),
        document_changes: None,
        change_annotations: None,
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

/// Where the file sets a setting: from its key to the end of its value.
fn place(file: &ModelFile<'_>, setting: &Setting) -> Option<Span> {
    let at = key_path(setting);
    let (key, item) = file
        .table_like(&at[..at.len() - 1])?
        .get_key_value(&setting.key)?;
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
