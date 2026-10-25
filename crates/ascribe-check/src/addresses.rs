//! The addresses the docs have published, against the base's (SPEC §7.10):
//! what a project with `ascribe.addresses` has stopped publishing that no
//! entry accounts for.
//!
//! The check can't read git, so its caller lists the base's addresses
//! ([`ascribe_resolve::addresses`]) and passes them in, with the working
//! tree's. Builds are matched by name. An address the base published and the
//! build doesn't now is accounted for when:
//!
//! - `[moved]` has its source, or the address itself, and the entry leads
//!   to a page a build publishes now;
//! - `[retired]` has its source, or the address itself;
//! - an availability mark takes its page out of this build while another
//!   build publishes the page: the mark is the decision;
//! - it's an id's, and the page it was on isn't at that address now: what
//!   happened to the page decides it.
//!
//! What's left is reported once per source: `address-lost` for a page and
//! its Markdown version, `anchor-lost` for an id, each listing its addresses
//! and, when it isn't every build compared, the builds. A build only the
//! base has is `addresses-build-gone`. Each is located at `ascribe.addresses`.

use std::collections::{BTreeMap, HashMap, HashSet};

use ascribe_core::{Issue, Location, RelPath, Span, diagnostics};
use ascribe_model::AddressFile;
use ascribe_model::addresses::split_place;
use ascribe_resolve::FileKind;
use ascribe_resolve::addresses::{AddressKind, BuildAddresses};

use crate::Diagnostic;
use crate::project::ADDRESSES_FILE_ID;

/// What a source lost, gathered across the builds.
#[derive(Default)]
struct Lost {
    /// The addresses, each once, in the order they were met.
    addresses: Vec<String>,
    /// The builds, each once, in the base's order.
    builds: Vec<String>,
}

impl Lost {
    fn add(&mut self, address: &str, build: &str) {
        if !self.addresses.iter().any(|a| a == address) {
            self.addresses.push(address.to_owned());
        }
        if !self.builds.iter().any(|b| b == build) {
            self.builds.push(build.to_owned());
        }
    }
}

/// The addresses `base` has that `now` doesn't and `file` doesn't account
/// for, as diagnostics at `ascribe.addresses`: an `address-lost` warning
/// for each page, an `anchor-lost` advice for each id, and an
/// `addresses-build-gone` advice for each build of `base` that `now`
/// doesn't have. `index` is the working tree's source index. Each of
/// `base`'s builds is compared with `now`'s of the same name; a caller that
/// checks only some builds passes only those.
pub fn compare(
    file: &AddressFile,
    index: &ascribe_resolve::Project,
    base: &[BuildAddresses],
    now: &[BuildAddresses],
) -> Vec<Diagnostic> {
    let now_by_name: HashMap<&str, &BuildAddresses> =
        now.iter().map(|b| (b.build.as_str(), b)).collect();
    // Where a `[moved]` entry leads counts only when a build publishes it.
    let published = |value: &str| {
        split_place(value)
            .and_then(|(path, _)| RelPath::parse(path).ok())
            .is_some_and(|path| now.iter().any(|b| b.publishes(&path)))
    };
    let accounted = |key: &str| {
        file.retired.contains_key(key) || file.moved.get(key).is_some_and(|e| published(&e.value))
    };
    let mut pages: BTreeMap<String, Lost> = BTreeMap::new();
    let mut anchors: BTreeMap<String, Lost> = BTreeMap::new();
    let mut gone: Vec<(&str, usize)> = Vec::new();
    let mut compared = 0;
    for before in base {
        let Some(after) = now_by_name.get(before.build.as_str()) else {
            gone.push((&before.build, before.addresses.len()));
            continue;
        };
        compared += 1;
        let has: HashSet<&str> = after.addresses.iter().map(|a| a.address.as_str()).collect();
        for address in &before.addresses {
            if has.contains(address.address.as_str())
                || accounted(&address.source)
                || accounted(&address.address)
            {
                continue;
            }
            match address.kind {
                AddressKind::Page | AddressKind::Markdown => {
                    if taken_out(index, &before.build, &address.page, now) {
                        continue;
                    }
                    pages
                        .entry(address.source.clone())
                        .or_default()
                        .add(&address.address, &before.build);
                }
                AddressKind::Anchor => {
                    let route = address
                        .address
                        .split_once('#')
                        .map_or(address.address.as_str(), |(route, _)| route);
                    if !has.contains(route) {
                        continue;
                    }
                    anchors
                        .entry(address.source.clone())
                        .or_default()
                        .add(&address.address, &before.build);
                }
            }
        }
    }
    let at = Location::new(ADDRESSES_FILE_ID, Span::empty(0));
    let in_builds = |lost: &Lost| {
        if lost.builds.len() >= compared {
            String::new()
        } else {
            format!(
                " in the build{} {}",
                if lost.builds.len() == 1 { "" } else { "s" },
                listed(
                    &lost
                        .builds
                        .iter()
                        .map(|b| format!("`{b}`"))
                        .collect::<Vec<_>>()
                )
            )
        }
    };
    let mut out = Vec::new();
    for (source, lost) in &pages {
        let mut issue = Issue::new(diagnostics::ADDRESS_LOST, at)
            .with_arg("source", source.clone())
            .with_arg("addresses", listed(&lost.addresses))
            .with_arg("builds", in_builds(lost));
        if lost.addresses.len() == 1 {
            issue = issue.with_variant("one");
        }
        out.push(with_builds(Diagnostic::from_issue(&issue), lost));
    }
    for (source, lost) in &anchors {
        let (path, id) = source.split_once('#').unwrap_or((source.as_str(), ""));
        let issue = Issue::new(diagnostics::ANCHOR_LOST, at)
            .with_arg("id", id.to_owned())
            .with_arg("path", path.to_owned())
            .with_arg("addresses", listed(&lost.addresses))
            .with_arg("builds", in_builds(lost));
        out.push(with_builds(Diagnostic::from_issue(&issue), lost));
    }
    for (build, count) in gone {
        let issue = Issue::new(diagnostics::ADDRESSES_BUILD_GONE, at)
            .with_arg("build", build.to_owned())
            .with_arg("count", count.to_string());
        out.push(Diagnostic::from_issue(&issue));
    }
    out
}

/// `addresses-base-unreadable`, at `ascribe.addresses`: the base couldn't be
/// read, for `reason`, so nothing was compared.
pub fn base_unreadable(reason: &str) -> Diagnostic {
    let issue = Issue::new(
        diagnostics::ADDRESSES_BASE_UNREADABLE,
        Location::new(ADDRESSES_FILE_ID, Span::empty(0)),
    )
    .with_arg("reason", reason.to_owned());
    Diagnostic::from_issue(&issue)
}

/// `addresses-base-unreadable` for a base with no project: `base`, as the
/// check names it, published nothing. It says nothing was compared, as a
/// project's first pull request has nothing to compare, so it's advice, not
/// a warning, unless `[checks]` gives it a level.
pub fn base_has_no_project(base: &str) -> Diagnostic {
    let issue = Issue::new(
        diagnostics::ADDRESSES_BASE_UNREADABLE,
        Location::new(ADDRESSES_FILE_ID, Span::empty(0)),
    )
    .with_variant("no-project")
    .with_arg("base", base.to_owned());
    let mut diagnostic = Diagnostic::from_issue(&issue);
    diagnostic.severity = crate::Severity::Advice;
    diagnostic
}

/// Whether an availability mark takes `page` out of `build` while another
/// build publishes it: the page is still a source, `build` drops it, and a
/// build of `now` publishes it.
fn taken_out(
    index: &ascribe_resolve::Project,
    build: &str,
    page: &RelPath,
    now: &[BuildAddresses],
) -> bool {
    let is_page = index.file(page).is_some_and(|f| f.kind == FileKind::Page);
    let dropped = index
        .model()
        .build(build)
        .is_some_and(|b| index.dropped(page, b).is_some());
    is_page && dropped && now.iter().any(|b| b.build != build && b.publishes(page))
}

fn with_builds(mut diagnostic: Diagnostic, lost: &Lost) -> Diagnostic {
    diagnostic.builds = lost.builds.clone();
    diagnostic
}

/// `a`, `a and b`, or `a, b, and c`.
fn listed(items: &[String]) -> String {
    match items {
        [] => String::new(),
        [one] => one.clone(),
        [a, b] => format!("{a} and {b}"),
        [rest @ .., last] => format!("{}, and {last}", rest.join(", ")),
    }
}
