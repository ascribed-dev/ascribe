//! The addresses the docs have published, against the base's (SPEC §7.10):
//! what a project with `ascribe.addresses` has stopped publishing that no
//! entry accounts for, how to account for it, and the file's own checks.
//!
//! The check can't read git, so its caller lists the base's addresses
//! ([`ascribe_resolve::addresses`]), with the base's own file, its pages'
//! text, and the renames git sees, and passes them in as a [`Base`], with
//! the working tree's as [`Here`]. Builds are matched by name. An address
//! the base published and the build doesn't now is accounted for when:
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
//! and, when it isn't every build compared, the builds. Each carries its
//! fixes, edits to `ascribe.addresses`: git's renames recorded, when git
//! sees the page renamed, which is safe, since git's rename is the
//! decision; the page forwarded to the pages most like it; and the page
//! retired, with the placeholder for its reason. A build only the base has
//! is `addresses-build-gone`. Each is located at `ascribe.addresses`.
//!
//! The file's own checks, each at its entry, are in [`entries`].

use std::collections::{BTreeMap, HashMap, HashSet};

use ascribe_core::{Applicability, Fix, Issue, Location, RelPath, Span, diagnostics};
use ascribe_model::AddressFile;
use ascribe_model::addresses::{RETIRED_PLACEHOLDER, split_place};
use ascribe_resolve::FileKind;
use ascribe_resolve::addresses::{AddressKind, BuildAddresses};

use crate::Diagnostic;
use crate::project::ADDRESSES_FILE_ID;

mod entries;
mod similar;

pub use entries::{entries, file_deleted};

/// What the check knows of the base: what it published, and what it had.
#[derive(Clone, Debug, Default)]
pub struct Base {
    /// Each build's addresses there, in its content model's order.
    pub builds: Vec<BuildAddresses>,
    /// Its `ascribe.addresses`, when it had one that reads.
    pub file: Option<AddressFile>,
    /// Each page's text there, by its content path, to find the page most
    /// like one that's lost.
    pub texts: BTreeMap<String, String>,
    /// The pages git sees renamed between the base and the working tree:
    /// each old content path, and the new one.
    pub renames: BTreeMap<String, String>,
}

impl Base {
    /// The base of the project `index`, read as a project: its builds'
    /// addresses, its file, and its pages' text. Nothing is renamed.
    pub fn of(index: &ascribe_resolve::Project) -> Base {
        Base {
            builds: ascribe_resolve::addresses::every_build(index),
            file: index.addresses_file().cloned(),
            texts: index
                .pages()
                .map(|page| (page.path.to_string(), page.source.to_string()))
                .collect(),
            renames: BTreeMap::new(),
        }
    }
}

/// The working tree, as the comparison reads it.
#[derive(Clone, Copy)]
pub struct Here<'a> {
    /// `ascribe.addresses`, read.
    pub file: &'a AddressFile,
    /// Its text, which a fix edits.
    pub text: &'a str,
    /// The source index.
    pub index: &'a ascribe_resolve::Project,
    /// Each build's addresses, for the builds checked.
    pub builds: &'a [BuildAddresses],
}

impl Here<'_> {
    /// Whether a build publishes the page at `path`.
    fn publishes(&self, path: &str) -> bool {
        RelPath::parse(path).is_ok_and(|path| self.builds.iter().any(|b| b.publishes(&path)))
    }

    /// A fix that edits `ascribe.addresses` into `after`, titled `title`;
    /// `None` when that changes nothing.
    fn fix(&self, title: String, after: &AddressFile, safe: bool) -> Option<Fix> {
        let edit = after.edit_from(self.text)?;
        Some(Fix {
            title,
            file: ADDRESSES_FILE_ID,
            edits: vec![edit],
            applicability: if safe {
                Applicability::Safe
            } else {
                Applicability::Unsafe
            },
        })
    }
}

/// The renames of `renames` that are lost pages: each old path that isn't
/// a source now, with no entry already leading where git sees it went, to a
/// new path that's a page of `index`. One for each page, in path order:
/// what `ascribe addresses move --from-git` records.
pub fn git_moves(
    file: &AddressFile,
    index: &ascribe_resolve::Project,
    renames: &BTreeMap<String, String>,
) -> Vec<(String, String)> {
    renames
        .iter()
        .filter(|(old, new)| {
            let gone = RelPath::parse(old).is_ok_and(|old| index.file(&old).is_none());
            let page = RelPath::parse(new)
                .is_ok_and(|new| index.file(&new).is_some_and(|f| f.kind == FileKind::Page));
            let recorded = file.moved.get(*old).is_some_and(|e| e.value == **new);
            gone && page && !recorded && ascribe_model::addresses::Key::parse(old).is_some()
        })
        .map(|(old, new)| (old.clone(), new.clone()))
        .collect()
}

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

/// The addresses `base` has that `here` doesn't and its file doesn't
/// account for, as diagnostics at `ascribe.addresses`, with their fixes: an
/// `address-lost` warning for each page, an `anchor-lost` advice for each
/// id, and an `addresses-build-gone` advice for each build of `base` that
/// `here` doesn't have. Each of `base`'s builds is compared with `here`'s of
/// the same name; a caller that checks only some builds passes only those.
pub fn compare(here: &Here<'_>, base: &Base) -> Vec<Diagnostic> {
    let Here {
        file,
        index,
        builds: now,
        ..
    } = *here;
    let now_by_name: HashMap<&str, &BuildAddresses> =
        now.iter().map(|b| (b.build.as_str(), b)).collect();
    // Where a `[moved]` entry leads in the end counts only when a build
    // publishes it.
    let published = |value: &str| {
        file.follow(value)
            .as_deref()
            .and_then(split_place)
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
    for before in &base.builds {
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
    // Recording git's renames is one fix, the same for every page git sees
    // renamed, so `--fix` applies it once.
    let git = {
        let moves = git_moves(file, index, &base.renames);
        let mut after = file.clone();
        for (old, new) in &moves {
            after.record_move(old, new);
        }
        (moves, after)
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
        for fix in page_fixes(here, base, &git, source) {
            issue = issue.with_fix(fix);
        }
        out.push(with_builds(Diagnostic::from_issue(&issue), lost));
    }
    for (source, lost) in &anchors {
        let (path, id) = source.split_once('#').unwrap_or((source.as_str(), ""));
        let mut issue = Issue::new(diagnostics::ANCHOR_LOST, at)
            .with_arg("id", id.to_owned())
            .with_arg("path", path.to_owned())
            .with_arg("addresses", listed(&lost.addresses))
            .with_arg("builds", in_builds(lost));
        for fix in anchor_fixes(here, source, path, id) {
            issue = issue.with_fix(fix);
        }
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

/// How many pages most like a lost one its fixes offer to forward it to.
const FORWARD_CHOICES: usize = 3;

/// The fixes of a lost page, `source`: git's renames recorded, when git
/// sees it renamed to a page; forwarded to each of the pages most like it,
/// the most like it first; and retired.
fn page_fixes(
    here: &Here<'_>,
    base: &Base,
    git: &(Vec<(String, String)>, AddressFile),
    source: &str,
) -> Vec<Fix> {
    let mut fixes = Vec::new();
    let (moves, recorded) = git;
    if let Some((_, new)) = moves.iter().find(|(old, _)| old == source)
        && here.publishes(new)
    {
        fixes.extend(here.fix(
            format!(
                "git sees it renamed to {new}: record git's renames, as `ascribe addresses move --from-git` does"
            ),
            recorded,
            true,
        ));
    }
    let candidates: Vec<(&str, &str)> = here
        .index
        .pages()
        .filter(|page| here.publishes(page.path.as_str()))
        .map(|page| (page.path.as_str(), &*page.source))
        .collect();
    let like = base
        .texts
        .get(source)
        .map(|text| similar::most_like(text, &candidates, FORWARD_CHOICES))
        .unwrap_or_default();
    for (i, page) in like.iter().enumerate() {
        let mut after = here.file.clone();
        after.record_move(source, page);
        let title = if i == 0 {
            format!(
                "Forward it to the page most like it, {page}: `ascribe addresses move {source} {page}`"
            )
        } else {
            format!(
                "Forward it to {page}, a page like it: `ascribe addresses move {source} {page}`"
            )
        };
        fixes.extend(here.fix(title, &after, false));
    }
    fixes.extend(retire_fix(here, source));
    fixes
}

/// The fixes of a lost id, `source`, `path#id`: forwarded to the same id on
/// each other page that has it, and retired.
fn anchor_fixes(here: &Here<'_>, source: &str, path: &str, id: &str) -> Vec<Fix> {
    let mut fixes = Vec::new();
    let mut offered: HashSet<String> = HashSet::new();
    for address in here.builds.iter().flat_map(|b| &b.addresses) {
        if address.kind != AddressKind::Anchor
            || address.page.as_str() == path
            || !address.address.ends_with(&format!("#{id}"))
        {
            continue;
        }
        let place = format!("{}#{id}", address.page);
        if offered.len() >= FORWARD_CHOICES || !offered.insert(place.clone()) {
            continue;
        }
        let mut after = here.file.clone();
        after.record_move(source, &place);
        fixes.extend(here.fix(
            format!(
                "Forward it to the same id on another page, {place}: `ascribe addresses move {source} {place}`"
            ),
            &after,
            false,
        ));
    }
    fixes.extend(retire_fix(here, source));
    fixes
}

/// Retiring `source`, with the placeholder for the reason, which is
/// reported until it's replaced.
fn retire_fix(here: &Here<'_>, source: &str) -> Option<Fix> {
    let mut after = here.file.clone();
    after.record_retired(source, RETIRED_PLACEHOLDER);
    here.fix(
        format!("Retire it, and write why: `ascribe addresses retire {source} --reason \"…\"`"),
        &after,
        false,
    )
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
pub(crate) fn listed(items: &[String]) -> String {
    match items {
        [] => String::new(),
        [one] => one.clone(),
        [a, b] => format!("{a} and {b}"),
        [rest @ .., last] => format!("{}, and {last}", rest.join(", ")),
    }
}
