//! The `ascribe/inventory` request: what a project has and how much each part
//! is used, for the editor's Pages and Content model views. It answers from
//! the current snapshot, so it includes unsaved edits, and counts with the
//! search Find All References lists from ([`ascribe_resolve::Project::uses`]),
//! so each count is the length of that list.

use std::collections::{BTreeSet, HashMap};

use ascribe_core::{LineIndex, RelPath};
use ascribe_model::TypeMatch;
use ascribe_resolve::Usable;
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
    /// The content model's entries: phrases, features, glossary terms,
    /// dimensions, note types, widgets, then builds, each kind in
    /// declaration order.
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
    /// Its content type; `null` when no one type applies.
    #[serde(rename = "type")]
    pub content_type: Option<String>,
    /// How many links from other files, and includes, name it.
    pub incoming: usize,
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
    pub uses: Option<usize>,
    /// Where `ascribe.toml` declares it; `null` for a built-in note type,
    /// and for an entry that can't be found there.
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
            content_type: match ctx.model.type_for(file.path.as_str()) {
                TypeMatch::One(t) => Some(t.name.clone()),
                TypeMatch::Ambiguous(_) | TypeMatch::None => None,
            },
            incoming: count(Usable::File(file.path.clone())),
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

    InventoryResult {
        model_uri: crate::uri::path_to_uri(&ctx.config).map(|uri| uri.as_str().to_owned()),
        content_uri: crate::uri::path_to_uri(&ctx.content_dir).map(|uri| uri.as_str().to_owned()),
        pages,
        fragments,
        orphans,
        model: model_entries(ctx, &counts),
    }
}

fn is_index(path: &str) -> bool {
    RelPath::parse(path).is_ok_and(|p| p.file_name() == Some("index.md"))
}

fn model_entries(ctx: &Ctx, counts: &HashMap<Usable, usize>) -> Vec<InventoryEntry> {
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
    for t in &model.glossary.terms {
        out.push(entry(
            ModelKind::Term,
            &t.id,
            Some(&t.term),
            Some(Usable::Term(t.id.clone())),
            Entry::Term(&t.id),
        ));
    }
    for d in &model.dimensions {
        out.push(entry(
            ModelKind::Dimension,
            &d.name,
            Some(d.label.as_str()).filter(|label| *label != d.name),
            Some(Usable::Dimension(d.name.clone())),
            Entry::Dimension(&d.name),
        ));
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
    out
}
