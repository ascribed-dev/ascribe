//! The page-level problems of one resolved page.
//!
//! Three sources feed it: the problems build resolution recorded on the page,
//! the source index's link-id problems for the links the page publishes, and
//! the duplicate ids and headings of its resolved headings.
//!
//! Where a problem is reported (SPEC §8.1): at the source location that
//! causes it; when that's inside included content, at the outermost include
//! site (`via[0]`), with the location in the fragment as related information;
//! and once, at the later of the places that together cause it.

use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Arc;

use ascribe_core::{
    Applicability, DiagnosticSlug, FileId, Fix, Issue, Location, Related, Span, TextEdit,
    diagnostics,
};
use ascribe_resolve::{IncludeSite, Project, ResolvedBlock, ResolvedKind, ResolvedPage};
use ascribe_syntax::BlockKind;

/// A page-level issue and where its cause is, for deciding whether any build
/// publishes that content.
#[derive(Clone, Debug)]
pub(super) struct Found {
    /// The issue, located where it's reported. Ids are the index's.
    pub issue: Issue,
    /// The cause: a span in a file, reached through these includes.
    pub cause: (FileId, Span, Arc<[IncludeSite]>),
}

/// The index's link problems by file, kept across builds because they don't
/// depend on one.
pub(super) type LinkProblems = RefCell<HashMap<FileId, Vec<Issue>>>;

/// Every page-level problem of `page`, in the order found, without repeats.
pub(super) fn check_page(index: &Project, page: &ResolvedPage, links: &LinkProblems) -> Vec<Found> {
    let mut found = Vec::new();
    recorded(index, page, &mut found);
    headings(index, page, &mut found);
    heading_levels(index, page, &mut found);
    link_ids(index, page, links, &mut found);
    let mut seen = Vec::new();
    found.retain(|f| {
        let key = identity(&f.issue);
        let new = !seen.contains(&key);
        if new {
            seen.push(key);
        }
        new
    });
    found
}

/// What makes two issues the same problem: the row, the place, the message
/// variant, and the message arguments other than `build`, which is how the
/// same problem in several builds becomes one.
pub(super) type Identity = (
    DiagnosticSlug,
    Location,
    Option<&'static str>,
    Vec<(&'static str, String)>,
);

pub(super) fn identity(issue: &Issue) -> Identity {
    (
        issue.slug,
        issue.location,
        issue.variant,
        issue
            .args
            .iter()
            .filter(|a| a.name != "build")
            .map(|a| (a.name, a.value.clone()))
            .collect(),
    )
}

/// What resolution recorded: expansion's problems, `available-exceeds-scope`,
/// `variant-no-arm-survives`, `link-id-removed`, and `link-page-dropped`.
fn recorded(index: &Project, page: &ResolvedPage, out: &mut Vec<Found>) {
    for problem in &page.problems {
        let inner = problem.issue.location;
        // A cycle is reported where it closes, in the file containing that
        // include; everything else at the outermost include site.
        let issue = if problem.issue.slug == diagnostics::INCLUDE_CYCLE {
            problem.issue.clone()
        } else {
            at_include_site(index, problem.issue.clone(), &problem.via)
        };
        out.push(Found {
            issue,
            cause: (inner.file, inner.span, problem.via.clone()),
        });
    }
}

/// Moves an issue found in included content to the include site, and keeps
/// its own location as related information (SPEC §8.1). An issue in the
/// page's own file stays where it is.
fn at_include_site(index: &Project, mut issue: Issue, via: &[IncludeSite]) -> Issue {
    let Some(site) = via.first() else {
        return issue;
    };
    let inner = issue.location;
    issue.location = Location::new(site.file, site.span);
    issue.related.insert(
        0,
        Related {
            location: inner,
            label: match index.path_of(inner.file) {
                Some(path) => format!("the cause is in the included file `{path}`"),
                None => "the cause is in the included file".to_owned(),
            },
        },
    );
    issue
}

/// The file an include directive names.
fn include_target(index: &Project, site: &IncludeSite) -> Option<String> {
    let file = index.file_by_id(site.file)?;
    let include = file.includes.iter().find(|i| i.span == site.span)?;
    Some(
        include
            .target
            .as_ref()
            .map_or_else(|| include.written.clone(), ToString::to_string),
    )
}

/// `id-duplicate` and `heading-duplicate-without-id`, from the page's headings
/// once includes and the build's modes are applied.
fn headings(index: &Project, page: &ResolvedPage, out: &mut Vec<Found>) {
    // The first heading with each page id, and each slug among the headings
    // without `@id` (the ones numbered against each other).
    let mut ids: HashMap<&str, Location> = HashMap::new();
    let mut slugs: HashMap<String, Location> = HashMap::new();
    let slugger = ascribe_resolve::slug::slugger_by_name(&index.model().consumer.slugger)
        .unwrap_or_else(ascribe_resolve::slug::default_slugger);
    for (block, heading) in page.headings() {
        let own = Location::new(block.file, block.span);
        let cause = (block.file, block.span, block.via.clone());
        let mut push = |issue: Issue| {
            out.push(Found {
                issue,
                cause: cause.clone(),
            });
        };
        match ids.get(heading.page_id.as_str()) {
            None => {
                ids.insert(&heading.page_id, own);
            }
            Some(first) => {
                // At the `@id` line for an explicit id, at the heading for
                // a slug (SPEC §8.1).
                let at = heading.explicit_at.unwrap_or(own);
                let mut issue = Issue::new(diagnostics::ID_DUPLICATE, at)
                    .with_arg("id", heading.page_id.clone())
                    .with_related(*first, "first used here");
                issue = match block.via.first().and_then(|s| include_target(index, s)) {
                    Some(path) => at_include_site(
                        index,
                        issue.with_variant("include").with_arg("path", path),
                        &block.via,
                    ),
                    None => at_include_site(index, issue, &block.via),
                };
                push(issue);
            }
        }
        // SPEC §5.5: the slug the text alone gives, among
        // headings without `@id`. An empty slug is `heading-empty-slug`'s.
        if heading.explicit {
            continue;
        }
        let slug = slugger.new_scope().slug(&heading.text);
        if slug.is_empty() {
            continue;
        }
        match slugs.get(&slug) {
            None => {
                slugs.insert(slug, own);
            }
            Some(first) => {
                let issue = Issue::new(diagnostics::HEADING_DUPLICATE_WITHOUT_ID, own)
                    .with_related(*first, "the same slug comes from this heading");
                push(at_include_site(index, issue, &block.via));
            }
        }
    }
}

/// `heading-level-skipped`: a heading more than one level below the one
/// before it, or at level 1, which the page's title is (the outputs make the
/// title the page's level-1 heading). Headings from includes count at the
/// levels they're written with (SPEC §4.2). After a skip, the next heading is
/// compared with the one that skipped, so a skipped section is reported once.
fn heading_levels(index: &Project, page: &ResolvedPage, out: &mut Vec<Found>) {
    if index
        .model()
        .checks
        .is_off(diagnostics::HEADING_LEVEL_SKIPPED)
    {
        return;
    }
    let mut previous = 1;
    for (block, _) in page.headings() {
        let ResolvedKind::Leaf(leaf) = &block.kind else {
            continue;
        };
        let BlockKind::Heading(heading) = &leaf.kind else {
            continue;
        };
        let level = heading.level;
        let issue = if level == 1 {
            Issue::new(
                diagnostics::HEADING_LEVEL_SKIPPED,
                Location::new(block.file, block.span),
            )
            .with_variant("title")
        } else if level > previous + 1 {
            Issue::new(
                diagnostics::HEADING_LEVEL_SKIPPED,
                Location::new(block.file, block.span),
            )
            .with_arg("level", level.to_string())
            .with_arg("previous", previous.to_string())
            .with_arg("expected", (previous + 1).to_string())
        } else {
            previous = level;
            continue;
        };
        let expected = if level == 1 { 2 } else { previous + 1 };
        previous = level;
        let issue = match level_fix(index, block, heading.setext, expected) {
            Some(fix) => issue.with_fix(fix),
            None => issue,
        };
        out.push(Found {
            issue: at_include_site(index, issue, &block.via),
            cause: (block.file, block.span, block.via.clone()),
        });
    }
}

/// The fix that makes a heading level `expected`, in the file it's written
/// in: another run of `#`, or, for a level-1 underlined heading, a `-`
/// underline.
fn level_fix(index: &Project, block: &ResolvedBlock, setext: bool, expected: u8) -> Option<Fix> {
    let source = &index.file_by_id(block.file)?.source;
    let text = source.get(block.span.range())?;
    let (marker, replacement) = if setext {
        let underline = text.trim_end().rfind('\n')? + 1;
        let start = underline + text[underline..].find('=')?;
        let end = start + text[start..].bytes().take_while(|b| *b == b'=').count();
        (Span::new(start, end), "-".repeat(end - start))
    } else {
        let start = text.find('#')?;
        let end = start + text[start..].bytes().take_while(|b| *b == b'#').count();
        (Span::new(start, end), "#".repeat(usize::from(expected)))
    };
    let base = block.span.start();
    Some(Fix {
        title: format!("Make it a level-{expected} heading"),
        file: block.file,
        edits: vec![TextEdit::replace(
            Span::new(base + marker.start(), base + marker.end()),
            replacement,
        )],
        // The page's outline changes, and the headings under it may need to
        // move too.
        applicability: Applicability::Unsafe,
    })
}

/// `link-id-missing`, which the source index finds
/// once for the file a link is written in, for the links this page publishes.
/// A link in an arm the build removed isn't the build's problem.
fn link_ids(index: &Project, page: &ResolvedPage, cache: &LinkProblems, out: &mut Vec<Found>) {
    page.visit(&mut |block: &ResolvedBlock| {
        if block.links.is_empty() {
            return;
        }
        let mut cache = cache.borrow_mut();
        let problems = cache.entry(block.file).or_insert_with(|| {
            index
                .path_of(block.file)
                .map(|path| {
                    index
                        .problems(path)
                        .into_iter()
                        .filter(|i| i.slug == diagnostics::LINK_ID_MISSING)
                        .collect()
                })
                .unwrap_or_default()
        });
        for issue in problems.iter() {
            let at = issue.location;
            // The issue is at the link's destination, or at the whole link
            // for a reference form: inside one of this block's links.
            let inside =
                |link: Span| link.start() <= at.span.start() && at.span.end() <= link.end();
            if at.file == block.file && block.links.iter().any(|l| inside(l.span)) {
                out.push(Found {
                    issue: at_include_site(index, issue.clone(), &block.via),
                    cause: (at.file, at.span, block.via.clone()),
                });
            }
        }
    });
}
