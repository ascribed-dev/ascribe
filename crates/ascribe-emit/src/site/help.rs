//! `_ascribe/help.json`: each permalink's section of the docs, for a
//! product to show where its user is, in a tooltip, a panel, or beside an
//! error (output-layout contract §1.7): its title, its summary, its text as
//! the plain output writes it, and its address. A section with a switcher
//! also has its text with each of the switcher's arms chosen. [`help`] is
//! one name's entry, as the file writes it, for `ascribe resolve go:<name>`.
//! Each entry is also an address of its own, `<folder><name>.json`
//! ([`entry_paths`]), which the headers give to any origin.

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
    help: BTreeMap<&'a str, HelpEntry>,
}

/// A permalink's entry in `_ascribe/help.json`: its section of the docs.
#[derive(Clone, Debug, Serialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
pub struct HelpEntry {
    /// The heading's text, or the page's title for a page's permalink, as
    /// plain text.
    pub title: String,
    /// The section's summary; absent when it has none.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    /// The section as the plain output writes it, in Markdown, with each
    /// image as its alt text.
    pub text: String,
    /// For a section with a switcher, its text with each arm chosen: keyed
    /// by dimension, then by value, both in the content model's order.
    /// Absent when the section has no switcher on a dimension.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(
        feature = "json-schema",
        schemars(with = "Option<BTreeMap<String, BTreeMap<String, String>>>")
    )]
    pub choices: Option<Choices>,
    /// The permalink's address; absent without `[consumer] site`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address: Option<String>,
    /// The page the permalink is recorded on, as a path from the content
    /// root.
    pub source: String,
}

/// What a build's `help.json` has under a name.
#[derive(Clone, Debug)]
pub enum Help {
    /// The name's entry.
    Entry(HelpEntry),
    /// A retired permalink, which has no entry.
    Retired {
        /// Why it was retired, as `ascribe.toml` writes it.
        reason: String,
        /// Where its address leads now, as `ascribe.toml` writes it.
        to: Option<String>,
    },
    /// No name the build answers to.
    Missing {
        /// The names the file has an entry for, in order.
        names: Vec<String>,
    },
}

/// The file's header for the build: the format's version, the build's name,
/// and its line, for a line of a versioned build that has more than one.
pub fn help_header<'a>(cx: &EmitContext<'a>) -> (u32, &'a str, Option<&'a str>) {
    let declared = cx.build;
    // A versioned build with one live line writes what it would unversioned.
    let line = declared
        .line
        .as_ref()
        .filter(|line| !(line.current && line.oldest.text == line.line.text));
    (
        VERSION,
        declared
            .line
            .as_ref()
            .map_or(declared.name.as_str(), |line| line.build.as_str()),
        line.map(|line| line.line.text.as_str()),
    )
}

/// What the build's `help.json` has under `name`: the entry the file
/// writes, the reason a retired permalink was retired, or the names it has.
pub fn help(cx: &EmitContext<'_>, build: &ResolvedBuild, name: &str) -> Help {
    let model = cx.model;
    let plain = PlainEmitter::new(model);
    let found = build.permalinks();
    let named = named(model, build, &found);
    let permalink = match named.get(name) {
        Some(
            Named::Declared(permalink)
            | Named::Alias {
                target: permalink, ..
            },
        ) => Some(*permalink),
        Some(Named::Retired { reason, .. } | Named::Removed { reason }) => {
            return Help::Retired {
                reason: (*reason).to_owned(),
                to: model
                    .permalinks
                    .retired(name)
                    .and_then(|retired| retired.to.clone()),
            };
        }
        None => None,
    };
    if let Some(entry) = permalink.and_then(|p| entry(cx, &plain, build, p, name)) {
        return Help::Entry(entry);
    }
    let names = named
        .into_iter()
        .filter(|(_, named)| matches!(named, Named::Declared(_) | Named::Alias { .. }))
        .map(|(name, _)| name.to_owned())
        .collect();
    Help::Missing { names }
}

/// The entries, by name, for each name the build answers to ([`named`]):
/// a permalink's own, and an alias's, which is the entry of the permalink it
/// ends at under its own name. A retired permalink has none.
pub(super) fn entries<'a>(
    cx: &EmitContext<'a>,
    build: &ResolvedBuild,
    found: &'a [Permalink],
) -> BTreeMap<&'a str, HelpEntry> {
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
pub(super) fn file(cx: &EmitContext<'_>, help: BTreeMap<&str, HelpEntry>) -> String {
    let (version, build, line) = help_header(cx);
    let file = File {
        version,
        build,
        line,
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
) -> Option<HelpEntry> {
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
    Some(HelpEntry {
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
) -> Option<Choices> {
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

/// A section's text with each arm chosen: each dimension, with each value's
/// text.
pub type Choices = Ordered<Ordered<String>>;

/// A table written in the order of its entries.
#[derive(Clone, Debug)]
pub struct Ordered<T>(pub Vec<(String, T)>);

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
