//! What other files can see of a file, and what of the content model each part
//! of the pipeline reads.
//!
//! Both are used to decide how much to redo, and both err toward doing more:
//! a false "changed" costs time, a false "unchanged" would be a wrong answer.

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use tessera_model::ContentModel;
use tessera_syntax::{Block, BlockKind, DirectiveLine};

use crate::index::FileIndex;

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
/// publishes it); its headings (level, text, source id, `@id`); and every
/// directive line and container or group opener as written, with how blocks
/// nest. Directives are included because `@available`, `@variant`, and
/// `@include` decide which headings survive a build and what a page holds.
///
/// Editing a paragraph, a code block, or the destination of a link leaves it
/// unchanged, so the pages that link to this one aren't re-resolved for it.
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
        heading.explicit_id.as_ref().map(|e| &e.id).hash(&mut h);
        heading.has_phrase.hash(&mut h);
    }
    blocks(&index.document.blocks, &index.source, &mut h);
    h.finish()
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
            warnings,
        } = model;
        // `parse` reads the directive schemas (built-ins and widgets) and the
        // note type names (`index_file`'s parse options).
        let note_names: Vec<&str> = notes.iter().map(|n| n.name.as_str()).collect();
        let parse = hash_of(&format!("{:?}", (model.directive_schemas(), note_names)));
        // `index` adds what indexing reads: phrases (heading text, destinations),
        // fragment patterns (page or fragment), the slugger, and the sources
        // snippets are read through (a snippet's code is part of the
        // expansion).
        let index = hash_of(&format!(
            "{parse}{:?}",
            (phrases, &fragments.patterns, &consumer.slugger, sources)
        ));
        let whole = hash_of(&format!(
            "{:?}",
            (
                (spec, project, types, fragments, dimensions),
                (version_scheme, lifecycle, features, notes, phrases),
                (
                    glossary,
                    image_attributes,
                    widgets,
                    consumer,
                    builds,
                    editor_build,
                    sources
                ),
            )
        ));
        let with_warnings = hash_of(&format!("{whole}{warnings:?}"));
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
