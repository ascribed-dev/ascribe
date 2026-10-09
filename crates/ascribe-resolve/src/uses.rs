//! Where things are used: the links and includes that name a page, a
//! fragment, or a heading, and the places pages use the content model's
//! entries. Find All References and the editor's inventory both ask this
//! search, so a count is always the length of the list.
//!
//! Everything comes from the source index, so it holds for every build:
//!
//! - a **file** (a page or a fragment) is used by every link from another
//!   file that the project resolved to it ([`Project::resolutions`]) and
//!   every `@include` of it;
//! - a **heading** by the links whose `#id` names it, on its own page or on
//!   a page that includes it (SPEC §4.2), and the includes of its section;
//! - a **phrase** by each declared `{key}`, wherever it's written;
//! - a **feature** by each availability spec that is its bare key: an
//!   `@available`, a table row's `available`, or a page's frontmatter;
//! - a **glossary term** by each occurrence of its text or an alias in prose,
//!   matched as the build matches it (whole words, the term's case rule),
//!   whether it links them first or every time. A term with
//!   `match = "marked"` has none: its occurrences are ordinary words until
//!   an author links one, and that link is a use of the page it names;
//! - a **dimension** by each `@variant` attribute and frontmatter `variant`
//!   key that names it, and each availability spec entry that names it or
//!   one of its values;
//! - a **note type** by each `@note` of that type (`note` when it gives none);
//! - a **widget** by each directive line, container, or arm that opens it.

use std::collections::HashMap;

use ascribe_core::schema::{Attributes, Builtin, DefaultValue};
use ascribe_core::{RelPath, Span};
use ascribe_model::{ContentModel, GlossaryMatch};
use ascribe_syntax::{Block, BlockKind, DirectiveLine, Inline, InlineKind};

use crate::build::{Terms, prose_lists};
use crate::index::walk::walk_blocks;
use crate::index::{FileIndex, RefKind};
use crate::project::{Project, Resolution};

/// Something a page can use.
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Usable {
    /// A page or a fragment, by content path.
    File(RelPath),
    /// A heading, by the file it's written in and its source id.
    Heading {
        /// The file the heading is written in.
        file: RelPath,
        /// Its source id.
        id: String,
    },
    /// A phrase, by key.
    Phrase(String),
    /// A feature, by key.
    Feature(String),
    /// A glossary term, by id.
    Term(String),
    /// A dimension, by name.
    Dimension(String),
    /// A note type.
    Note(String),
    /// A project widget, by name.
    Widget(String),
}

/// How a place uses what it names.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum UseKind {
    /// A link.
    Link,
    /// An `@include`.
    Include,
    /// A `{key}`.
    Phrase,
    /// An availability spec.
    Availability,
    /// An occurrence of a glossary term in prose.
    Term,
    /// A `@variant` attribute, or a frontmatter `variant` key.
    Variant,
    /// A `@note`.
    Note,
    /// A widget's directive.
    Widget,
}

/// One place something is used.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Use {
    /// The file it's written in.
    pub file: RelPath,
    /// Where: the link, the directive line, the `{key}`, the spec, the
    /// attribute, or the occurrence.
    pub span: Span,
    /// How.
    pub kind: UseKind,
}

impl Project {
    /// Every place `target` is used, file by file in content path order and
    /// in document order within a file.
    pub fn uses(&self, target: &Usable) -> Vec<Use> {
        let mut out = Vec::new();
        visit(self, Some(target), None, &mut |used, place| {
            if used == target {
                out.push(place);
            }
        });
        out
    }

    /// How many places use each thing that is used at all: for every
    /// `target`, the length of [`Project::uses`].
    pub fn use_counts(&self) -> HashMap<Usable, usize> {
        let mut counts = HashMap::new();
        visit(self, None, None, &mut |used, _| {
            if let Some(count) = counts.get_mut(used) {
                *count += 1;
            } else {
                counts.insert(used.clone(), 1);
            }
        });
        counts
    }

    /// Every use written in one file, with what it uses, in no set order. A
    /// link that names a heading is listed twice: as a use of the heading,
    /// and (unless it's on the heading's own page) of the page.
    pub fn uses_in(&self, path: &RelPath) -> Vec<(Usable, Use)> {
        let mut out = Vec::new();
        visit(self, None, Some(path), &mut |used, place| {
            out.push((used.clone(), place));
        });
        out
    }
}

/// Which kinds of use a search looks at: one kind for one target, every kind
/// for counting.
#[derive(Clone, Copy)]
struct Wants {
    files: bool,
    phrases: bool,
    availability: bool,
    terms: bool,
    directives: bool,
}

impl Wants {
    fn of(target: Option<&Usable>) -> Wants {
        let only = |files, phrases, availability, terms, directives| Wants {
            files,
            phrases,
            availability,
            terms,
            directives,
        };
        match target {
            None => only(true, true, true, true, true),
            Some(Usable::File(_) | Usable::Heading { .. }) => {
                only(true, false, false, false, false)
            }
            Some(Usable::Phrase(_)) => only(false, true, false, false, false),
            Some(Usable::Feature(_)) => only(false, false, true, false, false),
            Some(Usable::Term(_)) => only(false, false, false, true, false),
            Some(Usable::Dimension(_)) => only(false, false, true, false, true),
            Some(Usable::Note(_) | Usable::Widget(_)) => only(false, false, false, false, true),
        }
    }
}

/// Calls `f` with every use the search for `target` (every use, for `None`)
/// needs to see, in every file or in `only`, and what each one uses. It may
/// report uses of other things too; the caller keeps the ones it wants.
fn visit(
    project: &Project,
    target: Option<&Usable>,
    only: Option<&RelPath>,
    f: &mut dyn FnMut(&Usable, Use),
) {
    let wants = Wants::of(target);
    let model = project.model();
    let terms = (wants.terms && !model.glossary.terms.is_empty()).then(|| {
        Terms::new(
            model
                .glossary
                .terms
                .iter()
                .filter(|term| term.match_mode != GlossaryMatch::Marked),
        )
    });
    let default_note = default_note();
    let files: Box<dyn Iterator<Item = &FileIndex>> = match only {
        Some(path) => Box::new(project.file(path).into_iter()),
        None => Box::new(project.files()),
    };
    for file in files {
        let place = |span, kind| Use {
            file: file.path.clone(),
            span,
            kind,
        };
        if wants.files {
            file_uses(project, file, &mut |used, span, kind| {
                f(used, place(span, kind))
            });
        }
        if wants.phrases {
            for phrase in file.phrases.iter().filter(|p| p.declared) {
                f(
                    &Usable::Phrase(phrase.phrase.key.clone()),
                    place(phrase.phrase.span, UseKind::Phrase),
                );
            }
        }
        if wants.availability {
            availability_uses(model, file, &mut |used, span| {
                f(used, place(span, UseKind::Availability));
            });
        }
        if let Some(terms) = &terms {
            term_uses(terms, file, &mut |id, span| {
                f(&Usable::Term(id.to_owned()), place(span, UseKind::Term));
            });
        }
        if wants.directives {
            directive_uses(model, &default_note, file, &mut |used, span, kind| {
                f(used, place(span, kind))
            });
        }
    }
}

/// The links and includes in `file` that name a source file, and a heading
/// in it.
fn file_uses(project: &Project, file: &FileIndex, f: &mut dyn FnMut(&Usable, Span, UseKind)) {
    let resolutions = project.resolutions(&file.path);
    for (reference, resolution) in file.references.iter().zip(resolutions) {
        let Resolution::Source { target, id, .. } = resolution else {
            continue;
        };
        if reference.kind != RefKind::Link {
            continue;
        }
        // A page's links to its own headings use the headings, not the page.
        if target != &file.path {
            f(&Usable::File(target.clone()), reference.span, UseKind::Link);
        }
        if let Some(id) = id
            && let Some((written_in, heading)) = project.page_heading(target, id)
        {
            let heading = Usable::Heading {
                file: written_in,
                id: heading.source_id.clone(),
            };
            f(&heading, reference.span, UseKind::Link);
        }
    }
    for include in &file.includes {
        let Some(target) = include.target.as_ref().filter(|t| project.is_source(t)) else {
            continue;
        };
        f(
            &Usable::File(target.clone()),
            include.span,
            UseKind::Include,
        );
        if let Some(section) = &include.section
            && project.heading(target, section).is_some()
        {
            let heading = Usable::Heading {
                file: target.clone(),
                id: section.clone(),
            };
            f(&heading, include.span, UseKind::Include);
        }
    }
}

/// The availability specs in `file` that name a feature, a dimension, or a
/// dimension's value.
fn availability_uses(model: &ContentModel, file: &FileIndex, f: &mut dyn FnMut(&Usable, Span)) {
    let mut spec = |spec: &ascribe_core::availability::AvailabilitySpec, at: Span| {
        if let Some(name) = spec.bare_name()
            && model.feature(&name.text).is_some()
        {
            f(&Usable::Feature(name.text.clone()), at);
            return;
        }
        for entry in &spec.entries {
            let name = &entry.target.text;
            let dimension = model
                .dimension(name)
                .or_else(|| model.dimension_of_value(name));
            if let Some(dimension) = dimension {
                f(&Usable::Dimension(dimension.name.clone()), at);
            }
        }
    };
    for marker in &file.availability {
        if let (Some(Ok(parsed)), Some((_, at))) = (&marker.spec, &marker.primary) {
            spec(parsed, *at);
        }
    }
    if let Some(text) = file
        .frontmatter
        .as_ref()
        .and_then(|fm| fm.get("available"))
        .and_then(|v| v.as_str())
        && let Ok(parsed) = ascribe_core::availability::parse_availability(text, 0)
        && let Some(at) = frontmatter_key(file, "available")
    {
        spec(&parsed, at);
    }
}

/// The line of a top-level key of the frontmatter, from the key to the end
/// of its value on that line.
fn frontmatter_key(file: &FileIndex, key: &str) -> Option<Span> {
    let content = file.document.frontmatter.as_ref()?.content;
    let text = file.source.get(content.range())?;
    let mut at = content.start();
    for line in text.split_inclusive('\n') {
        if line
            .strip_prefix(key)
            .is_some_and(|rest| rest.trim_start().starts_with(':'))
        {
            return Some(Span::new(at, at + line.trim_end().len()));
        }
        at += line.len();
    }
    None
}

/// The occurrences of glossary terms in the prose of `file`: paragraphs,
/// table cells, and the text primaries of directive lines, outside links,
/// images, and code, as the build links them.
fn term_uses(terms: &Terms, file: &FileIndex, f: &mut dyn FnMut(&str, Span)) {
    walk_blocks(&file.document.blocks, &mut |block: &Block| {
        for list in prose_lists(&block.kind) {
            prose_terms(terms, list, &file.source, f);
        }
    });
}

fn prose_terms(terms: &Terms, inlines: &[Inline], source: &str, f: &mut dyn FnMut(&str, Span)) {
    for inline in inlines {
        match &inline.kind {
            InlineKind::Text(text) => {
                // A text with escapes or entities decoded can't be mapped back
                // more exactly than its node.
                let exact = source.get(inline.span.range()) == Some(text.as_str());
                let mut at = 0;
                while let Some((start, end, candidate)) = terms.next_match(text, at) {
                    at = end;
                    let span = if exact {
                        Span::new(inline.span.start() + start, inline.span.start() + end)
                    } else {
                        inline.span
                    };
                    f(&candidate.term, span);
                }
            }
            InlineKind::Emphasis(children) | InlineKind::Strong(children) => {
                prose_terms(terms, children, source, f);
            }
            _ => {}
        }
    }
}

/// The `@variant`s, `@note`s, and widgets in `file`, and its frontmatter
/// `variant` keys.
fn directive_uses(
    model: &ContentModel,
    default_note: &Option<String>,
    file: &FileIndex,
    f: &mut dyn FnMut(&Usable, Span, UseKind),
) {
    let mut line = |line: &DirectiveLine| {
        if model.widget(&line.name).is_some() {
            f(
                &Usable::Widget(line.name.clone()),
                line.name_span,
                UseKind::Widget,
            );
        }
        match line.name.as_str() {
            "variant" => {
                for attribute in line.attributes.iter().flat_map(|b| &b.attributes) {
                    if model.dimension(&attribute.key).is_some() {
                        f(
                            &Usable::Dimension(attribute.key.clone()),
                            attribute.span,
                            UseKind::Variant,
                        );
                    }
                }
            }
            "note" => {
                let given = line
                    .attributes
                    .as_ref()
                    .and_then(|b| b.get("type"))
                    .and_then(|a| a.value.as_ref());
                match given.and_then(|v| v.as_text().map(|text| (text, v.span()))) {
                    Some((text, span)) => {
                        f(&Usable::Note(text.to_owned()), span, UseKind::Note);
                    }
                    None if given.is_none() => {
                        if let Some(default) = default_note {
                            f(
                                &Usable::Note(default.clone()),
                                line.name_span,
                                UseKind::Note,
                            );
                        }
                    }
                    None => {}
                }
            }
            _ => {}
        }
    };
    walk_blocks(
        &file.document.blocks,
        &mut |block: &Block| match &block.kind {
            BlockKind::Directive(d) => line(d),
            BlockKind::Container(c) => line(&c.opener),
            BlockKind::Group(g) => {
                for arm in &g.arms {
                    line(&arm.opener);
                }
            }
            _ => {}
        },
    );
    if let Some(variant) = file
        .frontmatter
        .as_ref()
        .and_then(|fm| fm.get("variant"))
        .and_then(|v| v.as_mapping())
        && let Some(at) = frontmatter_key(file, "variant")
    {
        for name in variant.keys().filter_map(|k| k.as_str()) {
            if model.dimension(name).is_some() {
                f(&Usable::Dimension(name.to_owned()), at, UseKind::Variant);
            }
        }
    }
}

/// The type of a `@note` that gives none: its `type` attribute's default.
fn default_note() -> Option<String> {
    let Attributes::Declared(attributes) = Builtin::Note.schema().attributes else {
        return None;
    };
    match attributes.into_iter().find(|a| a.key == "type")?.default? {
        DefaultValue::Text(text) => Some(text),
        DefaultValue::Boolean(_) | DefaultValue::Set(_) => None,
    }
}
