//! Acknowledgements (SPEC §4.9): problems an author says are intended.
//!
//! A review check's problem can be acknowledged where the check reports it:
//! a page in its `intended` frontmatter, a block with `@intended` above it,
//! or a content model entry in `[[intended]]`. [`Acknowledgements`] reads
//! the valid ones (the checks report the rest, in `check_file` and the
//! content model's loader), and [`Acknowledgements::apply`] takes the
//! problems they cover out of a list of diagnostics, keeps them aside with
//! their reasons, and reports each acknowledgement that covered nothing.

use ascribe_core::diagnostics::ACKNOWLEDGEABLE;
use ascribe_core::intended::check_named;
use ascribe_core::{
    Applicability, DiagnosticSlug, FileId, Fix, Issue, Location, Place, Span, TextEdit, diagnostics,
};
use ascribe_model::{CheckLevel, ContentModel};
use ascribe_syntax::{Block, BlockKind, ParseOptions, PrimaryValue, bound_block, parse};
use serde_yaml_ng::Value;

use crate::yaml::YamlIndex;
use crate::{Diagnostic, Project, SourceFile};

/// The key a page's acknowledgements are under in its frontmatter.
pub const FRONTMATTER_KEY: &str = "intended";

/// One acknowledgement: a check, what it covers, and why.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Acknowledgement {
    /// The check whose problem is intended.
    pub check: DiagnosticSlug,
    /// Why.
    pub reason: String,
    /// Where it's written: the `@intended` line, or the `check` value of a
    /// frontmatter entry or of a `[[intended]]` table.
    pub at: Location,
    /// What it covers.
    pub covers: Covers,
    /// The text that removes it, in `at`'s file, when it can be told.
    removal: Option<Span>,
}

/// What an acknowledgement covers.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Covers {
    /// A page or fragment: problems located in the file.
    Page(FileId),
    /// A block: problems located in its span, or with a related place there
    /// (a problem in an included fragment is located at the include).
    Block(Location),
    /// A content model entry or an image: problems that name it, as
    /// [`entry_arg`](ascribe_core::intended::entry_arg) writes it.
    Entry(String),
}

/// A problem an acknowledgement covers: not reported, but counted and listed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Acknowledged {
    /// The problem.
    pub problem: Diagnostic,
    /// The acknowledgement's reason.
    pub reason: String,
    /// Where the acknowledgement is written.
    pub at: Location,
}

/// What [`Acknowledgements::apply`] returns.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Applied {
    /// The diagnostics to report: those no acknowledgement covers, then an
    /// `intended-unused` for each acknowledgement that covered nothing, when
    /// asked for.
    pub diagnostics: Vec<Diagnostic>,
    /// The problems acknowledged, in the order they were found.
    pub acknowledged: Vec<Acknowledged>,
}

/// The valid acknowledgements of a project, or of some of its files.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Acknowledgements {
    list: Vec<Acknowledgement>,
}

impl Acknowledgements {
    /// Every acknowledgement in the project: in its pages and fragments, and
    /// in its content model.
    pub fn of(project: &Project) -> Acknowledgements {
        Acknowledgements::read(project, None, ACKNOWLEDGEABLE)
    }

    /// The acknowledgements in `files` and in the content model: all that
    /// can cover a problem located in those files.
    pub fn in_files(project: &Project, files: &[FileId]) -> Acknowledgements {
        Acknowledgements::read(project, Some(files), ACKNOWLEDGEABLE)
    }

    fn read(
        project: &Project,
        files: Option<&[FileId]>,
        review: &[(DiagnosticSlug, Place)],
    ) -> Acknowledgements {
        let mut list = entries(project.model(), project.model_text());
        let sources = project
            .sources()
            .iter()
            .filter(|f| files.is_none_or(|ids| ids.contains(&f.id)));
        for file in sources {
            list.extend(read_file(project.model(), file, review));
        }
        Acknowledgements { list }
    }

    /// The acknowledgements, content model's first, then each file's in
    /// file order and source order.
    pub fn list(&self) -> &[Acknowledgement] {
        &self.list
    }

    /// Whether there are none.
    pub fn is_empty(&self) -> bool {
        self.list.is_empty()
    }

    /// Takes the problems an acknowledgement covers out of `diagnostics`.
    /// With `unused`, which a caller asks for only when it checked every
    /// build, each acknowledgement that covered nothing is reported as
    /// `intended-unused`, unless `[checks]` turns its check off.
    pub fn apply(
        &self,
        model: &ContentModel,
        diagnostics: Vec<Diagnostic>,
        unused: bool,
    ) -> Applied {
        if self.list.is_empty() {
            return Applied {
                diagnostics,
                acknowledged: Vec::new(),
            };
        }
        let mut used = vec![false; self.list.len()];
        let mut out = Applied::default();
        for d in diagnostics {
            let found = self.list.iter().position(|a| a.covers_problem(&d));
            match found {
                Some(i) => {
                    used[i] = true;
                    let a = &self.list[i];
                    out.acknowledged.push(Acknowledged {
                        problem: d,
                        reason: a.reason.clone(),
                        at: a.at,
                    });
                }
                None => out.diagnostics.push(d),
            }
        }
        if unused {
            for (a, used) in self.list.iter().zip(used) {
                if used || model.checks.level(a.check) == Some(CheckLevel::Off) {
                    continue;
                }
                out.diagnostics.push(a.unused());
            }
        }
        out
    }
}

impl Acknowledgement {
    /// Whether it covers a problem: the same check, at the place it covers.
    fn covers_problem(&self, d: &Diagnostic) -> bool {
        if d.slug != self.check {
            return false;
        }
        match &self.covers {
            Covers::Page(file) => d.location.file == *file,
            Covers::Block(block) => {
                let inside = |at: &Location| at.file == block.file && contains(block.span, at.span);
                inside(&d.location) || d.related.iter().any(|r| inside(&r.location))
            }
            Covers::Entry(entry) => d.subject.as_deref() == Some(entry.as_str()),
        }
    }

    /// The advice that it covers nothing, with a fix that removes it.
    fn unused(&self) -> Diagnostic {
        let mut issue = Issue::new(diagnostics::INTENDED_UNUSED, self.at)
            .with_arg("check", self.check.as_str());
        if let Some(span) = self.removal {
            issue = issue.with_fix(Fix {
                title: "Remove the acknowledgement".to_owned(),
                file: self.at.file,
                edits: vec![TextEdit::delete(span)],
                // It covers nothing, so removing it changes no output and
                // brings back no problem.
                applicability: Applicability::Safe,
            });
        }
        Diagnostic::from_issue(&issue)
    }
}

/// Whether `outer` holds `inner`.
fn contains(outer: Span, inner: Span) -> bool {
    outer.start() <= inner.start() && inner.end() <= outer.end()
}

/// The content model's `[[intended]]`, which the loader has checked.
fn entries(model: &ContentModel, text: &str) -> Vec<Acknowledgement> {
    model
        .intended
        .iter()
        .map(|i| Acknowledgement {
            check: i.check,
            reason: i.reason.clone(),
            at: Location::new(FileId::new(0), i.check_span),
            covers: Covers::Entry(ascribe_core::intended::entry_arg(i.kind, &i.name)),
            removal: table_removal(text, i.span),
        })
        .collect()
}

/// The lines of a `[[intended]]` table, from its header up to the next
/// table or the end of the file; `None` when it isn't written with a header
/// of its own.
fn table_removal(text: &str, header: Span) -> Option<Span> {
    if text.get(header.range())? != "[[intended]]" {
        return None;
    }
    let start = line_start(text, header.start());
    let after = line_end(text, header.end());
    let end = text
        .get(after..)?
        .split_inclusive('\n')
        .scan(after, |at, line| {
            let here = *at;
            *at += line.len();
            Some((here, line))
        })
        .find(|(_, line)| line.trim_start_matches([' ', '\t']).starts_with('['))
        .map_or(text.len(), |(at, _)| at);
    Some(Span::new(start, end))
}

/// The acknowledgements a source file holds that the checks accept: the
/// rest are reported by `check_file`.
fn read_file(
    model: &ContentModel,
    file: &SourceFile,
    review: &[(DiagnosticSlug, Place)],
) -> Vec<Acknowledgement> {
    // Most files have none, and a mention of the word is cheap to look for.
    if file.unreadable.is_some() || !file.text.contains(FRONTMATTER_KEY) {
        return Vec::new();
    }
    let options = ParseOptions::new(model.directive_schemas())
        .with_file(file.id)
        .with_note_types(model.notes.iter().map(|n| n.name.clone()).collect());
    let doc = parse(&file.text, &options);
    let mut out = Vec::new();
    if let Some(fm) = &doc.frontmatter {
        frontmatter(file, fm.content, review, &mut out);
    }
    blocks(file, &doc.blocks, review, &mut out);
    out
}

/// The `intended` entries of a page's frontmatter.
fn frontmatter(
    file: &SourceFile,
    content: Span,
    review: &[(DiagnosticSlug, Place)],
    out: &mut Vec<Acknowledgement>,
) {
    let text = file.text.get(content.range()).unwrap_or_default();
    let Ok(Value::Mapping(map)) = serde_yaml_ng::from_str::<Value>(text) else {
        return;
    };
    let Some(Value::Sequence(items)) = map.get(FRONTMATTER_KEY) else {
        return;
    };
    let index = YamlIndex::build(text, content.start());
    let removals = frontmatter_removals(&file.text, &index, items.len());
    for (i, item) in items.iter().enumerate() {
        let Value::Mapping(entry) = item else {
            continue;
        };
        let (Some(Value::String(check)), Some(Value::String(reason))) =
            (entry.get("check"), entry.get("reason"))
        else {
            continue;
        };
        let only_known = entry
            .keys()
            .all(|k| matches!(k.as_str(), Some("check" | "reason")));
        let Ok(slug) = check_named(check, Place::Page, review) else {
            continue;
        };
        if !only_known || reason.trim().is_empty() {
            continue;
        }
        let Some(node) = index.get(&format!("{FRONTMATTER_KEY}[{i}].check")) else {
            continue;
        };
        out.push(Acknowledgement {
            check: slug,
            reason: reason.trim().to_owned(),
            at: Location::new(file.id, node.value),
            covers: Covers::Page(file.id),
            removal: removals.get(i).copied().flatten(),
        });
    }
}

/// For each of `count` entries of `intended`, the lines that remove it: its
/// own, or the key's too when it's the only one. `None` for an entry that
/// doesn't start a line of its own (a flow sequence).
fn frontmatter_removals(text: &str, index: &YamlIndex, count: usize) -> Vec<Option<Span>> {
    let Some(key) = index.get(FRONTMATTER_KEY).and_then(|n| n.key) else {
        return vec![None; count];
    };
    // Where each entry's lines start, and the end of its last scalar's line.
    let lines: Vec<Option<(usize, usize)>> = (0..count)
        .map(|i| {
            let path = format!("{FRONTMATTER_KEY}[{i}]");
            let start = index.get(&path)?.value.start();
            let end = ["check", "reason"]
                .iter()
                .filter_map(|k| index.get(&format!("{path}.{k}")))
                .map(|n| n.value.end())
                .max()?;
            Some((line_start(text, start), line_end(text, end)))
        })
        .collect();
    let key_line = line_start(text, key.start());
    lines
        .iter()
        .enumerate()
        .map(|(i, entry)| {
            let (start, end) = (*entry)?;
            if start <= key_line {
                return None;
            }
            if count == 1 {
                return Some(Span::new(key_line, end));
            }
            // Up to the next entry, so a comment between them goes too.
            let end = match lines.get(i + 1) {
                Some(next) => next.map(|(s, _)| s)?,
                None => end,
            };
            Some(Span::new(start, end))
        })
        .collect()
}

/// The `@intended` lines among `list`, and in the blocks they contain.
fn blocks(
    file: &SourceFile,
    list: &[Block],
    review: &[(DiagnosticSlug, Place)],
    out: &mut Vec<Acknowledgement>,
) {
    for (i, block) in list.iter().enumerate() {
        match &block.kind {
            BlockKind::Directive(line) if line.name == FRONTMATTER_KEY => {
                let check = line
                    .attributes
                    .as_ref()
                    .and_then(|a| a.get("check"))
                    .and_then(|a| a.value.as_ref())
                    .and_then(|v| v.as_text());
                let reason = match &line.primary {
                    Some(PrimaryValue::Line(p)) => p.text.trim(),
                    _ => "",
                };
                let slug = check.and_then(|c| check_named(c, Place::Block, review).ok());
                let bound = bound_block(list, i).and_then(|k| list.get(k));
                if let (Some(slug), Some(bound), false) = (slug, bound, reason.is_empty()) {
                    out.push(Acknowledgement {
                        check: slug,
                        reason: reason.to_owned(),
                        at: Location::new(file.id, line.span),
                        covers: Covers::Block(Location::new(file.id, bound.span)),
                        removal: line_removal(&file.text, line.span),
                    });
                }
            }
            BlockKind::BlockQuote(q) => blocks(file, &q.children, review, out),
            BlockKind::List(l) => {
                for item in &l.items {
                    blocks(file, &item.children, review, out);
                }
            }
            BlockKind::Container(c) => blocks(file, &c.children, review, out),
            BlockKind::Group(g) => {
                for arm in &g.arms {
                    blocks(file, &arm.children, review, out);
                }
            }
            _ => {}
        }
    }
}

/// The whole line a directive is on, line ending included, when nothing but
/// indentation and block quote markers stand before it.
fn line_removal(text: &str, line: Span) -> Option<Span> {
    let start = line_start(text, line.start());
    let before = text.get(start..line.start())?;
    if !before.chars().all(|c| matches!(c, ' ' | '\t' | '>')) {
        return None;
    }
    Some(Span::new(start, line_end(text, line.end())))
}

/// The start of the line `at` is on.
fn line_start(text: &str, at: usize) -> usize {
    text.get(..at)
        .and_then(|before| before.rfind('\n'))
        .map_or(0, |i| i + 1)
}

/// Just past the line ending of the line `at` is on, or the end of the text.
fn line_end(text: &str, at: usize) -> usize {
    text.get(at..)
        .and_then(|after| after.find('\n'))
        .map_or(text.len(), |i| at + i + 1)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::*;
    use ascribe_core::RelPath;
    use std::path::PathBuf;

    // No check can be acknowledged yet, so these make two of them review
    // checks: `binding-blank-line` about a block, and
    // `container-nesting-deep` about a page.
    const REVIEW: &[(DiagnosticSlug, Place)] = &[
        (diagnostics::BINDING_BLANK_LINE, Place::Block),
        (diagnostics::CONTAINER_NESTING_DEEP, Place::Page),
    ];

    const MODEL: &str = "spec = \"0.1\"\n[types.page]\ndefault = true\n\
        [types.page.frontmatter]\ntitle = \"string\"\n";

    fn project(files: &[(&str, &str)]) -> Project {
        let model = ascribe_model::load_str(MODEL, FileId::new(0)).unwrap();
        let sources = Project::from_sources(
            files
                .iter()
                .map(|(p, t)| (RelPath::parse(p).unwrap(), (*t).to_owned())),
        );
        Project::from_parts(
            PathBuf::from("/p"),
            RelPath::parse("content").unwrap(),
            model,
            MODEL.to_owned(),
            sources,
        )
    }

    fn read(project: &Project) -> Acknowledgements {
        Acknowledgements::read(project, None, REVIEW)
    }

    /// A problem of `check` at `span` of the first file.
    fn problem(project: &Project, check: DiagnosticSlug, span: Span) -> Diagnostic {
        let at = Location::new(project.sources()[0].id, span);
        let issue = Issue::new(check, at)
            .with_arg("name", "note")
            .with_arg("depth", "3");
        Diagnostic::from_issue(&issue)
    }

    fn span_of(project: &Project, text: &str) -> Span {
        let source = &project.sources()[0].text;
        let start = source.find(text).expect("text in the page");
        Span::new(start, start + text.len())
    }

    const PAGE: &str = "---\ntitle: T\nintended:\n  - check: container-nesting-deep\n    \
        reason: The steps need the depth.\n---\n\n\
        @intended {check=binding-blank-line}: The gap is on purpose.\n\
        Bound paragraph.\n\nOther paragraph.\n";

    #[test]
    fn reads_frontmatter_and_directives() {
        let project = project(&[("page.md", PAGE)]);
        let found = read(&project);
        let got: Vec<_> = found
            .list()
            .iter()
            .map(|a| (a.check, a.reason.as_str()))
            .collect();
        assert_eq!(
            got,
            [
                (
                    diagnostics::CONTAINER_NESTING_DEEP,
                    "The steps need the depth."
                ),
                (diagnostics::BINDING_BLANK_LINE, "The gap is on purpose."),
            ]
        );
        let Covers::Block(block) = &found.list()[1].covers else {
            panic!("a block");
        };
        assert_eq!(block.span, span_of(&project, "Bound paragraph."));
    }

    #[test]
    fn covers_the_page_and_the_block_and_nothing_else() {
        let project = project(&[("page.md", PAGE)]);
        let found = read(&project);
        let bound = problem(
            &project,
            diagnostics::BINDING_BLANK_LINE,
            span_of(&project, "Bound"),
        );
        let other = problem(
            &project,
            diagnostics::BINDING_BLANK_LINE,
            span_of(&project, "Other"),
        );
        let page = problem(
            &project,
            diagnostics::CONTAINER_NESTING_DEEP,
            span_of(&project, "Other"),
        );
        let applied = found.apply(
            project.model(),
            vec![bound.clone(), other.clone(), page.clone()],
            true,
        );
        assert_eq!(applied.diagnostics, [other]);
        let acknowledged: Vec<_> = applied
            .acknowledged
            .iter()
            .map(|a| (a.problem.clone(), a.reason.as_str()))
            .collect();
        assert_eq!(
            acknowledged,
            [
                (bound, "The gap is on purpose."),
                (page, "The steps need the depth."),
            ]
        );
    }

    #[test]
    fn an_acknowledgement_that_covers_nothing_is_reported_with_a_fix() {
        let project = project(&[("page.md", PAGE)]);
        let applied = read(&project).apply(project.model(), Vec::new(), true);
        let slugs: Vec<_> = applied.diagnostics.iter().map(|d| d.slug).collect();
        assert_eq!(
            slugs,
            [diagnostics::INTENDED_UNUSED, diagnostics::INTENDED_UNUSED]
        );
        // Removing both gives the page without them.
        let text = &project.sources()[0].text;
        let mut edits: Vec<TextEdit> = applied
            .diagnostics
            .iter()
            .flat_map(|d| d.fixes[0].edits.clone())
            .collect();
        edits.sort_by_key(|e| std::cmp::Reverse(e.span.start()));
        let mut fixed = text.clone();
        for e in edits {
            fixed.replace_range(e.span.range(), &e.new_text);
        }
        assert_eq!(
            fixed,
            "---\ntitle: T\n---\n\nBound paragraph.\n\nOther paragraph.\n"
        );
        // Without `unused`, as for one build, nothing is said.
        let one_build = read(&project).apply(project.model(), Vec::new(), false);
        assert!(one_build.diagnostics.is_empty());
    }

    #[test]
    fn one_used_in_one_build_and_not_the_other_is_used() {
        // A page two builds treat differently: the block's problem is
        // reported in `cloud` only, which `diagnose` gives as its builds.
        let project = project(&[("page.md", PAGE)]);
        let mut bound = problem(
            &project,
            diagnostics::BINDING_BLANK_LINE,
            span_of(&project, "Bound"),
        );
        bound.builds = vec!["cloud".to_owned()];
        let page = problem(
            &project,
            diagnostics::CONTAINER_NESTING_DEEP,
            span_of(&project, "Other"),
        );
        let applied = read(&project).apply(project.model(), vec![bound, page], true);
        assert!(
            applied.diagnostics.is_empty(),
            "neither is unused: {:?}",
            applied.diagnostics
        );
        assert_eq!(applied.acknowledged[0].problem.builds, ["cloud"]);
    }

    #[test]
    fn one_of_several_frontmatter_entries_is_removed_alone() {
        let page = "---\ntitle: T\nintended:\n  - check: container-nesting-deep\n    \
            reason: One.\n  - check: container-nesting-deep\n    reason: Two.\n---\n\nText.\n";
        let project = project(&[("page.md", page)]);
        let found = read(&project);
        let removal = found.list()[1].removal.unwrap();
        let mut fixed = page.to_owned();
        fixed.replace_range(removal.range(), "");
        assert_eq!(
            fixed,
            "---\ntitle: T\nintended:\n  - check: container-nesting-deep\n    reason: One.\n---\n\nText.\n"
        );
    }

    #[test]
    fn invalid_ones_cover_nothing() {
        let page = "---\ntitle: T\nintended:\n  - check: binding-blank-line\n    reason: A block check.\n  \
            - check: container-nesting-deep\n---\n\n\
            @intended {check=container-nesting-deep}: A page check.\nText.\n\n\
            @intended {check=title-not-accepted}: Not a review check.\nText.\n";
        let project = project(&[("page.md", page)]);
        assert!(read(&project).is_empty());
    }

    #[test]
    fn a_block_covers_a_problem_in_a_container_and_one_located_at_an_include() {
        let page = "---\ntitle: T\n---\n\n@intended {check=binding-blank-line}: On purpose.\n\
            @note:\nInside.\n@end\n";
        let project = project(&[("page.md", page)]);
        let found = read(&project);
        let inside = problem(
            &project,
            diagnostics::BINDING_BLANK_LINE,
            span_of(&project, "Inside."),
        );
        let mut at_include = problem(
            &project,
            diagnostics::BINDING_BLANK_LINE,
            span_of(&project, "title"),
        );
        at_include.related.push(crate::RelatedInfo {
            location: Location::new(project.sources()[0].id, span_of(&project, "Inside.")),
            message: "in the included fragment".to_owned(),
        });
        let applied = found.apply(project.model(), vec![inside, at_include], true);
        assert!(applied.diagnostics.is_empty());
        assert_eq!(applied.acknowledged.len(), 2);
    }

    #[test]
    fn a_check_turned_off_leaves_its_acknowledgements_alone() {
        let project = project(&[("page.md", PAGE)]);
        let mut model = project.model().clone();
        model.checks.settings.push(ascribe_model::CheckSetting {
            slug: diagnostics::BINDING_BLANK_LINE,
            level: Some(CheckLevel::Off),
            limit: None,
        });
        let applied = read(&project).apply(&model, Vec::new(), true);
        assert_eq!(applied.diagnostics.len(), 1);
        assert_eq!(
            applied.diagnostics[0].message,
            "nothing here needs acknowledging any more: `container-nesting-deep` reports no \
             problem here in any build"
        );
    }

    #[test]
    fn an_entry_covers_the_problems_that_name_it() {
        let project = project(&[("page.md", "---\ntitle: T\n---\n")]);
        let text = "spec = \"0.1\"\n\n[[intended]]\ncheck = \"binding-blank-line\"\n\
            phrase = \"old\"\nreason = \"Kept.\"\n\n[phrases]\nold = \"x\"\n";
        let mut model = project.model().clone();
        let header = text.find("[[intended]]").unwrap();
        let body = text.find("check =").unwrap();
        model.intended.push(ascribe_model::Intended {
            check: diagnostics::BINDING_BLANK_LINE,
            kind: ascribe_core::EntryKind::Phrase,
            name: "old".to_owned(),
            reason: "Kept.".to_owned(),
            span: Span::new(header, header + "[[intended]]".len()),
            check_span: Span::new(body + 8, body + 28),
        });
        let found = Acknowledgements {
            list: entries(&model, text),
        };
        let mut named = problem(&project, diagnostics::BINDING_BLANK_LINE, Span::new(0, 1));
        named.subject = Some("phrase old".to_owned());
        let other = problem(&project, diagnostics::BINDING_BLANK_LINE, Span::new(0, 1));
        let applied = found.apply(&model, vec![named, other.clone()], true);
        assert_eq!(applied.diagnostics, [other]);
        let removal = found.list()[0].removal.unwrap();
        let mut fixed = text.to_owned();
        fixed.replace_range(removal.range(), "");
        assert_eq!(fixed, "spec = \"0.1\"\n\n[phrases]\nold = \"x\"\n");
    }
}
