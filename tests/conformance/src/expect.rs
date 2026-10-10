//! The contents of a case's `expect.yaml`.

use std::collections::BTreeMap;
use std::fmt;

use serde::{Deserialize, Serialize};

use crate::outline::{Outline, deserialize_outline, serialize_outline};

/// Everything a case expects. See `tests/conformance/README.md`.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Expect {
    /// What the case checks, in a sentence or two.
    #[serde(default)]
    pub description: Option<String>,
    /// SPEC.md sections the case covers, for example `"3.5"` or `"B"`.
    #[serde(default)]
    pub spec: Vec<String>,
    /// Area tags, used to route the case to adapters and to select cases.
    pub tags: Vec<String>,
    /// The expected outline of `input.md`. Single-file cases only.
    #[serde(
        default,
        deserialize_with = "deserialize_outline",
        serialize_with = "serialize_outline"
    )]
    pub outline: Option<Outline>,
    /// The file, relative to the case directory, that formatting `input.md`
    /// must produce (SPEC §8.3, canonical form). `input.md` itself says the
    /// input is already canonical. Single-file cases only. The runner also
    /// requires formatting the result to change nothing.
    #[serde(default)]
    pub formatted: Option<String>,
    /// Expected file-level diagnostics (SPEC §8.1). `None` means not checked;
    /// an empty list means none are expected.
    #[serde(default)]
    pub diagnostics: Option<Vec<ExpectedDiagnostic>>,
    /// Per-build expectations, keyed by build name from the content model.
    #[serde(default)]
    pub builds: BTreeMap<String, BuildExpect>,
    /// The day the checks run on, `YYYY-MM-DD`, for a case whose
    /// diagnostics depend on it (a review date). Without it, the checks have
    /// no day, and no review is overdue.
    #[serde(default)]
    pub today: Option<String>,
}

/// An expected diagnostic.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ExpectedDiagnostic {
    /// The diagnostic's slug from `tests/conformance/diagnostics.toml`.
    pub slug: String,
    /// The file, relative to the content root. Defaults to `input.md` in
    /// single-file cases; required in project cases.
    #[serde(default)]
    pub file: Option<String>,
    /// The 1-based line the diagnostic is reported at.
    pub line: u32,
    /// The 1-based column, counted in Unicode scalar values. Compared only when given.
    #[serde(default)]
    pub column: Option<u32>,
    /// `error`, `warning`, or `advice`, after the content model's `[checks]`
    /// levels. Compared only when given.
    #[serde(default)]
    pub severity: Option<String>,
}

impl fmt::Display for ExpectedDiagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} at {}:{}",
            self.slug,
            self.file.as_deref().unwrap_or("input.md"),
            self.line
        )?;
        if let Some(col) = self.column {
            write!(f, ":{col}")?;
        }
        if let Some(severity) = &self.severity {
            write!(f, " ({severity})")?;
        }
        Ok(())
    }
}

/// Expectations for one build.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct BuildExpect {
    /// Every page the build publishes, keyed by path relative to the content
    /// root, and what's expected of each. `None` means not checked.
    #[serde(default)]
    pub pages: Option<BTreeMap<String, Option<PageExpect>>>,
    /// Every asset the build copies, as source paths relative to the content
    /// root. `None` means not checked.
    #[serde(default)]
    pub assets: Option<Vec<String>>,
    /// Expected page-level diagnostics for this build. `None` means not checked.
    #[serde(default)]
    pub diagnostics: Option<Vec<ExpectedDiagnostic>>,
}

/// Expectations for one published page in one build.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PageExpect {
    /// The resolved page's outline: includes expanded, the build's modes
    /// applied, and phrases substituted.
    #[serde(
        default,
        deserialize_with = "deserialize_outline",
        serialize_with = "serialize_outline"
    )]
    pub outline: Option<Outline>,
    /// Expected outputs, by emitter.
    #[serde(default)]
    pub outputs: BTreeMap<OutputKind, OutputSlot>,
}

/// An output emitter (SPEC §9.4).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum OutputKind {
    /// Plain markdown.
    Plain,
    /// Site markdown with custom elements.
    Site,
    /// The JSON tree.
    Json,
}

impl fmt::Display for OutputKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            OutputKind::Plain => "plain",
            OutputKind::Site => "site",
            OutputKind::Json => "json",
        })
    }
}

/// Where an expected output is recorded.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(untagged)]
pub enum OutputSlot {
    /// `snapshot`: an `insta` snapshot in `tests/conformance/snapshots/`.
    Snapshot(SnapshotMarker),
    /// `{file: path}`: a file, relative to the case directory, compared exactly.
    File {
        /// The file's path.
        file: String,
    },
}

/// The literal word `snapshot`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum SnapshotMarker {
    /// `snapshot`.
    Snapshot,
}
