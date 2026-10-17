//! `_ascribe/nav.json`: the build's share of the project's navigation
//! ([`ascribe_resolve::navigation`]), for a site to draw its sidebar from
//! (output-layout contract §1.6, and the navigation contract), and in the
//! JSON output, for a consumer of its own.

use ascribe_model::Segment;
use ascribe_resolve::{NavPage, Navigation, Project, ResolvedBuild, SharedSection};
use serde::Serialize;

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

/// A label in the output's formatted form, as a page's `formatted.title`
/// is: HTML in the site output, pieces in the JSON output.
#[derive(Serialize)]
struct Formatted {
    label: serde_json::Value,
}

/// A label's formatted form in one output.
pub(crate) type Format = dyn Fn(&[Segment]) -> serde_json::Value;

/// The navigation, as JSON: the root section, each section with its folder,
/// its label, its own page, its pages, and its nested sections, in order,
/// and each page with its source path, its label, and its type. A build
/// with no pages has an empty root. `format` writes a label's formatted
/// form.
pub(crate) fn list(
    project: &Project,
    navigation: &Navigation,
    build: &ResolvedBuild,
    format: &Format,
) -> String {
    let share = navigation.share(project, build);
    let file = File {
        version: VERSION,
        root: section(&share.root, format),
    };
    // Serializing strings and lists of them can't fail.
    let mut text = serde_json::to_string_pretty(&file).unwrap_or_default();
    text.push('\n');
    text
}

fn section<'a>(s: &'a SharedSection<'a>, format: &Format) -> Section<'a> {
    Section {
        folder: s.folder,
        label: &s.label,
        formatted: s.formatted.map(|segments| Formatted {
            label: format(segments),
        }),
        page: s.page.as_ref().map(|p| page(p, format)),
        pages: s.pages.iter().map(|p| page(p, format)).collect(),
        sections: s.sections.iter().map(|n| section(n, format)).collect(),
    }
}

fn page<'a>(p: &'a NavPage<'a>, format: &Format) -> Page<'a> {
    Page {
        source: p.page.path.as_str(),
        label: &p.label,
        formatted: p.formatted.map(|segments| Formatted {
            label: format(segments),
        }),
        type_name: p.type_name,
    }
}
