//! `_ascribe/permalinks.json`: the permalinks a build publishes, for a site
//! to give each an address that forwards to its page or heading
//! (output-layout contract §1.2): each declared permalink, each alias of
//! one, each moved permalink whose page the build publishes, and each
//! retired permalink, which shows why it was removed.

use std::collections::BTreeMap;

use ascribe_model::{ContentModel, PermalinkRecords};
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
    /// For an alias, the name it leads to, as its record writes it.
    #[serde(skip_serializing_if = "Option::is_none")]
    alias: Option<&'a str>,
    /// For a permalink retired with a reason that still leads somewhere,
    /// why it was retired: one `ascribe.toml` retires with a `to`, until
    /// it's moved to `ascribe.addresses`, where a moved permalink has no
    /// reason.
    #[serde(skip_serializing_if = "Option::is_none")]
    retired: Option<&'a str>,
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
    /// An alias, and the permalink it ends at.
    Alias {
        /// The name it leads to, as its record writes it.
        written: &'a str,
        /// The permalink at the end of its chain.
        target: &'a Permalink,
    },
    /// A moved permalink, or one retired with a `to`, whose `to` the build
    /// publishes.
    Retired {
        /// Where `to` leads.
        to: Destination,
        /// Why it was retired; none for one moved.
        reason: Option<&'a str>,
    },
    /// A retired permalink with nowhere to lead in this build.
    Removed {
        /// Why it was retired.
        reason: &'a str,
    },
}

/// Every name the build answers to, from the permalinks it publishes
/// (`found`, [`ResolvedBuild::permalinks`]) and the project's records of
/// its permalinks (`records`, [`ascribe_resolve::Project::permalink_records`]):
/// a name published twice, which `permalink-duplicate` reports, with its
/// first page; a retired or moved name, unless it's declared too
/// (`permalink-name-conflict`, where the declaration wins), a moved one only
/// when the build publishes its page; and an alias, unless its name is
/// taken, when the build publishes the permalink it ends at.
pub(super) fn named<'a>(
    records: &'a PermalinkRecords,
    build: &ResolvedBuild,
    found: &'a [Permalink],
) -> BTreeMap<&'a str, Named<'a>> {
    let mut out: BTreeMap<&str, Named<'_>> = BTreeMap::new();
    for permalink in found {
        out.entry(permalink.name.as_str())
            .or_insert(Named::Declared(permalink));
    }
    let names = records;
    for retired in &names.retired {
        if out.contains_key(retired.name.as_str()) {
            continue;
        }
        let reason = retired.reason.as_deref();
        let to = retired
            .to
            .as_deref()
            .and_then(|to| build.retired_destination(to));
        let named = match (to, reason) {
            (Some(to), reason) => Named::Retired { to, reason },
            (None, Some(reason)) => Named::Removed { reason },
            (None, None) => continue,
        };
        out.insert(retired.name.as_str(), named);
    }
    for alias in &names.aliases {
        if out.contains_key(alias.name.as_str()) {
            continue;
        }
        let Some(end) = names.alias_end(&alias.name) else {
            continue;
        };
        let Some(target) = found.iter().find(|p| p.name == end) else {
            continue;
        };
        out.insert(
            alias.name.as_str(),
            Named::Alias {
                written: alias.target.as_str(),
                target,
            },
        );
    }
    out
}

/// The list, as JSON. A build with no permalinks has an empty table, so a
/// reader can tell "none" from an output without the file. Each name is
/// listed as [`named`] finds it: an alias with the page and anchor of the
/// permalink it ends at; a moved permalink with its `to`'s; and a retired
/// one under `removed`, or, retired with a `to` the build publishes, with
/// its `to`'s. `dimensions` is the content model's dimensions, each with its
/// values in declaration order: the parameters an address takes.
pub(super) fn list(
    model: &ContentModel,
    records: &PermalinkRecords,
    build: &ResolvedBuild,
) -> String {
    let found = build.permalinks();
    let mut permalinks: BTreeMap<&str, Entry<'_>> = BTreeMap::new();
    let mut removed = BTreeMap::new();
    for (name, named) in named(records, build, &found) {
        let entry = match named {
            Named::Declared(permalink) => Entry {
                route: permalink.route.clone(),
                fragment: permalink.anchor.clone(),
                source: permalink.page.to_string(),
                alias: None,
                retired: None,
            },
            Named::Alias { written, target } => Entry {
                route: target.route.clone(),
                fragment: target.anchor.clone(),
                source: target.page.to_string(),
                alias: Some(written),
                retired: None,
            },
            Named::Retired { to, reason } => Entry {
                route: to.route,
                fragment: to.anchor,
                source: to.page.to_string(),
                alias: None,
                retired: reason,
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
