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
//! | `avail` | `@available` specs, the `available` frontmatter key, and table rows' `available` |
//! | `frontmatter` | Content type, fields, reserved keys, `variant` |
//! | `acknowledge` | What `@intended` and `intended` frontmatter name (SPEC §4.9) |
//! | `content` | A page's description and review date, code blocks' languages |
//! | `comment` | HTML comments, which a memo would keep out of the published page |
//! | `memo` | Container memos that say nothing |
//! | `refs` | `@include` targets, `@snippet` addresses, link destinations, image sources and alt text |
//! | `sources` | `ascribe.lock`, and the copies of sources in other repositories |
//! | `template` | What a page still has of its type's template, never filled in |

mod acknowledge;
mod attrs;
mod avail;
mod comment;
mod content;
mod frontmatter;
mod memo;
mod refs;
mod sources;
pub(crate) mod template;
pub(crate) mod text;

use ascribe_core::info_string::InfoString;
use ascribe_core::{
    Applicability, DirectiveSchema, FileId, Fix, Issue, Location, Span, TextEdit, diagnostics,
};
use ascribe_model::{ContentModel, ContentType};
use ascribe_syntax::{
    Block, BlockKind, DirectiveLine, Heading, Inline, InlineKind, ParseOptions, PrimaryValue, parse,
};

use crate::{Diagnostic, Project, SourceFile};

pub(crate) use sources::{check_examples, check_sources};

/// State shared by the checks of one file.
struct Ctx<'a> {
    project: &'a Project,
    model: &'a ContentModel,
    file: &'a SourceFile,
    id: FileId,
    schemas: Vec<DirectiveSchema>,
    /// The file's link reference definitions, whose destinations hold the
    /// phrases of the reference forms that use them.
    definitions: Vec<ascribe_syntax::LinkDefinition>,
    /// The text of the file's first paragraph, for a prompt about its
    /// description.
    first_paragraph: Option<String>,
    /// The page's content type, once the frontmatter is checked: `None` for
    /// a fragment, and for a page whose type is missing or unknown.
    page_type: Option<&'a ContentType>,
    /// Each mark in the file that says something, with where it applies:
    /// the whole file for the page's `available` key.
    mark_scopes: Vec<(Span, Span)>,
    /// Whether a code block's info string looks marked as an example of an
    /// operation's body, so the file's examples are checked once its blocks
    /// are walked.
    marked_examples: bool,
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

    fn page_type(&self) -> Option<&ContentType> {
        self.page_type
    }
}

/// Checks one source file at file level (SPEC §8.1): the parser's issues,
/// then every check that needs the content model or the file system.
/// Diagnostics come back in source order.
pub fn check_file(project: &Project, file: &SourceFile) -> Vec<Diagnostic> {
    if let Some(failure) = &file.unreadable {
        let at = Location::new(file.id, Span::empty(0));
        let issue = Issue::new(diagnostics::SOURCE_UNREADABLE, at);
        let issue = if failure.not_utf8 {
            issue.with_variant("encoding")
        } else {
            issue.with_arg("reason", failure.reason.clone())
        };
        return vec![Diagnostic::from_issue(&issue)];
    }
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
        definitions: doc.definitions.clone(),
        first_paragraph: content::first_paragraph(&file.text, &doc.blocks),
        page_type: None,
        mark_scopes: Vec::new(),
        marked_examples: false,
        issues: doc.issues.clone(),
    };
    cx.check_frontmatter(doc.frontmatter.as_ref());
    cx.collect_mark_scopes(&doc.blocks);
    cx.blocks(&doc.blocks);
    if cx.marked_examples {
        cx.check_marked_examples();
    }
    cx.check_template_body(&doc.blocks);
    cx.parser_fixes();
    // A stable sort keeps the parser's issues before a check's at one place.
    let mut issues = cx.issues;
    issues.sort_by_key(|i| (i.location.span.start(), i.location.span.end()));
    issues.iter().map(Diagnostic::from_issue).collect()
}

impl Ctx<'_> {
    /// Fixes for the parser's issues whose replacement is certain: a
    /// misspelled directive (the suggested one) and text that doesn't belong
    /// on a directive line (remove it).
    fn parser_fixes(&mut self) {
        let text = self.file.text.clone();
        let id = self.id;
        for issue in &mut self.issues {
            let span = issue.location.span;
            let fix = if issue.slug == diagnostics::DIRECTIVE_UNKNOWN
                && issue.variant == Some("suggestion")
            {
                let Some(suggestion) = issue.arg("suggestion").map(str::to_owned) else {
                    continue;
                };
                // `@warning:` becomes `@note {type=warning}:`, colon included.
                let mut end = span.end();
                if suggestion.ends_with(':') {
                    let rest = &text[end..];
                    let after = rest.trim_start_matches([' ', '\t']);
                    if after.starts_with(':') {
                        end += rest.len() - after.len() + 1;
                    }
                }
                Fix {
                    title: format!("Replace with `{suggestion}`"),
                    file: id,
                    edits: vec![TextEdit::replace(Span::new(span.start(), end), suggestion)],
                    // The nearest name, which may not be the one meant.
                    applicability: Applicability::Unsafe,
                }
            } else if issue.slug == diagnostics::DIRECTIVE_EXTRA_TEXT {
                let before = &text[..span.start()];
                let start = before.trim_end_matches([' ', '\t']).len();
                Fix {
                    title: format!("Remove `{}`", issue.arg("extra").unwrap_or_default()),
                    file: id,
                    edits: vec![TextEdit::delete(Span::new(start, span.end()))],
                    // The text may be meant for the primary or the content.
                    applicability: Applicability::Unsafe,
                }
            } else {
                continue;
            };
            issue.fixes.push(fix);
        }
    }

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
                    for row in &t.rows {
                        if let Some(block) = &row.attributes {
                            self.check_row(block);
                        }
                        for cell in &row.cells {
                            self.inlines(&cell.inlines);
                        }
                    }
                }
                BlockKind::Directive(d) => self.directive(d),
                BlockKind::Container(c) => {
                    self.directive(&c.opener);
                    self.check_memo(block, c);
                    self.blocks(&c.children);
                }
                BlockKind::Group(g) => {
                    for arm in &g.arms {
                        self.directive(&arm.opener);
                        self.blocks(&arm.children);
                    }
                }
                BlockKind::CodeBlock(code) => {
                    self.check_code_language(block, code);
                    let info = InfoString::parse(&code.info);
                    self.marked_examples |=
                        info.get("request").is_some() || info.get("response").is_some();
                }
                BlockKind::HtmlBlock(html) => self.check_html_block(blocks, i, html),
                BlockKind::ThematicBreak | BlockKind::End(_) | BlockKind::Title(_) => {}
            }
        }
    }

    fn inlines(&mut self, inlines: &[Inline]) {
        for inline in inlines {
            match &inline.kind {
                InlineKind::Phrase(p) => {
                    // Every inline position where a
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
                    } else {
                        self.check_double_braces(p);
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
                | InlineKind::HardBreak => {}
                InlineKind::Html(_) => self.check_html_inline(inline.span),
            }
        }
    }

    /// A heading with no `@id` (SPEC §5.5) whose slug changes whenever a
    /// phrase's value does, or whose slug is empty.
    fn heading(&mut self, siblings: &[Block], index: usize, span: Span, h: &Heading) {
        let has_id = siblings[index + 1..]
            .iter()
            .take_while(|b| !matches!(b.kind, BlockKind::Heading(_)))
            .any(|b| {
                matches!(
                    &b.kind,
                    BlockKind::Directive(d)
                        if d.name == "id" && d.binding == Some(ascribe_syntax::Bound::Heading)
                )
            });
        if has_id {
            return;
        }
        if self.has_declared_phrase(&h.inlines) {
            let issue = Issue::new(diagnostics::HEADING_PHRASE_WITHOUT_ID, self.location(span));
            self.report(issue);
        }
        // The slug the text alone gives, as the source index computes it.
        let text = ascribe_resolve::heading_text(&h.inlines, self.model);
        let slugger = ascribe_resolve::slug::slugger_by_name(&self.model.consumer.slugger)
            .unwrap_or_else(ascribe_resolve::slug::default_slugger);
        if slugger.new_scope().slug(&text).is_empty() {
            let issue = Issue::new(diagnostics::HEADING_EMPTY_SLUG, self.location(span));
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
            ("id", Some(PrimaryValue::Identifier(p))) => self.check_name(&p.text, p.span, None),
            ("include", Some(PrimaryValue::Identifier(p))) => {
                self.check_include(&p.text, p.span);
                self.check_include_heading(d, &p.text);
            }
            ("available", Some(PrimaryValue::Line(p))) => {
                let removal = memo::lines_removal(self.source(), d.span.start(), d.span.end());
                let removal = self.nested_removal(d.span, d.span.start(), removal);
                self.check_availability_text(&p.text, p.span.start(), p.span, true, removal);
            }
            ("snippet", Some(PrimaryValue::Identifier(_))) => self.check_snippet(d),
            ("operation", Some(PrimaryValue::Identifier(p))) => {
                self.check_operation(&p.text, p.span)
            }
            ("intended", _) => self.check_intended_directive(d),
            _ => {}
        }
    }

    /// `{heading=false}` only applies to an include of a section (SPEC §4.2).
    fn check_include_heading(&mut self, d: &DirectiveLine, primary: &str) {
        let Some(attribute) = d.attributes.as_ref().and_then(|a| a.get("heading")) else {
            return;
        };
        let off = attribute.value.as_ref().and_then(|v| v.as_text()) == Some("false");
        if off
            && ascribe_resolve::include_target(primary, &self.file.path)
                .section
                .is_none()
        {
            let issue = Issue::new(
                diagnostics::INCLUDE_HEADING_WITHOUT_ID,
                self.location(attribute.span),
            );
            self.report(issue);
        }
    }

    /// A declared phrase directly between two more braces,
    /// `{{key}}`, is `{`, the phrase, and `}`: almost always a substitution
    /// from another tool that wasn't converted. An escaped outer brace
    /// (`\{{key}}`) is meant, and isn't reported.
    fn check_double_braces(&mut self, p: &ascribe_syntax::Phrase) {
        let text = self.file.text.as_bytes();
        let (start, end) = (p.span.start(), p.span.end());
        let opened = start >= 1
            && text.get(start - 1) == Some(&b'{')
            && !(start >= 2 && text.get(start - 2) == Some(&b'\\'));
        let closed = text.get(end) == Some(&b'}');
        if !(opened && closed) {
            return;
        }
        let whole = Span::new(start - 1, end + 1);
        let issue = Issue::new(diagnostics::PHRASE_DOUBLE_BRACES, self.location(whole))
            .with_arg("key", format!("{{{}}}", p.key))
            .with_fix(Fix {
                title: "Remove the outer braces".to_owned(),
                file: self.id,
                edits: vec![
                    TextEdit::delete(Span::new(start - 1, start)),
                    TextEdit::delete(Span::new(end, end + 1)),
                ],
                // The page shows the braces now, and one may be meant.
                applicability: Applicability::Unsafe,
            });
        self.report(issue);
    }

    /// An `@id`, or with the variant `permalink` a page's `permalink`
    /// (SPEC §2.1): letters, digits, hyphens, underscores, and periods
    /// (SPEC §4.1).
    fn check_name(&mut self, id: &str, span: Span, variant: Option<&'static str>) {
        if !ascribe_core::is_id(id) {
            let mut issue = Issue::new(diagnostics::ID_INVALID, self.location(span))
                .with_arg("id", id.to_owned());
            if let Some(variant) = variant {
                issue = issue.with_variant(variant);
            }
            self.report(issue);
        }
    }
}
