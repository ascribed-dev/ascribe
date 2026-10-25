//! The JSON Schema validator, behind one seam: a document holding a schema
//! and a JSON text go in, and the places where the text disagrees with the
//! schema come out, each as a byte range of the text. Nothing else of
//! Ascribe's is imported here, so the validator, `boon`, can be swapped or
//! moved into a process of its own without touching its callers
//! (decision #920).
//!
//! The document is added whole, as one resource named by a URN, and the
//! schema is compiled from its JSON pointer in it, so the document's own
//! `#…` references resolve. Nothing is loaded from a file or the network: a
//! reference into another document isn't checked, and is said.

mod spans;

use std::collections::BTreeSet;
use std::error::Error;
use std::ops::Range;

use boon::{CompileError, Compiler, ErrorKind, InstanceToken, Schemas, Type, ValidationError};
use serde_json::Value;

pub(crate) use spans::token;

/// The name the document is added under.
const BASE: &str = "urn:ascribe:schema";

/// How many patterns are left out of one schema before giving up on it.
const PATTERNS: usize = 256;

/// Which JSON Schema the document's schemas are written in.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Dialect {
    /// Draft 4.
    Draft4,
    /// Draft 2020-12.
    Draft2020,
}

/// One place where the text disagrees with the schema.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Disagreement {
    /// Where, in the text: the value, a property that isn't allowed's key,
    /// or the opening brace of an object missing a required property.
    pub at: Range<usize>,
    /// What the text has there: "`name` is a number".
    pub found: String,
    /// What the schema says instead: "it's a string".
    pub expected: String,
}

/// What checking a text found.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Checked {
    /// The text was checked: every disagreement, none when it agrees, and
    /// the patterns the validator couldn't read, which were left out so the
    /// rest of the schema was still checked.
    Done {
        /// The disagreements, in the text's order.
        disagreements: Vec<Disagreement>,
        /// The patterns left out, as the schema writes them.
        patterns_left_out: Vec<String>,
    },
    /// The text isn't one JSON value.
    NotJson {
        /// Where reading it stopped.
        at: usize,
        /// The reader's reason.
        reason: String,
    },
    /// The schema refers into another document, which isn't loaded, so
    /// nothing was checked.
    Elsewhere {
        /// The reference, as written.
        reference: String,
    },
    /// The schema couldn't be compiled: the validator's reason.
    Unusable(String),
}

/// Checks `text` against the schema at `pointer` in `document`.
pub fn check(document: &Value, dialect: Dialect, pointer: &str, text: &str) -> Checked {
    let instance: Value = match serde_json::from_str(text) {
        Ok(value) => value,
        Err(error) => {
            return Checked::NotJson {
                at: offset_of(text, error.line(), error.column()),
                reason: reason_of(&error),
            };
        }
    };
    if let Some(reference) = elsewhere(document, pointer) {
        return Checked::Elsewhere { reference };
    }
    let mut document = document.clone();
    let mut patterns_left_out = Vec::new();
    let (schemas, index) = loop {
        match compile(&document, dialect, pointer) {
            Ok(compiled) => break compiled,
            Err(CompileError::InvalidRegex { url, regex, .. })
                if patterns_left_out.len() < PATTERNS && leave_out(&mut document, &url) =>
            {
                patterns_left_out.push(regex);
            }
            // The metaschema asserts that a pattern is a regular expression
            // the validator can read.
            Err(CompileError::ValidationError { url, src })
                if patterns_left_out.len() < PATTERNS =>
            {
                // The error's places are in the schema at `url`.
                let base = url
                    .split_once('#')
                    .map(|(_, f)| decode(f))
                    .unwrap_or_default();
                let unread: Vec<String> = unread_patterns(&src)
                    .into_iter()
                    .map(|p| format!("{base}{p}"))
                    .collect();
                if unread.is_empty() {
                    return Checked::Unusable(src.to_string());
                }
                for pointer in unread {
                    let removed = pointer
                        .rsplit_once('/')
                        .and_then(|(parent, _)| document.pointer_mut(parent))
                        .and_then(Value::as_object_mut)
                        .and_then(|schema| schema.remove("pattern"));
                    match removed {
                        Some(Value::String(regex)) => patterns_left_out.push(regex),
                        Some(_) => {}
                        None => return Checked::Unusable(src.to_string()),
                    }
                }
            }
            Err(error) => return Checked::Unusable(error.to_string()),
        }
    };
    let Some(places) = spans::places(text) else {
        // `serde_json` read it, so the walk over the same text can't fail.
        return Checked::Unusable("the example's values couldn't be placed".to_owned());
    };
    let mut disagreements = Vec::new();
    if let Err(error) = schemas.validate(&instance, index) {
        let mut leaves = Vec::new();
        collect(&error, &mut leaves);
        for leaf in leaves {
            disagreements.extend(describe(leaf, &instance, &places));
        }
    }
    let mut seen = BTreeSet::new();
    disagreements.retain(|d| seen.insert((d.at.start, d.at.end, d.found.clone())));
    disagreements.sort_by_key(|d| (d.at.start, d.at.end));
    Checked::Done {
        disagreements,
        patterns_left_out,
    }
}

/// Refuses every address, so nothing outside the document is ever read.
struct NoLoader;

impl boon::UrlLoader for NoLoader {
    fn load(&self, url: &str) -> Result<Value, Box<dyn Error>> {
        Err(format!("`{url}` isn't loaded").into())
    }
}

// `boon`'s own error, which the caller matches once.
#[allow(clippy::result_large_err)]
fn compile(
    document: &Value,
    dialect: Dialect,
    pointer: &str,
) -> Result<(Schemas, boon::SchemaIndex), CompileError> {
    let mut compiler = Compiler::new();
    compiler.use_loader(Box::new(NoLoader));
    compiler.set_default_draft(match dialect {
        Dialect::Draft4 => boon::Draft::V4,
        Dialect::Draft2020 => boon::Draft::V2020_12,
    });
    compiler.enable_format_assertions();
    compiler.add_resource(BASE, document.clone())?;
    let mut schemas = Schemas::new();
    let index = compiler.compile(&format!("{BASE}#{}", encode(pointer)), &mut schemas)?;
    Ok((schemas, index))
}

/// Removes the `pattern` or `patternProperties` an `InvalidRegex` error's
/// address names. `false` when the address names neither.
fn leave_out(document: &mut Value, url: &str) -> bool {
    let Some((_, fragment)) = url.split_once('#') else {
        return false;
    };
    let pointer = decode(fragment);
    let Some((parent, keyword)) = pointer.rsplit_once('/') else {
        return false;
    };
    if !matches!(keyword, "pattern" | "patternProperties") {
        return false;
    }
    document
        .pointer_mut(parent)
        .and_then(Value::as_object_mut)
        .and_then(|schema| schema.remove(keyword))
        .is_some()
}

/// The pointers to the patterns the metaschema found the validator can't
/// read, from its error.
fn unread_patterns(error: &ValidationError<'_, '_>) -> Vec<String> {
    let mut leaves = Vec::new();
    collect(error, &mut leaves);
    let mut out: Vec<String> = leaves
        .iter()
        .filter(|leaf| matches!(leaf.kind, ErrorKind::Format { want: "regex", .. }))
        .map(|leaf| leaf.instance_location.to_string())
        .collect();
    if out.len() != leaves.len() || !out.iter().all(|p| p.ends_with("/pattern")) {
        return Vec::new();
    }
    out.dedup();
    out
}

/// The first reference the schema at `pointer` reaches, through its own
/// `#…` references, that isn't into the same document.
fn elsewhere(document: &Value, pointer: &str) -> Option<String> {
    let mut seen = BTreeSet::new();
    let mut next = vec![pointer.to_owned()];
    while let Some(pointer) = next.pop() {
        if !seen.insert(pointer.clone()) {
            continue;
        }
        let Some(schema) = document.pointer(&pointer) else {
            continue;
        };
        let mut values = vec![schema];
        while let Some(value) = values.pop() {
            match value {
                Value::Object(map) => {
                    if let Some(Value::String(reference)) = map.get("$ref") {
                        match reference.strip_prefix('#') {
                            Some(local) => next.push(decode(local)),
                            None => return Some(reference.clone()),
                        }
                    }
                    values.extend(map.values());
                }
                Value::Array(items) => values.extend(items),
                _ => {}
            }
        }
    }
    None
}

/// The errors worth reporting under `error`: its leaves, with a failed
/// `anyOf` or `oneOf` narrowed to the one alternative the value was likely
/// meant to be, the only one it has the type of.
fn collect<'e, 's, 'v>(
    error: &'e ValidationError<'s, 'v>,
    out: &mut Vec<&'e ValidationError<'s, 'v>>,
) {
    match &error.kind {
        ErrorKind::AnyOf | ErrorKind::OneOf(None) => {
            let meant: Vec<_> = error
                .causes
                .iter()
                .filter(|cause| !only_type(cause, error))
                .collect();
            match meant.as_slice() {
                [one] => collect(one, out),
                _ => out.push(error),
            }
        }
        _ if error.causes.is_empty() => out.push(error),
        _ => {
            for cause in &error.causes {
                collect(cause, out);
            }
        }
    }
}

/// Whether every leaf of `branch` is a wrong type at `of`'s value: an
/// alternative of another type.
fn only_type(branch: &ValidationError<'_, '_>, of: &ValidationError<'_, '_>) -> bool {
    let mut leaves = Vec::new();
    collect(branch, &mut leaves);
    !leaves.is_empty()
        && leaves.iter().all(|leaf| {
            matches!(leaf.kind, ErrorKind::Type { .. })
                && leaf.instance_location.to_string() == of.instance_location.to_string()
        })
}

/// One error as disagreements, each at its place in the text.
fn describe(
    error: &ValidationError<'_, '_>,
    instance: &Value,
    places: &std::collections::HashMap<String, spans::Place>,
) -> Vec<Disagreement> {
    let pointer = error.instance_location.to_string();
    let name = name_of(&error.instance_location.tokens);
    let whole = places.get("").map_or(0..0, |p| p.value.clone());
    let value_at = places
        .get(&pointer)
        .map_or(whole.clone(), |p| p.value.clone());
    let value = instance.pointer(&pointer).unwrap_or(&Value::Null);
    let one = |found: String, expected: String| {
        vec![Disagreement {
            at: value_at.clone(),
            found,
            expected,
        }]
    };
    match &error.kind {
        ErrorKind::Type { got, want } => one(
            format!("{name} is {}", a(*got)),
            format!("it's {}", or(want.iter().map(a))),
        ),
        ErrorKind::Required { want } => {
            let brace = value_at.start..value_at.start + 1;
            let missing: Vec<String> = want.iter().map(|w| format!("`{w}`")).collect();
            let verb = if missing.len() == 1 { "is" } else { "are" };
            vec![Disagreement {
                at: brace,
                found: format!("{name} has no {}", and(missing.clone())),
                expected: format!("{} {verb} required", and(missing)),
            }]
        }
        ErrorKind::AdditionalProperties { got } => got
            .iter()
            .map(|property| {
                let member = format!("{pointer}/{}", token(property));
                let at = places
                    .get(&member)
                    .and_then(|p| p.key.clone())
                    .unwrap_or_else(|| value_at.clone());
                let mut tokens: Vec<String> = error
                    .instance_location
                    .tokens
                    .iter()
                    .map(token_text)
                    .collect();
                tokens.push(property.to_string());
                Disagreement {
                    at,
                    found: format!("{} is here", path_name(&tokens)),
                    expected: "it isn't one of the object's properties".to_owned(),
                }
            })
            .collect(),
        // Every alternative is of another type: the types they allow.
        ErrorKind::AnyOf | ErrorKind::OneOf(None)
            if !error.causes.is_empty() && error.causes.iter().all(|c| only_type(c, error)) =>
        {
            let mut got = None;
            let mut wanted: Vec<Type> = Vec::new();
            for cause in &error.causes {
                let mut leaves = Vec::new();
                collect(cause, &mut leaves);
                for leaf in leaves {
                    if let ErrorKind::Type { got: g, want } = &leaf.kind {
                        got = Some(*g);
                        for kind in want.iter() {
                            if !wanted.contains(&kind) {
                                wanted.push(kind);
                            }
                        }
                    }
                }
            }
            match got {
                Some(got) => one(
                    format!("{name} is {}", a(got)),
                    format!("it's {}", or(wanted.into_iter().map(a))),
                ),
                None => Vec::new(),
            }
        }
        ErrorKind::AnyOf | ErrorKind::OneOf(None) => one(
            format!("{name} matches none of the schemas it can be"),
            if matches!(error.kind, ErrorKind::AnyOf) {
                "it matches at least one".to_owned()
            } else {
                "it matches exactly one".to_owned()
            },
        ),
        other => {
            let (found, expected) = at_value(other, &name, value);
            one(found, expected)
        }
    }
}

/// What the value has, and what the schema says instead, for an error that's
/// at the value alone.
fn at_value(kind: &ErrorKind<'_, '_>, name: &str, value: &Value) -> (String, String) {
    match kind {
        ErrorKind::Enum { want } => (
            format!("{name} is {}", shown(value)),
            format!("it's one of {}", or(want.iter().map(shown))),
        ),
        ErrorKind::Const { want } => (
            format!("{name} is {}", shown(value)),
            format!("it's {}", shown(want)),
        ),
        ErrorKind::Format { want, .. } => (
            format!("{name} is {}", shown(value)),
            format!("its format is `{want}`"),
        ),
        ErrorKind::Pattern { want, .. } => (
            format!("{name} is {}", shown(value)),
            format!("it matches `{want}`"),
        ),
        ErrorKind::MinLength { got, want } => (
            format!("{name} is {got} characters long"),
            format!("it's at least {want}"),
        ),
        ErrorKind::MaxLength { got, want } => (
            format!("{name} is {got} characters long"),
            format!("it's at most {want}"),
        ),
        ErrorKind::Minimum { got, want } => {
            (format!("{name} is {got}"), format!("it's at least {want}"))
        }
        ErrorKind::Maximum { got, want } => {
            (format!("{name} is {got}"), format!("it's at most {want}"))
        }
        ErrorKind::ExclusiveMinimum { got, want } => {
            (format!("{name} is {got}"), format!("it's more than {want}"))
        }
        ErrorKind::ExclusiveMaximum { got, want } => {
            (format!("{name} is {got}"), format!("it's less than {want}"))
        }
        ErrorKind::MultipleOf { got, want } => (
            format!("{name} is {got}"),
            format!("it's a multiple of {want}"),
        ),
        ErrorKind::MinItems { got, want } => (
            format!("{name} has {}", count(*got, "item")),
            format!("it has at least {want}"),
        ),
        ErrorKind::MaxItems { got, want } => (
            format!("{name} has {}", count(*got, "item")),
            format!("it has at most {want}"),
        ),
        ErrorKind::MinProperties { got, want } => (
            format!("{name} has {}", count(*got, "property")),
            format!("it has at least {want}"),
        ),
        ErrorKind::MaxProperties { got, want } => (
            format!("{name} has {}", count(*got, "property")),
            format!("it has at most {want}"),
        ),
        ErrorKind::UniqueItems {
            got: [first, second],
        } => (
            format!("items {first} and {second} of {name} are the same"),
            "its items are unique".to_owned(),
        ),
        ErrorKind::AdditionalItems { got } => (
            format!("{name} has {}", count(*got, "item")),
            "it has no more items than its schema lists".to_owned(),
        ),
        ErrorKind::Contains | ErrorKind::MinContains { .. } => (
            format!("{name} has too few items that match its schema"),
            "more of them do".to_owned(),
        ),
        ErrorKind::MaxContains { .. } => (
            format!("{name} has too many items that match its schema"),
            "fewer of them do".to_owned(),
        ),
        ErrorKind::Dependency { prop, missing }
        | ErrorKind::DependentRequired { prop, missing } => (
            format!(
                "{name} has `{prop}` and no {}",
                and(missing.iter().map(|m| format!("`{m}`")).collect())
            ),
            format!("`{prop}` comes with them"),
        ),
        ErrorKind::OneOf(Some((first, second))) => (
            format!(
                "{name} matches schemas {} and {} of the ones it can be",
                first + 1,
                second + 1
            ),
            "it matches exactly one".to_owned(),
        ),
        ErrorKind::Not => (format!("{name} matches a schema"), "it doesn't".to_owned()),
        ErrorKind::FalseSchema => (
            format!("{name} is here"),
            "nothing is allowed there".to_owned(),
        ),
        other => (format!("{name} doesn't match"), other.to_string()),
    }
}

fn token_text(token: &InstanceToken<'_>) -> String {
    match token {
        InstanceToken::Prop(name) => name.to_string(),
        InstanceToken::Item(index) => index.to_string(),
    }
}

/// How a message names the value at `tokens`: `` `address.city` ``, or
/// `` `tags[0]` ``, or "the example" for the whole text.
fn name_of(tokens: &[InstanceToken<'_>]) -> String {
    let mut out = String::new();
    for token in tokens {
        match token {
            InstanceToken::Prop(name) => {
                if !out.is_empty() {
                    out.push('.');
                }
                out.push_str(name);
            }
            InstanceToken::Item(index) => out.push_str(&format!("[{index}]")),
        }
    }
    if out.is_empty() {
        "the example".to_owned()
    } else {
        format!("`{out}`")
    }
}

/// [`name_of`], for tokens as text, the last a property's name.
fn path_name(tokens: &[String]) -> String {
    format!("`{}`", tokens.join("."))
}

/// A type with its article: "a string", "an object", "null".
fn a(kind: Type) -> String {
    match kind {
        Type::Null => "null".to_owned(),
        Type::Integer | Type::Array | Type::Object => format!("an {kind}"),
        _ => format!("a {kind}"),
    }
}

/// A value as a message shows it, cut short when it's long.
fn shown(value: &Value) -> String {
    let text = value.to_string();
    if text.chars().count() > 40 {
        let cut: String = text.chars().take(37).collect();
        format!("`{cut}…`")
    } else {
        format!("`{text}`")
    }
}

fn count(n: usize, noun: &str) -> String {
    match (n, noun) {
        (1, _) => format!("1 {noun}"),
        (_, "property") => format!("{n} properties"),
        _ => format!("{n} {noun}s"),
    }
}

fn or(items: impl Iterator<Item = String>) -> String {
    list(items.collect(), "or")
}

fn and(items: Vec<String>) -> String {
    list(items, "and")
}

fn list(items: Vec<String>, word: &str) -> String {
    match items.as_slice() {
        [] => String::new(),
        [one] => one.clone(),
        [rest @ .., last] => format!("{} {word} {last}", rest.join(", ")),
    }
}

/// The byte offset of `serde_json`'s 1-based line and column (in bytes):
/// where it stopped reading.
fn offset_of(text: &str, line: usize, column: usize) -> usize {
    let start: usize = text
        .split_inclusive('\n')
        .take(line.saturating_sub(1))
        .map(str::len)
        .sum();
    (start + column.saturating_sub(1)).min(text.len())
}

/// `serde_json`'s reason, without the line and column it adds.
fn reason_of(error: &serde_json::Error) -> String {
    let text = error.to_string();
    match text.rfind(" at line ") {
        Some(at) => text[..at].to_owned(),
        None => text,
    }
}

/// A JSON pointer percent-encoded, as a URI fragment may hold it.
pub(crate) fn encode(pointer: &str) -> String {
    let mut out = String::with_capacity(pointer.len());
    for byte in pointer.bytes() {
        if byte.is_ascii_alphanumeric() || b"-._~/".contains(&byte) {
            out.push(char::from(byte));
        } else {
            out.push_str(&format!("%{byte:02X}"));
        }
    }
    out
}

/// A URI fragment percent-decoded.
pub(crate) fn decode(fragment: &str) -> String {
    let bytes = fragment.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        let hex = bytes
            .get(i + 1..i + 3)
            .and_then(|h| std::str::from_utf8(h).ok())
            .and_then(|h| u8::from_str_radix(h, 16).ok());
        match (bytes[i], hex) {
            (b'%', Some(byte)) => {
                out.push(byte);
                i += 3;
            }
            (byte, _) => {
                out.push(byte);
                i += 1;
            }
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}
