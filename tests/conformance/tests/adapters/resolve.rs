//! The adapter for `tessera-resolve`'s build resolution: what phase 12
//! produces.
//!
//! It handles the `resolve` tag (resolution passes and build modes, SPEC
//! §9.2, §9.3). For a build it resolves every page of the case's project with
//! the build's modes and gives, for each published page, the **resolved
//! outline**: the page after includes, availability, the build's modes, and
//! phrases (SPEC §9.2 steps 1 to 4). Its text is still the page's source text
//! (links keep their file-path destinations and empty link text, heading ids
//! aren't shown, glossary terms aren't linked), with each substituted phrase
//! replaced by its value. It also gives the assets the build copies and the
//! page-level problems the resolution records.
//!
//! The problems are `variant-no-arm-survives`, `available-exceeds-scope`,
//! `link-id-removed`, `link-page-dropped`, and what expansion found
//! (`include-cycle`, `include-id-missing`), at the locations SPEC §8.1 and Q20
//! give them. Phase 14 reports them; this adapter only reads them off the
//! resolved pages, for cases that expect them.

use tessera_conformance::outline::normalize_ws;
use tessera_conformance::{
    AdapterError, AdapterResult, Arm, BuildResult, Case, ConformanceAdapter, Diagnostic, Directive,
    Form, Group, Node, PageResult,
};
use tessera_core::{LineIndex, Span, WideEncoding};
use tessera_resolve::{
    DefaultRouter, IncludeSite, Project, ResolvedBlock, ResolvedBuild, ResolvedKind, ResolvedPage,
    Substitution,
};
use tessera_syntax::{BlockKind, Bound, DirectiveLine, InlineKind, PrimaryValue, raw_text};

use super::include::load_project;
use super::structure::attributes;

/// Handles the cases that expect resolved pages, assets, or the page-level
/// problems a build records.
pub struct ResolveAdapter;

impl ConformanceAdapter for ResolveAdapter {
    fn name(&self) -> &str {
        "resolve"
    }

    fn handles_tag(&self, tag: &str) -> bool {
        tag == "resolve"
    }

    fn build(&self, case: &Case, build: &str) -> AdapterResult<BuildResult> {
        resolve_build(case, build).map(Some)
    }

    /// A case that resolves and expects no file-level diagnostics (such as
    /// `samples/include-and-selection`) gets them from the whole file-level
    /// check, as `tessera check` runs it: the resolution passes don't report.
    fn diagnostics(&self, case: &Case) -> AdapterResult<Vec<Diagnostic>> {
        super::file_level_diagnostics(case).map(Some)
    }
}

fn err(e: impl std::fmt::Display) -> AdapterError {
    AdapterError(e.to_string())
}

/// Resolves the named build of a case.
pub fn resolve_build(case: &Case, name: &str) -> Result<BuildResult, AdapterError> {
    let project = load_project(case)?;
    let build = project
        .model()
        .build(name)
        .ok_or_else(|| err(format!("the case's model has no build `{name}`")))?;
    let router = DefaultRouter::from_consumer(&project.model().consumer);
    let resolved = project.resolve_build(build, &router);
    result(&project, &resolved)
}

fn result(project: &Project, resolved: &ResolvedBuild) -> Result<BuildResult, AdapterError> {
    let mut out = BuildResult::default();
    for page in &resolved.pages {
        out.pages.insert(
            page.path.to_string(),
            PageResult {
                outline: Some(outline(project, &page.blocks)?),
                outputs: Default::default(),
            },
        );
        for problem in &page.problems {
            out.diagnostics
                .push(diagnostic(project, page, &problem.issue.slug, problem)?);
        }
    }
    out.assets = resolved.assets().iter().map(|p| p.to_string()).collect();
    out.diagnostics.sort_by(|a, b| {
        (&a.file, a.line, a.column, &a.slug).cmp(&(&b.file, b.line, b.column, &b.slug))
    });
    out.diagnostics.dedup();
    Ok(out)
}

/// A page problem as a conformance diagnostic. A page-level report goes at the
/// outermost include site (SPEC §8.1), except a cycle, which is reported
/// where it closes (Q20).
fn diagnostic(
    project: &Project,
    _page: &ResolvedPage,
    slug: &tessera_core::DiagnosticSlug,
    problem: &tessera_resolve::PageProblem,
) -> Result<Diagnostic, AdapterError> {
    let at: Option<&IncludeSite> = match (slug.as_str(), problem.via.first()) {
        ("include-cycle", _) | (_, None) => None,
        (_, Some(site)) => Some(site),
    };
    let (file, span) = match at {
        Some(site) => (site.file, site.span),
        None => (problem.issue.location.file, problem.issue.location.span),
    };
    let index = project
        .file_by_id(file)
        .ok_or_else(|| err(format!("no file {file}")))?;
    let pos = LineIndex::new(&index.source)
        .wide_line_col(WideEncoding::Utf32, span.start())
        .ok_or_else(|| err(format!("bad location in {}", index.path)))?;
    Ok(Diagnostic {
        slug: slug.as_str().to_owned(),
        file: index.path.to_string(),
        line: pos.line + 1,
        column: pos.col + 1,
    })
}

// ---------------------------------------------------------------------------
// The resolved tree as an outline

/// The text of `span` in the block's file, with the block's substituted
/// phrases replaced by their values, container prefixes removed, and
/// whitespace normalized.
fn text(source: &str, span: Span, subs: &[Substitution]) -> String {
    let mut patched = source.get(span.range()).unwrap_or("").to_owned();
    let mut inside: Vec<&Substitution> = subs
        .iter()
        .filter(|s| s.span.start() >= span.start() && s.span.end() <= span.end())
        .collect();
    inside.sort_by_key(|s| std::cmp::Reverse(s.span.start()));
    for s in inside {
        let from = s.span.start() - span.start();
        let to = s.span.end() - span.start();
        patched.replace_range(from..to, &s.value);
    }
    normalize_ws(&raw_text(&patched, Span::new(0, patched.len())))
}

fn outline(project: &Project, blocks: &[ResolvedBlock]) -> Result<Vec<Node>, AdapterError> {
    let mut out = Vec::new();
    for block in blocks {
        let index = project
            .file_by_id(block.file)
            .ok_or_else(|| err(format!("no file {}", block.file)))?;
        let source: &str = &index.source;
        let subs = &block.substitutions;
        let children = |blocks: &[ResolvedBlock]| outline(project, blocks);
        let node = match &block.kind {
            ResolvedKind::Leaf(leaf) => match &leaf.kind {
                BlockKind::Heading(h) => Some(Node::Heading {
                    level: h.level,
                    text: Some(text(source, h.content, subs)),
                }),
                BlockKind::Paragraph(p) => Some(paragraph(source, block, p)),
                BlockKind::CodeBlock(c) => Some(Node::Code {
                    text: Some(c.literal.clone()),
                    info: Some(c.info.clone()),
                    fenced: Some(c.fenced),
                }),
                BlockKind::HtmlBlock(h) => Some(Node::Html {
                    text: Some(h.literal.trim_end().to_owned()),
                }),
                BlockKind::ThematicBreak => Some(Node::ThematicBreak),
                BlockKind::Table(_) => Some(Node::Table),
                BlockKind::Directive(line) => Some(Node::Directive(directive(
                    source,
                    block,
                    line,
                    Form::Line,
                    Vec::new(),
                ))),
                _ => None,
            },
            ResolvedKind::BlockQuote { children: c } => Some(Node::Blockquote {
                children: children(c)?,
            }),
            ResolvedKind::List {
                ordered,
                start,
                items,
                ..
            } => Some(Node::List {
                ordered: *ordered,
                start: *start,
                children: items
                    .iter()
                    .map(|item| {
                        Ok(Node::Item {
                            children: children(&item.children)?,
                        })
                    })
                    .collect::<Result<_, AdapterError>>()?,
            }),
            ResolvedKind::Container {
                opener,
                children: c,
                ..
            } => Some(Node::Directive(directive(
                source,
                block,
                opener,
                Form::Container,
                children(c)?,
            ))),
            ResolvedKind::Group { name, arms, .. } => Some(Node::Group(Group {
                name: name.clone(),
                arms: arms
                    .iter()
                    .map(|arm| {
                        Ok(Arm {
                            attributes: attributes(&arm.opener),
                            title: title(source, block, &arm.opener),
                            children: children(&arm.children)?,
                        })
                    })
                    .collect::<Result<_, AdapterError>>()?,
            })),
        };
        out.extend(node);
    }
    Ok(out)
}

fn title(source: &str, block: &ResolvedBlock, line: &DirectiveLine) -> Option<String> {
    line.title
        .as_ref()
        .map(|t| text(source, t.content, &block.substitutions))
}

fn directive(
    source: &str,
    block: &ResolvedBlock,
    line: &DirectiveLine,
    form: Form,
    children: Vec<Node>,
) -> Directive {
    let subs = &block.substitutions;
    let primary = match &line.primary {
        Some(PrimaryValue::Text(p)) => Some(text(source, p.span, subs)),
        Some(PrimaryValue::Identifier(p)) => Some(p.text.clone()),
        // A surviving `@available` shows the spec a feature key stands for
        // (SPEC §4.4, Q25).
        Some(PrimaryValue::Line(p)) => Some(
            block
                .annotation
                .as_ref()
                .map_or_else(|| p.text.clone(), |a| a.text.clone()),
        ),
        Some(PrimaryValue::Unexpected(_)) | None => None,
    };
    let binding = line.binding.map(|b| match b {
        Bound::Own => tessera_conformance::Binding::SelfBinding,
        Bound::Heading => tessera_conformance::Binding::Heading,
        Bound::FollowingBlock => tessera_conformance::Binding::FollowingBlock,
        Bound::Unbound => tessera_conformance::Binding::Unbound,
    });
    Directive {
        name: line.name.clone(),
        form,
        attributes: attributes(line),
        primary,
        title: title(source, block, line),
        binding,
        children,
    }
}

/// A paragraph, or an image when the paragraph is one image alone.
fn paragraph(source: &str, block: &ResolvedBlock, p: &tessera_syntax::Paragraph) -> Node {
    if let [only] = p.inlines.as_slice()
        && let InlineKind::Image(image) = &only.kind
    {
        return Node::Image {
            src: image.destination.clone(),
            alt: Some(text(source, image.alt, &block.substitutions)),
            title: image.title.clone(),
            attributes: super::inline::attributes(image),
        };
    }
    Node::Paragraph {
        text: Some(text(source, block.span, &block.substitutions)),
    }
}
