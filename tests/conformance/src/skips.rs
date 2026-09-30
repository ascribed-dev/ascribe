//! `SKIPS.toml`: the deliberate, visible list of skipped cases and tags.

use std::fmt;
use std::path::Path;

use serde::Deserialize;

/// A check the runner performs on a case.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Check {
    /// The top-level `outline`.
    Outline,
    /// The top-level `diagnostics`.
    Diagnostics,
    /// Everything under `builds`.
    Builds,
    /// The top-level `formatted`.
    Format,
}

impl fmt::Display for Check {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Check::Outline => "outline",
            Check::Diagnostics => "diagnostics",
            Check::Builds => "builds",
            Check::Format => "format",
        })
    }
}

/// What a skip entry applies to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SkipTarget {
    /// Every case carrying this tag.
    Tag(String),
    /// One case, by id.
    Case(String),
}

impl fmt::Display for SkipTarget {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SkipTarget::Tag(t) => write!(f, "tag `{t}`"),
            SkipTarget::Case(c) => write!(f, "case `{c}`"),
        }
    }
}

/// One `[[skip]]` entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SkipEntry {
    /// The tag or case skipped.
    pub target: SkipTarget,
    /// Why, in words a reader of the test output understands.
    pub reason: String,
    /// When set, only these checks are skipped and the rest run. When unset,
    /// the whole case is skipped.
    pub checks: Option<Vec<Check>>,
}

impl SkipEntry {
    /// Whether the entry skips whole cases rather than some checks.
    pub fn is_full(&self) -> bool {
        self.checks.is_none()
    }
}

/// A problem in `SKIPS.toml`.
#[derive(Debug, thiserror::Error)]
pub enum SkipsError {
    /// The file couldn't be read.
    #[error("couldn't read SKIPS.toml: {0}")]
    Io(#[from] std::io::Error),
    /// The file isn't valid TOML or has unknown fields.
    #[error("invalid SKIPS.toml: {0}")]
    Toml(#[from] toml::de::Error),
    /// An entry breaks a rule.
    #[error("SKIPS.toml entry {index}: {message}")]
    Entry {
        /// The entry's 1-based position.
        index: usize,
        /// What's wrong.
        message: String,
    },
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawFile {
    #[serde(default)]
    skip: Vec<RawEntry>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawEntry {
    tag: Option<String>,
    case: Option<String>,
    reason: String,
    checks: Option<Vec<Check>>,
}

/// The parsed `SKIPS.toml`.
#[derive(Debug, Clone, Default)]
pub struct Skips {
    /// Every entry, in file order.
    pub entries: Vec<SkipEntry>,
}

impl Skips {
    /// Loads `SKIPS.toml`. A missing file means no skips.
    pub fn load(path: &Path) -> Result<Skips, SkipsError> {
        if !path.exists() {
            return Ok(Skips::default());
        }
        Skips::parse(&std::fs::read_to_string(path)?)
    }

    /// Parses the text of `SKIPS.toml`.
    pub fn parse(text: &str) -> Result<Skips, SkipsError> {
        let raw: RawFile = toml::from_str(text)?;
        let mut entries: Vec<SkipEntry> = Vec::new();
        for (i, e) in raw.skip.into_iter().enumerate() {
            let index = i + 1;
            let bad = |message: &str| {
                Err(SkipsError::Entry {
                    index,
                    message: message.into(),
                })
            };
            let target = match (e.tag, e.case) {
                (Some(t), None) => SkipTarget::Tag(t),
                (None, Some(c)) => SkipTarget::Case(c),
                _ => return bad("give exactly one of `tag` or `case`"),
            };
            if e.reason.trim().is_empty() {
                return bad("`reason` must not be empty");
            }
            if e.checks.as_ref().is_some_and(Vec::is_empty) {
                return bad("`checks`, when given, must not be empty");
            }
            if entries.iter().any(|x| x.target == target) {
                return bad(&format!("{target} is listed more than once"));
            }
            entries.push(SkipEntry {
                target,
                reason: e.reason,
                checks: e.checks,
            });
        }
        Ok(Skips { entries })
    }

    /// The entry for a tag, if any.
    pub fn for_tag(&self, tag: &str) -> Option<&SkipEntry> {
        self.entries
            .iter()
            .find(|e| matches!(&e.target, SkipTarget::Tag(t) if t == tag))
    }

    /// The entry for a case, if any.
    pub fn for_case(&self, id: &str) -> Option<&SkipEntry> {
        self.entries
            .iter()
            .find(|e| matches!(&e.target, SkipTarget::Case(c) if c == id))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_tag_and_case_entries() {
        let skips = Skips::parse(
            r#"
[[skip]]
tag = "parser"
reason = "no adapter yet"

[[skip]]
case = "structure/foo"
reason = "diagnostics aren't checked yet"
checks = ["diagnostics"]
"#,
        )
        .unwrap();
        assert!(skips.for_tag("parser").unwrap().is_full());
        let case = skips.for_case("structure/foo").unwrap();
        assert_eq!(case.checks, Some(vec![Check::Diagnostics]));
    }

    #[test]
    fn rejects_bad_entries() {
        let both = "[[skip]]\ntag = \"a\"\ncase = \"b\"\nreason = \"x\"\n";
        assert!(Skips::parse(both).is_err());
        let no_reason = "[[skip]]\ntag = \"a\"\nreason = \" \"\n";
        assert!(Skips::parse(no_reason).is_err());
        let dup = "[[skip]]\ntag = \"a\"\nreason = \"x\"\n[[skip]]\ntag = \"a\"\nreason = \"y\"\n";
        assert!(Skips::parse(dup).is_err());
        let typo = "[[skip]]\ntags = \"a\"\nreason = \"x\"\n";
        assert!(Skips::parse(typo).is_err());
    }
}
