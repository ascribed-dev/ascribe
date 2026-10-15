//! Find All References (`textDocument/references`): every place a page, a
//! fragment, a heading, or a content model entry is used, from the search
//! the inventory counts with ([`ascribe_resolve::Project::uses`]).
//!
//! What the cursor asks about:
//!
//! - on a link or an `@include`, what it names: the heading after `#`, or
//!   else the file;
//! - on a `{key}`, a feature key in an availability spec, a glossary term in
//!   prose, a `@note` or its type, a widget, or a `@variant` or its
//!   attribute, that phrase, feature, term, note type, widget, or dimension;
//! - on a heading, or the primary of its `@id`, that heading;
//! - at the start of the file, or on its frontmatter or title, the file.

use ascribe_core::{LineIndex, RelPath};
use ascribe_resolve::Usable;
use ascribe_syntax::BlockKind;
use lsp_types::{Location, Position, Range};

use crate::definition::{Entry, find_entry};
use crate::nav::{Ctx, Lines, directive_at, identifier_primary};

/// The places that use what's at `position`, and its declaration first when
/// `include_declaration`. `None` when nothing there is used.
pub(crate) fn references(
    ctx: &Ctx,
    position: Position,
    include_declaration: bool,
) -> Option<Vec<Location>> {
    let file = ctx.file()?;
    let index = LineIndex::new(&file.source);
    let offset = ctx.encoding.offset_lenient(&index, &file.source, position);
    let target = target_at(ctx, offset)?;
    let mut lines = Lines::new(ctx);
    let mut out = Vec::new();
    if include_declaration && let Some(declared) = declaration(ctx, &mut lines, &target) {
        out.push(declared);
    }
    for place in ctx.snapshot.uses(&target) {
        out.push(Location {
            uri: ctx.uri_of(&place.file)?,
            range: lines.range(&place.file, place.span)?,
        });
    }
    Some(out)
}

/// What the cursor at `offset` of the requested file asks about.
pub(crate) fn target_at(ctx: &Ctx, offset: usize) -> Option<Usable> {
    let file = ctx.file()?;
    if offset == 0 {
        return Some(Usable::File(ctx.path.clone()));
    }
    // The smallest use under the cursor; of a link's two (the heading it
    // names and its page), the heading.
    let uses = ctx.snapshot.uses_in(&ctx.path);
    let best = uses
        .iter()
        .filter(|(_, place)| place.span.start() <= offset && offset <= place.span.end())
        .min_by_key(|(used, place)| (place.span.len(), !matches!(used, Usable::Heading { .. })));
    if let Some((used, _)) = best {
        return Some(used.clone());
    }
    let directive = directive_at(&file.document.blocks, offset);
    // A directive's name asks about the note type, widget, or dimension its
    // line uses.
    if let Some(directive) = directive
        && directive.name_span.start() <= offset
        && offset <= directive.name_span.end()
        && let Some((used, _)) = uses.iter().find(|(used, place)| {
            matches!(
                used,
                Usable::Note(_) | Usable::Widget(_) | Usable::Dimension(_)
            ) && directive.span.start() <= place.span.start()
                && place.span.end() <= directive.span.end()
        })
    {
        return Some(used.clone());
    }
    if let Some(directive) = directive
        && directive.name == "id"
        && let Some((id, span)) = identifier_primary(directive)
        && span.start() <= offset
        && offset <= span.end()
    {
        return Some(Usable::Heading {
            file: ctx.path.clone(),
            id: id.to_owned(),
        });
    }
    if let Some(heading) = file
        .headings
        .iter()
        .filter(|h| h.span.start() <= offset && offset <= h.span.end())
        .min_by_key(|h| h.span.len())
    {
        return Some(Usable::Heading {
            file: ctx.path.clone(),
            id: heading.source_id.clone(),
        });
    }
    let in_frontmatter = file
        .document
        .frontmatter
        .as_ref()
        .is_some_and(|fm| fm.span.start() <= offset && offset <= fm.span.end());
    let in_title = file.document.blocks.iter().any(|block| {
        matches!(block.kind, BlockKind::Title(_))
            && block.span.start() <= offset
            && offset <= block.span.end()
    });
    (in_frontmatter || in_title).then(|| Usable::File(ctx.path.clone()))
}

/// Where `target` is declared: the file's start, the heading, or the
/// entry in `ascribe.toml`.
fn declaration(ctx: &Ctx, lines: &mut Lines<'_>, target: &Usable) -> Option<Location> {
    let entry = match target {
        Usable::File(path) => return file_start(ctx, path),
        Usable::Heading { file, id } => {
            let heading = ctx.snapshot.heading(file, id)?;
            return lines.location(file, Some(heading));
        }
        Usable::Phrase(key) => Entry::Phrase(key),
        Usable::Feature(key) => Entry::Feature(key),
        Usable::Term(id) => Entry::Term(id),
        Usable::Dimension(name) => Entry::Dimension(name),
        Usable::Note(name) => Entry::Note(name),
        Usable::Widget(name) => Entry::Widget(name),
    };
    let span = find_entry(&ctx.model_text, &entry)?;
    Some(Location {
        uri: crate::uri::path_to_uri(&ctx.config)?,
        range: ctx.encoding.range(&LineIndex::new(&ctx.model_text), span),
    })
}

fn file_start(ctx: &Ctx, path: &RelPath) -> Option<Location> {
    ctx.snapshot.file(path)?;
    Some(Location {
        uri: ctx.uri_of(path)?,
        range: Range::default(),
    })
}
