//! Frontmatter (SPEC §2.1, §2.2, §7.2): the content type a page gets, its
//! fields, the reserved keys, and the values of `available` and `variant`.
//!
//! `ascribe_model::validate_frontmatter` checks fields against a schema, but
//! it gets parsed YAML and so can't say where anything is. This module
//! reads the YAML text too, with [`YamlIndex`], and moves each problem to the
//! key or value that causes it. A problem with no place of its own, such as a
//! missing field, is reported at the file's first line (SPEC §8.1).

use ascribe_core::Router;
use ascribe_core::{Applicability, Fix, Issue, Location, Span, TextEdit, diagnostics};
use ascribe_model::{
    FieldRole, FrontmatterSchema, Listing, ListingLevel, TypeMatch, validate_frontmatter,
};
use ascribe_resolve::{AstroRouter, llms, pages};
use ascribe_syntax::Frontmatter;
use serde_yaml_ng::Value;

use super::Ctx;
use super::text::quoted_and;
use crate::yaml::{Node, YamlIndex};

impl Ctx<'_> {
    pub(super) fn check_frontmatter(&mut self, frontmatter: Option<&Frontmatter>) {
        let first_line = self.first_line();
        let path = self.file.path.as_str().to_owned();
        let fragment = self.model.is_fragment(&path);
        let schema: &FrontmatterSchema = if fragment {
            &self.model.fragments.frontmatter
        } else {
            match self.model.type_for(&path) {
                TypeMatch::One(t) => &t.frontmatter,
                TypeMatch::Ambiguous(types) => {
                    let names: Vec<&str> = types.iter().map(|t| t.name.as_str()).collect();
                    let issue = Issue::new(
                        diagnostics::CONTENT_TYPE_UNRESOLVED,
                        Location::new(self.id, first_line),
                    )
                    .with_arg("types", quoted_and(&names));
                    self.report(issue);
                    return;
                }
                TypeMatch::None => {
                    let issue = Issue::new(
                        diagnostics::CONTENT_TYPE_UNRESOLVED,
                        Location::new(self.id, first_line),
                    )
                    .with_variant("none");
                    self.report(issue);
                    return;
                }
            }
        };

        let (value, index) = match frontmatter {
            Some(fm) => {
                let content = self.file.text.get(fm.content.range()).unwrap_or_default();
                let value = if content.trim().is_empty() {
                    Value::Null
                } else {
                    match serde_yaml_ng::from_str::<Value>(content) {
                        Ok(v) => v,
                        Err(e) => {
                            // SPEC §8.2: at the YAML
                            // parser's position.
                            let at = e.location().map_or(first_line, |l| {
                                Span::empty(fm.content.start() + l.index())
                            });
                            let issue = Issue::new(
                                diagnostics::FRONTMATTER_SYNTAX,
                                Location::new(self.id, at),
                            )
                            .with_arg("detail", e.to_string());
                            self.report(issue);
                            return;
                        }
                    }
                };
                (value, YamlIndex::build(content, fm.content.start()))
            }
            None => (Value::Null, YamlIndex::default()),
        };

        let at = Location::new(self.id, first_line);
        for issue in validate_frontmatter(schema, &value, at) {
            let issue = self.relocate(issue, &index);
            self.report(issue);
        }
        if !fragment {
            self.check_reserved_keys(&value, &index, first_line);
            self.check_roles(schema, &value, &index, first_line);
            self.check_description_length(schema, &value, &index);
        }
        self.check_intended_key(&value, &index, first_line);
    }

    /// A description longer than its line in `llms.txt` should be, when the
    /// project publishes one (`[consumer] agents = true`).
    fn check_description_length(
        &mut self,
        schema: &FrontmatterSchema,
        value: &Value,
        index: &YamlIndex,
    ) {
        if !self.model.consumer.agents {
            return;
        }
        let Some(field) = schema.field_with_role(FieldRole::Description) else {
            return;
        };
        let Some(raw) = value.get(field.name.as_str()).and_then(Value::as_str) else {
            return;
        };
        let Some(text) = llms::description_text(self.model, raw) else {
            return;
        };
        let length = text.chars().count();
        if length <= llms::DESCRIPTION_LIMIT {
            return;
        }
        let Some(node) = index.get(&field.name) else {
            return;
        };
        let issue = Issue::new(
            diagnostics::DESCRIPTION_TOO_LONG,
            Location::new(self.id, node.value),
        )
        .with_arg("length", length.to_string())
        .with_arg("limit", llms::DESCRIPTION_LIMIT.to_string());
        self.report(issue);
    }

    /// The first line of the file, without its line ending. Never empty
    /// unless the file is: a diagnostic on a blank first line covers its line
    /// break, so it always has a span to show (SPEC §8.1).
    fn first_line(&self) -> Span {
        let text = &self.file.text;
        let end = text.find(['\n', '\r']).unwrap_or(text.len());
        if end == 0 && !text.is_empty() {
            let width = text.chars().next().map_or(0, char::len_utf8);
            return Span::new(0, width);
        }
        Span::new(0, end)
    }

    /// Moves an issue from the file's first line to the key or value it's
    /// about, and words a string field's number or boolean as a quoting
    /// problem (the registry's `quote` variant).
    fn relocate(&self, mut issue: Issue, index: &YamlIndex) -> Issue {
        let path = match issue.slug {
            s if s == diagnostics::FRONTMATTER_UNKNOWN_KEY
                || s == diagnostics::FRONTMATTER_RESERVED_IN_FRAGMENT =>
            {
                issue.arg("key")
            }
            s if s == diagnostics::FRONTMATTER_TYPE_MISMATCH => issue.arg("field"),
            _ => None,
        };
        let Some(node) = path.and_then(|p| index.get(p)) else {
            return issue;
        };
        let is_type = issue.slug == diagnostics::FRONTMATTER_TYPE_MISMATCH;
        let span = match (is_type, node.scalar, node.key) {
            (true, true, _) => node.value,
            (_, _, Some(key)) => key,
            _ => node.value,
        };
        issue.location = Location::new(self.id, span);

        // A misspelled key has one certain replacement: the suggestion.
        if issue.slug == diagnostics::FRONTMATTER_UNKNOWN_KEY
            && issue.variant == Some("suggestion")
            && let (Some(suggestion), Some(key)) = (issue.arg("suggestion"), node.key)
        {
            let fix = Fix {
                title: format!("Rename the key to `{suggestion}`"),
                file: self.id,
                edits: vec![TextEdit::replace(key, suggestion.to_owned())],
                // The nearest key, which may not be the one meant.
                applicability: Applicability::Unsafe,
            };
            issue = issue.with_fix(fix);
        }

        let quotable = is_type
            && node.plain
            && issue.arg("expected") == Some("a string")
            && matches!(issue.arg("found"), Some("a number" | "a boolean"));
        if quotable {
            let raw = self
                .file
                .text
                .get(node.value.range())
                .unwrap_or_default()
                .to_owned();
            let field = issue.arg("field").unwrap_or_default().to_owned();
            let found = issue.arg("found").unwrap_or_default().to_owned();
            issue.variant = Some("quote");
            issue.args.clear();
            issue = issue
                .with_arg("field", field)
                .with_arg("value", raw)
                .with_arg("found", found);
        }
        issue
    }

    /// The reserved keys on a page (SPEC §2.1): `available` holds an
    /// availability spec, `variant` a mapping of dimensions to values,
    /// `permalink` a name with the characters an `@id` has, and `listing`
    /// one of its values.
    fn check_reserved_keys(&mut self, value: &Value, index: &YamlIndex, first_line: Span) {
        // SPEC §2.1: a value of the wrong shape is a type mismatch.
        let Value::Mapping(map) = value else { return };
        if let Some(v) = map.get("available") {
            let node = index.get("available");
            let whole = node.map_or(first_line, |n| n.key.unwrap_or(n.value));
            match v {
                Value::String(text) => {
                    let (offset, precise) = node
                        .and_then(|n| self.scalar_offset(n, text))
                        .map_or((0, false), |o| (o, true));
                    self.check_availability_text(text, offset, whole, precise);
                }
                other => self.mismatch(
                    "available",
                    "an availability spec, such as `cloud, self-managed 3.3`",
                    other,
                    node.map_or(first_line, |n| n.value),
                ),
            }
        }
        if let Some(v) = map.get("variant") {
            self.check_variant_key(v, index, first_line);
        }
        if let Some(v) = map.get("permalink") {
            let node = index.get("permalink");
            let at = node.map_or(first_line, |n| n.value);
            match v {
                Value::String(name) => self.check_name(name, at, Some("permalink")),
                other => self.mismatch("permalink", "a name, such as `cli-reference`", other, at),
            }
        }
        if let Some(v) = map.get(Listing::KEY) {
            let node = index.get(Listing::KEY);
            let at = node.map_or(first_line, |n| n.value);
            match v.as_str().and_then(Listing::from_name) {
                Some(listing) => {
                    if let Some(node) = node {
                        self.check_listing_rule(listing, node);
                        if listing == Listing::Unlisted {
                            self.check_listing_fetched(node);
                        }
                    }
                }
                None => {
                    let values: Vec<&str> = Listing::ALL.iter().map(|l| l.name()).collect();
                    self.mismatch(
                        Listing::KEY,
                        &format!("one of: {}", values.join(", ")),
                        v,
                        at,
                    );
                }
            }
        }
    }

    /// A page whose `listing` is looser than the rule of `[listing]` it's
    /// under (`listing-loosened`). The fix removes `listing`.
    fn check_listing_rule(&mut self, listing: Listing, node: &Node) {
        let Some(rule) = self.model.listing_rule(self.file.path.as_str()) else {
            return;
        };
        if listing >= rule.listing {
            return;
        }
        let mut issue = Issue::new(
            diagnostics::LISTING_LOOSENED,
            Location::new(self.id, node.value),
        )
        .with_arg("listing", listing.name())
        .with_arg("rule", rule.text.clone())
        .with_arg("line", rule.line.to_string());
        if rule.level == ListingLevel::Site {
            issue = issue
                .with_variant("default")
                .with_arg("strict", rule.listing.name());
        }
        if let Some(fix) = self.remove_listing(node, "Remove `listing` from the page", true) {
            issue = issue.with_fix(fix);
        }
        self.report(issue);
    }

    /// A fix that removes the page's `listing` line, line break and all.
    /// `None` when the key's place isn't known.
    fn remove_listing(&self, node: &Node, title: &str, safe: bool) -> Option<Fix> {
        let key = node.key?;
        let text = &self.file.text;
        let start = text[..key.start()].rfind('\n').map_or(0, |n| n + 1);
        let end = text[key.start()..]
            .find('\n')
            .map_or(text.len(), |n| key.start() + n + 1);
        Some(Fix {
            title: title.to_owned(),
            file: self.id,
            edits: vec![TextEdit::replace(Span::new(start, end), String::new())],
            applicability: if safe {
                Applicability::Safe
            } else {
                Applicability::Unsafe
            },
        })
    }

    /// A page that's `unlisted` at an address the project's own `[crawlers]
    /// lines` disallow for every crawler, which then never reads that it
    /// isn't to be listed (`listing-disallowed`). The fix removes `listing`.
    fn check_listing_fetched(&mut self, node: &Node) {
        let lines = pages::robots_lines(self.model);
        let route = AstroRouter::from_consumer(&self.model.consumer).route(&self.file.path);
        let Some(at) = pages::disallowing(&lines, &route) else {
            return;
        };
        let Some(line) = self.model.crawlers.lines.get(at) else {
            return;
        };
        let mut issue = Issue::new(
            diagnostics::LISTING_DISALLOWED,
            Location::new(self.id, node.value),
        )
        .with_arg("line", line.text.clone());
        // It lists the page, which may not be what was meant.
        if let Some(fix) = self.remove_listing(node, "Remove `listing: unlisted`", false) {
            issue = issue.with_fix(fix);
        }
        self.report(issue);
    }

    /// `variant`: a mapping from dimension names to a value or a list of
    /// values, validated like `@variant` attributes (SPEC §4.3).
    fn check_variant_key(&mut self, value: &Value, index: &YamlIndex, first_line: Span) {
        let node_span = |path: &str| {
            index
                .get(path)
                .map_or(first_line, |n| n.key.unwrap_or(n.value))
        };
        let Value::Mapping(map) = value else {
            self.mismatch(
                "variant",
                "a mapping from dimension names to values",
                value,
                node_span("variant"),
            );
            return;
        };
        for (key, v) in map {
            let Some(name) = key.as_str() else { continue };
            let path = format!("variant.{name}");
            let key_span = node_span(&path);
            let items: Vec<(&Value, String)> = match v {
                Value::Sequence(items) => items
                    .iter()
                    .enumerate()
                    .map(|(i, item)| (item, format!("{path}[{i}]")))
                    .collect(),
                other => vec![(other, path.clone())],
            };
            let mut values = Vec::new();
            let mut ok = true;
            for (item, item_path) in items {
                match item {
                    Value::String(s) => {
                        let span = index.get(&item_path).map_or(key_span, |n| n.value);
                        values.push((s.clone(), span));
                    }
                    other => {
                        ok = false;
                        self.mismatch(
                            &path,
                            "a value or a list of values",
                            other,
                            index.get(&item_path).map_or(key_span, |n| n.value),
                        );
                    }
                }
            }
            if ok {
                self.check_dimension(name, key_span, &values);
            }
        }
    }

    pub(super) fn mismatch(&mut self, field: &str, expected: &str, found: &Value, at: Span) {
        let issue = Issue::new(
            diagnostics::FRONTMATTER_TYPE_MISMATCH,
            Location::new(self.id, at),
        )
        .with_arg("field", field.to_owned())
        .with_arg("expected", expected.to_owned())
        .with_arg("found", describe(found));
        self.report(issue);
    }

    /// Where a scalar's text starts in the file, when it can be told: a plain
    /// scalar starts at its value, and a quoted one just after its quote,
    /// provided the text between the quotes is the value as written.
    fn scalar_offset(&self, node: &Node, text: &str) -> Option<usize> {
        let start = node.value.start();
        let source = self.file.text.get(start..)?;
        if node.plain && source.starts_with(text) {
            return Some(start);
        }
        let inner = source.get(1..)?;
        (matches!(source.as_bytes().first(), Some(b'"' | b'\'')) && inner.starts_with(text))
            .then_some(start + 1)
    }
}

/// How YAML read a value, for messages.
fn describe(v: &Value) -> String {
    match v {
        Value::Null => "empty".into(),
        Value::Bool(_) => "a boolean".into(),
        Value::Number(_) => "a number".into(),
        Value::String(_) => "a string".into(),
        Value::Sequence(_) => "a list".into(),
        Value::Mapping(_) => "a mapping".into(),
        Value::Tagged(t) => describe(&t.value),
    }
}
