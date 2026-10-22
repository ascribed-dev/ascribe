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
    FieldRole, FrontmatterSchema, Listing, ListingLevel, Purpose, TYPE_KEY, TypeMatch,
    crawler_tokens, inline, types, validate_frontmatter,
};
use ascribe_resolve::{
    AstroRouter, Local, NAV_LABEL, REPLACES, RefKind, Target, llms, pages, reference_target,
};
use ascribe_syntax::{BlockKind, Frontmatter, Inline, InlineKind, ParseOptions};
use serde_yaml_ng::Value;

use super::Ctx;
use super::text::{quoted_and, suggest};
use crate::roles;
use crate::yaml::{Node, YamlIndex};

impl Ctx<'_> {
    pub(super) fn check_frontmatter(&mut self, frontmatter: Option<&Frontmatter>) {
        let first_line = self.first_line();
        let path = self.file.path.as_str().to_owned();
        let fragment = self.model.is_fragment(&path);

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

        let schema: &FrontmatterSchema = if fragment {
            &self.model.fragments.frontmatter
        } else {
            match self.model.type_of(Some(&value)) {
                TypeMatch::One(t) => {
                    self.page_type = Some(t);
                    &t.frontmatter
                }
                TypeMatch::Missing => {
                    self.report_type_missing(frontmatter, first_line);
                    return;
                }
                TypeMatch::Unknown => {
                    self.report_type_unknown(&value, &index, first_line);
                    return;
                }
            }
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
            self.check_description_links(schema, &value, &index);
            self.check_template_fields(&value, &index);
        }
        self.check_intended_key(&value, &index, first_line);
    }

    /// A page that names no content type, in a project that declares types
    /// (SPEC §7.2), at its first line. A fix adds each declared type, as the
    /// frontmatter's first key, where `ascribe fmt` keeps it.
    fn report_type_missing(&mut self, frontmatter: Option<&Frontmatter>, first_line: Span) {
        let names: Vec<&str> = self.model.types.iter().map(|t| t.name.as_str()).collect();
        let mut issue = Issue::new(
            diagnostics::CONTENT_TYPE_MISSING,
            Location::new(self.id, first_line),
        )
        .with_arg("values", names.join(", "));
        for name in &names {
            let edit = match frontmatter {
                Some(fm) => TextEdit::insert(fm.content.start(), format!("{TYPE_KEY}: {name}\n")),
                None => TextEdit::insert(0, format!("---\n{TYPE_KEY}: {name}\n---\n\n")),
            };
            issue = issue.with_fix(Fix {
                title: format!("Add `{TYPE_KEY}: {name}`"),
                file: self.id,
                edits: vec![edit],
                // Which type a page is is the author's to choose.
                applicability: Applicability::Unsafe,
            });
        }
        self.report(issue);
    }

    /// A page whose `type` names no declared content type (SPEC §7.2), at
    /// its value. The fix is the closest declared name, when one is close.
    fn report_type_unknown(&mut self, value: &Value, index: &YamlIndex, first_line: Span) {
        let names: Vec<&str> = if self.model.declares_types() {
            self.model.types.iter().map(|t| t.name.as_str()).collect()
        } else {
            Vec::new()
        };
        let node = index.get(TYPE_KEY);
        let at = node.map_or(first_line, |n| n.value);
        let mut issue = Issue::new(
            diagnostics::CONTENT_TYPE_UNKNOWN,
            Location::new(self.id, at),
        )
        .with_arg("values", names.join(", "));
        match value.get(TYPE_KEY) {
            Some(Value::String(declared)) => {
                issue = issue.with_arg("type", declared.clone());
                if names.is_empty() {
                    issue = issue.with_variant("none");
                    if let Some(node) = node
                        && let Some(fix) = self.remove_line(node, "Remove `type`", true)
                    {
                        issue = issue.with_fix(fix);
                    }
                } else if let Some(suggestion) = suggest(declared, names.iter().copied()) {
                    issue = issue
                        .with_variant("suggestion")
                        .with_arg("suggestion", suggestion.to_owned());
                    if node.is_some_and(|n| n.scalar) {
                        issue = issue.with_fix(Fix {
                            title: format!("Change the type to `{suggestion}`"),
                            file: self.id,
                            edits: vec![TextEdit::replace(at, suggestion.to_owned())],
                            // The nearest name, which may not be the one meant.
                            applicability: Applicability::Unsafe,
                        });
                    }
                }
            }
            found => {
                issue = issue
                    .with_variant("not-a-name")
                    .with_arg("found", found.map_or_else(String::new, types::found));
            }
        }
        self.report(issue);
    }

    /// A description the page writes that's longer than its line in
    /// `llms.txt` should be, when the project publishes one (`[consumer]
    /// agents = true`). A field's default is reported once, where it's
    /// written ([`crate::descriptions`]), not on each page that leaves the
    /// field to it.
    fn check_description_length(
        &mut self,
        schema: &FrontmatterSchema,
        value: &Value,
        index: &YamlIndex,
    ) {
        let Some(field) = schema.field_with_role(FieldRole::Description) else {
            return;
        };
        let Some(raw) = value
            .get(field.name.as_str())
            .and_then(llms::own_description)
        else {
            return;
        };
        let Some(text) = llms::description_text(self.model, raw) else {
            return;
        };
        let length = text.chars().count();
        if !roles::description_too_long(self.model, length) {
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

    /// A description holding Markdown link syntax, which it shows as
    /// written: a description is plain text (decision #455).
    fn check_description_links(
        &mut self,
        schema: &FrontmatterSchema,
        value: &Value,
        index: &YamlIndex,
    ) {
        if self.model.checks.is_off(diagnostics::DESCRIPTION_LINK) {
            return;
        }
        let Some(field) = schema.field_with_role(FieldRole::Description) else {
            return;
        };
        let Some(raw) = value.get(field.name.as_str()).and_then(Value::as_str) else {
            return;
        };
        let Some(node) = index.get(&field.name) else {
            return;
        };
        let parsed = ascribe_syntax::parse(raw, &ParseOptions::default());
        let mut links = Vec::new();
        for block in &parsed.blocks {
            if let BlockKind::Paragraph(p) = &block.kind {
                collect_links(&p.inlines, &mut links);
            }
        }
        // A plain scalar is written as it is in the file, so a link's place
        // in the value is its place in the file.
        let written = node.plain && self.file.text.get(node.value.range()) == Some(raw);
        for link in links {
            let source = raw.get(link.span.range()).unwrap_or_default();
            let text = link_text(raw, link);
            let mut issue = Issue::new(
                diagnostics::DESCRIPTION_LINK,
                Location::new(self.id, node.value),
            )
            .with_arg("link", source.to_owned());
            if written {
                let base = node.value.start();
                let at = Span::new(base + link.span.start(), base + link.span.end());
                issue.location = Location::new(self.id, at);
                if keeps_plain(raw, link.span, text) {
                    issue = issue.with_fix(Fix {
                        title: format!("Keep the link's text, `{text}`"),
                        file: self.id,
                        edits: vec![TextEdit::replace(at, text.to_owned())],
                        // It changes the description's text.
                        applicability: Applicability::Unsafe,
                    });
                }
            }
            self.report(issue);
        }
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
    /// `permalink` a name with the characters an `@id` has, `replaces` the
    /// path of a page, `nav-label` a string on one line, and `listing` one
    /// of its values.
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
                    let removal = node.filter(|_| precise).and_then(|n| self.key_removal(n));
                    self.check_availability_text(text, offset, whole, precise, removal);
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
        if let Some(v) = map.get(REPLACES) {
            let node = index.get(REPLACES);
            let at = node.map_or(first_line, |n| n.value);
            match v {
                Value::String(written) => self.check_replaces(written, at),
                other => self.mismatch(
                    REPLACES,
                    "the path of the page it replaces, such as `install-legacy.md`",
                    other,
                    at,
                ),
            }
        }
        if let Some(v) = map.get(NAV_LABEL) {
            let node = index.get(NAV_LABEL);
            let at = node.map_or(first_line, |n| n.value);
            match v {
                Value::String(label) if label.contains(['\n', '\r']) => {
                    let issue = Issue::new(
                        diagnostics::FRONTMATTER_TYPE_MISMATCH,
                        Location::new(self.id, at),
                    )
                    .with_arg("field", NAV_LABEL)
                    .with_arg("expected", "a label on one line, such as `Keys`")
                    .with_arg("found", "a string of more than one line");
                    self.report(issue);
                }
                Value::String(label) => self.check_nav_label_length(label, map.get("title"), at),
                other => self.mismatch(NAV_LABEL, "a label on one line, such as `Keys`", other, at),
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

    /// A `nav-label` longer than the title it shortens (`nav-label-long`),
    /// each counted in characters as a reader sees it, without code spans'
    /// backticks when the title is read with inline markup.
    fn check_nav_label_length(&mut self, label: &str, title: Option<&Value>, at: Span) {
        let Some(title) = title.and_then(Value::as_str) else {
            return;
        };
        let inline = self
            .page_type
            .and_then(|t| t.frontmatter.fields.iter().find(|f| f.name == "title"))
            .is_some_and(|f| f.inline.is_some());
        let length = |text: &str| {
            let text = if inline {
                inline::plain_text(&inline::parse(text, &|_| None))
            } else {
                text.to_owned()
            };
            text.trim().chars().count()
        };
        let (label_length, title_length) = (length(label), length(title));
        if label_length <= title_length {
            return;
        }
        let issue = Issue::new(diagnostics::NAV_LABEL_LONG, Location::new(self.id, at))
            .with_arg("length", label_length.to_string())
            .with_arg("title", title_length.to_string());
        self.report(issue);
    }

    /// A `replaces` that doesn't name a page (`replaces-not-a-page`): it's
    /// read as a link destination is, and names a page without a heading.
    fn check_replaces(&mut self, written: &str, at: Span) {
        let target = reference_target(RefKind::Link, written, &[], &self.file.path, self.model);
        let variant = match &target {
            Target::Local(local) if local.source && local.fragment.is_some() => Some("heading"),
            Target::Local(Local {
                source: true,
                path: Some(path),
                ..
            }) => match self.project.source_at(path) {
                None => None,
                Some(_) if self.model.is_fragment(path.as_str()) => Some("fragment"),
                Some(_) => return,
            },
            _ => Some("other"),
        };
        let mut issue = Issue::new(diagnostics::REPLACES_NOT_A_PAGE, Location::new(self.id, at))
            .with_arg("path", written.to_owned());
        if let Some(variant) = variant {
            issue = issue.with_variant(variant);
        }
        self.report(issue);
    }

    /// A page whose `listing` is looser than the rule of `[listing]` it's
    /// under (`listing-loosened`). The fix removes `listing`.
    fn check_listing_rule(&mut self, listing: Listing, node: &Node) {
        let type_name = self.page_type().map(|t| t.name.as_str());
        let Some(rule) = self.model.listing_rule(self.file.path.as_str(), type_name) else {
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
        if let Some(fix) = self.remove_line(node, "Remove `listing` from the page", true) {
            issue = issue.with_fix(fix);
        }
        self.report(issue);
    }

    /// The lines of a top-level key whose value is on the key's line, line
    /// break and all. `None` when the key's place isn't known, or its value
    /// runs onto another line.
    fn key_removal(&self, node: &Node) -> Option<Span> {
        let key = node.key?;
        let text = &self.file.text;
        let start = text[..key.start()].rfind('\n').map_or(0, |n| n + 1);
        let end = text[key.start()..]
            .find('\n')
            .map_or(text.len(), |n| key.start() + n + 1);
        (node.value.end() <= end).then(|| Span::new(start, end))
    }

    /// A fix that removes a top-level key's line, line break and all.
    /// `None` when the key's place isn't known.
    fn remove_line(&self, node: &Node, title: &str, safe: bool) -> Option<Fix> {
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

    /// A page that's `unlisted` at an address a search crawler may not fetch,
    /// which then never reads that it isn't to be listed
    /// (`listing-disallowed`). Each search crawler reads the one group that
    /// applies to it: `*` for every search engine Ascribe doesn't name, and
    /// for each search token Ascribe knows, the group a disallowed `search`
    /// writes, or the project's own that names it, or `*`. The first found is
    /// reported, in that order. The fix removes `listing`.
    fn check_listing_fetched(&mut self, node: &Node) {
        let lines = pages::robots_lines(self.model);
        let route = AstroRouter::from_consumer(&self.model.consumer).route(&self.file.path);
        let line_text = |at: usize| self.model.crawlers.lines.get(at).map(|l| l.text.clone());
        let mut issue = Issue::new(
            diagnostics::LISTING_DISALLOWED,
            Location::new(self.id, node.value),
        );
        if let Some(line) = pages::disallowing(&lines, &route).and_then(line_text) {
            issue = issue.with_arg("line", line);
        } else if self.model.crawlers.disallows(Purpose::Search) {
            let tokens: Vec<&str> = crawler_tokens(Purpose::Search).collect();
            issue = issue
                .with_variant("purpose")
                .with_arg("purpose", Purpose::Search.name())
                .with_arg("crawler", quoted_and(&tokens));
        } else if let Some((token, line)) = crawler_tokens(Purpose::Search).find_map(|token| {
            pages::disallowing_for(&lines, token, &route)
                .and_then(line_text)
                .map(|line| (token, line))
        }) {
            issue = issue
                .with_variant("crawler")
                .with_arg("line", line)
                .with_arg("crawler", quoted_and(&[token]));
        } else {
            return;
        }
        // It lists the page, which may not be what was meant.
        if let Some(fix) = self.remove_line(node, "Remove `listing: unlisted`", false) {
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

/// The links among `inlines`, outermost first, at any depth.
fn collect_links<'a>(inlines: &'a [Inline], out: &mut Vec<&'a Inline>) {
    for inline in inlines {
        match &inline.kind {
            InlineKind::Link(_) => out.push(inline),
            InlineKind::Emphasis(children) | InlineKind::Strong(children) => {
                collect_links(children, out);
            }
            _ => {}
        }
    }
}

/// A link's text as written in `raw`: what's between its brackets, or an
/// autolink's address.
fn link_text<'a>(raw: &'a str, link: &Inline) -> &'a str {
    let InlineKind::Link(l) = &link.kind else {
        return "";
    };
    match (l.children.first(), l.children.last()) {
        (Some(first), Some(last)) => raw
            .get(first.span.start()..last.span.end())
            .unwrap_or_default(),
        _ => "",
    }
}

/// Whether replacing `span` of the plain scalar `raw` with `text` leaves a
/// plain scalar with the text expected, so the fix can't break the YAML.
fn keeps_plain(raw: &str, span: Span, text: &str) -> bool {
    let (Some(before), Some(after)) = (raw.get(..span.start()), raw.get(span.end()..)) else {
        return false;
    };
    let fixed = format!("{before}{text}{after}");
    let parsed = serde_yaml_ng::from_str::<Value>(&format!("d: {fixed}"));
    matches!(parsed, Ok(v) if v.get("d").and_then(Value::as_str) == Some(fixed.as_str()))
}
