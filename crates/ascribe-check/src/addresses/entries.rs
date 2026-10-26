//! `ascribe.addresses`' own checks (SPEC §7.10): whether each entry still
//! does what it says, each reported at its key.
//!
//! - **`addresses-entry-live`**, advice: a key whose source is back, as
//!   after a page is restored. Its fix removes the entry.
//! - **`addresses-entry-taken`**, an error: an address of a key that
//!   another source publishes now, so an old link lands on another page.
//! - **`addresses-entry-unpublished`**, advice: an entry the base's file
//!   doesn't have, whose key the base never published, as the editor's
//!   record of a move on a branch leaves. Its fix removes it. An entry the
//!   base's file has is never reported.
//! - **`addresses-entry-not-a-page`**, an error: a `[moved]` value that
//!   isn't a page, or a heading on one, in each build that published its
//!   key.
//! - **`addresses-chain`**, advice: a `[moved]` value that's another entry's
//!   key. Its fix forwards it to where the chain ends.
//! - **`addresses-anchor-collision`**, an error: an `#id` entry from a page
//!   that's gone, whose id is live on the page its page's addresses lead
//!   to, so a link to it lands there.
//! - **`addresses-soft-404`**, advice: many addresses forwarded to one
//!   index page.
//!
//! A `permalink:` key isn't checked here: what it records is checked against
//! the content with the project's other permalinks, in `permalinks.rs`.
//!
//! What needs the base is checked only when there's one. A key written
//! twice is found when the file is read (`AddressFile::parse`), and
//! `addresses-file-deleted` by [`file_deleted`].

use std::collections::{BTreeMap, HashSet};

use ascribe_core::{FileId, Issue, Location, RelPath, Span, diagnostics};
use ascribe_model::addresses::{Entry, Key, Step, split_place};
use ascribe_resolve::FileKind;
use ascribe_resolve::addresses::AddressKind;

use super::{Base, Here, listed};
use crate::Diagnostic;
use crate::project::ADDRESSES_FILE_ID;

/// How many addresses forwarded to one index page make it a soft 404.
const SOFT_404: usize = 3;

/// The problems with `here`'s entries, each at its key, in file order: its
/// `[moved]` table's, then its `[retired]` table's, then the soft 404s.
pub fn entries(here: &Here<'_>, base: Option<&Base>) -> Vec<Diagnostic> {
    let mut out = Vec::new();
    for (table, map) in [("moved", &here.file.moved), ("retired", &here.file.retired)] {
        for (key, entry) in map {
            if matches!(Key::parse(key), Some(Key::Permalink(_))) {
                continue;
            }
            let at = Location::new(ADDRESSES_FILE_ID, entry.span);
            if let Some(found) = whether_kept(here, base, table, key, at) {
                out.push(found);
                continue;
            }
            if table == "moved" {
                out.extend(not_a_page(here, base, key, &entry.value, at));
                out.extend(chain(here, key, &entry.value, at));
                out.extend(anchor_collision(here, key, &entry.value, at));
            }
        }
    }
    out.extend(soft_404s(here));
    out
}

/// `addresses-file-deleted`, at `ascribe.toml`: the base had an
/// `ascribe.addresses` with `count` entries, and the working tree has none.
pub fn file_deleted(count: usize) -> Diagnostic {
    let issue = Issue::new(
        diagnostics::ADDRESSES_FILE_DELETED,
        Location::new(FileId::new(0), Span::empty(0)),
    )
    .with_arg(
        "count",
        if count == 1 {
            "1 entry".to_owned()
        } else {
            format!("{count} entries")
        },
    );
    Diagnostic::from_issue(&issue)
}

/// Whether the entry for `key` still keeps an address: `addresses-entry-live`
/// when its source is back, `addresses-entry-taken` when another source has
/// its address, and `addresses-entry-unpublished` when the base never
/// published it and doesn't have the entry.
fn whether_kept(
    here: &Here<'_>,
    base: Option<&Base>,
    table: &'static str,
    key: &str,
    at: Location,
) -> Option<Diagnostic> {
    let parsed = Key::parse(key)?;
    let live = match parsed {
        Key::Source(path) => RelPath::parse(path).is_ok_and(|p| here.index.file(&p).is_some()),
        Key::Id { .. } => here
            .builds
            .iter()
            .flat_map(|b| &b.addresses)
            .any(|a| a.source == key),
        Key::Address(_) | Key::Permalink(_) => false,
    };
    if live {
        let issue = Issue::new(diagnostics::ADDRESSES_ENTRY_LIVE, at)
            .with_arg("key", key.to_owned())
            .with_arg("table", table);
        return Some(with_removal(here, issue, key));
    }
    // An address another source publishes now: one the key had at the
    // base, or, for an address key, the address itself.
    let had: Vec<&str> = match parsed {
        Key::Address(address) => vec![address],
        _ => base
            .iter()
            .flat_map(|b| &b.builds)
            .flat_map(|b| &b.addresses)
            .filter(|a| a.source == key)
            .map(|a| a.address.as_str())
            .collect(),
    };
    let taken = here
        .builds
        .iter()
        .flat_map(|b| &b.addresses)
        .find(|a| a.source != key && had.iter().any(|h| same_address(h, &a.address)));
    if let Some(now) = taken {
        let issue = Issue::new(diagnostics::ADDRESSES_ENTRY_TAKEN, at)
            .with_arg("address", now.address.clone())
            .with_arg("key", key.to_owned())
            .with_arg("source", now.source.clone());
        return Some(Diagnostic::from_issue(&issue));
    }
    let base = base?;
    let in_base_file = base.file.as_ref().is_some_and(|f| f.entry(key).is_some());
    if matches!(parsed, Key::Address(_)) || in_base_file || !had.is_empty() {
        return None;
    }
    let issue = Issue::new(diagnostics::ADDRESSES_ENTRY_UNPUBLISHED, at)
        .with_arg("key", key.to_owned())
        .with_arg("table", table);
    Some(with_removal(here, issue, key))
}

/// `issue`, with the safe fix that removes the entry for `key`.
fn with_removal(here: &Here<'_>, mut issue: Issue, key: &str) -> Diagnostic {
    let mut after = here.file.clone();
    after.remove(key);
    if let Some(fix) = here.fix("Remove the entry".to_owned(), &after, true) {
        issue = issue.with_fix(fix);
    }
    Diagnostic::from_issue(&issue)
}

/// Whether two addresses are one to a host: the same once case and a final
/// `/` are ignored.
fn same_address(a: &str, b: &str) -> bool {
    let norm = |s: &str| {
        let trimmed = s.trim_end_matches('/');
        if trimmed.is_empty() { "/" } else { trimmed }.to_lowercase()
    };
    norm(a) == norm(b)
}

/// `addresses-entry-not-a-page` for `key`, forwarded to `value`: the value
/// isn't a page of the project, the page has no heading with its id, or a
/// build that published the key doesn't publish the page.
fn not_a_page(
    here: &Here<'_>,
    base: Option<&Base>,
    key: &str,
    value: &str,
    at: Location,
) -> Option<Diagnostic> {
    // A chain is judged by where it ends; a cycle, or a chain that ends at
    // a retired entry, is `addresses-chain`'s.
    let end = here.file.follow(value)?;
    let (path, id) = split_place(&end)?;
    let page = RelPath::parse(path).ok()?;
    let issue = |variant: Option<&'static str>| {
        let issue = Issue::new(diagnostics::ADDRESSES_ENTRY_NOT_A_PAGE, at)
            .with_arg("key", key.to_owned())
            .with_arg("value", value.to_owned());
        match variant {
            Some(variant) => issue.with_variant(variant),
            None => issue,
        }
    };
    if !here
        .index
        .file(&page)
        .is_some_and(|f| f.kind == FileKind::Page)
    {
        return Some(Diagnostic::from_issue(&issue(None)));
    }
    if let Some(id) = id
        && here.index.page_heading(&page, id).is_none()
    {
        let issue = issue(Some("no-heading"))
            .with_arg("page", path.to_owned())
            .with_arg("id", id.to_owned());
        return Some(Diagnostic::from_issue(&issue));
    }
    // The builds that published the key, of those checked; or, without
    // them, any build.
    let published_in: Vec<&str> = base
        .iter()
        .flat_map(|b| &b.builds)
        .filter(|b| {
            b.addresses
                .iter()
                .any(|a| a.source == key || a.address == key)
        })
        .map(|b| b.build.as_str())
        .filter(|name| here.builds.iter().any(|b| b.build == *name))
        .collect();
    let missing: Vec<String> = if published_in.is_empty() {
        if here.publishes(path) {
            Vec::new()
        } else {
            return Some(Diagnostic::from_issue(
                &issue(Some("builds")).with_arg("builds", "no build publishes"),
            ));
        }
    } else {
        here.builds
            .iter()
            .filter(|b| published_in.contains(&b.build.as_str()) && !b.publishes(&page))
            .map(|b| format!("`{}`", b.build))
            .collect()
    };
    if missing.is_empty() {
        return None;
    }
    let builds = format!(
        "the build{} {} {}",
        if missing.len() == 1 { "" } else { "s" },
        listed(&missing),
        if missing.len() == 1 {
            "doesn't publish"
        } else {
            "don't publish"
        }
    );
    Some(Diagnostic::from_issue(
        &issue(Some("builds")).with_arg("builds", builds),
    ))
}

/// `addresses-chain` for `key`, forwarded to `value`, when `value` is
/// another entry's key, or a heading on a page that's one; with the safe
/// fix that forwards it to where the chain ends, or retires it for the
/// reason the chain ends at. A cycle has no fix.
fn chain(here: &Here<'_>, key: &str, value: &str, at: Location) -> Option<Diagnostic> {
    let file = here.file;
    let next = |value: &str| file.step(value);
    let first = next(value)?;
    let issue = Issue::new(diagnostics::ADDRESSES_CHAIN, at)
        .with_arg("key", key.to_owned())
        .with_arg("value", value.to_owned());
    let mut seen: HashSet<String> = HashSet::from([key.to_owned()]);
    let mut current = value.to_owned();
    let mut after = file.clone();
    let end = loop {
        if !seen.insert(current.clone()) {
            break None;
        }
        match next(&current) {
            Some(Step::Moved(to)) => current = to,
            Some(Step::Retired(reason)) => {
                after.record_retired(key, &reason);
                break Some(());
            }
            None => {
                after
                    .moved
                    .insert(key.to_owned(), Entry::new(current.clone()));
                break Some(());
            }
        }
    };
    let issue = match (&first, end) {
        (_, None) => issue.with_variant("cycle"),
        (Step::Retired(_), _) => issue.with_variant("retired"),
        (Step::Moved(to), _) => issue.with_arg("next", to.clone()),
    };
    let issue = match end.and_then(|()| {
        here.fix(
            "Forward it to where the chain ends".to_owned(),
            &after,
            true,
        )
    }) {
        Some(fix) => issue.with_fix(fix),
        None => issue,
    };
    Some(Diagnostic::from_issue(&issue))
}

/// `addresses-anchor-collision` for `key`, an id on a page that's gone,
/// forwarded to `value`: the page its page's addresses lead to has a live
/// heading with the same id, which a link to it lands on instead.
fn anchor_collision(here: &Here<'_>, key: &str, value: &str, at: Location) -> Option<Diagnostic> {
    let Some(Key::Id { path, id }) = Key::parse(key) else {
        return None;
    };
    if RelPath::parse(path).is_ok_and(|p| here.index.file(&p).is_some()) {
        return None;
    }
    let (to, _) = split_place(value)?;
    // Where the page's own addresses lead: its entry's page, or else where
    // the id was forwarded.
    let lands = match here
        .file
        .moved
        .get(path)
        .and_then(|e| split_place(&e.value))
    {
        Some((page, None)) => page,
        _ => to,
    };
    if value == format!("{lands}#{id}") {
        return None;
    }
    let live = here.builds.iter().flat_map(|b| &b.addresses).any(|a| {
        a.kind == AddressKind::Anchor
            && a.page.as_str() == lands
            && a.address.ends_with(&format!("#{id}"))
    });
    if !live {
        return None;
    }
    let issue = Issue::new(diagnostics::ADDRESSES_ANCHOR_COLLISION, at)
        .with_arg("key", key.to_owned())
        .with_arg("value", value.to_owned())
        .with_arg("id", id.to_owned())
        .with_arg("page", lands.to_owned());
    Some(Diagnostic::from_issue(&issue))
}

/// `addresses-soft-404` for each index page that [`SOFT_404`] or more
/// `[moved]` entries forward to, at the first of them.
fn soft_404s(here: &Here<'_>) -> Vec<Diagnostic> {
    let mut to: BTreeMap<&str, Vec<&Entry>> = BTreeMap::new();
    for (key, entry) in &here.file.moved {
        if matches!(Key::parse(key), Some(Key::Permalink(_))) {
            continue;
        }
        let value = entry.value.as_str();
        if value == "index.md" || value.ends_with("/index.md") {
            to.entry(value).or_default().push(entry);
        }
    }
    to.into_iter()
        .filter(|(_, entries)| entries.len() >= SOFT_404)
        .map(|(value, entries)| {
            let first = entries
                .iter()
                .map(|e| e.span)
                .min_by_key(|s| s.start())
                .unwrap_or(Span::empty(0));
            let what = if value == "index.md" {
                "the docs' home page"
            } else {
                "a section's index page"
            };
            let issue = Issue::new(
                diagnostics::ADDRESSES_SOFT_404,
                Location::new(ADDRESSES_FILE_ID, first),
            )
            .with_arg("count", entries.len().to_string())
            .with_arg("value", value.to_owned())
            .with_arg("what", what);
            Diagnostic::from_issue(&issue)
        })
        .collect()
}
