//! Go to definition (SPEC §10): links and includes go to their target file or
//! heading, `@id`s to the heading they name, and phrases and feature keys to
//! their entry in `ascribe.toml`.

use ascribe_core::{LineIndex, RelPath, Span};
use ascribe_resolve::Resolution;
use lsp_types::{Location, Position};

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
        // A link's `#id` can name a heading from a fragment the target page
        // includes (SPEC §4.2): that heading, in the fragment.
        Resolution::Source {
            target,
            id: Some(id),
            ..
        } if ctx.snapshot.heading(target, id).is_none() => {
            match ctx.snapshot.page_heading(target, id) {
                Some((written_in, heading)) => lines.location(&written_in, Some(heading)),
                None => source_location(ctx, lines, target, None),
            }
        }
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

/// An entry of `ascribe.toml` that can be found by its text.
pub(crate) enum Entry<'a> {
    /// A phrase, by key.
    Phrase(&'a str),
    /// A feature, by key.
    Feature(&'a str),
    /// A value of a dimension, as `values` of `[dimensions.<name>]` lists it.
    DimensionValue {
        /// The dimension's name.
        dimension: &'a str,
        /// The value.
        value: &'a str,
    },
}

/// The entry in `ascribe.toml`: a phrase's line under `[phrases]`, a
/// feature's `[features.<key>]` table, or a dimension's value.
fn model_entry(ctx: &Ctx, entry: Entry<'_>) -> Option<Location> {
    let span = find_entry(&ctx.model_text, &entry)?;
    let index = LineIndex::new(&ctx.model_text);
    Some(Location {
        uri: crate::uri::path_to_uri(&ctx.config)?,
        range: ctx.encoding.range(&index, span),
    })
}

/// The span of an entry in the model's text: a phrase's key, a feature's
/// table header, or a dimension value's text inside its quotes. The model
/// keeps no spans, so this reads the text; it finds entries written as
/// tables and keys, which is how `ascribe.toml` declares them.
pub(crate) fn find_entry(text: &str, entry: &Entry<'_>) -> Option<Span> {
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
        if let Entry::DimensionValue { dimension, value } = entry
            && table == format!("dimensions.{dimension}")
            && let Some((name, _)) = trimmed.split_once('=')
            && name.trim() == "values"
        {
            let from = start + (line.len() - line.trim_start().len()) + name.len() + 1;
            return array_string(text, from, value);
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

/// The span of the string `wanted`, inside its quotes, in the TOML array
/// that starts at or after `from` (after a key's `=`): the array may span
/// lines and hold comments.
fn array_string(text: &str, from: usize, wanted: &str) -> Option<Span> {
    let rest = text.get(from..)?;
    let open = rest.find('[')?;
    let mut chars = rest[open + 1..].char_indices();
    let base = from + open + 1;
    while let Some((i, c)) = chars.next() {
        match c {
            ']' => return None,
            '#' => {
                for (_, c) in chars.by_ref() {
                    if c == '\n' {
                        break;
                    }
                }
            }
            '"' | '\'' => {
                let inner = i + 1;
                let mut end = None;
                let mut escaped = false;
                for (j, d) in chars.by_ref() {
                    if escaped {
                        escaped = false;
                    } else if d == '\\' && c == '"' {
                        escaped = true;
                    } else if d == c {
                        end = Some(j);
                        break;
                    }
                }
                let end = end?;
                if &rest[open + 1 + inner..open + 1 + end] == wanted {
                    return Some(Span::new(base + inner, base + end));
                }
            }
            _ => {}
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

    #[test]
    fn dimension_values_are_found_in_their_array() {
        let text = "[dimensions.pm]\nlabel = \"Package manager\"\nvalues = [\"npm\", 'pnpm',\n  # yarn = \"no\"\n  \"yarn\",\n]\n\n[dimensions.os]\nvalues = [\"linux\"]\n";
        let find = |dimension, value| {
            find_entry(text, &Entry::DimensionValue { dimension, value })
                .map(|span| &text[span.range()])
        };
        assert_eq!(find("pm", "npm"), Some("npm"));
        assert_eq!(find("pm", "pnpm"), Some("pnpm"));
        let yarn = find_entry(
            text,
            &Entry::DimensionValue {
                dimension: "pm",
                value: "yarn",
            },
        )
        .expect("found");
        assert_eq!(&text[yarn.start() - 1..yarn.end() + 1], "\"yarn\"");
        assert!(
            text[..yarn.start()].ends_with("  \""),
            "not the commented one"
        );
        assert_eq!(find("os", "linux"), Some("linux"));
        assert_eq!(find("os", "npm"), None);
        assert_eq!(find("pm", "no"), None);
    }
}
