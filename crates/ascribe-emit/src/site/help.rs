//! `_ascribe/help.json`: each permalink's section of the docs, for a
//! product to show where its user is, in a tooltip, a panel, or beside an
//! error (output-layout contract §1.7): its title, its summary, its text as
//! the plain output writes it, and its address. A section with a switcher
//! also has its text with each of the switcher's arms chosen.

use std::collections::BTreeMap;

use ascribe_model::{ContentModel, TrailingSlash};
use ascribe_resolve::{PageSection, Permalink, ResolvedBlock, ResolvedBuild, ResolvedKind, select};
use serde::Serialize;
use serde::ser::{SerializeMap, Serializer};

use super::arm_values;
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
struct Entry {
    title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    summary: Option<String>,
    text: String,
    /// For a section with a switcher, its text with each arm chosen.
    #[serde(skip_serializing_if = "Option::is_none")]
    choices: Option<Ordered<Ordered<String>>>,
    /// The permalink's address, with `[consumer] site`.
    #[serde(skip_serializing_if = "Option::is_none")]
    address: Option<String>,
    source: String,
}

/// The file, as JSON, with an entry for each name the build answers to
/// ([`named`]): a permalink's own, and an alias's, which is the entry of the
/// permalink it ends at under its own name. A retired permalink has none. A
/// build with no permalinks has an empty table.
pub(super) fn file(cx: &EmitContext<'_>, build: &ResolvedBuild) -> String {
    let model = cx.model;
    let plain = PlainEmitter::new(model);
    let found = build.permalinks();
    let mut help = BTreeMap::new();
    for (name, named) in named(model, build, &found) {
        let permalink = match named {
            Named::Declared(permalink) => permalink,
            Named::Alias { target, .. } => target,
            Named::Retired { .. } | Named::Removed { .. } => continue,
        };
        if let Some(entry) = entry(cx, &plain, build, permalink, name) {
            help.insert(name, entry);
        }
    }
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
    let choices = choices(cx.model, &section, |blocks| {
        let chosen = PageSection {
            heading: section.heading,
            blocks,
            arms: section.arms.clone(),
        };
        render_section(&page_cx, page, &chosen).text
    });
    let title = match section.heading.and_then(|h| h.heading.as_ref()) {
        Some(ids) => ids.text.clone(),
        None => page.title.clone().unwrap_or_default(),
    };
    Some(Entry {
        title,
        summary: rendered.summary,
        text: rendered.text,
        choices,
        address: address(cx.model, name),
        source: permalink.page.to_string(),
    })
}

/// A section's text with each value of each dimension its switchers are on
/// chosen, as a build that selects that value alone writes it: keyed by
/// dimension, then by value, both in the content model's order, each
/// `render`ed from the section's blocks with that value selected. A
/// switcher's dimensions are those its arms name; a value is one an arm
/// names, alone or in a set. A labeled group isn't along a dimension, so it
/// adds none and stays as it is in each. `None` when the section has no
/// switcher on a dimension.
fn choices(
    model: &ContentModel,
    section: &PageSection<'_>,
    render: impl Fn(&[ResolvedBlock]) -> String,
) -> Option<Ordered<Ordered<String>>> {
    let mut named: Vec<(String, String)> = Vec::new();
    for block in section.blocks {
        block.visit(&mut |b| {
            let ResolvedKind::Group { name, arms, .. } = &b.kind else {
                return;
            };
            if name != "variant" || arms.len() < 2 {
                return;
            }
            for arm in arms {
                for (dimension, values) in arm_values(model, &arm.opener) {
                    for value in values {
                        let pair = (dimension.clone(), value);
                        if !named.contains(&pair) {
                            named.push(pair);
                        }
                    }
                }
            }
        });
    }
    let choices: Vec<_> = model
        .dimensions
        .iter()
        .filter_map(|dimension| {
            let values: Vec<_> = dimension
                .values
                .iter()
                .filter(|v| {
                    named
                        .iter()
                        .any(|(d, n)| *d == dimension.name && *n == v.value)
                })
                .map(|v| {
                    let selection = [(dimension.name.clone(), vec![v.value.clone()])];
                    let chosen = select(section.blocks, &selection, model);
                    (v.value.clone(), render(&chosen))
                })
                .collect();
            (!values.is_empty()).then(|| (dimension.name.clone(), Ordered(values)))
        })
        .collect();
    (!choices.is_empty()).then_some(Ordered(choices))
}

/// A table written in the order of its entries.
struct Ordered<T>(Vec<(String, T)>);

impl<T: Serialize> Serialize for Ordered<T> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(Some(self.0.len()))?;
        for (key, value) in &self.0 {
            map.serialize_entry(key, value)?;
        }
        map.end()
    }
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
