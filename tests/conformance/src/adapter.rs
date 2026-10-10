//! The adapter interface that connects the harness to the Ascribe crates.

use std::collections::BTreeMap;
use std::fmt;

use crate::case::Case;
use crate::expect::OutputKind;
use crate::outline::Outline;

/// A diagnostic an implementation reported.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    /// The slug from `tests/conformance/diagnostics.toml`.
    pub slug: String,
    /// The file, relative to the case's content root, with `/` separators.
    /// `input.md` in single-file cases.
    pub file: String,
    /// The 1-based line.
    pub line: u32,
    /// The 1-based column, counted in Unicode scalar values.
    pub column: u32,
    /// `error`, `warning`, or `advice`, when the adapter reports it: the
    /// severity after the content model's `[checks]` levels.
    pub severity: Option<String>,
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} at {}:{}:{}",
            self.slug, self.file, self.line, self.column
        )?;
        if let Some(severity) = &self.severity {
            write!(f, " ({severity})")?;
        }
        Ok(())
    }
}

/// What a build of a case produced.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct BuildResult {
    /// Every published page, keyed by source path relative to the content root.
    pub pages: BTreeMap<String, PageResult>,
    /// Every asset copied, as source paths relative to the content root.
    pub assets: Vec<String>,
    /// Page-level diagnostics for this build.
    pub diagnostics: Vec<Diagnostic>,
}

/// What a build produced for one page.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct PageResult {
    /// The resolved page's outline, if the adapter can produce one.
    pub outline: Option<Outline>,
    /// Emitted outputs, by emitter.
    pub outputs: BTreeMap<OutputKind, String>,
}

/// An adapter couldn't process a case, for a reason other than a wrong result.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("{0}")]
pub struct AdapterError(pub String);

/// The result of one adapter call: `Ok(None)` means this adapter doesn't
/// produce that result (yet), so the runner asks the next adapter.
pub type AdapterResult<T> = Result<Option<T>, AdapterError>;

/// Connects the harness to an implementation.
///
/// An adapter declares the tags it handles, and produces results for cases
/// carrying them. Every method has a default that produces nothing, so an
/// adapter implements only what it covers. Adapters live in the
/// conformance crate's `tests/adapters/` module and are registered in
/// `tests/conformance.rs`; the harness itself depends on no Ascribe crate.
pub trait ConformanceAdapter {
    /// A short name for messages, for example `syntax`.
    fn name(&self) -> &str;

    /// Whether this adapter handles cases with this area tag. Handling a tag
    /// means its cases run instead of needing a skip entry.
    fn handles_tag(&self, tag: &str) -> bool;

    /// The outline of a single-file case's `input.md`.
    fn outline(&self, case: &Case) -> AdapterResult<Outline> {
        let _ = case;
        Ok(None)
    }

    /// File-level diagnostics for every source file in the case.
    fn diagnostics(&self, case: &Case) -> AdapterResult<Vec<Diagnostic>> {
        let _ = case;
        Ok(None)
    }

    /// `source` formatted into canonical form (SPEC §8.3), under the case's
    /// content model. The runner calls it with the case's `input.md`, and again
    /// with the result to check that formatting is idempotent.
    fn format(&self, case: &Case, source: &str) -> AdapterResult<String> {
        let _ = (case, source);
        Ok(None)
    }

    /// The result of the named build of the case.
    fn build(&self, case: &Case, build: &str) -> AdapterResult<BuildResult> {
        let _ = (case, build);
        Ok(None)
    }
}

/// The adapters available to a run.
#[derive(Default)]
pub struct Registry {
    adapters: Vec<Box<dyn ConformanceAdapter>>,
}

impl Registry {
    /// An empty registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds an adapter. Earlier adapters are asked first.
    pub fn register(&mut self, adapter: impl ConformanceAdapter + 'static) -> &mut Self {
        self.adapters.push(Box::new(adapter));
        self
    }

    /// Whether any adapter handles the tag.
    pub fn handles_tag(&self, tag: &str) -> bool {
        self.adapters.iter().any(|a| a.handles_tag(tag))
    }

    /// The adapters that handle at least one of the tags, in registration order.
    pub fn for_tags<'a>(
        &'a self,
        tags: &'a [&'a str],
    ) -> impl Iterator<Item = &'a dyn ConformanceAdapter> + 'a {
        self.adapters
            .iter()
            .map(|a| a.as_ref())
            .filter(move |a| tags.iter().any(|t| a.handles_tag(t)))
    }
}
