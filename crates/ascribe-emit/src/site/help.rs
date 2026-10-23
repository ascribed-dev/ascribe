//! `_ascribe/help.json`: each permalink's section of the docs, for a
//! product to show where its user is, in a tooltip, a panel, or beside an
//! error (output-layout contract §1.7): its title, its summary, its text as
//! the plain output writes it, and its address. Each entry is also an
//! address of its own, `<folder><name>.json` ([`entry_paths`]), which the
//! headers give to any origin.

use std::collections::BTreeMap;

use ascribe_model::{ContentModel, TrailingSlash};
use ascribe_resolve::{Permalink, ResolvedBuild};
use serde::Serialize;

use super::permalinks::{Named, named};
use crate::emitter::{EmitContext, Emitter, PageContext};
use crate::plain::{PlainEmitter, render_section};

/// The version of the file's format.
const VERSION: u32 = 1;

#[derive(Serialize)]
struct File<'a> {
    version: u32,
    build: &'a str,
    /// The line, for a line of a versioned build.
    #[serde(skip_serializing_if = "Option::is_none")]
    line: Option<&'a str>,
    help: BTreeMap<&'a str, Entry>,
}

#[derive(Serialize)]
pub(super) struct Entry {
    title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    summary: Option<String>,
    text: String,
    /// The permalink's address, with `[consumer] site`.
    #[serde(skip_serializing_if = "Option::is_none")]
    address: Option<String>,
    source: String,
}

/// The entries, by name, for each name the build answers to ([`named`]):
/// a permalink's own, and an alias's, which is the entry of the permalink it
/// ends at under its own name. A retired permalink has none.
pub(super) fn entries<'a>(
    cx: &EmitContext<'a>,
    build: &ResolvedBuild,
    found: &'a [Permalink],
) -> BTreeMap<&'a str, Entry> {
    let model = cx.model;
    let plain = PlainEmitter::new(model);
    let mut help = BTreeMap::new();
    for (name, named) in named(model, build, found) {
        let permalink = match named {
            Named::Declared(permalink) => permalink,
            Named::Alias { target, .. } => target,
            Named::Retired { .. } | Named::Removed { .. } => continue,
        };
        if let Some(entry) = entry(cx, &plain, build, permalink, name) {
            help.insert(name, entry);
        }
    }
    help
}

/// The file, as JSON, with `help`'s entries. A build with no permalinks has
/// an empty table.
pub(super) fn file(cx: &EmitContext<'_>, help: BTreeMap<&str, Entry>) -> String {
    let declared = &cx.build;
    // A versioned build with one live line writes what it would unversioned.
    let line = declared
        .line
        .as_ref()
        .filter(|line| !(line.current && line.oldest.text == line.line.text));
    let file = File {
        version: VERSION,
        build: declared
            .line
            .as_ref()
            .map_or(declared.name.as_str(), |line| line.build.as_str()),
        line: line.map(|line| line.line.text.as_str()),
        help,
    };
    // Serializing strings and maps of strings can't fail.
    let mut text = serde_json::to_string_pretty(&file).unwrap_or_default();
    text.push('\n');
    text
}

/// The path, from the host's root, each of `names`' entries is served at on
/// its own: `<folder><name>.json`. The folder is the permalinks' path for
/// the current line and a build that isn't versioned, and `go/` under an
/// older line's base path, `/docs/3.4/go/`, since the permalinks' addresses
/// are the current line's.
pub(super) fn entry_paths<'a>(
    cx: &EmitContext<'_>,
    names: impl IntoIterator<Item = &'a str>,
) -> Vec<String> {
    let consumer = &cx.model.consumer;
    let folder = match cx.build.line_folder() {
        Some(_) => format!("{}/go/", consumer.base_path.trim_end_matches('/')),
        None => consumer.permalink_path(),
    };
    names
        .into_iter()
        .map(|name| format!("{folder}{name}.json"))
        .collect()
}

/// The entry of `permalink` under `name`: its section rendered as the plain
/// output writes it, read from the page it's recorded on.
fn entry(
    cx: &EmitContext<'_>,
    plain: &PlainEmitter,
    build: &ResolvedBuild,
    permalink: &Permalink,
    name: &str,
) -> Option<Entry> {
    let (page, section) = build.section(permalink)?;
    let page_cx = PageContext::new(cx, plain.page_path(&page.path), plain);
    let rendered = render_section(&page_cx, page, &section);
    let title = match section.heading.and_then(|h| h.heading.as_ref()) {
        Some(ids) => ids.text.clone(),
        None => page.title.clone().unwrap_or_default(),
    };
    Some(Entry {
        title,
        summary: rendered.summary,
        text: rendered.text,
        address: address(cx.model, name),
        source: permalink.page.to_string(),
    })
}

/// A permalink's address: the site's origin, the permalink path, and the
/// name, ending as `trailing-slash` says. `None` without `[consumer] site`.
fn address(model: &ContentModel, name: &str) -> Option<String> {
    let consumer = &model.consumer;
    let origin = consumer.site.as_deref()?.trim_end_matches('/');
    let slash = match consumer.trailing_slash {
        TrailingSlash::Always => "/",
        TrailingSlash::Never => "",
    };
    Some(format!(
        "{origin}{}{name}{slash}",
        consumer.permalink_path()
    ))
}
