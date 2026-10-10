//! The per-file index: what's true of one source file, independent of any
//! build and of every other file.
//!
//! [`index_file`] is a pure function of the file's id, path, and text, the
//! content model, and the slugger. It never touches the file system, and it
//! doesn't know which other files exist, so the result can be cached and
//! reused until the file's text or the model changes.
//!
//! What it records:
//!
//! - the parsed tree and the frontmatter, and the file's **title**;
//! - the **headings**, with their sections and **source ids** (SPEC §5.5);
//! - the **includes**, each with the path it names, resolved (SPEC §4.2);
//! - the **references**: links and images, each resolved from this file,
//!   and so each **asset reference** among them;
//! - every **phrase candidate**, those in the frontmatter fields that take
//!   phrases included, and every **`@available`** directive;
//! - every **`@snippet`**, with its address read (SPEC §4.8).

mod frontmatter;
mod headings;
mod refs;
pub(crate) mod walk;

use std::sync::Arc;

use ascribe_core::availability::{AvailabilityError, AvailabilitySpec, parse_availability};
use ascribe_core::{AttributeValue, FileId, RelPath, Slugger, Span};
use ascribe_model::ContentModel;
use ascribe_syntax::{
    Block, BlockKind, Bound, DirectiveLine, InlineKind, ParseOptions, ParsedDocument, Phrase,
    PrimaryValue, TableRow, parse,
};

use crate::snippet::SnippetUse;

pub use headings::{ExplicitId, Heading, plain_text as heading_text};
pub use refs::{Local, RefKind, Reference, Target};
pub(crate) use refs::{substitute, target_of};

/// Whether a file is published (SPEC §2.2).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum FileKind {
    /// Published on its own.
    Page,
    /// Exists only to be included: a path segment begins with `_`, or a
    /// fragment pattern in the content model matches.
    Fragment,
}

/// Everything the index knows about one source file.
#[derive(Clone, Debug)]
pub struct FileIndex {
    /// The file's id in its project.
    pub file: FileId,
    /// The file's content path.
    pub path: RelPath,
    /// Page or fragment.
    pub kind: FileKind,
    /// The file's text. Every span in the index and the tree points into it.
    pub source: Arc<str>,
    /// The parsed file, with its parser issues.
    pub document: Arc<ParsedDocument>,
    /// The frontmatter, when the file has one that is valid YAML. A file
    /// whose frontmatter doesn't parse has `None`; the file-level checks
    /// report it.
    pub frontmatter: Option<serde_yaml_ng::Value>,
    /// The file's title: its frontmatter `title`, when that is a string.
    /// A link with no text takes it (SPEC §5.2).
    pub title: Option<String>,
    /// Every heading, in document order.
    pub headings: Vec<Heading>,
    /// Every `@include`, in document order.
    pub includes: Vec<Include>,
    /// Every link and image, in document order. An image is an **asset
    /// reference**; so is a link whose target isn't a source file.
    pub references: Vec<Reference>,
    /// Every phrase candidate, in document order.
    pub phrases: Vec<PhraseUse>,
    /// Every `@available` directive, in document order.
    pub availability: Vec<AvailabilityMarker>,
    /// Every `@snippet` directive, in document order.
    pub snippets: Vec<SnippetUse>,
}

impl FileIndex {
    /// The heading with this source id: the first, if a file repeats one
    /// (which page-level checks report). An empty id names nothing.
    pub fn heading_by_id(&self, id: &str) -> Option<&Heading> {
        if id.is_empty() {
            return None;
        }
        self.headings.iter().find(|h| h.source_id == id)
    }

    /// The include whose directive has this span.
    pub(crate) fn include_at(&self, span: Span) -> Option<&Include> {
        self.includes.iter().find(|i| i.span == span)
    }

    /// The `@snippet` whose directive has this span.
    pub fn snippet_at(&self, span: Span) -> Option<&SnippetUse> {
        self.snippets.iter().find(|s| s.span == span)
    }
}

/// An `@include` directive (SPEC §4.2).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Include {
    /// The directive line, from `@` to the end of the primary.
    pub span: Span,
    /// The primary: the path, and the `#id` if it has one. `None` when the
    /// directive has no primary (which the parser reports).
    pub primary: Option<Span>,
    /// The path as written, without the `#id`.
    pub written: String,
    /// The file it names, as a content path, resolved from this file (or from
    /// the content root when the path starts with `/`). It starts with `..`
    /// when the path leads outside the content root. `None` without a
    /// primary.
    pub target: Option<RelPath>,
    /// The id after `#`: the source id of the heading whose section is
    /// included. `None` includes the whole file; so does an empty id
    /// (`file.md#`).
    pub section: Option<String>,
    /// Whether the included section keeps its own heading: `false` only for
    /// `{heading=false}` (SPEC §4.2). Any other value keeps it, and the file
    /// checks report a value that isn't a boolean.
    pub heading: bool,
}

/// Where a phrase candidate is.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PhrasePlace {
    /// In text outside headings: prose, link text, alt text, titles, table
    /// cells, and text primaries.
    Text,
    /// In a heading's text.
    Heading,
    /// In an inline link or image destination, or an autolink's.
    Destination,
    /// In a fenced code block that opts in with `phrases=true`.
    Code,
    /// In a frontmatter field that takes phrases. Only declared candidates
    /// are listed here: in the frontmatter, the rest is literal text.
    Frontmatter,
}

/// A phrase candidate: `{key}` (SPEC §5.1), declared or not.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PhraseUse {
    /// The candidate.
    pub phrase: Phrase,
    /// Whether the content model declares the key. Only a declared key is a
    /// phrase; otherwise the text is literal.
    pub declared: bool,
    /// Where it is.
    pub place: PhrasePlace,
}

/// An `@available` directive, or the `available` attribute of a table row
/// (SPEC §4.4).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AvailabilityMarker {
    /// The directive line, or the row's attribute block.
    pub span: Span,
    /// The primary's text and span, if the directive has one; for a row, the
    /// attribute's value, without its quotes.
    pub primary: Option<(String, Span)>,
    /// What the directive binds, as the structure pass decided. `None` for a
    /// row.
    pub binding: Option<Bound>,
    /// The primary read as an availability spec. A bare name may be a feature
    /// key instead; build resolution decides, from the model.
    pub spec: Option<Result<AvailabilitySpec, AvailabilityError>>,
}

/// Indexes one file.
///
/// `path` is the file's content path, `source` its text. The result depends on
/// nothing else: not on the file system, and not on other files.
pub fn index_file(
    file: FileId,
    path: &RelPath,
    source: &str,
    model: &ContentModel,
    slugger: &dyn Slugger,
) -> FileIndex {
    let document = parse_source(file, source, model);
    index_parsed(
        file,
        path,
        Arc::from(source),
        Arc::new(document),
        model,
        slugger,
    )
}

/// Parses one file's text the way [`index_file`] does. The result depends on
/// the text, the file's id (it's in every location), and the model's
/// directive keywords and note types alone, which is what lets the
/// incremental update reuse it ([`crate::incremental`]).
pub(crate) fn parse_source(file: FileId, source: &str, model: &ContentModel) -> ParsedDocument {
    let options = ParseOptions::new(model.directive_schemas())
        .with_file(file)
        .with_note_types(model.notes.iter().map(|n| n.name.clone()).collect());
    parse(source, &options)
}

/// Indexes a file that is already parsed ([`parse_source`] of the same
/// `source`, `file`, and model): the second half of [`index_file`].
pub(crate) fn index_parsed(
    file: FileId,
    path: &RelPath,
    source: Arc<str>,
    document: Arc<ParsedDocument>,
    model: &ContentModel,
    slugger: &dyn Slugger,
) -> FileIndex {
    let text: &str = &source;
    let frontmatter = document.frontmatter.as_ref().and_then(|fm| {
        let yaml = text.get(fm.content.range())?;
        // An empty frontmatter block reads as no keys.
        match serde_yaml_ng::from_str::<serde_yaml_ng::Value>(yaml) {
            Ok(serde_yaml_ng::Value::Null) => {
                Some(serde_yaml_ng::Value::Mapping(Default::default()))
            }
            Ok(value) => Some(value),
            Err(_) => None,
        }
    });
    let title = frontmatter
        .as_ref()
        .and_then(|v| v.get("title"))
        .and_then(|t| t.as_str())
        .map(str::to_owned);

    let headings = headings::collect(&document.blocks, model, slugger);
    let mut includes = Vec::new();
    let mut references = Vec::new();
    let mut phrases = Vec::new();
    let mut availability = Vec::new();
    let mut snippets = Vec::new();

    let mut visit = |block: &Block| {
        for inlines in walk::own_inlines(block) {
            refs::collect_references(
                inlines,
                text,
                path,
                model,
                &document.definitions,
                &mut references,
            );
            collect_phrases(inlines, block, model, &mut phrases);
        }
        match &block.kind {
            BlockKind::CodeBlock(code) => {
                for phrase in code.phrases.iter().flatten() {
                    phrases.push(phrase_use(phrase, PhrasePlace::Code, model));
                }
            }
            BlockKind::Directive(line) if line.name == "include" => {
                includes.push(include_of(line, path));
            }
            BlockKind::Directive(line) if line.name == "available" => {
                availability.push(availability_of(line, text));
            }
            BlockKind::Directive(line) if line.name == "snippet" => {
                snippets.push(SnippetUse::of(line));
            }
            BlockKind::Table(table) => {
                availability.extend(table.rows.iter().filter_map(row_availability));
            }
            _ => {}
        }
    };
    walk::walk_blocks(&document.blocks, &mut visit);
    if let Some(fm) = &document.frontmatter {
        phrases.extend(frontmatter::phrases(text, fm.content, path.as_str(), model));
    }
    // A definition's destination holds phrases too, once, wherever the
    // links that use it are.
    for definition in &document.definitions {
        for phrase in &definition.destination_phrases {
            phrases.push(phrase_use(phrase, PhrasePlace::Destination, model));
        }
    }
    // Destinations are found through the inlines above; sort the phrases they
    // add into document order.
    phrases.sort_by_key(|p: &PhraseUse| p.phrase.span);

    let kind = if model.is_fragment(path.as_str()) {
        FileKind::Fragment
    } else {
        FileKind::Page
    };
    FileIndex {
        file,
        path: path.clone(),
        kind,
        source,
        document,
        frontmatter,
        title,
        headings,
        includes,
        references,
        phrases,
        availability,
        snippets,
    }
}

fn phrase_use(phrase: &Phrase, place: PhrasePlace, model: &ContentModel) -> PhraseUse {
    PhraseUse {
        phrase: phrase.clone(),
        declared: model.has_phrase(&phrase.key),
        place,
    }
}

/// Phrase candidates in a block's own inlines: text ones, and those in
/// destinations.
fn collect_phrases(
    inlines: &[ascribe_syntax::Inline],
    block: &Block,
    model: &ContentModel,
    out: &mut Vec<PhraseUse>,
) {
    let text_place = if matches!(block.kind, BlockKind::Heading(_)) {
        PhrasePlace::Heading
    } else {
        PhrasePlace::Text
    };
    walk::walk_inlines(inlines, &mut |inline| match &inline.kind {
        InlineKind::Phrase(p) => out.push(phrase_use(p, text_place, model)),
        InlineKind::Link(l) => {
            for p in &l.destination_phrases {
                out.push(phrase_use(p, PhrasePlace::Destination, model));
            }
        }
        InlineKind::Image(i) => {
            for p in &i.destination_phrases {
                out.push(phrase_use(p, PhrasePlace::Destination, model));
            }
        }
        _ => {}
    });
}

fn include_of(line: &DirectiveLine, written_in: &RelPath) -> Include {
    let heading = line
        .attributes
        .as_ref()
        .and_then(|a| a.get("heading"))
        .and_then(|a| a.value.as_ref())
        .and_then(|v| v.as_text())
        != Some("false");
    let Some(PrimaryValue::Identifier(primary)) = &line.primary else {
        return Include {
            span: line.span,
            primary: line.primary.as_ref().map(PrimaryValue::span),
            written: String::new(),
            target: None,
            section: None,
            heading,
        };
    };
    let crate::references::IncludeTarget {
        written,
        target,
        section,
    } = crate::references::include_target(&primary.text, written_in);
    Include {
        span: line.span,
        primary: Some(primary.span),
        written,
        target,
        section,
        heading,
    }
}

/// The `available` attribute of a table row, when it has one whose value is
/// text (a value set is the wrong type, which the checks report).
fn row_availability(row: &TableRow) -> Option<AvailabilityMarker> {
    let block = row.attributes.as_ref()?;
    let value = block.get("available")?.value.as_ref()?;
    let (text, span) = attribute_text(value)?;
    let spec = Some(parse_availability(&text, span.start()));
    Some(AvailabilityMarker {
        span: block.span,
        primary: Some((text, span)),
        binding: None,
        spec,
    })
}

/// An attribute value's text and the span it's written in: a quoted value's
/// span is inside its quotes, unless escapes make the text differ from the
/// source, when it's the whole value.
fn attribute_text(value: &AttributeValue) -> Option<(String, Span)> {
    match value {
        AttributeValue::Token(t) => Some((t.text.clone(), t.span)),
        AttributeValue::Quoted { text, span } => {
            let inner = Span::new(span.start() + 1, span.end().saturating_sub(1));
            let span = if inner.len() == text.len() {
                inner
            } else {
                *span
            };
            Some((text.clone(), span))
        }
        AttributeValue::Set { .. } => None,
    }
}

fn availability_of(line: &DirectiveLine, source: &str) -> AvailabilityMarker {
    let primary = match &line.primary {
        Some(PrimaryValue::Line(p)) => Some((p.text.clone(), p.span)),
        Some(PrimaryValue::Text(p)) => source
            .get(p.span.range())
            .map(|text| (text.to_owned(), p.span)),
        _ => None,
    };
    let spec = primary
        .as_ref()
        .map(|(text, span)| parse_availability(text, span.start()));
    AvailabilityMarker {
        span: line.span,
        primary,
        binding: line.binding,
        spec,
    }
}
