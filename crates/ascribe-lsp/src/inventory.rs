//! The `ascribe/inventory` request: what a project has and how much each part
//! is used, and its navigation, for the editor's Pages and Content model
//! views. It answers from
//! the current snapshot, so it includes unsaved edits, and counts with the
//! search Find All References lists from ([`ascribe_resolve::Project::uses`]),
//! so each count is the length of that list.

use std::collections::{BTreeSet, HashMap};

use ascribe_core::{LineIndex, RelPath};
use ascribe_model::Listing;
use ascribe_resolve::{Project, Section, Usable};
use lsp_types::{Range, TextDocumentIdentifier};
use serde::{Deserialize, Serialize};

use crate::definition::{Entry, find_entry};
use crate::nav::Ctx;

/// The request's method name.
pub const METHOD: &str = "ascribe/inventory";

/// The parameters of `ascribe/inventory`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InventoryParams {
    /// Any of the project's files: a source file, `ascribe.toml`, or another
    /// file in the project's folder.
    pub text_document: TextDocumentIdentifier,
}

/// The answer to `ascribe/inventory`. A document that isn't one of the
/// project's files gets empty lists.
#[derive(Debug, Default, Serialize, PartialEq)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct InventoryResult {
    /// The `file:` URI of the project's `ascribe.toml`, which the ranges of
    /// declarations are in.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model_uri: Option<String>,
    /// The `file:` URI of the content root, which content paths are
    /// relative to.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_uri: Option<String>,
    /// The pages, by content path.
    pub pages: Vec<InventoryPage>,
    /// The fragments, by content path.
    pub fragments: Vec<InventoryFragment>,
    /// The content paths of the pages no other file links to or includes,
    /// other than index pages (`index.md`, in any folder). Without a
    /// navigation file a reader may still reach them, so this is a hint.
    pub orphans: Vec<String>,
    /// The project's navigation (SPEC §7.8), `ascribe.nav`'s order over the
    /// default: its root section. `null` for a document that isn't one of
    /// the project's files.
    pub navigation: Option<InventorySection>,
    /// The content model's entries: phrases, features, glossary terms,
    /// dimensions, note types, widgets, builds, then page content types,
    /// each kind in declaration order.
    pub model: Vec<InventoryEntry>,
}

/// A page.
#[derive(Debug, Serialize, PartialEq)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct InventoryPage {
    /// Its content path.
    pub path: String,
    /// Its title (frontmatter `title`).
    pub title: Option<String>,
    /// Its label where it's an entry in a list, as the navigation shows it:
    /// as plain text, its `nav-label`, else its title, else its file name
    /// without `.md`.
    pub label: String,
    /// The `nav-label` its frontmatter gives it, as written, when it gives
    /// one.
    pub nav_label: Option<String>,
    /// Its content type; `null` when no one type applies.
    #[serde(rename = "type")]
    pub content_type: Option<String>,
    /// How many links from other files, and includes, name it.
    pub incoming: usize,
    /// The builds that publish it, in declaration order: the ones This
    /// page's Builds row says publish it.
    pub builds: Vec<String>,
}

/// A section of the navigation: a folder and its pages, in order.
#[derive(Debug, Serialize, PartialEq)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct InventorySection {
    /// Its folder, from the content root, ending in `/`; `""` for the root.
    pub folder: String,
    /// Its label: `label` from `ascribe.nav`, else its own page's title as
    /// plain text, else its folder's name, and "Start" for the root.
    pub label: String,
    /// The `label` `ascribe.nav` gives it, when it gives one.
    pub label_set: Option<String>,
    /// The content path of its own page: its `index.md`, or the page beside
    /// the folder with its name, which is then one of its parent's `pages`
    /// too, and the section sits where it does.
    pub page: Option<String>,
    /// Its pages in order, its own page not among them.
    pub pages: Vec<InventoryNavPage>,
    /// Its nested sections: each whose own page is among `pages` sits
    /// there; the rest come after the pages, in this order.
    pub sections: Vec<InventorySection>,
    /// Whether `ascribe.nav` lists it.
    pub listed: bool,
    /// Whether `ascribe.nav` lists its pages, so a page it leaves out isn't
    /// placed.
    pub lists_pages: bool,
}

/// A page in a section of the navigation.
#[derive(Debug, Serialize, PartialEq)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct InventoryNavPage {
    /// Its content path, as `pages` has it.
    pub path: String,
    /// Whether it has its place: its section lists no pages, or lists it,
    /// or it's `unlisted`. One that isn't comes after those that are, and
    /// `nav-page-unplaced` reports it.
    pub placed: bool,
    /// Whether it's `unlisted`: no build's sidebar shows it, so it needs no
    /// place.
    pub unlisted: bool,
}

/// A fragment.
#[derive(Debug, Serialize, PartialEq)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct InventoryFragment {
    /// Its content path.
    pub path: String,
    /// The content paths of the files that include it, in order.
    pub included_by: Vec<String>,
}

/// An entry of the content model.
#[derive(Debug, Serialize, PartialEq)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct InventoryEntry {
    /// What kind of entry it is.
    pub kind: ModelKind,
    /// Its key, id, or name, as pages write it.
    pub key: String,
    /// Its label, name, or term, where it has one besides its key.
    pub label: Option<String>,
    /// How many places use it; `null` for a build, which pages don't name.
    /// For a content type, how many pages have it.
    pub uses: Option<usize>,
    /// Where it's declared: in `ascribe.toml`, or, for a glossary term, in
    /// `page`. `null` for a built-in note type, the implicit `page` type and
    /// `site` build, and an entry that can't be found there.
    #[cfg_attr(
        feature = "json-schema",
        schemars(with = "Option<crate::schema::LspRange>")
    )]
    pub declaration: Option<Range>,
    /// For a glossary term, the content path of the file its heading is
    /// written in, which `declaration` is a range in: its glossary page, or
    /// a fragment the page includes. `null` for an entry of `ascribe.toml`.
    pub page: Option<String>,
    /// A dimension's values, in declaration order; left out for every other
    /// kind.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub values: Vec<InventoryValue>,
}

/// A value of a dimension.
#[derive(Debug, Serialize, PartialEq)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct InventoryValue {
    /// The value, as pages write it.
    pub value: String,
    /// Its label, where it has one besides the value.
    pub label: Option<String>,
    /// Where the dimension's `values` in `ascribe.toml` has it, inside its
    /// quotes; `null` when it can't be found there.
    #[cfg_attr(
        feature = "json-schema",
        schemars(with = "Option<crate::schema::LspRange>")
    )]
    pub declaration: Option<Range>,
}

/// A kind of content model entry.
#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub enum ModelKind {
    /// A phrase.
    Phrase,
    /// A feature.
    Feature,
    /// A glossary term.
    Term,
    /// A dimension.
    Dimension,
    /// A note type.
    Note,
    /// A project widget.
    Widget,
    /// A build.
    Build,
    /// A page content type.
    Type,
}

/// The project's inventory.
pub(crate) fn inventory(ctx: &Ctx) -> InventoryResult {
    let snapshot = &ctx.snapshot;
    let counts = snapshot.use_counts();
    let count = |used: Usable| counts.get(&used).copied().unwrap_or(0);

    let mut pages: Vec<InventoryPage> = snapshot
        .pages()
        .map(|file| InventoryPage {
            path: file.path.to_string(),
            title: file.title.clone(),
            label: ascribe_resolve::file_label(file, &ctx.model),
            nav_label: file
                .frontmatter
                .as_ref()
                .and_then(|f| f.get(ascribe_resolve::NAV_LABEL))
                .and_then(|l| l.as_str())
                .map(str::to_owned),
            content_type: file.content_type(&ctx.model).map(|t| t.name.clone()),
            incoming: count(Usable::File(file.path.clone())),
            builds: crate::page_rows::builds(ctx, file)
                .into_iter()
                .filter(|b| b.left_out.is_none())
                .map(|b| b.name)
                .collect(),
        })
        .collect();
    pages.sort_by(|a, b| a.path.cmp(&b.path));
    let orphans = pages
        .iter()
        .filter(|p| p.incoming == 0 && !is_index(&p.path))
        .map(|p| p.path.clone())
        .collect();

    let mut fragments: Vec<InventoryFragment> = snapshot
        .fragments()
        .map(|file| InventoryFragment {
            path: file.path.to_string(),
            included_by: snapshot
                .includers(&file.path)
                .iter()
                .map(|edge| edge.file.to_string())
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect(),
        })
        .collect();
    fragments.sort_by(|a, b| a.path.cmp(&b.path));

    let model = model_entries(ctx, &counts, &pages);
    InventoryResult {
        model_uri: crate::uri::path_to_uri(&ctx.config).map(|uri| uri.as_str().to_owned()),
        content_uri: crate::uri::path_to_uri(&ctx.content_dir).map(|uri| uri.as_str().to_owned()),
        pages,
        fragments,
        orphans,
        navigation: Some(section(
            snapshot.project(),
            &snapshot.project().navigation().root,
        )),
        model,
    }
}

/// A section of the navigation, as the inventory has it.
fn section(project: &Project, section: &Section) -> InventorySection {
    let entry = project.nav_file().and_then(|f| f.section(&section.folder));
    let listed: Option<Vec<&str>> = entry
        .and_then(|e| e.pages.as_ref())
        .map(|pages| pages.iter().map(|(name, _)| name.as_str()).collect());
    InventorySection {
        folder: section.folder.clone(),
        label: section.label(project),
        label_set: section.label.clone(),
        page: section.page.as_ref().map(ToString::to_string),
        pages: section
            .pages
            .iter()
            .map(|page| {
                let unlisted = project
                    .file(page)
                    .is_some_and(|f| f.listing(project.model()) == Listing::Unlisted);
                InventoryNavPage {
                    path: page.to_string(),
                    placed: unlisted
                        || listed.as_ref().is_none_or(|names| {
                            names.contains(&page.file_name().unwrap_or_default())
                        }),
                    unlisted,
                }
            })
            .collect(),
        sections: section
            .sections
            .iter()
            .map(|nested| self::section(project, nested))
            .collect(),
        listed: section.listed,
        lists_pages: listed.is_some(),
    }
}

fn is_index(path: &str) -> bool {
    RelPath::parse(path).is_ok_and(|p| p.file_name() == Some("index.md"))
}

fn model_entries(
    ctx: &Ctx,
    counts: &HashMap<Usable, usize>,
    pages: &[InventoryPage],
) -> Vec<InventoryEntry> {
    let model = &ctx.model;
    let lines = LineIndex::new(&ctx.model_text);
    let entry =
        |kind, key: &str, label: Option<&str>, used: Option<Usable>, declared: Entry<'_>| {
            InventoryEntry {
                kind,
                key: key.to_owned(),
                label: label.map(str::to_owned),
                uses: used.map(|used| counts.get(&used).copied().unwrap_or(0)),
                declaration: find_entry(&ctx.model_text, &declared)
                    .map(|span| ctx.encoding.range(&lines, span)),
                page: None,
                values: Vec::new(),
            }
        };
    let mut out = Vec::new();
    for p in &model.phrases {
        out.push(entry(
            ModelKind::Phrase,
            &p.key,
            Some(&p.value),
            Some(Usable::Phrase(p.key.clone())),
            Entry::Phrase(&p.key),
        ));
    }
    for f in &model.features {
        out.push(entry(
            ModelKind::Feature,
            &f.key,
            Some(&f.name),
            Some(Usable::Feature(f.key.clone())),
            Entry::Feature(&f.key),
        ));
    }
    for t in ctx.snapshot.glossary().terms {
        let declaration = ctx
            .snapshot
            .file(&t.written_in)
            .map(|file| ctx.encoding.range(&LineIndex::new(&file.source), t.heading));
        out.push(InventoryEntry {
            kind: ModelKind::Term,
            key: t.id.clone(),
            label: Some(t.term.clone()),
            uses: Some(counts.get(&Usable::Term(t.id)).copied().unwrap_or(0)),
            declaration,
            page: Some(t.written_in.to_string()),
            values: Vec::new(),
        });
    }
    for d in &model.dimensions {
        let mut dimension = entry(
            ModelKind::Dimension,
            &d.name,
            Some(d.label.as_str()).filter(|label| *label != d.name),
            Some(Usable::Dimension(d.name.clone())),
            Entry::Dimension(&d.name),
        );
        dimension.values = d
            .values
            .iter()
            .map(|v| InventoryValue {
                value: v.value.clone(),
                label: Some(v.label.as_str())
                    .filter(|label| *label != v.value)
                    .map(str::to_owned),
                declaration: find_entry(
                    &ctx.model_text,
                    &Entry::DimensionValue {
                        dimension: &d.name,
                        value: &v.value,
                    },
                )
                .map(|span| ctx.encoding.range(&lines, span)),
            })
            .collect();
        out.push(dimension);
    }
    for n in &model.notes {
        out.push(entry(
            ModelKind::Note,
            &n.name,
            Some(&n.label),
            Some(Usable::Note(n.name.clone())),
            Entry::Note(&n.name),
        ));
    }
    for w in &model.widgets {
        let name = &w.schema.name;
        out.push(entry(
            ModelKind::Widget,
            name,
            w.schema.description.as_deref(),
            Some(Usable::Widget(name.clone())),
            Entry::Widget(name),
        ));
    }
    for b in &model.builds {
        out.push(entry(
            ModelKind::Build,
            &b.name,
            None,
            None,
            Entry::Build(&b.name),
        ));
    }
    for t in &model.types {
        let mut page_type = entry(
            ModelKind::Type,
            &t.name,
            t.purpose.as_deref(),
            Some(Usable::Type(t.name.clone())),
            Entry::Type(&t.name),
        );
        // The implicit type's pages name no type, so no page uses it there.
        if t.implicit {
            page_type.uses = Some(
                pages
                    .iter()
                    .filter(|p| p.content_type.as_deref() == Some(t.name.as_str()))
                    .count(),
            );
            page_type.declaration = None;
        }
        out.push(page_type);
    }
    out
}
