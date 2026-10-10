//! `_ascribe/permalinks.json`: the permalinks a build publishes, for a site
//! to give each an address that forwards to its page or heading
//! (output-layout contract §1.2).

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
    permalinks: BTreeMap<&'a str, Entry<'a>>,
}

#[derive(Serialize)]
struct Entry<'a> {
    route: &'a str,
    /// The heading's anchor; none for a page's own permalink.
    #[serde(skip_serializing_if = "Option::is_none")]
    fragment: Option<&'a str>,
    source: String,
}

/// The list, as JSON. A build with no permalinks has an empty table, so a
/// reader can tell "none" from an output without the file. A name the build
/// publishes twice, which `permalink-duplicate` reports, is listed with its
/// first page.
pub(super) fn list(model: &ContentModel, build: &ResolvedBuild) -> String {
    let found = build.permalinks();
    let mut permalinks = BTreeMap::new();
    for permalink in &found {
        permalinks
            .entry(permalink.name.as_str())
            .or_insert_with(|| Entry {
                route: &permalink.route,
                fragment: permalink.anchor.as_deref(),
                source: permalink.page.to_string(),
            });
    }
    let list = List {
        version: VERSION,
        path: model.consumer.permalink_path(),
        permalinks,
    };
    // Serializing strings and maps of strings can't fail.
    let mut text = serde_json::to_string_pretty(&list).unwrap_or_default();
    text.push('\n');
    text
}
