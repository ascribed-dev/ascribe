//! The page-level problems of one resolved page.
//!
//! Three sources feed it: the problems build resolution recorded on the page,
//! the source index's link-id problems for the links the page publishes, and
//! the duplicate ids and headings of its resolved headings.
//!
//! Where a problem is reported (SPEC §8.1, Q20): at the source location that
//! causes it; when that's inside included content, at the outermost include
//! site (`via[0]`), with the location in the fragment as related information;
//! and once, at the later of the places that together cause it.

use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Arc;

use tessera_core::{DiagnosticSlug, FileId, Issue, Location, Related, Span, diagnostics};
use tessera_resolve::{IncludeSite, Project, ResolvedBlock, ResolvedPage};

/// A page-level issue and where its cause is, for deciding whether any build
/// publishes that content (Q101).
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
        // include (Q20); everything else at the outermost include site.
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
    // The first heading with each page id and each text.
    let mut ids: HashMap<&str, Location> = HashMap::new();
    let mut texts: HashMap<&str, Location> = HashMap::new();
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
        // SPEC-QUESTION(Q103): equal text, not equal slugs.
        if heading.text.is_empty() {
            continue;
        }
        match texts.get(heading.text.as_str()) {
            None => {
                texts.insert(&heading.text, own);
            }
            Some(first) if !heading.explicit => {
                let issue = Issue::new(diagnostics::HEADING_DUPLICATE_WITHOUT_ID, own)
                    .with_related(*first, "the same text is used here");
                push(at_include_site(index, issue, &block.via));
            }
            Some(_) => {}
        }
    }
}

/// `link-id-missing` and `link-id-in-fragment`, which the source index finds
/// once for the file a link is written in, for the links this page publishes.
/// A link in an arm the build removed isn't the build's problem (Q81).
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
                        .filter(|i| {
                            i.slug == diagnostics::LINK_ID_MISSING
                                || i.slug == diagnostics::LINK_ID_IN_FRAGMENT
                        })
                        .collect()
                })
                .unwrap_or_default()
        });
        for issue in problems.iter() {
            let at = issue.location;
            // The issue is at the link's destination, or at the whole link
            // for a reference form (Q53): inside one of this block's links.
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
