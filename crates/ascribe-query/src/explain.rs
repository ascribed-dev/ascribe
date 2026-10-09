//! What a diagnostic means: its message, how to fix it, and a short
//! wrong-and-right example, from the diagnostics registry
//! (`tests/conformance/diagnostics.toml`), which is embedded in the binary.
//!
//! Many diagnostics fire only under a particular content model, so each
//! example is a page, `page.md`, checked under the explain model
//! ([`EXPLAIN_MODEL`]), with the example's own `example.model` laid over it,
//! and with its `example.files` beside it. A test checks that each example's
//! wrong page has its diagnostic and its right page doesn't
//! ([`example_slugs`]).

use std::path::PathBuf;
use std::sync::Arc;

use ascribe_check::{Entry, Example, Level, Registry, Severity};
use ascribe_core::{FileId, RelPath};
use ascribe_model::edit_distance;
use ascribe_resolve::{Layout, MemoryFs};
use serde::Serialize;

use crate::{ASCRIBE_VERSION, QueryError, SCHEMA_VERSION};

/// The content model the examples are checked under.
pub const EXPLAIN_MODEL: &str = include_str!("../../../tests/conformance/explain-model.toml");

/// The content path of an example's page.
pub const EXAMPLE_PAGE: &str = "page.md";

/// What `ascribe explain <CODE>` answers: one diagnostic.
#[derive(Debug, Serialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
pub struct Explanation {
    /// The version of this schema. It changes only when a field is removed
    /// or changes meaning.
    pub schema_version: u32,
    /// The version of Ascribe that wrote it.
    pub ascribe_version: &'static str,
    /// The code, such as `ASC036`.
    pub code: String,
    /// The diagnostic's name, such as `link-target-missing`.
    pub slug: String,
    /// `error` or `warning`.
    pub severity: &'static str,
    /// `file` for a problem found in each file on its own; `page` for one
    /// found in each page as a build resolves it.
    pub level: &'static str,
    /// The message, with its placeholders (`{path}`) as written.
    pub message: String,
    /// The other ways the message is worded, by the case they're for, in
    /// name order.
    pub variants: Vec<MessageVariant>,
    /// How to fix it, in general. `null` for a diagnostic that's no longer
    /// reported.
    pub fix: Option<String>,
    /// The address of its entry in the diagnostics reference.
    pub docs: String,
    /// A page that has the problem and the same page fixed; `null` when the
    /// diagnostic has no example.
    pub example: Option<ExampleText>,
}

/// Another wording of a diagnostic's message.
#[derive(Debug, Serialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
pub struct MessageVariant {
    /// The case it's for.
    pub name: String,
    /// The message, with its placeholders as written.
    pub message: String,
}

/// A diagnostic's example.
#[derive(Debug, Serialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
pub struct ExampleText {
    /// The page, `page.md`, with the problem.
    pub wrong: String,
    /// The same page, fixed.
    pub right: String,
    /// The content model the example assumes, when the problem depends on
    /// one: the example's own fragment, or else the explain model. `null`
    /// when it's a problem under any content model.
    pub model: Option<String>,
    /// The project's other files the example assumes, in path order.
    pub files: Vec<ExampleFile>,
}

/// A file an example assumes besides its page.
#[derive(Debug, Serialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
pub struct ExampleFile {
    /// Its content path.
    pub path: String,
    /// Its text.
    pub text: String,
}

/// What `ascribe explain --list` answers: every diagnostic.
#[derive(Debug, Serialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
pub struct DiagnosticList {
    /// The version of this schema. It changes only when a field is removed
    /// or changes meaning.
    pub schema_version: u32,
    /// The version of Ascribe that wrote it.
    pub ascribe_version: &'static str,
    /// Every diagnostic, in code order.
    pub diagnostics: Vec<ListedDiagnostic>,
}

/// A diagnostic in the list.
#[derive(Debug, Serialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
pub struct ListedDiagnostic {
    /// The code.
    pub code: String,
    /// The slug.
    pub slug: String,
    /// `error` or `warning`.
    pub severity: &'static str,
}

/// The diagnostic with this code (`ASC036`, in any case) or slug, explained.
/// `docs_site` is the docs site's address, with no trailing slash.
///
/// # Errors
///
/// No diagnostic has that code or slug; the error names the closest.
pub fn explain(code_or_slug: &str, docs_site: &str) -> Result<Explanation, QueryError> {
    let registry = Registry::global();
    let given = code_or_slug.trim();
    let Some(entry) = registry.find(given) else {
        return Err(QueryError::UnknownDiagnostic {
            given: given.to_owned(),
            closest: closest(registry, given),
        });
    };
    let mut variants: Vec<MessageVariant> = entry
        .messages
        .iter()
        .map(|(name, message)| MessageVariant {
            name: name.clone(),
            message: message.clone(),
        })
        .collect();
    variants.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(Explanation {
        schema_version: SCHEMA_VERSION,
        ascribe_version: ASCRIBE_VERSION,
        code: entry.code.clone(),
        slug: entry.slug.clone(),
        severity: severity(entry.severity),
        level: match entry.level {
            Level::File => "file",
            Level::Page => "page",
        },
        message: entry.message.clone(),
        variants,
        fix: entry.fix.clone(),
        docs: docs_url(entry, docs_site),
        example: entry.example.as_ref().map(|e| example_text(entry, e)),
    })
}

/// Every diagnostic, in code order.
pub fn list() -> DiagnosticList {
    DiagnosticList {
        schema_version: SCHEMA_VERSION,
        ascribe_version: ASCRIBE_VERSION,
        diagnostics: Registry::global()
            .entries()
            .map(|e| ListedDiagnostic {
                code: e.code.clone(),
                slug: e.slug.clone(),
                severity: severity(e.severity),
            })
            .collect(),
    }
}

fn severity(severity: Severity) -> &'static str {
    match severity {
        Severity::Error => "error",
        Severity::Warning => "warning",
    }
}

/// The address of a diagnostic's entry in the diagnostics reference, whose
/// headings are `ASC036: link-target-missing`.
fn docs_url(entry: &Entry, docs_site: &str) -> String {
    format!(
        "{docs_site}/reference/diagnostics/#{}-{}",
        entry.code.to_lowercase(),
        entry.slug
    )
}

/// The codes whose code or slug is closest to `given`, with their slugs, at
/// most three.
fn closest(registry: &Registry, given: &str) -> Vec<String> {
    let given = given.to_lowercase();
    let mut scored: Vec<(usize, &Entry)> = registry
        .entries()
        .map(|e| {
            let by_code = edit_distance(&given, &e.code.to_lowercase());
            let by_slug = edit_distance(&given, &e.slug);
            // A typo in the start of a slug: `link-targt`.
            let by_start = e
                .slug
                .get(..given.len())
                .map_or(usize::MAX, |start| edit_distance(&given, start));
            (by_code.min(by_slug).min(by_start), e)
        })
        .filter(|(distance, e)| {
            *distance <= (given.len() / 3).max(2) || e.slug.contains(given.as_str())
        })
        .collect();
    scored.sort_by_key(|(distance, e)| (*distance, e.code.clone()));
    scored
        .into_iter()
        .take(3)
        .map(|(_, e)| format!("{} {}", e.code, e.slug))
        .collect()
}

fn example_text(entry: &Entry, example: &Example) -> ExampleText {
    // The model is shown only when the example needs it: its own fragment
    // always, the explain model when the problem isn't found without it.
    let model = match &example.model {
        Some(fragment) => Some(fragment.clone()),
        None => {
            let anywhere = example_slugs(example, &example.wrong, Base::Bare)
                .is_ok_and(|slugs| slugs.contains(&entry.slug));
            (!anywhere).then(|| EXPLAIN_MODEL.to_owned())
        }
    };
    ExampleText {
        wrong: example.wrong.clone(),
        right: example.right.clone(),
        model,
        files: example
            .files
            .iter()
            .map(|(path, text)| ExampleFile {
                path: path.clone(),
                text: text.clone(),
            })
            .collect(),
    }
}

/// The content model an example is checked under, before its own fragment.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Base {
    /// The explain model, [`EXPLAIN_MODEL`].
    Explain,
    /// A model that declares nothing.
    Bare,
}

/// Why an example couldn't be checked.
#[derive(Debug, thiserror::Error)]
pub enum ExampleError {
    /// A model isn't TOML.
    #[error("the model isn't TOML: {0}")]
    Toml(String),
    /// The model doesn't load.
    #[error("the model has errors: {0}")]
    Model(String),
    /// A path isn't a content path.
    #[error("`{0}` isn't a content path")]
    Path(String),
}

/// The slugs of the diagnostics `ascribe check` reports for an example's
/// project with `page` as `page.md`, in the order reported.
///
/// # Errors
///
/// The example's model or a file's path is invalid.
pub fn example_slugs(
    example: &Example,
    page: &str,
    base: Base,
) -> Result<Vec<String>, ExampleError> {
    let text = example_model(example, base)?;
    let model = ascribe_model::load_str(&text, FileId::new(0)).map_err(|issues| {
        ExampleError::Model(
            issues
                .iter()
                .map(|i| Registry::global().message(i))
                .collect::<Vec<_>>()
                .join("; "),
        )
    })?;
    let layout = Layout::from_model(&model);
    let mut files = vec![(EXAMPLE_PAGE.to_owned(), page.to_owned())];
    files.extend(example.files.iter().cloned());
    let mut fs = MemoryFs::new(&layout);
    let mut sources = Vec::new();
    for (path, text) in &files {
        fs = fs.with_source(path, text);
        if ascribe_resolve::is_source_path(
            &RelPath::parse(path).map_err(|_| ExampleError::Path(path.clone()))?,
        ) {
            sources.push((
                RelPath::parse(path).map_err(|_| ExampleError::Path(path.clone()))?,
                text.clone(),
            ));
        }
    }
    let project = ascribe_check::Project::from_parts_with_fs(
        PathBuf::from("/explain"),
        layout.content_root.clone(),
        model,
        text,
        ascribe_check::Project::from_sources(sources),
        Arc::new(fs),
    );
    let diagnostics = ascribe_check::diagnose(&project, &[])
        .map(|d| d.diagnostics)
        .unwrap_or_default();
    Ok(diagnostics.iter().map(|d| d.slug.to_string()).collect())
}

/// The text of the model an example is checked under: `base` with the
/// example's own fragment laid over it, table by table.
fn example_model(example: &Example, base: Base) -> Result<String, ExampleError> {
    let base_text = match base {
        Base::Explain => EXPLAIN_MODEL,
        Base::Bare => "spec = \"0.1\"\n",
    };
    let Some(fragment) = &example.model else {
        return Ok(base_text.to_owned());
    };
    let parse = |text: &str| {
        text.parse::<toml::Table>()
            .map_err(|e| ExampleError::Toml(e.to_string()))
    };
    let mut merged = parse(base_text)?;
    overlay(&mut merged, parse(fragment)?);
    toml::to_string(&merged).map_err(|e| ExampleError::Toml(e.to_string()))
}

/// Lays `over` over `under`: a table in both is merged, anything else in
/// `over` replaces what `under` has.
fn overlay(under: &mut toml::Table, over: toml::Table) {
    for (key, value) in over {
        match (under.get_mut(&key), value) {
            (Some(toml::Value::Table(inner)), toml::Value::Table(value)) => overlay(inner, value),
            (_, value) => {
                under.insert(key, value);
            }
        }
    }
}
