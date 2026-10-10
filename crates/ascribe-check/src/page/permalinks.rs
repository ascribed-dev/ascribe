//! `permalink-duplicate`: a permalink a build publishes twice (SPEC §2.1,
//! §4.1): on two pages, or as a page's own and a heading's. Two headings on
//! one page are `id-duplicate`'s.

use std::collections::HashMap;

use ascribe_core::{Issue, Location, diagnostics};
use ascribe_resolve::{Permalink, Project, ResolvedPage};

use super::collect::{Found, at_include_site, include_target};

/// Each permalink the build publishes after the first place that has it,
/// reported where it's declared, naming the first.
pub(super) fn duplicates(index: &Project, pages: &[&ResolvedPage], out: &mut Vec<Found>) {
    let mut first: HashMap<String, Permalink> = HashMap::new();
    for permalink in ascribe_resolve::permalinks(pages.iter().copied()) {
        let Some(earlier) = first.get(&permalink.name) else {
            first.insert(permalink.name.clone(), permalink);
            continue;
        };
        let headings = earlier.anchor.is_some() && permalink.anchor.is_some();
        if headings && earlier.page == permalink.page {
            continue;
        }
        let mut issue = Issue::new(diagnostics::PERMALINK_DUPLICATE, permalink.at)
            .with_arg("name", permalink.name.clone())
            .with_arg("page", earlier.page.to_string())
            .with_related(reported_at(earlier), format!("also on `{}`", earlier.page));
        let fragment = permalink
            .via
            .first()
            .and_then(|site| include_target(index, site));
        if let Some(path) = fragment {
            issue = issue.with_variant("include").with_arg("path", path);
        }
        out.push(Found {
            issue: at_include_site(index, issue, &permalink.via),
            cause: (permalink.at.file, permalink.at.span, permalink.via.clone()),
        });
    }
}

/// Where a permalink is reported on its page: its `@id` line, or the include
/// that brings it there.
fn reported_at(permalink: &Permalink) -> Location {
    permalink
        .via
        .first()
        .map_or(permalink.at, |site| Location::new(site.file, site.span))
}
