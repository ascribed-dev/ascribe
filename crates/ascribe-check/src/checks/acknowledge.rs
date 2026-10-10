//! Acknowledgements (SPEC §4.9): `@intended` lines and `intended`
//! frontmatter name a review check acknowledged where it reports its
//! problems, and give a reason. What they cover is matched in
//! [`crate::intended`]; these checks report the ones that can't cover
//! anything.

use ascribe_core::diagnostics::ACKNOWLEDGEABLE;
use ascribe_core::intended::{NameProblem, check_named};
use ascribe_core::{Issue, Location, Place, Span, diagnostics};
use ascribe_syntax::DirectiveLine;
use serde_yaml_ng::Value;

use super::Ctx;
use super::text::suggest;
use crate::intended::FRONTMATTER_KEY;
use crate::yaml::YamlIndex;

impl Ctx<'_> {
    /// `@intended`: it names a check acknowledged at a block. A missing
    /// reason is the parser's (a required primary).
    pub(super) fn check_intended_directive(&mut self, d: &DirectiveLine) {
        let check = d.attributes.as_ref().and_then(|a| a.get("check"));
        match check.and_then(|a| a.value.as_ref()) {
            Some(value) => {
                // A value of the wrong form is the attribute checks'.
                if let Some(name) = value.as_text() {
                    self.check_intended_name(name, value.span(), Place::Block);
                }
            }
            None if check.is_none() => self.report(
                Issue::new(diagnostics::INTENDED_CHECK, self.location(d.name_span))
                    .with_variant("missing")
                    .with_arg("example", "{check=<check>}"),
            ),
            // A bare key: the parser reported it.
            None => {}
        }
    }

    /// The `intended` frontmatter key: a list of entries, each with the
    /// `check` it acknowledges at the page and a `reason`.
    pub(super) fn check_intended_key(
        &mut self,
        value: &Value,
        index: &YamlIndex,
        first_line: Span,
    ) {
        let Value::Mapping(map) = value else { return };
        let Some(list) = map.get(FRONTMATTER_KEY) else {
            return;
        };
        let node_span = |path: &str| {
            index
                .get(path)
                .map_or(first_line, |n| n.key.unwrap_or(n.value))
        };
        let Value::Sequence(items) = list else {
            self.mismatch(
                FRONTMATTER_KEY,
                "a list of acknowledgements, each with a `check` and a `reason`",
                list,
                index.get(FRONTMATTER_KEY).map_or(first_line, |n| n.value),
            );
            return;
        };
        for (i, item) in items.iter().enumerate() {
            let path = format!("{FRONTMATTER_KEY}[{i}]");
            let Value::Mapping(entry) = item else {
                let at = index.get(&path).map_or(first_line, |n| n.value);
                self.report(Issue::new(diagnostics::INTENDED_ENTRY, self.location(at)));
                continue;
            };
            let item_span = index.get(&path).map_or(first_line, |n| n.value);
            for key in entry.keys() {
                let name = key.as_str().unwrap_or_default();
                if matches!(name, "check" | "reason") {
                    continue;
                }
                let at = node_span(&format!("{path}.{name}"));
                self.report(
                    Issue::new(diagnostics::INTENDED_ENTRY, self.location(at))
                        .with_variant("key")
                        .with_arg("key", name.to_owned()),
                );
            }
            let check = match entry.get("check") {
                None => {
                    self.report(
                        Issue::new(diagnostics::INTENDED_CHECK, self.location(item_span))
                            .with_variant("missing")
                            .with_arg("example", "check: <check>"),
                    );
                    None
                }
                Some(Value::String(name)) => {
                    let at = index
                        .get(&format!("{path}.check"))
                        .map_or(item_span, |n| n.value);
                    self.check_intended_name(name, at, Place::Page);
                    Some(name.clone())
                }
                Some(other) => {
                    let at = index
                        .get(&format!("{path}.check"))
                        .map_or(item_span, |n| n.value);
                    self.mismatch(&format!("{path}.check"), "a check's name", other, at);
                    None
                }
            };
            let reason_ok =
                matches!(entry.get("reason"), Some(Value::String(r)) if !r.trim().is_empty());
            if !reason_ok {
                let at = index
                    .get(&format!("{path}.reason"))
                    .map_or(item_span, |n| n.key.unwrap_or(n.value));
                self.report(
                    Issue::new(diagnostics::INTENDED_ENTRY, self.location(at))
                        .with_variant("reason")
                        .with_arg("check", check.unwrap_or_else(|| "a check".to_owned())),
                );
            }
        }
    }

    /// A check's name, written where `written` problems are acknowledged.
    fn check_intended_name(&mut self, name: &str, at: Span, written: Place) {
        let problem = match check_named(name, written, ACKNOWLEDGEABLE) {
            Ok(_) => return,
            Err(problem) => problem,
        };
        let issue = Issue::new(diagnostics::INTENDED_CHECK, Location::new(self.id, at))
            .with_arg("check", name.to_owned());
        let issue = match problem {
            NameProblem::Unknown => {
                match suggest(name, diagnostics::ALL.iter().map(|s| s.as_str())) {
                    Some(s) => issue.with_variant("suggestion").with_arg("suggestion", s),
                    None => issue,
                }
            }
            NameProblem::NotReview => issue.with_variant("not-review"),
            NameProblem::Place(place) => issue
                .with_variant("place")
                .with_arg("place", place.noun())
                .with_arg("written", place.written()),
        };
        self.report(issue);
    }
}
