//! `permalink-duplicate`: a permalink a build publishes twice (SPEC §2.1,
//! §4.1): on two pages, or as a page's own and a heading's. Names are
//! compared ignoring case, as build names are, since a site may serve every
//! spelling of an address at one. Two headings with one id on one page are
//! `id-duplicate`'s.

use std::collections::HashMap;

use ascribe_core::{Issue, Location, diagnostics};
use ascribe_resolve::{Permalink, Project, ResolvedPage};

use super::collect::{Found, at_include_site, include_target};

/// Each permalink the build publishes after the first place that has it,
/// reported where it's declared, naming the first. A name that differs from
/// the first only in case is reported with the variant `case`, naming both
/// spellings.
pub(super) fn duplicates(index: &Project, pages: &[&ResolvedPage], out: &mut Vec<Found>) {
    let mut first: HashMap<String, Permalink> = HashMap::new();
    for permalink in ascribe_resolve::permalinks(pages.iter().copied()) {
        let key = permalink.name.to_ascii_lowercase();
        let Some(earlier) = first.get(&key) else {
            first.insert(key, permalink);
            continue;
        };
        let same = earlier.name == permalink.name;
        let headings = earlier.anchor.is_some() && permalink.anchor.is_some();
        if same && headings && earlier.page == permalink.page {
            continue;
        }
        let mut issue = Issue::new(diagnostics::PERMALINK_DUPLICATE, permalink.at)
            .with_arg("name", permalink.name.clone())
            .with_arg("page", earlier.page.to_string())
            .with_related(reported_at(earlier), related_label(earlier, same));
        if !same {
            issue = issue.with_arg("other", earlier.name.clone());
        }
        let fragment = permalink
            .via
            .first()
            .and_then(|site| include_target(index, site));
        issue = match (fragment, same) {
            (Some(path), true) => issue.with_variant("include").with_arg("path", path),
            (Some(path), false) => issue.with_variant("include-case").with_arg("path", path),
            (None, true) => issue,
            (None, false) => issue.with_variant("case"),
        };
        out.push(Found {
            issue: at_include_site(index, issue, &permalink.via),
            cause: (permalink.at.file, permalink.at.span, permalink.via.clone()),
        });
    }
}

/// The label on the first place a duplicate permalink is, naming its
/// spelling when that differs.
fn related_label(earlier: &Permalink, same: bool) -> String {
    if same {
        format!("also on `{}`", earlier.page)
    } else {
        format!("`{}` on `{}`", earlier.name, earlier.page)
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
