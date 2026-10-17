//! `_ascribe/nav.json`: the build's share of the project's navigation
//! ([`ascribe_resolve::navigation`]), for a site to draw its sidebar from
//! (output-layout contract §1.6, and the navigation contract).

use ascribe_resolve::{NavPage, Navigation, Project, ResolvedBuild, SharedSection};
use serde::Serialize;

use super::frontmatter::html;

/// The version of the file's format.
const VERSION: u32 = 1;

#[derive(Serialize)]
struct File<'a> {
    version: u32,
    root: Section<'a>,
}

#[derive(Serialize)]
struct Section<'a> {
    folder: &'a str,
    label: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    formatted: Option<Formatted>,
    #[serde(skip_serializing_if = "Option::is_none")]
    page: Option<Page<'a>>,
    pages: Vec<Page<'a>>,
    sections: Vec<Section<'a>>,
}

#[derive(Serialize)]
struct Page<'a> {
    source: &'a str,
    label: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    formatted: Option<Formatted>,
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    type_name: Option<&'a str>,
}

/// A label in the site output's formatted form, HTML, as a page's
/// `formatted.title` is.
#[derive(Serialize)]
struct Formatted {
    label: String,
}

/// The navigation, as JSON: the root section, each section with its folder,
/// its label, its own page, its pages, and its nested sections, in order,
/// and each page with its source path, its label, and its type. A build
/// with no pages has an empty root.
pub(super) fn list(project: &Project, navigation: &Navigation, build: &ResolvedBuild) -> String {
    let share = navigation.share(project, build);
    let file = File {
        version: VERSION,
        root: section(&share.root),
    };
    // Serializing strings and lists of them can't fail.
    let mut text = serde_json::to_string_pretty(&file).unwrap_or_default();
    text.push('\n');
    text
}

fn section<'a>(s: &'a SharedSection<'a>) -> Section<'a> {
    Section {
        folder: s.folder,
        label: &s.label,
        formatted: s.formatted.map(|segments| Formatted {
            label: html(segments),
        }),
        page: s.page.as_ref().map(page),
        pages: s.pages.iter().map(page).collect(),
        sections: s.sections.iter().map(section).collect(),
    }
}

fn page<'a>(p: &'a NavPage<'a>) -> Page<'a> {
    Page {
        source: p.page.path.as_str(),
        label: &p.label,
        formatted: p.formatted.map(|segments| Formatted {
            label: html(segments),
        }),
        type_name: p.type_name,
    }
}
