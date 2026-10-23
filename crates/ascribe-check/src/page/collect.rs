//! The page-level problems of one resolved page.
//!
//! Three sources feed it: the problems build resolution recorded on the page,
//! the source index's link-id problems for the links the page publishes, and
//! what's found on the resolved page itself: the duplicate ids and headings
//! of its headings, a section's second summary, a permalink with no summary,
//! and a summary too long.
//!
//! Where a problem is reported (SPEC §8.1): at the source location that
//! causes it; when that's inside included content, at the outermost include
//! site (`via[0]`), with the location in the fragment as related information;
//! and once, at the later of the places that together cause it. The summary
//! checks are the exception: a permalink or a summary in a fragment is
//! reported in the fragment, once, however many pages include it, so a
//! reader isn't sent to a page that includes it.

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
pub(crate) struct Found {
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
    title_heading(index, page, &mut found);
    definitions(index, page, &mut found);
    link_ids(index, page, links, &mut found);
    summaries(index, page, &mut found);
    summaries_missing(index, page, &mut found);
    summaries_long(index, page, &mut found);
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
pub(super) fn at_include_site(index: &Project, mut issue: Issue, via: &[IncludeSite]) -> Issue {
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
pub(super) fn include_target(index: &Project, site: &IncludeSite) -> Option<String> {
    let file = index.file_by_id(site.file)?;
    let include = file.includes.iter().find(|i| i.span == site.span)?;
    Some(
        include
            .target
            .as_ref()
            .map_or_else(|| include.written.clone(), ToString::to_string),
    )
}

/// `summary-repeated`: each summary after the first in its section, once
/// includes and the build's modes are applied (SPEC §4.12).
fn summaries(index: &Project, page: &ResolvedPage, out: &mut Vec<Found>) {
    let mut firsts: Vec<(Option<&str>, Location)> = Vec::new();
    for found in ascribe_resolve::find_summaries(&page.blocks) {
        let directive = found.directive;
        let ResolvedKind::Leaf(ascribe_syntax::Block {
            kind: BlockKind::Directive(line),
            ..
        }) = &directive.kind
        else {
            continue;
        };
        let at = Location::new(directive.file, line.name_span);
        let ids = found.heading.and_then(|h| h.heading.as_ref());
        let anchor = ids.map(|ids| ids.page_id.as_str());
        let Some(&(_, first)) = firsts.iter().find(|(a, _)| *a == anchor) else {
            firsts.push((anchor, at));
            continue;
        };
        let issue = Issue::new(diagnostics::SUMMARY_REPEATED, at)
            .with_related(first, "the section's summary");
        let issue = match ids {
            Some(ids) => issue.with_arg("heading", ids.text.clone()),
            None => issue.with_variant("opening"),
        };
        out.push(Found {
            issue: at_include_site(index, issue, &directive.via),
            cause: (directive.file, directive.span, directive.via.clone()),
        });
    }
}

/// `summary-missing`: a heading whose `@id` marks it a permalink, with no
/// summary in its section once includes are expanded, at the `@id` line, in
/// the file it's written in. A page's own permalink has its description,
/// which `page-description-missing` reports.
fn summaries_missing(index: &Project, page: &ResolvedPage, out: &mut Vec<Found>) {
    if index.model().checks.is_off(diagnostics::SUMMARY_MISSING) {
        return;
    }
    for (block, heading) in page.headings() {
        let Some(at) = heading.explicit_at.filter(|_| heading.permalink) else {
            continue;
        };
        if page.summary(Some(&heading.page_id)).is_some() {
            continue;
        }
        let issue = Issue::new(diagnostics::SUMMARY_MISSING, at)
            .with_arg("name", heading.page_id.clone())
            .with_arg("section", section_text(index, block));
        out.push(Found {
            issue,
            cause: (at.file, at.span, block.via.clone()),
        });
    }
}

/// The most characters of a section's text a prompt quotes.
const MAX_SECTION: usize = 800;

/// The text of a heading's section as written in its file, for a prompt
/// that asks for its summary: at most [`MAX_SECTION`] characters.
fn section_text(index: &Project, block: &ResolvedBlock) -> String {
    let Some(file) = index.file_by_id(block.file) else {
        return String::new();
    };
    let Some(text) = file
        .heading_at(block.span)
        .and_then(|h| file.source.get(h.section.range()))
    else {
        return String::new();
    };
    let text = text.trim_end();
    let mut quoted: String = text.chars().take(MAX_SECTION).collect();
    if quoted.len() < text.len() {
        quoted.push('…');
    }
    quoted
}

/// `summary-long`: a section's summary as long as the limit or longer,
/// measured as plain text on one line ([`ascribe_resolve::SectionSummary::text`]),
/// at its `@summary` line, in the file it's written in.
fn summaries_long(index: &Project, page: &ResolvedPage, out: &mut Vec<Found>) {
    let checks = &index.model().checks;
    if checks.is_off(diagnostics::SUMMARY_LONG) {
        return;
    }
    let limit = summary_limit(checks);
    for summary in &page.summaries {
        let length = summary.text().chars().count() as u64;
        if length < limit {
            continue;
        }
        let Some(directive) = summary_line(page, summary) else {
            continue;
        };
        let at = Location::new(directive.file, directive.span);
        let issue = Issue::new(diagnostics::SUMMARY_LONG, at)
            .with_arg("length", length.to_string())
            .with_arg("limit", limit.to_string());
        out.push(Found {
            issue,
            cause: (at.file, at.span, directive.via.clone()),
        });
    }
}

/// The length at which `summary-long` reports a summary: `[checks.summary-long]
/// limit`, or [`super::SUMMARY_LIMIT`].
pub fn summary_limit(checks: &ascribe_model::Checks) -> u64 {
    checks
        .limit(diagnostics::SUMMARY_LONG)
        .unwrap_or(super::SUMMARY_LIMIT)
}

/// The `@summary` line that gives a section its summary.
fn summary_line<'p>(
    page: &'p ResolvedPage,
    summary: &ascribe_resolve::SectionSummary,
) -> Option<&'p ResolvedBlock> {
    ascribe_resolve::find_summaries(&page.blocks)
        .into_iter()
        .find(|found| {
            found.block.file == summary.block.file && found.block.span == summary.block.span
        })
        .map(|found| found.directive)
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

/// `title-heading-differs`: a page whose own top heading, the first level-1
/// heading it publishes, says something its title doesn't. A page's layout
/// draws its title as the top heading, so most pages write none and aren't
/// looked at; one that writes its own, as a landing page does in its hero,
/// is. They agree when either contains the other, ignoring case and runs of
/// spaces, so a heading that adds to the title, or a title that adds the
/// product's name, isn't reported. Reported at the title, with the heading
/// as related.
fn title_heading(index: &Project, page: &ResolvedPage, out: &mut Vec<Found>) {
    if index
        .model()
        .checks
        .is_off(diagnostics::TITLE_HEADING_DIFFERS)
    {
        return;
    }
    let Some(title) = page.title.as_deref() else {
        return;
    };
    let Some((block, ids)) = page.headings().into_iter().find(|(_, ids)| ids.level == 1) else {
        return;
    };
    let words = |text: &str| {
        text.split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
            .to_lowercase()
    };
    let (said, heading) = (words(title), words(&ids.text));
    if said.is_empty() || heading.is_empty() || said.contains(&heading) || heading.contains(&said) {
        return;
    }
    let Some(file) = index.file(&page.path) else {
        return;
    };
    let span = crate::across::title_span(file);
    let issue = Issue::new(
        diagnostics::TITLE_HEADING_DIFFERS,
        Location::new(page.file, span),
    )
    .with_arg("title", title.to_owned())
    .with_arg("heading", ids.text.clone())
    .with_related(
        Location::new(block.file, block.span),
        "the page's own top heading",
    );
    out.push(Found {
        issue,
        cause: (page.file, span, Arc::from([])),
    });
}

/// `glossary-definition-long`: on a glossary page, a term whose definition,
/// as this build derives it, is longer than a description may be (one
/// limit for both, `llms::DESCRIPTION_LIMIT`), at its heading.
fn definitions(index: &Project, page: &ResolvedPage, out: &mut Vec<Found>) {
    let checks = &index.model().checks;
    if checks.is_off(diagnostics::GLOSSARY_DEFINITION_LONG)
        || !ascribe_resolve::glossary_targets(index.model()).contains(&page.path)
    {
        return;
    }
    let limit = ascribe_resolve::llms::DESCRIPTION_LIMIT;
    let glossary = index.glossary();
    for (block, _) in page.headings() {
        let Some(term) = glossary.term_at(block.file, block.span) else {
            continue;
        };
        let long = term
            .definition_in(&page.blocks)
            .is_some_and(|d| d.chars().count() > limit);
        if !long {
            continue;
        }
        let issue = Issue::new(
            diagnostics::GLOSSARY_DEFINITION_LONG,
            Location::new(block.file, block.span),
        )
        .with_arg("term", term.term.clone())
        .with_arg("limit", limit.to_string())
        .with_arg("build", page.build.clone());
        out.push(Found {
            issue: at_include_site(index, issue, &block.via),
            cause: (block.file, block.span, block.via.clone()),
        });
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
