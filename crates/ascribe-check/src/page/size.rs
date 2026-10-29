//! `page-size`: a page whose plain Markdown, in a build, is as long as the
//! limit or longer. The plain output is what an agent reads when a site
//! serves a page's Markdown, so the check measures it, rendered as the build
//! renders it. Rendering is `ascribe-emit`'s, which this crate doesn't use:
//! the caller gives the renderer, a [`PlainMarkdown`], with
//! [`Project::with_plain_markdown`](crate::Project::with_plain_markdown).

use std::ops::Range;
use std::path::Path;

use ascribe_core::{FileId, Issue, Location, Span, diagnostics};
use ascribe_model::Build;
use ascribe_resolve::{Project, ResolvedPage};

use super::collect::Found;

/// Renders one page's plain Markdown in a build, as the build writes it:
/// `ascribe_emit::plain_markdown`. `None` when the page can't be rendered,
/// which its other problems say.
pub type PlainMarkdown =
    fn(index: &Project, root: &Path, build: &Build, page: &ResolvedPage) -> Option<String>;

/// The size a page passes under when no limit is set: the Web Documentation
/// Delivery Spec's (0.6.0), in characters of Markdown.
pub const DEFAULT_LIMIT: u64 = 50_000;

/// The most sections a prompt lists.
const MAX_SECTIONS: usize = 10;

/// The most parts, includes and sections, the problem names as related
/// information.
const MAX_PARTS: usize = 5;

/// The limit a project sets, or the default; `None` when the check is off.
pub fn limit(index: &Project) -> Option<u64> {
    let checks = &index.model().checks;
    if checks.is_off(diagnostics::PAGE_SIZE) {
        return None;
    }
    Some(
        checks
            .limit(diagnostics::PAGE_SIZE)
            .unwrap_or(DEFAULT_LIMIT),
    )
}

/// The page's `page-size` problem in `build`, if its plain Markdown has
/// `limit` characters or more. It's reported at the page's first line.
pub(super) fn page_size(
    render: PlainMarkdown,
    index: &Project,
    root: &Path,
    build: &Build,
    page: &ResolvedPage,
    limit: u64,
) -> Option<Found> {
    let text = render(index, root, build, page)?;
    let size = text.chars().count() as u64;
    if size < limit {
        return None;
    }
    let first_line = first_line(index, page.file);
    let mut issue = Issue::new(diagnostics::PAGE_SIZE, Location::new(page.file, first_line))
        .with_arg("size", thousands(size))
        .with_arg("limit", thousands(limit))
        .with_arg("build", build.name.clone())
        .with_arg("sections", sections(&text));
    for (at, size) in largest_parts(render, index, root, build, page) {
        let label = format!("`{}`: {} characters", line_at(index, at), thousands(size));
        issue = issue.with_related(at, label);
    }
    Some(Found {
        issue,
        cause: (page.file, first_line, Default::default()),
    })
}

/// The size `page-size` measures: the characters of the page's plain
/// Markdown in `build`, as `render` writes it. `None` when the page can't
/// be rendered, which its other problems say.
pub fn markdown_size(
    render: PlainMarkdown,
    index: &Project,
    root: &Path,
    build: &Build,
    page: &ResolvedPage,
) -> Option<u64> {
    render(index, root, build, page).map(|text| text.chars().count() as u64)
}

/// The page's largest parts, largest first, at most [`MAX_PARTS`]: its
/// sections, each from a heading of the page's top level to the next, located
/// at the heading, wherever it's written; and its includes, each the run of
/// top-level blocks one include of the page's own file brought, located at
/// the `@include` line. An include that's exactly one section is named as
/// the section. Each part's size is what it adds to the page's Markdown.
fn largest_parts(
    render: PlainMarkdown,
    index: &Project,
    root: &Path,
    build: &Build,
    page: &ResolvedPage,
) -> Vec<(Location, u64)> {
    let blocks = &page.blocks;
    let mut sections: Vec<(Location, Range<usize>)> = Vec::new();
    for (i, block) in blocks.iter().enumerate() {
        if block.heading.is_some() {
            if let Some(last) = sections.last_mut() {
                last.1.end = i;
            }
            let line = first_line_of(index, block.file, block.span);
            sections.push((Location::new(block.file, line), i..blocks.len()));
        }
    }
    let mut includes: Vec<(Location, Range<usize>)> = Vec::new();
    for (i, block) in blocks.iter().enumerate() {
        let Some(site) = block.via.first() else {
            continue;
        };
        let at = Location::new(site.file, site.span);
        match includes.last_mut() {
            Some((last, range)) if *last == at && range.end == i => range.end = i + 1,
            _ => includes.push((at, i..i + 1)),
        }
    }
    includes.retain(|(_, range)| !sections.iter().any(|(_, s)| s == range));
    // One copy of the page without its blocks, given each part's in turn:
    // copying the whole page for each part cost a page of several hundred
    // sections seconds (#894).
    let mut part = ResolvedPage {
        blocks: Vec::new(),
        ..page.clone()
    };
    let empty = markdown_size(render, index, root, build, &part).unwrap_or(0);
    let mut parts: Vec<(Location, u64)> = sections
        .into_iter()
        .chain(includes)
        .filter_map(|(at, range)| {
            part.blocks = blocks[range].to_vec();
            let size = markdown_size(render, index, root, build, &part)?;
            Some((at, size.saturating_sub(empty)))
        })
        .collect();
    parts.sort_by_key(|(_, size)| std::cmp::Reverse(*size));
    parts.truncate(MAX_PARTS);
    parts
}

/// The text of the first line at `at`, trimmed.
fn line_at(index: &Project, at: Location) -> String {
    let text = index.file_by_id(at.file).map_or("", |f| &*f.source);
    text.get(at.span.start()..at.span.end())
        .unwrap_or_default()
        .trim()
        .to_owned()
}

/// The size an issue's `size` argument says, for keeping the largest.
pub(super) fn size_of(issue: &Issue) -> u64 {
    issue
        .arg("size")
        .map(|s| s.replace(',', ""))
        .and_then(|s| s.parse().ok())
        .unwrap_or(0)
}

/// The first line of a file, without its line ending.
fn first_line(index: &Project, file: FileId) -> Span {
    first_line_of(index, file, Span::new(0, 0))
}

/// The first line of `span` in `file`, without its line ending.
fn first_line_of(index: &Project, file: FileId, span: Span) -> Span {
    let text = index.file_by_id(file).map_or("", |f| &*f.source);
    let rest = text.get(span.start()..).unwrap_or_default();
    let end = rest.find(['\n', '\r']).unwrap_or(rest.len());
    Span::new(span.start(), span.start() + end)
}

/// The page's sections, largest first: each heading of the Markdown, from
/// the title down, with the characters from it to the next heading. At most
/// [`MAX_SECTIONS`], one to a line.
fn sections(markdown: &str) -> String {
    let mut found: Vec<(String, u64)> = Vec::new();
    let mut fence: Option<(u8, usize)> = None;
    for line in markdown.split_inclusive('\n') {
        let trimmed = line.trim_start_matches(' ');
        let run = |marker: u8| trimmed.bytes().take_while(|b| *b == marker).count();
        match fence {
            Some((marker, width)) => {
                if run(marker) >= width && trimmed[run(marker)..].trim().is_empty() {
                    fence = None;
                }
            }
            None => {
                for marker in *b"`~" {
                    if run(marker) >= 3 {
                        fence = Some((marker, run(marker)));
                    }
                }
            }
        }
        let heading = fence.is_none()
            && line.starts_with('#')
            && line.trim_start_matches('#').starts_with(' ');
        if heading || found.is_empty() {
            let name = if heading {
                line.trim_end()
            } else {
                "(the start)"
            };
            found.push((name.to_owned(), 0));
        }
        if let Some((_, size)) = found.last_mut() {
            *size += line.chars().count() as u64;
        }
    }
    found.sort_by_key(|f| std::cmp::Reverse(f.1));
    let shown = found.len().min(MAX_SECTIONS);
    let mut lines: Vec<String> = found
        .iter()
        .take(MAX_SECTIONS)
        .map(|(name, size)| format!("- `{name}`: {} characters", thousands(*size)))
        .collect();
    if found.len() > shown {
        lines.push(format!("and {} smaller sections", found.len() - shown));
    }
    lines.join("\n")
}

/// A number with commas between its thousands: `61,234`.
pub fn thousands(n: u64) -> String {
    let digits = n.to_string();
    let mut out = String::new();
    for (i, c) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i).is_multiple_of(3) {
            out.push(',');
        }
        out.push(c);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn numbers_have_commas() {
        assert_eq!(thousands(7), "7");
        assert_eq!(thousands(50_000), "50,000");
        assert_eq!(thousands(1_234_567), "1,234,567");
    }

    #[test]
    fn sections_are_largest_first_and_skip_code() {
        let md = "# Title\n\nIntro.\n\n## Small\n\nx\n\n## Large\n\n```sh\n# not a heading\nlong long long long\n```\n";
        let listed = sections(md);
        let names: Vec<&str> = listed.lines().collect();
        assert!(names[0].starts_with("- `## Large`"), "{listed}");
        assert!(names[1].starts_with("- `# Title`"), "{listed}");
        assert!(!listed.contains("not a heading"), "{listed}");
    }
}
