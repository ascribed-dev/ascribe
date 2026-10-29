//! Go to definition (SPEC §10): links and includes go to their target file or
//! heading, `@id`s to the heading they name, a glossary term in prose to its
//! heading, phrases and feature keys to their entry in `ascribe.toml`, a
//! `@snippet`'s address to its region's start tag in the code file, and a
//! link to an operation, or an `@operation`'s, to the operation in its API's
//! spec.

use ascribe_core::{LineIndex, RelPath, Span};
use ascribe_model::RELEASE_DIMENSION;
use ascribe_resolve::{Resolution, Usable};
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
        Some(Hit::Operation(link, _)) => {
            let resolution = crate::nav::operation_resolution(ctx, link)?;
            resolution_location(ctx, &mut lines, &resolution)
        }
        Some(Hit::Include(include)) => {
            let target = include.target.as_ref()?;
            source_location(ctx, &mut lines, target, include.section.as_deref())
        }
        Some(Hit::Availability(text, _)) => feature_entry(ctx, text),
        Some(Hit::Frontmatter(text, _)) => feature_entry(ctx, &text),
        Some(Hit::Listing(_)) => listing_rule(ctx),
        Some(Hit::Replaces(_)) => {
            let replaced = ctx.snapshot.replaces(&ctx.path)?;
            source_location(ctx, &mut lines, &replaced, None)
        }
        Some(Hit::Snippet(snippet)) => crate::examples::snippet_definition(ctx, snippet),
        Some(Hit::Directive(_)) => id_definition(ctx, &mut lines, offset),
        None => term_definition(ctx, &mut lines, offset)
            .or_else(|| id_definition(ctx, &mut lines, offset)),
    }
}

/// On an occurrence of a glossary term in prose, the term's heading.
fn term_definition(ctx: &Ctx, lines: &mut Lines<'_>, offset: usize) -> Option<Location> {
    let id = ctx
        .snapshot
        .uses_in(&ctx.path)
        .into_iter()
        .find_map(|(used, place)| match used {
            Usable::Term(id) if place.span.start() <= offset && offset <= place.span.end() => {
                Some(id)
            }
            _ => None,
        })?;
    let glossary = ctx.snapshot.glossary();
    let term = glossary.term(&id)?;
    let heading = ctx
        .snapshot
        .file(&term.written_in)?
        .heading_at(term.heading)?;
    lines.location(&term.written_in, Some(heading))
}

/// The rule of `[listing]` the page is under, in `ascribe.toml`.
fn listing_rule(ctx: &Ctx) -> Option<Location> {
    let rule = ctx
        .model
        .listing_rule(ctx.path.as_str(), ctx.page_type_name())?;
    let index = LineIndex::new(&ctx.model_text);
    Some(Location {
        uri: crate::uri::path_to_uri(&ctx.config)?,
        range: ctx.encoding.range(&index, rule.span),
    })
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
        Resolution::Operation { api, id } => operation_location(ctx, api, id),
        _ => None,
    }
}

/// The spec of a declared API, at the line of the operation a link names:
/// at its top when the spec doesn't have it, or its line isn't known.
fn operation_location(ctx: &Ctx, api: &str, id: &str) -> Option<Location> {
    let declared = ctx.model.api(api)?;
    let path = ascribe_core::path::normalize(&ctx.config.parent()?.join(&declared.spec));
    let line = ctx
        .snapshot
        .project()
        .apis()
        .spec(api)
        .and_then(|spec| spec.named(id))
        .and_then(|named| named.operation().line)
        .and_then(|line| u32::try_from(line).ok())
        .unwrap_or(0);
    let start = Position { line, character: 0 };
    Some(Location {
        uri: crate::uri::path_to_uri(&path)?,
        range: lsp_types::Range { start, end: start },
    })
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
    /// A dimension, by name.
    Dimension(&'a str),
    /// A declared note type.
    Note(&'a str),
    /// A project widget, by name.
    Widget(&'a str),
    /// A build, by name.
    Build(&'a str),
    /// A page content type, by name.
    Type(&'a str),
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

impl Entry<'_> {
    /// The header of the entry's own table, without brackets or quotes, for
    /// an entry declared as one.
    fn table(&self) -> Option<String> {
        match self {
            Entry::Feature(key) => Some(format!("features.{key}")),
            Entry::Dimension(name) => Some(format!("dimensions.{name}")),
            Entry::Note(name) => Some(format!("notes.{name}")),
            Entry::Widget(name) => Some(format!("widgets.{name}")),
            Entry::Build(name) => Some(format!("builds.{name}")),
            Entry::Type(name) => Some(format!("types.{name}")),
            Entry::Phrase(_) | Entry::DimensionValue { .. } => None,
        }
    }
}

/// The span of an entry in the model's text: a phrase's key, the header of
/// the table that declares a feature, dimension, note type, widget, build,
/// or content type, or a dimension value's text inside its quotes. The model
/// keeps no spans, so this reads the text; it finds entries written as tables
/// and keys, which is how `ascribe.toml` declares them. The implicit
/// `release` dimension is found at the table of its first target, the first
/// `[releases.<target>]` no declared dimension has, and each of its values
/// at its own.
pub(crate) fn find_entry(text: &str, entry: &Entry<'_>) -> Option<Span> {
    find_declared(text, entry).or_else(|| match entry {
        Entry::Dimension(RELEASE_DIMENSION) => find_header(text, |table| {
            table
                .strip_prefix("releases.")
                .is_some_and(|target| !target.contains('.') && !declared_value(text, target))
        }),
        Entry::DimensionValue {
            dimension: RELEASE_DIMENSION,
            value,
        } => find_header(text, |table| {
            table.strip_prefix("releases.") == Some(*value)
        }),
        _ => None,
    })
}

/// Whether `value` is a declared dimension's name or one of its `values`,
/// which the model leaves out of the implicit dimension.
fn declared_value(text: &str, value: &str) -> bool {
    text.lines()
        .filter(|line| line.trim_start().starts_with('['))
        .filter_map(|line| {
            header_name(line.trim())
                .strip_prefix("dimensions.")
                .map(str::to_owned)
        })
        .any(|dimension| {
            dimension == value
                || find_declared(
                    text,
                    &Entry::DimensionValue {
                        dimension: &dimension,
                        value,
                    },
                )
                .is_some()
        })
}

/// The span of the first table header whose name, without brackets or
/// quotes, `wanted` takes.
fn find_header(text: &str, wanted: impl Fn(&str) -> bool) -> Option<Span> {
    let mut at = 0;
    for line in text.split_inclusive('\n') {
        let start = at;
        at += line.len();
        let trimmed = line.trim();
        if trimmed.starts_with('[') && wanted(&header_name(trimmed)) {
            let lead = line.len() - line.trim_start().len();
            return Some(Span::new(start + lead, start + lead + trimmed.len()));
        }
    }
    None
}

/// A table header's name, without brackets, quotes, or spaces.
fn header_name(header: &str) -> String {
    header
        .trim_start_matches('[')
        .split(']')
        .next()
        .unwrap_or_default()
        .chars()
        .filter(|c| !c.is_whitespace() && *c != '"' && *c != '\'')
        .collect()
}

/// [`find_entry`] for an entry `ascribe.toml` declares in its own words.
fn find_declared(text: &str, entry: &Entry<'_>) -> Option<Span> {
    let own_table = entry.table();
    // A content type declared only through its nested tables,
    // `[types.page.frontmatter]` without `[types.page]`, is found at the
    // first one.
    let nested_prefix = match entry {
        Entry::Type(name) => Some(format!("types.{name}.")),
        _ => None,
    };
    let mut nested = None;
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
            table = header_name(trimmed);
            let lead = line.len() - line.trim_start().len();
            let span = Span::new(start + lead, start + lead + trimmed.len());
            if own_table.as_ref() == Some(&table) {
                return Some(span);
            }
            if nested.is_none()
                && nested_prefix
                    .as_ref()
                    .is_some_and(|prefix| table.starts_with(prefix.as_str()))
            {
                nested = Some(span);
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
    nested
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
    fn tables_are_found_by_their_header() {
        let text = "[widgets.quill-lab]\nforms = [\"line\"]\n\n[widgets.quill-lab.attributes]\nlab = \"string\"\n\n[builds.\"self-managed-3.3\"]\nvariants = \"switch\"\n\n[notes.security]\n[dimensions.pm]\n";
        let find = |entry| find_entry(text, &entry).map(|span| &text[span.range()]);
        assert_eq!(
            find(Entry::Widget("quill-lab")),
            Some("[widgets.quill-lab]")
        );
        assert_eq!(
            find(Entry::Build("self-managed-3.3")),
            Some("[builds.\"self-managed-3.3\"]")
        );
        assert_eq!(find(Entry::Note("security")), Some("[notes.security]"));
        assert_eq!(find(Entry::Dimension("pm")), Some("[dimensions.pm]"));
        assert_eq!(find(Entry::Note("tip")), None);
        let text =
            "[types.guide]\npurpose = \"Teach\"\n\n[types.guide.frontmatter]\ntitle = \"string\"\n";
        let span = find_entry(text, &Entry::Type("guide")).expect("found");
        assert_eq!(&text[span.range()], "[types.guide]");
        // Declared only through a nested table: found there.
        let text = "[types.page.frontmatter]\ntitle = \"string\"\n\n[types.pages]\n";
        let span = find_entry(text, &Entry::Type("page")).expect("found");
        assert_eq!(&text[span.range()], "[types.page.frontmatter]");
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

    #[test]
    fn the_implicit_release_dimension_is_found_at_its_targets_tables() {
        let text = "[releases]\nlabel = \"Product\"\n\n[releases.agent]\nlive = [\"1.2\"]\n\n[releases.\"cli\"]\nlive = [\"0.4\"]\n";
        let find = |entry| find_entry(text, &entry).map(|span| &text[span.range()]);
        assert_eq!(find(Entry::Dimension("release")), Some("[releases.agent]"));
        let value = |value| Entry::DimensionValue {
            dimension: "release",
            value,
        };
        assert_eq!(find(value("cli")), Some("[releases.\"cli\"]"));
        assert_eq!(find(value("web")), None);
        // A target a declared dimension has isn't the implicit dimension's.
        let text = "[dimensions.deployment]\nvalues = [\"cloud\", \"self-managed\"]\n\n[releases.self-managed]\nlive = [\"3.4\"]\n\n[releases.agent]\nlive = [\"1.2\"]\n";
        assert_eq!(
            find_entry(text, &Entry::Dimension("release")).map(|span| &text[span.range()]),
            Some("[releases.agent]")
        );
        // Nor is a target named for a declared dimension.
        let text = "[dimensions.agent]\nvalues = [\"x\"]\n\n[releases.agent]\nlive = [\"1.2\"]\n\n[releases.cli]\nlive = [\"0.4\"]\n";
        assert_eq!(
            find_entry(text, &Entry::Dimension("release")).map(|span| &text[span.range()]),
            Some("[releases.cli]")
        );
        // A declared `release` dimension is found where it's declared.
        let text = "[dimensions.release]\nvalues = [\"stable\"]\n";
        assert_eq!(
            find_entry(text, &Entry::Dimension("release")).map(|span| &text[span.range()]),
            Some("[dimensions.release]")
        );
    }
}
