//! What other files can see of a file, and what of the content model each part
//! of the pipeline reads.
//!
//! Both are used to decide how much to redo, and both err toward doing more:
//! a false "changed" costs time, a false "unchanged" would be a wrong answer.

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use ascribe_model::ContentModel;
use ascribe_syntax::{Block, BlockKind, Bound, DirectiveLine, bound_heading};

use crate::index::FileIndex;
use crate::project::Project;

pub(crate) fn hash_of(value: &(impl Hash + ?Sized)) -> u64 {
    let mut hasher = DefaultHasher::new();
    value.hash(&mut hasher);
    hasher.finish()
}

/// A hash of what the rest of the project can see of a file: everything about
/// it that a link to it, or an include of it, can depend on, and nothing that
/// only shows up in the text of a paragraph.
///
/// That is: whether it's a page or a fragment; its frontmatter as written
/// (the title, and `variant` and `available`, which decide whether a build
/// publishes it); its headings (level, text, source id, `@id`); every
/// directive line and container or group opener as written, with how blocks
/// nest; and, for a heading marked `@term`, the block that opens its section,
/// which is the term's definition on a glossary page (SPEC §5.4). Directives
/// are included because `@available`, `@variant`, `@include`, and `@term`
/// decide which headings survive a build, what a page holds, and what the
/// glossary's terms are.
///
/// Editing a paragraph, a code block, or the destination of a link leaves it
/// unchanged, so the pages that link to this one aren't re-resolved for it,
/// unless the paragraph is a term's definition.
pub(crate) fn structure_signature(index: &FileIndex) -> u64 {
    let mut h = DefaultHasher::new();
    index.kind.hash(&mut h);
    if let Some(front) = &index.document.frontmatter {
        index.source.get(front.content.range()).hash(&mut h);
    }
    for heading in &index.headings {
        heading.level.hash(&mut h);
        heading.text.hash(&mut h);
        heading.source_id.hash(&mut h);
        heading
            .explicit_id
            .as_ref()
            .map(|e| (&e.id, e.permalink))
            .hash(&mut h);
        heading.has_phrase.hash(&mut h);
    }
    blocks(&index.document.blocks, &index.source, &mut h);
    definitions(&index.document.blocks, &index.source, &mut h);
    h.finish()
}

/// A hash of what every page can see of the glossary: each term's id, text,
/// aliases, settings, and pages, and the text of the paragraphs that may
/// define it, wherever they're written. A fragment a glossary page includes
/// can hold a definition its own [`structure_signature`] doesn't see (one
/// included right under a term's heading), so an edit to it that leaves its
/// structure alone is compared by this.
pub(crate) fn glossary_signature(project: &Project) -> u64 {
    let mut h = DefaultHasher::new();
    for term in project.glossary().terms {
        term.id.hash(&mut h);
        term.term.hash(&mut h);
        term.aliases.hash(&mut h);
        term.case_sensitive.hash(&mut h);
        (term.match_mode as u8).hash(&mut h);
        term.pages.hash(&mut h);
        term.definition.len().hash(&mut h);
        for at in &term.definition {
            project
                .file_by_id(at.file)
                .and_then(|f| f.source.get(at.span.range()))
                .hash(&mut h);
        }
    }
    h.finish()
}

/// The text of the block that opens each `@term` heading's section, in every
/// list of blocks.
fn definitions(list: &[Block], source: &str, h: &mut DefaultHasher) {
    for (i, block) in list.iter().enumerate() {
        match &block.kind {
            BlockKind::Directive(line) if line.name == "term" => {
                if let Some(heading) = bound_heading(list, i)
                    && let Some(first) = list[heading + 1..]
                        .iter()
                        .find(|b| b.binding() != Some(Bound::Heading))
                {
                    source.get(first.span.range()).hash(h);
                }
            }
            BlockKind::BlockQuote(q) => definitions(&q.children, source, h),
            BlockKind::List(l) => {
                for item in &l.items {
                    definitions(&item.children, source, h);
                }
            }
            BlockKind::Container(c) => definitions(&c.children, source, h),
            BlockKind::Group(g) => {
                for arm in &g.arms {
                    definitions(&arm.children, source, h);
                }
            }
            _ => {}
        }
    }
}

fn line(line: &DirectiveLine, source: &str, h: &mut DefaultHasher) {
    source.get(line.span.range()).hash(h);
    if let Some(title) = &line.title {
        source.get(title.span.range()).hash(h);
    }
}

fn blocks(list: &[Block], source: &str, h: &mut DefaultHasher) {
    for block in list {
        match &block.kind {
            BlockKind::Directive(l) => {
                1u8.hash(h);
                line(l, source, h);
            }
            BlockKind::Heading(_) => 2u8.hash(h),
            BlockKind::BlockQuote(q) => {
                3u8.hash(h);
                blocks(&q.children, source, h);
                4u8.hash(h);
            }
            BlockKind::List(l) => {
                5u8.hash(h);
                for item in &l.items {
                    6u8.hash(h);
                    blocks(&item.children, source, h);
                }
                7u8.hash(h);
            }
            BlockKind::Container(c) => {
                8u8.hash(h);
                line(&c.opener, source, h);
                blocks(&c.children, source, h);
                9u8.hash(h);
            }
            BlockKind::Group(g) => {
                10u8.hash(h);
                for arm in &g.arms {
                    11u8.hash(h);
                    line(&arm.opener, source, h);
                    blocks(&arm.children, source, h);
                }
                12u8.hash(h);
            }
            _ => {}
        }
    }
}

/// The paths of the fields that take phrases, each with whether it's read
/// with inline markup.
fn phrase_fields(fields: &[ascribe_model::Field], prefix: &str, out: &mut Vec<(String, bool)>) {
    for field in fields {
        let path = format!("{prefix}{}", field.name);
        if field.phrases {
            out.push((path.clone(), field.inline.is_some()));
        }
        let mut ty = &field.ty;
        while let ascribe_model::FieldType::List(inner) = ty {
            ty = inner;
        }
        if let ascribe_model::FieldType::Object(inner) = ty {
            phrase_fields(inner, &format!("{path}."), out);
        }
    }
}

/// How much of the pipeline a change to the content model reaches.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ModelImpact {
    /// Only what `ascribe.toml` says about itself changed (a warning, or where
    /// something is in the file). No file's result changes; the caller
    /// refreshes the diagnostics of `ascribe.toml` (file id 0).
    Warnings,
    /// Something changed that later passes read: dimensions, lifecycle
    /// states, features, builds, glossary, types, frontmatter schemas, image
    /// attributes, or the consumer profile's routes. Nothing is reparsed or
    /// re-indexed; every file is re-checked and every page re-resolved.
    Resolution,
    /// The index reads something that changed: the phrases, the fragment
    /// patterns, the slugger, or the sources. Every file is re-indexed (its parse is
    /// reused); every file is re-checked and every page re-resolved.
    Index,
    /// The directive keyword set (built-ins and widgets) or the note types
    /// changed, which changes how **every** file parses, including files no
    /// one has open. Every file is reparsed.
    Keywords,
}

/// What each stage reads of the model, as hashes to compare.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Fingerprints {
    parse: u64,
    index: u64,
    whole: u64,
    with_warnings: u64,
}

impl Fingerprints {
    pub(crate) fn of(model: &ContentModel) -> Fingerprints {
        // Destructured without `..`, so a field added to the model is a
        // compile error here until someone decides which stage reads it.
        let ContentModel {
            spec,
            project,
            types,
            fragments,
            dimensions,
            version_scheme,
            releases,
            planned_label,
            lifecycle,
            features,
            notes,
            phrases,
            glossary,
            image_attributes,
            widgets,
            consumer,
            builds,
            sources,
            editor_build,
            checks,
            intended,
            permalinks,
            crawlers,
            listing,
            warnings,
        } = model;
        // `parse` reads the directive schemas (built-ins and widgets) and the
        // note type names (`index_file`'s parse options).
        let note_names: Vec<&str> = notes.iter().map(|n| n.name.as_str()).collect();
        let parse = hash_of(&format!("{:?}", (model.directive_schemas(), note_names)));
        // `index` adds what indexing reads: phrases (heading text, destinations),
        // fragment patterns (page or fragment), the slugger, the sources
        // snippets are read through (a snippet's code is part of the
        // expansion), and which frontmatter fields of which types take
        // phrases. Where each is declared changes nothing a file reads.
        let read_through: Vec<_> = sources
            .iter()
            .map(|s| (&s.name, &s.path, &s.include, &s.ignore, &s.git))
            .collect();
        let phrase_fields: Vec<_> = types
            .iter()
            .map(|t| {
                let mut fields = Vec::new();
                phrase_fields(&t.frontmatter.fields, "", &mut fields);
                (&t.name, t.implicit, fields)
            })
            .collect();
        // Where a phrase, a feature, a glossary page, or a release line
        // is declared is read only by the diagnostics of `ascribe.toml`.
        let phrase_values: Vec<_> = phrases.iter().map(|p| (&p.key, &p.value)).collect();
        let feature_specs: Vec<_> = features
            .iter()
            .map(|f| (&f.key, &f.name, &f.available_text, &f.available))
            .collect();
        let glossary_pages: Vec<_> = glossary.pages.iter().map(|p| &p.path).collect();
        let declared: Vec<_> = phrases
            .iter()
            .map(|p| p.span)
            .chain(features.iter().map(|f| f.span))
            .chain(glossary.pages.iter().map(|p| p.span))
            .chain(releases.iter().flat_map(|r| {
                r.lines()
                    .into_iter()
                    .map(|(l, _)| l.span)
                    .chain(r.ends.iter().map(|e| e.line.span))
            }))
            .collect();
        let lines: Vec<_> = releases
            .iter()
            .map(|r| {
                let lines: Vec<_> = r
                    .lines()
                    .into_iter()
                    .map(|(l, status)| (&l.text, &l.components, status))
                    .collect();
                let ends: Vec<_> = r.ends.iter().map(|e| (&e.line.text, e.date)).collect();
                let archived: Vec<_> = r.archived.iter().map(|a| (&a.url, a.ended)).collect();
                (&r.target, lines, ends, archived)
            })
            .collect();
        let index = hash_of(&format!(
            "{parse}{:?}",
            (
                phrase_values.clone(),
                &fragments.patterns,
                &consumer.slugger,
                &read_through,
                phrase_fields
            )
        ));
        let whole = hash_of(&format!(
            "{:?}",
            (
                (spec, project, types, fragments, dimensions),
                (
                    (version_scheme, planned_label, lines),
                    lifecycle,
                    feature_specs,
                    notes,
                    phrase_values
                ),
                (
                    (glossary.match_mode, glossary.case_sensitive, glossary_pages),
                    image_attributes,
                    widgets,
                    consumer,
                    builds,
                    editor_build,
                    &read_through,
                    checks,
                    intended,
                    permalinks,
                    crawlers,
                    listing
                ),
            )
        ));
        let with_warnings = hash_of(&format!("{whole}{warnings:?}{declared:?}"));
        Fingerprints {
            parse,
            index,
            whole,
            with_warnings,
        }
    }

    /// What changed going from `self` to `next`; `None` if nothing did.
    pub(crate) fn impact_of(&self, next: &Fingerprints) -> Option<ModelImpact> {
        if self.parse != next.parse {
            Some(ModelImpact::Keywords)
        } else if self.index != next.index {
            Some(ModelImpact::Index)
        } else if self.whole != next.whole {
            Some(ModelImpact::Resolution)
        } else if self.with_warnings != next.with_warnings {
            Some(ModelImpact::Warnings)
        } else {
            None
        }
    }

    pub(crate) fn parse(&self) -> u64 {
        self.parse
    }
}
