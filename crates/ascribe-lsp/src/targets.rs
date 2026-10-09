//! The `ascribe/targets` request: what an action can point at or use in a
//! page's project, so a client can offer choices instead of asking for
//! syntax. A client asks for the kinds it needs.
//!
//! The lists come from the current snapshot and model, so they include
//! unsaved edits. Pages, headings, and fragments are the ones completion
//! offers, with the paths written the way completion writes them; images and
//! the files of the model's sources are listed through the project's
//! [`FileSystem`], and a snippet's regions are read by the code that resolves
//! `@snippet` ([`source_files`]), so every address listed resolves.

use std::collections::HashSet;

use ascribe_core::schema::{AttributeType, Attributes, DefaultValue, SetMember};
use ascribe_core::{Binding, LineIndex, Primary, RelPath, image_media_type};
use ascribe_model::TypeMatch;
use ascribe_resolve::{CodeFiles, FileSystem, in_nested_project, source_files};
use ascribe_syntax::BlockKind;
use lsp_types::{Range, TextDocumentIdentifier};
use serde::{Deserialize, Serialize};

use crate::definition::{Entry, find_entry};
use crate::nav::{Ctx, encode_destination, relative_path};

/// The request's method name.
pub const METHOD: &str = "ascribe/targets";

/// The parameters of `ascribe/targets`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TargetsParams {
    /// The page the targets are for: paths are written from it.
    pub text_document: TextDocumentIdentifier,
    /// The kinds of target to list.
    pub kinds: Vec<TargetKind>,
}

/// A kind of target.
#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum TargetKind {
    /// Pages.
    Pages,
    /// The headings of pages.
    Headings,
    /// Fragments.
    Fragments,
    /// Image files.
    Images,
    /// The files of the content model's sources.
    Snippets,
    /// Phrases.
    Phrases,
    /// Note types.
    Notes,
    /// Dimensions.
    Dimensions,
    /// Project widgets.
    Widgets,
    /// Features.
    Features,
    /// Builds.
    Builds,
}

/// The answer to `ascribe/targets`: a list for each kind asked for, and no
/// others. A document that isn't a source file of the project gets no
/// lists.
#[derive(Debug, Default, Serialize, PartialEq)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct TargetsResult {
    /// The `file:` URI of the project's `ascribe.toml`, which the ranges of
    /// declarations are in.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model_uri: Option<String>,
    /// Pages, by content path.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pages: Option<Vec<TargetPage>>,
    /// The headings a link can name, page by page in content path order,
    /// each page's in document order: a page's own, and those of the
    /// fragments it includes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub headings: Option<Vec<TargetHeading>>,
    /// Fragments, by content path.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fragments: Option<Vec<TargetFragment>>,
    /// The image files under the content root, by path.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub images: Option<Vec<TargetImage>>,
    /// The content model's sources, in the order it declares them, with the
    /// files a snippet can take code from.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub snippets: Option<Vec<TargetSource>>,
    /// Phrases, in declaration order.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phrases: Option<Vec<TargetPhrase>>,
    /// Note types: the built-ins, then the declared ones.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<Vec<TargetNote>>,
    /// Dimensions, in declaration order.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dimensions: Option<Vec<TargetDimension>>,
    /// Project widgets, in declaration order.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub widgets: Option<Vec<TargetWidget>>,
    /// Features, in declaration order.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub features: Option<Vec<TargetFeature>>,
    /// Builds, in declaration order.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub builds: Option<Vec<TargetBuild>>,
}

/// A page.
#[derive(Debug, Serialize, PartialEq)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct TargetPage {
    /// Its content path.
    pub path: String,
    /// Its title (frontmatter `title`).
    pub title: Option<String>,
    /// Its content type; `null` when no one type applies.
    #[serde(rename = "type")]
    pub content_type: Option<String>,
    /// The destination of a link to it from the requesting page.
    pub link: String,
}

/// A heading a link can name.
#[derive(Debug, Serialize, PartialEq)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct TargetHeading {
    /// The content path of the page a link names it on.
    pub page: String,
    /// The heading's text, phrases replaced by their values.
    pub text: String,
    /// Its id.
    pub id: String,
    /// 1 to 6.
    pub level: u8,
    /// The destination of a link to it from the requesting page: `page.md#id`,
    /// or `#id` on the requesting page itself.
    pub link: String,
}

/// A fragment.
#[derive(Debug, Serialize, PartialEq)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct TargetFragment {
    /// Its content path.
    pub path: String,
    /// The path an `@include` on the requesting page writes for it.
    pub include: String,
    /// Whether its first block is a heading.
    pub starts_with_heading: bool,
}

/// An image file.
#[derive(Debug, Serialize, PartialEq)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct TargetImage {
    /// Its path from the project root.
    pub path: String,
    /// The source of an image of it on the requesting page.
    pub link: String,
}

/// A source of the content model and its files.
#[derive(Debug, Serialize, PartialEq)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct TargetSource {
    /// The source's name.
    pub name: String,
    /// The files a snippet can take code from, by path.
    pub files: Vec<TargetSourceFile>,
}

/// A file a snippet can take code from.
#[derive(Debug, Serialize, PartialEq)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct TargetSourceFile {
    /// Its path relative to the source's folder.
    pub path: String,
    /// The address a `@snippet` writes for the whole file.
    pub address: String,
    /// Its regions, in the order they start.
    pub regions: Vec<TargetRegion>,
}

/// A region of a source file.
#[derive(Debug, Serialize, PartialEq)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct TargetRegion {
    /// Its name.
    pub name: String,
    /// The address a `@snippet` writes for it.
    pub address: String,
}

/// A phrase.
#[derive(Debug, Serialize, PartialEq)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct TargetPhrase {
    /// The key.
    pub key: String,
    /// The value.
    pub value: String,
    /// Its key in `ascribe.toml`; `null` when it can't be found there.
    #[cfg_attr(
        feature = "json-schema",
        schemars(with = "Option<crate::schema::LspRange>")
    )]
    pub range: Option<Range>,
}

/// A note type.
#[derive(Debug, Serialize, PartialEq)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct TargetNote {
    /// The type, as `@note {type=…}` writes it.
    #[serde(rename = "type")]
    pub note_type: String,
    /// Its label.
    pub label: String,
}

/// A dimension.
#[derive(Debug, Serialize, PartialEq)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct TargetDimension {
    /// Its name.
    pub name: String,
    /// Its label.
    pub label: String,
    /// Its values, in display order.
    pub values: Vec<TargetDimensionValue>,
}

/// A value of a dimension.
#[derive(Debug, Serialize, PartialEq)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct TargetDimensionValue {
    /// The value.
    pub value: String,
    /// Its label.
    pub label: String,
    /// Whether it's versionless.
    pub versionless: bool,
    /// The value in its dimension's `values` in `ascribe.toml`, inside its
    /// quotes; `null` when it can't be found there.
    #[cfg_attr(
        feature = "json-schema",
        schemars(with = "Option<crate::schema::LspRange>")
    )]
    pub range: Option<Range>,
}

/// A project widget.
#[derive(Debug, Serialize, PartialEq)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct TargetWidget {
    /// Its name.
    pub name: String,
    /// What it's for, as the content model describes it.
    pub description: Option<String>,
    /// Whether it may be one line.
    pub line: bool,
    /// Whether it may be a container.
    pub container: bool,
    /// Whether its openers form groups of arms.
    pub groupable: bool,
    /// What its line form takes after the colon.
    pub primary: PrimaryKind,
    /// What its line form applies to; `null` when it has no line form.
    pub binding: Option<BindingKind>,
    /// Its attributes, in canonical order.
    pub attributes: Vec<TargetAttribute>,
}

/// What a directive takes after its colon.
#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub enum PrimaryKind {
    /// Nothing.
    None,
    /// One token: a path, id, or key.
    Identifier,
    /// Inline text.
    Text,
    /// An availability spec.
    Availability,
}

/// What a line-form directive applies to.
#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub enum BindingKind {
    /// Its own primary, or nothing.
    #[serde(rename = "self")]
    SelfBound,
    /// The heading at the start of its section.
    Heading,
    /// The block below it.
    Block,
    /// The section at the top of one, else the block below it.
    HeadingOrBlock,
}

/// An attribute a widget accepts.
#[derive(Debug, Serialize, PartialEq)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct TargetAttribute {
    /// The key.
    pub key: String,
    /// The value's type.
    #[serde(rename = "type")]
    pub value_type: AttributeKind,
    /// The values it allows, for `enum` and a `set` of named values; empty
    /// otherwise.
    pub values: Vec<String>,
    /// Whether every use must give it.
    pub required: bool,
    /// The value used when it's left out, as written (a set's members joined
    /// by `|`).
    pub default: Option<String>,
    /// What it's for.
    pub description: Option<String>,
}

/// An attribute's type.
#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub enum AttributeKind {
    /// Any text.
    String,
    /// A number.
    Number,
    /// `true` or `false`.
    Boolean,
    /// One of `values`.
    Enum,
    /// One or more values joined by `|`: any of `values`, or anything when
    /// `values` is empty.
    Set,
    /// A note type.
    NoteType,
}

/// A feature.
#[derive(Debug, Serialize, PartialEq)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct TargetFeature {
    /// Its key.
    pub key: String,
    /// Its name.
    pub name: String,
    /// Its availability spec, as written.
    pub availability: String,
    /// Its `[features.<key>]` table header in `ascribe.toml`; `null` when it
    /// can't be found there.
    #[cfg_attr(
        feature = "json-schema",
        schemars(with = "Option<crate::schema::LspRange>")
    )]
    pub range: Option<Range>,
}

/// A build.
#[derive(Debug, Serialize, PartialEq)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct TargetBuild {
    /// Its name.
    pub name: String,
    /// Whether it's the editor's build (`[editor] build`).
    pub editor: bool,
}

/// The targets of the requested kinds, for the requested page.
pub(crate) fn targets(ctx: &Ctx, kinds: &[TargetKind]) -> TargetsResult {
    let wants = |kind| kinds.contains(&kind).then_some(());
    let model_index = LineIndex::new(&ctx.model_text);
    let declared = |entry: Entry<'_>| {
        find_entry(&ctx.model_text, &entry).map(|span| ctx.encoding.range(&model_index, span))
    };
    TargetsResult {
        model_uri: crate::uri::path_to_uri(&ctx.config).map(|uri| uri.as_str().to_owned()),
        pages: wants(TargetKind::Pages).map(|()| pages(ctx)),
        headings: wants(TargetKind::Headings).map(|()| headings(ctx)),
        fragments: wants(TargetKind::Fragments).map(|()| fragments(ctx)),
        images: wants(TargetKind::Images).map(|()| images(ctx)),
        snippets: wants(TargetKind::Snippets).map(|()| snippets(ctx)),
        phrases: wants(TargetKind::Phrases).map(|()| {
            ctx.model
                .phrases
                .iter()
                .map(|p| TargetPhrase {
                    key: p.key.clone(),
                    value: p.value.clone(),
                    range: declared(Entry::Phrase(&p.key)),
                })
                .collect()
        }),
        notes: wants(TargetKind::Notes).map(|()| {
            ctx.model
                .notes
                .iter()
                .map(|n| TargetNote {
                    note_type: n.name.clone(),
                    label: n.label.clone(),
                })
                .collect()
        }),
        dimensions: wants(TargetKind::Dimensions).map(|()| {
            ctx.model
                .dimensions
                .iter()
                .map(|d| TargetDimension {
                    name: d.name.clone(),
                    label: d.label.clone(),
                    values: d
                        .values
                        .iter()
                        .map(|v| TargetDimensionValue {
                            value: v.value.clone(),
                            label: v.label.clone(),
                            versionless: v.versionless,
                            range: declared(Entry::DimensionValue {
                                dimension: &d.name,
                                value: &v.value,
                            }),
                        })
                        .collect(),
                })
                .collect()
        }),
        widgets: wants(TargetKind::Widgets).map(|()| widgets(ctx)),
        features: wants(TargetKind::Features).map(|()| {
            ctx.model
                .features
                .iter()
                .map(|f| TargetFeature {
                    key: f.key.clone(),
                    name: f.name.clone(),
                    availability: f.available_text.clone(),
                    range: declared(Entry::Feature(&f.key)),
                })
                .collect()
        }),
        builds: wants(TargetKind::Builds).map(|()| {
            ctx.model
                .builds
                .iter()
                .map(|b| TargetBuild {
                    name: b.name.clone(),
                    editor: b.name == ctx.model.editor_build,
                })
                .collect()
        }),
    }
}

/// The destination of a link to `target` from the requesting page.
fn link_to(ctx: &Ctx, target: &RelPath) -> String {
    encode_destination(&relative_path(target, &ctx.path))
}

fn pages(ctx: &Ctx) -> Vec<TargetPage> {
    let mut pages: Vec<TargetPage> = ctx
        .snapshot
        .pages()
        .map(|file| TargetPage {
            path: file.path.to_string(),
            title: file.title.clone(),
            content_type: match ctx.model.type_for(file.path.as_str()) {
                TypeMatch::One(t) => Some(t.name.clone()),
                TypeMatch::Ambiguous(_) | TypeMatch::None => None,
            },
            link: link_to(ctx, &file.path),
        })
        .collect();
    pages.sort_by(|a, b| a.path.cmp(&b.path));
    pages
}

/// The headings a link can name: a page's own, and its fragments' (SPEC
/// §4.2), each id once, as completion offers them.
fn headings(ctx: &Ctx) -> Vec<TargetHeading> {
    let snapshot = &ctx.snapshot;
    let mut pages: Vec<_> = snapshot.pages().collect();
    pages.sort_by(|a, b| a.path.cmp(&b.path));
    let mut out = Vec::new();
    for file in pages {
        let headings: Vec<_> = match snapshot.expansion(&file.path) {
            Some(page) => page
                .headings(snapshot)
                .into_iter()
                .map(|(_, h)| h)
                .collect(),
            None => file.headings.iter().collect(),
        };
        let page_link = if file.path == ctx.path {
            String::new()
        } else {
            link_to(ctx, &file.path)
        };
        let mut seen = HashSet::new();
        for h in headings {
            if h.source_id.is_empty() || !seen.insert(h.source_id.as_str()) {
                continue;
            }
            out.push(TargetHeading {
                page: file.path.to_string(),
                text: h.text.clone(),
                id: h.source_id.clone(),
                level: h.level,
                link: format!("{page_link}#{}", encode_destination(&h.source_id)),
            });
        }
    }
    out
}

fn fragments(ctx: &Ctx) -> Vec<TargetFragment> {
    let mut fragments: Vec<TargetFragment> = ctx
        .snapshot
        .fragments()
        .map(|file| TargetFragment {
            path: file.path.to_string(),
            include: link_to(ctx, &file.path),
            starts_with_heading: file
                .document
                .blocks
                .first()
                .is_some_and(|b| matches!(b.kind, BlockKind::Heading(_))),
        })
        .collect();
    fragments.sort_by(|a, b| a.path.cmp(&b.path));
    fragments
}

/// The image files under the content root, listed through the project's
/// file system: not in a folder whose name starts with `.`, in
/// `node_modules`, in a nested project's folder, or in the output directory.
fn images(ctx: &Ctx) -> Vec<TargetImage> {
    let layout = ctx.snapshot.layout();
    let root = &layout.content_root;
    let nested = ctx.snapshot.nested_projects();
    let mut out = Vec::new();
    for path in ctx.fs.files_in(root) {
        if image_media_type(path.extension().unwrap_or_default()).is_none()
            || path.starts_with(&layout.output_dir)
        {
            continue;
        }
        let Some(content) = path.relative_to(root) else {
            continue;
        };
        if content
            .segments()
            .any(|s| s.starts_with('.') || s == "node_modules")
            || in_nested_project(&content, nested)
        {
            continue;
        }
        out.push(TargetImage {
            path: path.to_string(),
            link: link_to(ctx, &content),
        });
    }
    out
}

fn snippets(ctx: &Ctx) -> Vec<TargetSource> {
    let fs: &dyn FileSystem = &*ctx.fs;
    let code = CodeFiles::new();
    ctx.model
        .sources
        .iter()
        .map(|source| TargetSource {
            name: source.name.clone(),
            files: source_files(source, fs, &code)
                .into_iter()
                .map(|f| TargetSourceFile {
                    regions: f
                        .regions
                        .iter()
                        .map(|r| TargetRegion {
                            name: r.clone(),
                            address: format!("{}#{r}", f.address),
                        })
                        .collect(),
                    path: f.path,
                    address: f.address,
                })
                .collect(),
        })
        .collect()
}

fn widgets(ctx: &Ctx) -> Vec<TargetWidget> {
    ctx.model
        .widgets
        .iter()
        .map(|w| {
            let schema = &w.schema;
            let attributes = match &schema.attributes {
                Attributes::Declared(list) => list
                    .iter()
                    .map(|a| {
                        let (value_type, values) = match &a.ty {
                            AttributeType::String => (AttributeKind::String, Vec::new()),
                            AttributeType::Number => (AttributeKind::Number, Vec::new()),
                            AttributeType::Boolean => (AttributeKind::Boolean, Vec::new()),
                            AttributeType::Enum(values) => (AttributeKind::Enum, values.clone()),
                            AttributeType::Set(SetMember::String) => {
                                (AttributeKind::Set, Vec::new())
                            }
                            AttributeType::Set(SetMember::Enum(values)) => {
                                (AttributeKind::Set, values.clone())
                            }
                            AttributeType::NoteType => (AttributeKind::NoteType, Vec::new()),
                        };
                        TargetAttribute {
                            key: a.key.clone(),
                            value_type,
                            values,
                            required: a.required,
                            default: a.default.as_ref().map(|d| match d {
                                DefaultValue::Text(text) => text.clone(),
                                DefaultValue::Boolean(b) => b.to_string(),
                                DefaultValue::Set(members) => members.join("|"),
                            }),
                            description: a.description.clone(),
                        }
                    })
                    .collect(),
                Attributes::Dimensions => Vec::new(),
            };
            TargetWidget {
                name: schema.name.clone(),
                description: schema.description.clone(),
                line: schema.forms.line,
                container: schema.forms.container,
                groupable: schema.groupable,
                primary: match schema.primary {
                    Primary::None => PrimaryKind::None,
                    Primary::Identifier { .. } => PrimaryKind::Identifier,
                    Primary::Text { .. } => PrimaryKind::Text,
                    Primary::Availability { .. } => PrimaryKind::Availability,
                },
                binding: schema.binding.map(|b| match b {
                    Binding::SelfBound => BindingKind::SelfBound,
                    Binding::Heading => BindingKind::Heading,
                    Binding::Block => BindingKind::Block,
                    Binding::HeadingOrBlock => BindingKind::HeadingOrBlock,
                }),
                attributes,
            }
        })
        .collect()
}
