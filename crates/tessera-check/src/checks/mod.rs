//! The file-level checks.
//!
//! [`check_file`] parses a file with the project's directive schemas, turns
//! the parser's issues into diagnostics, and walks the tree for the checks
//! that need the content model or the file system. Each check lives in its
//! own module:
//!
//! | Module | Checks |
//! |---|---|
//! | `attrs` | Attribute keys and types, `@variant` dimensions, required attributes |
//! | `avail` | `@available` specs and the `available` frontmatter key |
//! | `frontmatter` | Content type, fields, reserved keys, `variant` |
//! | `refs` | `@include` targets, link destinations, image sources and alt text |

mod attrs;
mod avail;
mod frontmatter;
mod refs;
mod text;

use tessera_core::{DirectiveSchema, FileId, Issue, Location, Span, diagnostics};
use tessera_model::ContentModel;
use tessera_syntax::{
    Block, BlockKind, DirectiveLine, Heading, Inline, InlineKind, ParseOptions, PrimaryValue, parse,
};

use crate::{Diagnostic, Project, SourceFile};

/// State shared by the checks of one file.
struct Ctx<'a> {
    project: &'a Project,
    model: &'a ContentModel,
    file: &'a SourceFile,
    id: FileId,
    schemas: Vec<DirectiveSchema>,
    issues: Vec<Issue>,
}

impl Ctx<'_> {
    fn report(&mut self, issue: Issue) {
        self.issues.push(issue);
    }

    fn source(&self) -> &str {
        &self.file.text
    }

    fn schema(&self, name: &str) -> Option<&DirectiveSchema> {
        self.schemas.iter().find(|s| s.name == name)
    }

    fn location(&self, span: Span) -> Location {
        Location::new(self.id, span)
    }
}

/// Checks one source file at file level (SPEC §8.1): the parser's issues,
/// then every check that needs the content model or the file system.
/// Diagnostics come back in source order.
pub fn check_file(project: &Project, file: &SourceFile) -> Vec<Diagnostic> {
    let model = project.model();
    let options = ParseOptions::new(model.directive_schemas())
        .with_file(file.id)
        .with_note_types(model.notes.iter().map(|n| n.name.clone()).collect());
    let doc = parse(&file.text, &options);
    let mut cx = Ctx {
        project,
        model,
        file,
        id: file.id,
        schemas: model.directive_schemas(),
        issues: doc.issues.clone(),
    };
    cx.check_frontmatter(doc.frontmatter.as_ref());
    cx.blocks(&doc.blocks);
    // A stable sort keeps the parser's issues before a check's at one place.
    let mut issues = cx.issues;
    issues.sort_by_key(|i| (i.location.span.start(), i.location.span.end()));
    issues.iter().map(Diagnostic::from_issue).collect()
}

impl Ctx<'_> {
    fn blocks(&mut self, blocks: &[Block]) {
        for (i, block) in blocks.iter().enumerate() {
            match &block.kind {
                BlockKind::Heading(h) => {
                    self.inlines(&h.inlines);
                    self.heading(blocks, i, block.span, h);
                }
                BlockKind::Paragraph(p) => self.inlines(&p.inlines),
                BlockKind::BlockQuote(q) => self.blocks(&q.children),
                BlockKind::List(l) => {
                    for item in &l.items {
                        self.blocks(&item.children);
                    }
                }
                BlockKind::Table(t) => {
                    for cell in t.rows.iter().flat_map(|r| &r.cells) {
                        self.inlines(&cell.inlines);
                    }
                }
                BlockKind::Directive(d) => self.directive(d),
                BlockKind::Container(c) => {
                    self.directive(&c.opener);
                    self.blocks(&c.children);
                }
                BlockKind::Group(g) => {
                    for arm in &g.arms {
                        self.directive(&arm.opener);
                        self.blocks(&arm.children);
                    }
                }
                BlockKind::CodeBlock(_)
                | BlockKind::HtmlBlock(_)
                | BlockKind::ThematicBreak
                | BlockKind::End(_)
                | BlockKind::Title(_) => {}
            }
        }
    }

    fn inlines(&mut self, inlines: &[Inline]) {
        for inline in inlines {
            match &inline.kind {
                InlineKind::Phrase(p) => {
                    // SPEC-QUESTION(Q56): every inline position where a
                    // candidate is recorded, and no destination, fence, or
                    // frontmatter.
                    // SPEC §5.1: a `{key}` in prose whose key isn't declared
                    // is literal text today and would silently become a
                    // phrase if the key were declared later.
                    if !self.model.has_phrase(&p.key) {
                        let issue =
                            Issue::new(diagnostics::PHRASE_UNDECLARED, self.location(p.span))
                                .with_arg("key", format!("{{{}}}", p.key));
                        self.report(issue);
                    }
                }
                InlineKind::Emphasis(children) | InlineKind::Strong(children) => {
                    self.inlines(children);
                }
                InlineKind::Link(link) => {
                    self.check_link(inline.span, link);
                    self.inlines(&link.children);
                }
                InlineKind::Image(image) => {
                    self.check_image(inline.span, image);
                    self.inlines(&image.children);
                }
                InlineKind::Text(_)
                | InlineKind::Code(_)
                | InlineKind::SoftBreak
                | InlineKind::HardBreak
                | InlineKind::Html(_) => {}
            }
        }
    }

    /// A heading with a phrase and no `@id` (SPEC §5.5): its slug changes
    /// whenever the phrase's value does.
    fn heading(&mut self, siblings: &[Block], index: usize, span: Span, h: &Heading) {
        if !self.has_declared_phrase(&h.inlines) {
            return;
        }
        let has_id = siblings[index + 1..]
            .iter()
            .take_while(|b| !matches!(b.kind, BlockKind::Heading(_)))
            .any(|b| {
                matches!(
                    &b.kind,
                    BlockKind::Directive(d)
                        if d.name == "id" && d.binding == Some(tessera_syntax::Bound::Heading)
                )
            });
        if !has_id {
            let issue = Issue::new(diagnostics::HEADING_PHRASE_WITHOUT_ID, self.location(span));
            self.report(issue);
        }
    }

    fn has_declared_phrase(&self, inlines: &[Inline]) -> bool {
        inlines.iter().any(|i| match &i.kind {
            InlineKind::Phrase(p) => self.model.has_phrase(&p.key),
            InlineKind::Emphasis(c) | InlineKind::Strong(c) => self.has_declared_phrase(c),
            InlineKind::Link(l) => self.has_declared_phrase(&l.children),
            InlineKind::Image(image) => self.has_declared_phrase(&image.children),
            _ => false,
        })
    }

    /// Everything about one directive line (line form, container opener, or
    /// arm opener).
    fn directive(&mut self, d: &DirectiveLine) {
        if let Some(title) = &d.title {
            self.inlines(&title.inlines);
        }
        if let Some(PrimaryValue::Text(text)) = &d.primary {
            self.inlines(&text.inlines);
        }
        let Some(schema) = self.schema(&d.name).cloned() else {
            return;
        };
        self.check_directive_attributes(
            &schema,
            d.attributes.as_ref(),
            d.attributes_closed,
            d.name_span,
        );
        // An unclosed attribute block hides the rest of the line (SPEC §3.3).
        if !d.attributes_closed {
            return;
        }
        match (d.name.as_str(), &d.primary) {
            ("id", Some(PrimaryValue::Identifier(p))) => self.check_id(&p.text, p.span),
            ("include", Some(PrimaryValue::Identifier(p))) => self.check_include(&p.text, p.span),
            ("available", Some(PrimaryValue::Line(p))) => {
                self.check_availability_text(&p.text, p.span.start(), p.span, true);
            }
            _ => {}
        }
    }

    /// `@id` values: letters, digits, and hyphens (SPEC §4.1).
    fn check_id(&mut self, id: &str, span: Span) {
        let valid = !id.is_empty() && id.chars().all(|c| c.is_ascii_alphanumeric() || c == '-');
        if !valid {
            let issue = Issue::new(diagnostics::ID_INVALID, self.location(span))
                .with_arg("id", id.to_owned());
            self.report(issue);
        }
    }
}
