//! Go to definition (SPEC §10): links and includes go to their target file or
//! heading, `@id`s to the heading they name, and phrases and feature keys to
//! their entry in `ascribe.toml`.

use lsp_types::{Location, Position};
use tessera_core::{LineIndex, RelPath, Span};
use tessera_resolve::Resolution;

use crate::nav::{Ctx, Hit, Lines, directive_at, hit_at, identifier_primary};

/// Where the thing at a position is defined.
pub(crate) fn definition(ctx: &Ctx, position: Position) -> Option<Location> {
    let file = ctx.file()?;
    let index = LineIndex::new(&file.source);
    let offset = ctx.encoding.offset_lenient(&index, &file.source, position);
    let mut lines = Lines::new(ctx);
    match hit_at(file, offset) {
        Some(Hit::Phrase(use_)) => model_entry(ctx, Entry::Phrase(&use_.phrase.key)),
        Some(Hit::Reference(i, _)) => {
            let resolution = ctx.snapshot.resolutions(&ctx.path).get(i)?;
            resolution_location(ctx, &mut lines, resolution)
        }
        Some(Hit::Include(include)) => {
            let target = include.target.as_ref()?;
            source_location(ctx, &mut lines, target, include.section.as_deref())
        }
        Some(Hit::Availability(text, _)) => feature_entry(ctx, text),
        Some(Hit::Frontmatter(text, _)) => feature_entry(ctx, &text),
        Some(Hit::Directive(_)) | None => id_definition(ctx, &mut lines, offset),
    }
}

/// The location a link or image names.
pub(crate) fn resolution_location(
    ctx: &Ctx,
    lines: &mut Lines<'_>,
    resolution: &Resolution,
) -> Option<Location> {
    match resolution {
        Resolution::Source { target, id, .. } => source_location(ctx, lines, target, id.as_deref()),
        Resolution::Asset { path, .. } => lines.location(path, None),
        _ => None,
    }
}

/// A source file, or the heading in it with this source id (the file when the
/// id names none).
pub(crate) fn source_location(
    ctx: &Ctx,
    lines: &mut Lines<'_>,
    target: &RelPath,
    id: Option<&str>,
) -> Option<Location> {
    let file = ctx.snapshot.file(target)?;
    let heading = id.and_then(|id| file.heading_by_id(id));
    lines.location(target, heading)
}

/// On the primary of an `@id`, the heading it names.
fn id_definition(ctx: &Ctx, lines: &mut Lines<'_>, offset: usize) -> Option<Location> {
    let file = ctx.file()?;
    let d = directive_at(&file.document.blocks, offset)?;
    if d.name != "id" {
        return None;
    }
    let (_, primary) = identifier_primary(d)?;
    if !(primary.start() <= offset && offset <= primary.end()) {
        return None;
    }
    let heading = file
        .headings
        .iter()
        .find(|h| h.explicit_id.as_ref().is_some_and(|e| e.span == d.span))?;
    lines.location(&ctx.path, Some(heading))
}

fn feature_entry(ctx: &Ctx, text: &str) -> Option<Location> {
    let key = text.trim();
    ctx.model.feature(key)?;
    model_entry(ctx, Entry::Feature(key))
}

enum Entry<'a> {
    Phrase(&'a str),
    Feature(&'a str),
}

/// The entry in `ascribe.toml`: a phrase's line under `[phrases]`, or a
/// feature's `[features.<key>]` table.
fn model_entry(ctx: &Ctx, entry: Entry<'_>) -> Option<Location> {
    let span = find_entry(&ctx.model_text, &entry)?;
    let index = LineIndex::new(&ctx.model_text);
    Some(Location {
        uri: crate::uri::path_to_uri(&ctx.config)?,
        range: ctx.encoding.range(&index, span),
    })
}

/// The span of an entry's key (or of a feature's table header) in the model.
fn find_entry(text: &str, entry: &Entry<'_>) -> Option<Span> {
    let mut table = String::new();
    let mut at = 0;
    for line in text.split_inclusive('\n') {
        let start = at;
        at += line.len();
        let trimmed = line.trim();
        if trimmed.starts_with('#') || trimmed.is_empty() {
            continue;
        }
        if trimmed.starts_with('[') {
            table = trimmed
                .trim_start_matches('[')
                .split(']')
                .next()
                .unwrap_or_default()
                .chars()
                .filter(|c| !c.is_whitespace() && *c != '"' && *c != '\'')
                .collect();
            if let Entry::Feature(key) = entry
                && table == format!("features.{key}")
            {
                let lead = line.len() - line.trim_start().len();
                return Some(Span::new(start + lead, start + lead + trimmed.len()));
            }
            continue;
        }
        if let Entry::Phrase(key) = entry
            && table == "phrases"
        {
            let name = trimmed.split('=').next().unwrap_or_default().trim();
            if name.trim_matches(|c| c == '"' || c == '\'') == *key && trimmed.contains('=') {
                let lead = line.len() - line.trim_start().len();
                return Some(Span::new(start + lead, start + lead + name.len()));
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn entries_are_found_in_the_model() {
        let text = "spec = \"0.1\"\n\n[phrases]\nproduct = \"Quill\"\n\"api\" = \"x\"\n\n[features.sso]\nname = \"SSO\"\n";
        let p = find_entry(text, &Entry::Phrase("product")).expect("found");
        assert_eq!(&text[p.range()], "product");
        let p = find_entry(text, &Entry::Phrase("api")).expect("found");
        assert_eq!(&text[p.range()], "\"api\"");
        let p = find_entry(text, &Entry::Feature("sso")).expect("found");
        assert_eq!(&text[p.range()], "[features.sso]");
        assert!(find_entry(text, &Entry::Phrase("name")).is_none());
    }
}
