//! `page-size`: a page whose plain Markdown, in a build, is as long as the
//! limit or longer. The plain output is what an agent reads when a site
//! serves a page's Markdown, so the check measures it, rendered as the build
//! renders it.

use std::path::Path;

use ascribe_core::{FileId, Issue, Location, Span, diagnostics};
use ascribe_emit::{EmitContext, PlainEmitter, emit_page};
use ascribe_model::Build;
use ascribe_resolve::{Project, ResolvedPage};

use super::collect::Found;

/// The size a page passes under when no limit is set: the Web Documentation
/// Delivery Spec's (0.6.0), in characters of Markdown.
pub const DEFAULT_LIMIT: u64 = 50_000;

/// The most sections a prompt lists.
const MAX_SECTIONS: usize = 10;

/// The limit a project sets, or the default; `None` when the check is off.
pub(super) fn limit(index: &Project) -> Option<u64> {
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
    index: &Project,
    root: &Path,
    build: &Build,
    page: &ResolvedPage,
    limit: u64,
) -> Option<Found> {
    let cx = EmitContext::new(index, root, build);
    // As the build writes it: with the outputs for agents on, the pointer
    // to llms.txt is part of the page.
    let text = emit_page(&PlainEmitter::new(index.model()), &cx, page)
        .ok()?
        .text;
    let size = text.chars().count() as u64;
    if size < limit {
        return None;
    }
    let first_line = first_line(index, page.file);
    let issue = Issue::new(diagnostics::PAGE_SIZE, Location::new(page.file, first_line))
        .with_arg("size", thousands(size))
        .with_arg("limit", thousands(limit))
        .with_arg("build", build.name.clone())
        .with_arg("sections", sections(&text));
    Some(Found {
        issue,
        cause: (page.file, first_line, Default::default()),
    })
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
    let text = index.file_by_id(file).map_or("", |f| &*f.source);
    let end = text.find(['\n', '\r']).unwrap_or(text.len());
    Span::new(0, end)
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
fn thousands(n: u64) -> String {
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
