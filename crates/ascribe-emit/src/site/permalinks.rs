//! `_ascribe/permalinks.json`: the permalinks a build publishes, for a site
//! to give each an address that forwards to its page or heading
//! (output-layout contract §1.2): each declared permalink, and each name
//! `ascribe.addresses` moves or retires, which forwards to where it was
//! moved or shows why it was removed.

use std::collections::BTreeMap;

use ascribe_model::{ContentModel, End, PERMALINK_PREFIX, RecordedPermalinks};
use ascribe_resolve::{Destination, Permalink, ResolvedBuild};
use serde::Serialize;

/// The version of the list's format.
const VERSION: u32 = 1;

#[derive(Serialize)]
struct List<'a> {
    version: u32,
    path: String,
    dimensions: BTreeMap<&'a str, Vec<&'a str>>,
    permalinks: BTreeMap<&'a str, Entry<'a>>,
    /// The retired permalinks with nowhere to lead in this build: written
    /// only when there's one.
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    removed: BTreeMap<&'a str, Removed<'a>>,
}

#[derive(Clone, Serialize)]
struct Entry<'a> {
    route: String,
    /// The heading's anchor; none for a page's own permalink.
    #[serde(skip_serializing_if = "Option::is_none")]
    fragment: Option<String>,
    source: String,
    /// For a name moved to another permalink, that permalink's name, as
    /// `ascribe.addresses` writes it after `go:`.
    #[serde(skip_serializing_if = "Option::is_none")]
    alias: Option<&'a str>,
    /// For a name moved to a page, where, as `ascribe.addresses` writes it.
    #[serde(skip_serializing_if = "Option::is_none")]
    moved: Option<&'a str>,
}

#[derive(Serialize)]
struct Removed<'a> {
    reason: &'a str,
}

/// What a name the build answers to leads to.
pub(super) enum Named<'a> {
    /// A permalink the build publishes: the first, for a name it publishes
    /// twice.
    Declared(&'a Permalink),
    /// A name moved to another permalink, and the permalink it ends at.
    Alias {
        /// The name it was moved to, as `ascribe.addresses` writes it after
        /// `go:`.
        written: &'a str,
        /// The permalink at the end of its chain.
        target: &'a Permalink,
    },
    /// A name moved to a page the build publishes.
    Moved {
        /// Where it leads.
        to: Destination,
        /// Where it was moved, as `ascribe.addresses` writes it.
        written: &'a str,
    },
    /// A retired name, or one moved to a name that's retired.
    Removed {
        /// Why it was retired.
        reason: &'a str,
    },
}

/// Every name the build answers to, from the permalinks it publishes
/// (`found`, [`ResolvedBuild::permalinks`]) and what `ascribe.addresses`
/// records (`recorded`): a name published twice, which
/// `permalink-duplicate` reports, with its first page; a retired name; a
/// name moved to a page, when the build publishes it; and one moved to
/// another permalink, when the build publishes the permalink it ends at.
/// A name the content declares is the content's, whatever the file says of
/// it (`addresses-entry-live`).
pub(super) fn named<'a>(
    recorded: &'a RecordedPermalinks,
    build: &ResolvedBuild,
    found: &'a [Permalink],
) -> BTreeMap<&'a str, Named<'a>> {
    let mut out: BTreeMap<&str, Named<'_>> = BTreeMap::new();
    for permalink in found {
        out.entry(permalink.name.as_str())
            .or_insert(Named::Declared(permalink));
    }
    for entry in &recorded.names {
        if out.contains_key(entry.name.as_str()) {
            continue;
        }
        let named = match &entry.end {
            End::Retired(reason) => Named::Removed { reason },
            End::Cycle => continue,
            End::Permalink(end) => {
                let Some(target) = found.iter().find(|p| p.name == *end) else {
                    continue;
                };
                Named::Alias {
                    written: entry.alias_of().unwrap_or(end),
                    target,
                }
            }
            End::Place(place) => {
                let Some(to) = build.retired_destination(place) else {
                    continue;
                };
                Named::Moved {
                    to,
                    written: entry.value.as_str(),
                }
            }
        };
        out.insert(entry.name.as_str(), named);
    }
    out
}

/// The list, as JSON. A build with no permalinks has an empty table, so a
/// reader can tell "none" from an output without the file. Each name is
/// listed as [`named`] finds it: a name moved to another permalink with the
/// page and anchor of the permalink it ends at, one moved to a page with
/// that page's, and a retired one under `removed`. `dimensions` is the
/// content model's dimensions, each with its values in declaration order:
/// the parameters an address takes.
pub(super) fn list(
    model: &ContentModel,
    recorded: &RecordedPermalinks,
    build: &ResolvedBuild,
) -> String {
    let found = build.permalinks();
    let mut permalinks: BTreeMap<&str, Entry<'_>> = BTreeMap::new();
    let mut removed = BTreeMap::new();
    for (name, named) in named(recorded, build, &found) {
        let entry = match named {
            Named::Declared(permalink) => Entry {
                route: permalink.route.clone(),
                fragment: permalink.anchor.clone(),
                source: permalink.page.to_string(),
                alias: None,
                moved: None,
            },
            Named::Alias { written, target } => Entry {
                route: target.route.clone(),
                fragment: target.anchor.clone(),
                source: target.page.to_string(),
                alias: Some(written),
                moved: None,
            },
            Named::Moved { to, written } => Entry {
                route: to.route,
                fragment: to.anchor,
                source: to.page.to_string(),
                alias: written.strip_prefix(PERMALINK_PREFIX),
                moved: (!written.starts_with(PERMALINK_PREFIX)).then_some(written),
            },
            Named::Removed { reason } => {
                removed.insert(name, Removed { reason });
                continue;
            }
        };
        permalinks.insert(name, entry);
    }
    let list = List {
        version: VERSION,
        path: model.consumer.permalink_path(),
        dimensions: model
            .dimensions
            .iter()
            .map(|dimension| {
                let values = dimension.values.iter().map(|v| v.value.as_str()).collect();
                (dimension.name.as_str(), values)
            })
            .collect(),
        permalinks,
        removed,
    };
    // Serializing strings and maps of strings can't fail.
    let mut text = serde_json::to_string_pretty(&list).unwrap_or_default();
    text.push('\n');
    text
}
