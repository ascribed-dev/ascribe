//! `@available` specs, in a directive, in `available` frontmatter, or in a
//! table row's attribute block (SPEC §4.4).

use ascribe_core::availability::parse_availability;
use ascribe_core::{
    AttributeBlock, AttributeSchema, AttributeType, AttributeValue, Issue, Location, Span,
    diagnostics,
};
use ascribe_model::AvailabilityProblem;
use ascribe_syntax::{Block, BlockKind, PrimaryValue, bound_block, bound_heading};

use super::Ctx;
use super::attrs::Owner;
use super::text::quoted_list;

impl Ctx<'_> {
    /// A table row's attribute block: `available` is its only key, and its
    /// value is a spec.
    pub(super) fn check_row(&mut self, block: &AttributeBlock) {
        let declared = [AttributeSchema {
            key: "available".to_owned(),
            ty: AttributeType::String,
            required: false,
            default: None,
            description: None,
        }];
        self.check_block(block, &declared, Owner::Row);
        let removal = self.nested_removal(block.span, block.span.start(), self.row_removal(block));
        match block.get("available").and_then(|a| a.value.as_ref()) {
            Some(AttributeValue::Token(t)) => {
                self.check_availability_text(&t.text, t.span.start(), t.span, true, removal);
            }
            Some(AttributeValue::Quoted { text, span }) => {
                // Escapes inside the quotes move the spec off its source.
                let precise = text.len() + 2 == span.len();
                let removal = removal.filter(|_| precise);
                self.check_availability_text(text, span.start() + 1, *span, precise, removal);
            }
            // A value set is the wrong type, which `check_block` reported.
            Some(AttributeValue::Set { .. }) | None => {}
        }
    }

    /// Records where each mark in `blocks` that says something applies, as
    /// [`crate::releases::reads_as_unmarked`] tells: the section of the
    /// heading an `@available` line binds, or the block it binds. A mark
    /// inside one of them can't go, since its content would read as that
    /// mark instead.
    pub(super) fn collect_mark_scopes(&mut self, blocks: &[Block]) {
        for (i, block) in blocks.iter().enumerate() {
            match &block.kind {
                BlockKind::Directive(d) if d.name == "available" => {
                    let Some(PrimaryValue::Line(p)) = &d.primary else {
                        continue;
                    };
                    if self.reads_as_unmarked(&p.text, p.span.start()) {
                        continue;
                    }
                    let scope = if let Some(h) = bound_heading(blocks, i) {
                        Some(section(blocks, h))
                    } else {
                        bound_block(blocks, i).map(|k| blocks[k].span)
                    };
                    if let Some(scope) = scope {
                        self.mark_scopes.push((d.span, scope));
                    }
                }
                BlockKind::BlockQuote(q) => self.collect_mark_scopes(&q.children),
                BlockKind::List(l) => {
                    for item in &l.items {
                        self.collect_mark_scopes(&item.children);
                    }
                }
                BlockKind::Container(c) => self.collect_mark_scopes(&c.children),
                BlockKind::Group(g) => {
                    for arm in &g.arms {
                        self.collect_mark_scopes(&arm.children);
                    }
                }
                _ => {}
            }
        }
    }

    /// Whether the spec `text`, at `offset`, reads as no mark at every live
    /// line. One that doesn't parse says something, to be safe.
    pub(super) fn reads_as_unmarked(&self, text: &str, offset: usize) -> bool {
        parse_availability(text, offset).is_ok_and(|spec| {
            let spec = self.model.resolve_planned(&spec).spec;
            crate::releases::reads_as_unmarked(self.model, &spec)
        })
    }

    /// `removal` for the mark `mark`, which applies from `at`, when the
    /// mark may go: the file isn't a fragment, whose content reads as the
    /// scope it's included in, and no other mark that says something
    /// applies at `at`.
    pub(super) fn nested_removal(
        &self,
        mark: Span,
        at: usize,
        removal: Option<Span>,
    ) -> Option<Span> {
        let inside = self
            .mark_scopes
            .iter()
            .any(|(other, scope)| *other != mark && scope.start() <= at && at < scope.end());
        removal.filter(|_| !inside && !self.model.is_fragment(self.file.path.as_str()))
    }

    /// What removing a row's attribute block deletes: the block and the
    /// space before it, keeping one after the `|` when the block is alone
    /// in its cell. `None` when the block holds more than `available`.
    fn row_removal(&self, block: &AttributeBlock) -> Option<Span> {
        if block.attributes.len() != 1 {
            return None;
        }
        let before = self.source().get(..block.span.start())?;
        let text = before.trim_end_matches([' ', '\t']);
        let start = if text.ends_with('|') {
            (text.len() + 1).min(block.span.start())
        } else {
            text.len()
        };
        Some(Span::new(start, block.span.end()))
    }

    /// Parses a spec and checks it against the content model: syntax,
    /// targets, states, versionless targets, history order, whether
    /// `[releases]` can place each version, and whether the mark still does
    /// anything.
    ///
    /// `offset` is where `text` starts in the file. When `precise` is false
    /// the offset can't be trusted (a multi-line or escaped YAML scalar), so
    /// everything is reported at `whole`. `removal` is what removing the
    /// whole mark deletes, when that's known.
    pub(super) fn check_availability_text(
        &mut self,
        text: &str,
        offset: usize,
        whole: Span,
        precise: bool,
        removal: Option<Span>,
    ) {
        let at = |span: Span| Location::new(self.id, if precise { span } else { whole });
        let spec = match parse_availability(text, offset) {
            Ok(spec) => spec,
            Err(e) => {
                let issue = Issue::new(diagnostics::AVAILABLE_SYNTAX, at(e.span))
                    .with_arg("spec", text.trim().to_owned())
                    .with_arg("detail", e.detail);
                self.report(issue);
                return;
            }
        };
        // `planned` reads as the line it names (SPEC §4.4); one that names
        // none is its own problem, and the rest of the spec is checked as
        // written.
        let resolved = self.model.resolve_planned(&spec);
        let spec = resolved.spec;
        let states: Vec<&str> = self
            .model
            .lifecycle
            .iter()
            .map(|s| s.name.as_str())
            .collect();
        let mut issues: Vec<Issue> = resolved
            .problems
            .into_iter()
            .map(|p| {
                let issue = Issue::new(diagnostics::AVAILABLE_PLANNED_LINE, at(p.span))
                    .with_arg("target", p.target.text);
                if p.lines.is_empty() {
                    issue
                } else {
                    let lines: Vec<&str> = p.lines.iter().map(|l| l.text.as_str()).collect();
                    issue
                        .with_variant("several")
                        .with_arg("lines", quoted_list(&lines))
                }
            })
            .collect();
        for problem in self.model.check_availability(&spec) {
            issues.push(match problem {
                AvailabilityProblem::UnknownTarget(name) => {
                    Issue::new(diagnostics::AVAILABLE_UNKNOWN, at(name.span))
                        .with_arg("name", name.text)
                }
                AvailabilityProblem::UnknownState(name) => {
                    Issue::new(diagnostics::AVAILABLE_UNKNOWN, at(name.span))
                        .with_variant("state")
                        .with_arg("name", name.text)
                        .with_arg("states", quoted_list(&states))
                }
                AvailabilityProblem::VersionlessVersion { target, version } => {
                    Issue::new(diagnostics::AVAILABLE_VERSIONLESS, at(version.span))
                        .with_arg("target", target.text)
                }
                AvailabilityProblem::DimensionVersion {
                    target,
                    version,
                    example,
                } => Issue::new(diagnostics::AVAILABLE_VERSIONLESS, at(version.span))
                    .with_variant("dimension")
                    .with_arg("target", target.text)
                    .with_arg("example", example),
                AvailabilityProblem::HistoryOrder {
                    target,
                    later,
                    earlier,
                    span,
                } => Issue::new(diagnostics::AVAILABLE_HISTORY_ORDER, at(span))
                    .with_arg("target", target.text)
                    .with_arg("later", later)
                    .with_arg("earlier", earlier),
            });
        }
        // The list places the versions of a spec that's otherwise right, and
        // only a spec it places is asked whether any output shows it: a
        // mistyped version is one problem, not also content in no line.
        if issues.is_empty() {
            issues = crate::releases::placement_issues(self.model, &spec.entries, &at, precise);
        }
        if issues.is_empty() {
            issues = crate::releases::inert_issues(self.model, &spec, &at, precise, removal);
        }
        for issue in issues {
            self.report(issue);
        }
    }
}

/// The section of the heading `blocks[heading]`: from it up to the next
/// heading of its level or a higher one, or the end of `blocks`.
fn section(blocks: &[Block], heading: usize) -> Span {
    let level = match &blocks[heading].kind {
        BlockKind::Heading(h) => h.level,
        _ => 0,
    };
    let start = blocks[heading].span.start();
    let end = blocks[heading + 1..]
        .iter()
        .find(|b| matches!(&b.kind, BlockKind::Heading(h) if h.level <= level))
        .map_or_else(
            || blocks.last().map_or(start, |b| b.span.end()),
            |b| b.span.start(),
        );
    Span::new(start, end)
}
