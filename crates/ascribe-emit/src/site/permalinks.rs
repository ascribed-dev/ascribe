//! `_ascribe/permalinks.json`: the permalinks a build publishes, for a site
//! to give each an address that forwards to its page or heading
//! (output-layout contract §1.2): each declared permalink, each alias of
//! one, and each retired permalink, which forwards to its `to` or shows why
//! it was removed.

use std::collections::BTreeMap;

use ascribe_model::ContentModel;
use ascribe_resolve::ResolvedBuild;
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
    /// For an alias, the name it leads to, as `ascribe.toml` writes it.
    #[serde(skip_serializing_if = "Option::is_none")]
    alias: Option<&'a str>,
    /// For a retired permalink, why it was retired.
    #[serde(skip_serializing_if = "Option::is_none")]
    retired: Option<&'a str>,
}

#[derive(Serialize)]
struct Removed<'a> {
    reason: &'a str,
}

/// The list, as JSON. A build with no permalinks has an empty table, so a
/// reader can tell "none" from an output without the file. A name the build
/// publishes twice, which `permalink-duplicate` reports, is listed with its
/// first page. An alias is listed when the build publishes the permalink it
/// ends at, with that permalink's page and anchor; a retired permalink with
/// its `to`'s when the build publishes that page, and under `removed`
/// otherwise. `dimensions` is the content model's dimensions, each with its
/// values in declaration order: the parameters an address takes.
pub(super) fn list(model: &ContentModel, build: &ResolvedBuild) -> String {
    let found = build.permalinks();
    let mut permalinks: BTreeMap<&str, Entry<'_>> = BTreeMap::new();
    for permalink in &found {
        permalinks
            .entry(permalink.name.as_str())
            .or_insert_with(|| Entry {
                route: permalink.route.clone(),
                fragment: permalink.anchor.clone(),
                source: permalink.page.to_string(),
                alias: None,
                retired: None,
            });
    }
    let names = &model.permalinks;
    let mut removed = BTreeMap::new();
    for retired in &names.retired {
        if permalinks.contains_key(retired.name.as_str()) {
            // Declared too: `permalink-name-conflict`, and the declaration
            // wins.
            continue;
        }
        let to = retired
            .to
            .as_deref()
            .and_then(|to| build.retired_destination(to));
        match to {
            Some(to) => {
                permalinks.insert(
                    retired.name.as_str(),
                    Entry {
                        route: to.route,
                        fragment: to.anchor,
                        source: to.page.to_string(),
                        alias: None,
                        retired: Some(retired.reason.as_str()),
                    },
                );
            }
            None => {
                removed.insert(
                    retired.name.as_str(),
                    Removed {
                        reason: retired.reason.as_str(),
                    },
                );
            }
        }
    }
    for alias in &names.aliases {
        if permalinks.contains_key(alias.name.as_str()) || removed.contains_key(alias.name.as_str())
        {
            continue;
        }
        let Some(end) = names.alias_end(&alias.name) else {
            continue;
        };
        let Some(target) = found.iter().find(|p| p.name == end) else {
            continue;
        };
        permalinks.insert(
            alias.name.as_str(),
            Entry {
                route: target.route.clone(),
                fragment: target.anchor.clone(),
                source: target.page.to_string(),
                alias: Some(alias.target.as_str()),
                retired: None,
            },
        );
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
