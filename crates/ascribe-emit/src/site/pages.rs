//! `_ascribe/pages.json`: the build's page records
//! ([`ascribe_resolve::pages`]), for a site to write each page's head, its
//! sitemap, and its `robots.txt` from (output-layout contract §1.3).

use std::collections::BTreeMap;

use ascribe_model::ContentModel;
use ascribe_resolve::ResolvedBuild;
use ascribe_resolve::pages::{PageRecord, records};
use serde::Serialize;

/// The version of the file's format.
const VERSION: u32 = 1;

#[derive(Serialize)]
struct List<'a> {
    version: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    site: Option<&'a str>,
    pages: BTreeMap<&'a str, Entry<'a>>,
    robots: Robots<'a>,
}

#[derive(Serialize)]
struct Entry<'a> {
    source: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    title: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    canonical: Option<&'a str>,
    listing: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    markdown: Option<&'a str>,
}

#[derive(Serialize)]
struct Robots<'a> {
    lines: &'a [String],
}

/// The records, as JSON: every page the build publishes, keyed by its route
/// and sorted by it, and the project's `[crawlers] lines` as a host reads
/// them. A build with no pages has an empty table, so a reader can tell
/// "none" from an output without the file. Two pages with one route, which
/// the site output refuses, are listed with the first.
pub(super) fn list(model: &ContentModel, build: &ResolvedBuild) -> String {
    let records = records(model, &build.pages);
    let mut pages = BTreeMap::new();
    for record in &records.pages {
        pages
            .entry(record.route.as_str())
            .or_insert_with(|| entry(record));
    }
    let list = List {
        version: VERSION,
        site: records.site.as_deref(),
        pages,
        robots: Robots {
            lines: &records.robots_lines,
        },
    };
    // Serializing strings and maps of strings can't fail.
    let mut text = serde_json::to_string_pretty(&list).unwrap_or_default();
    text.push('\n');
    text
}

fn entry(record: &PageRecord) -> Entry<'_> {
    Entry {
        source: record.source.as_str(),
        title: record.title.as_deref(),
        description: record.description.as_deref(),
        canonical: record.canonical.as_deref(),
        listing: record.listing.name(),
        markdown: record.markdown.as_deref(),
    }
}
