//! The JSON output (SPEC §9.4): the resolved tree, for custom consumers.
//!
//! One document per page, at the page's path with `.json` in place of `.md`
//! (output-layout contract, §1.1). The schema is versioned by
//! `schemaVersion` and documented in the crate's README.

use serde::Serialize;
use tessera_core::availability::parse_availability;
use tessera_core::{AssetUse, FileId, Location, RelPath, Span};
use tessera_resolve::{
    Annotation, Availability, GlossaryUse, HeadingIds, IncludeSite, LinkTarget, RefKind,
    ResolvedBlock, ResolvedKind, ResolvedLink, ResolvedPage, Scope, Substitution,
};
use tessera_syntax::{BlockKind, DirectiveLine, Inline, InlineKind, PrimaryValue};

use crate::emitter::{Emitter, PageContext};
use crate::error::EmitError;
use crate::labels::{attribute_values, availability_display, dimensional_label, plain_text};

// Resolved Q118: the JSON output's shape (SPEC §9.4 says only "the
// resolved tree, for custom consumers").

/// The version of the JSON schema. It changes only when a field is removed or
/// changes meaning; fields can be added without a new version.
pub const JSON_SCHEMA_VERSION: u32 = 1;

/// The JSON emitter.
#[derive(Clone, Copy, Debug, Default)]
pub struct JsonEmitter;

impl Emitter for JsonEmitter {
    fn name(&self) -> &'static str {
        "json"
    }

    fn page_path(&self, page: &RelPath) -> RelPath {
        let text = page.as_str();
        let stem = text.strip_suffix(".md").unwrap_or(text);
        RelPath::parse(&format!("{stem}.json")).unwrap_or_else(|_| page.clone())
    }

    fn render_page(&self, cx: &PageContext<'_>, page: &ResolvedPage) -> Result<String, EmitError> {
        let doc = Writer { cx }.page(page);
        let text = serde_json::to_string_pretty(&doc).map_err(|e| EmitError::Render {
            page: page.path.to_string(),
            message: e.to_string(),
        })?;
        let mut text = compact_scalar_arrays(&text);
        text.push('\n');
        Ok(text)
    }
}

// ---------------------------------------------------------------------------
// The document

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct PageDoc {
    schema_version: u32,
    format: &'static str,
    build: String,
    path: String,
    route: String,
    site: Option<String>,
    title: Option<String>,
    frontmatter: Option<serde_json::Value>,
    availability: Option<AvailabilityJson>,
    headings: Vec<HeadingJson>,
    assets: Vec<AssetJson>,
    blocks: Vec<BlockJson>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct HeadingJson {
    level: u8,
    text: String,
    id: String,
    source_id: String,
    explicit: bool,
    source: SourceJson,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct AssetJson {
    /// The asset's source path, relative to the content root.
    source: String,
    /// The copy's path in this output.
    path: String,
    /// How this page refers to the copy.
    reference: String,
    kind: &'static str,
    fragment: Option<String>,
    written_in: Option<String>,
    span: SpanJson,
}

/// A byte range in a source file, `[start, end]`, counted in UTF-8 bytes from
/// the start of the file (frontmatter included).
#[derive(Clone, Copy, Serialize)]
struct SpanJson(usize, usize);

impl From<Span> for SpanJson {
    fn from(span: Span) -> SpanJson {
        SpanJson(span.start(), span.end())
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SourceJson {
    /// The file the node is written in, relative to the content root.
    file: Option<String>,
    span: SpanJson,
    /// First and last line, counted from 1.
    lines: Option<[u32; 2]>,
    /// The includes it came through, outermost first.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    via: Vec<ViaJson>,
}

#[derive(Serialize)]
struct ViaJson {
    file: Option<String>,
    span: SpanJson,
    line: Option<u32>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct AvailabilityJson {
    /// The spec as written (for a feature key, the spec it stands for).
    text: String,
    /// The spec with the content model's display labels.
    display: String,
    scope: &'static str,
    feature: Option<String>,
    source: Option<SourceRef>,
    /// The scope it sits in, whose availability it inherits too. The chain
    /// stops before the page's availability, which is in the page's own
    /// `availability` and applies to every block.
    #[serde(skip_serializing_if = "Option::is_none")]
    enclosing: Option<Box<AvailabilityJson>>,
}

#[derive(Serialize)]
struct SourceRef {
    file: Option<String>,
    span: SpanJson,
}

#[derive(Serialize)]
struct AttrJson {
    key: String,
    values: Vec<String>,
}

// Blocks.

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct BlockJson {
    #[serde(flatten)]
    body: BlockBody,
    source: SourceJson,
    #[serde(skip_serializing_if = "Option::is_none")]
    availability: Option<AvailabilityJson>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    links: Vec<LinkJson>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    glossary: Vec<GlossaryJson>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    substitutions: Vec<SubstitutionJson>,
}

#[derive(Serialize)]
#[serde(tag = "type", rename_all = "camelCase")]
enum BlockBody {
    #[serde(rename_all = "camelCase")]
    Heading {
        level: u8,
        id: Option<String>,
        source_id: Option<String>,
        explicit: bool,
        text: Option<String>,
        inlines: Vec<InlineJson>,
    },
    Paragraph {
        inlines: Vec<InlineJson>,
    },
    Code {
        fenced: bool,
        info: String,
        literal: String,
    },
    BlockQuote {
        children: Vec<BlockJson>,
    },
    List {
        ordered: bool,
        start: Option<u64>,
        tight: bool,
        items: Vec<ItemJson>,
    },
    Html {
        literal: String,
    },
    ThematicBreak,
    Table {
        /// Each column's alignment: `none`, `left`, `center`, or `right`.
        alignments: Vec<&'static str>,
        rows: Vec<RowJson>,
    },
    /// A line-form directive that survived: `@available` with its annotation,
    /// or a `@note`, `@steps`, `@details`, or widget that binds the block
    /// that follows it.
    Directive {
        name: String,
        attributes: Vec<AttrJson>,
        title: Option<Vec<InlineJson>>,
        primary: Option<PrimaryJson>,
        binding: Option<&'static str>,
        annotation: Option<AnnotationJson>,
    },
    Container {
        name: String,
        attributes: Vec<AttrJson>,
        title: Option<Vec<InlineJson>>,
        children: Vec<BlockJson>,
    },
    Group {
        name: String,
        arms: Vec<ArmJson>,
    },
}

#[derive(Serialize)]
struct ItemJson {
    children: Vec<BlockJson>,
}

#[derive(Serialize)]
struct RowJson {
    header: bool,
    cells: Vec<CellJson>,
}

#[derive(Serialize)]
struct CellJson {
    inlines: Vec<InlineJson>,
}

#[derive(Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
enum PrimaryJson {
    Text { inlines: Vec<InlineJson> },
    Identifier { text: String },
    Line { text: String },
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct AnnotationJson {
    text: String,
    display: Option<String>,
    feature: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ArmJson {
    /// The arm's label: its title as plain text, or the display labels of the
    /// values it names.
    label: Option<String>,
    attributes: Vec<AttrJson>,
    title: Option<Vec<InlineJson>>,
    span: SpanJson,
    children: Vec<BlockJson>,
}

// Links, glossary, substitutions.

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct LinkJson {
    span: SpanJson,
    kind: &'static str,
    destination: String,
    target: TargetJson,
}

#[derive(Serialize)]
#[serde(tag = "type", rename_all = "camelCase")]
enum TargetJson {
    External,
    #[serde(rename_all = "camelCase")]
    Page {
        page: String,
        id: Option<String>,
        url: String,
        text_filled: bool,
    },
    #[serde(rename_all = "camelCase")]
    Asset {
        source: String,
        path: String,
        reference: String,
        url: Option<String>,
        fragment: Option<String>,
    },
    Unresolved,
}

#[derive(Serialize)]
struct GlossaryJson {
    term: String,
    text: String,
    url: String,
}

#[derive(Serialize)]
struct SubstitutionJson {
    span: SpanJson,
    key: String,
    value: String,
}

// Inlines.

#[derive(Serialize)]
struct InlineJson {
    #[serde(flatten)]
    body: InlineBody,
    span: SpanJson,
}

#[derive(Serialize)]
#[serde(tag = "type", rename_all = "camelCase")]
enum InlineBody {
    Text {
        value: String,
    },
    Code {
        value: String,
    },
    SoftBreak,
    HardBreak,
    Html {
        value: String,
    },
    Emphasis {
        children: Vec<InlineJson>,
    },
    Strong {
        children: Vec<InlineJson>,
    },
    Link {
        destination: String,
        title: Option<String>,
        target: Option<TargetJson>,
        children: Vec<InlineJson>,
    },
    Image {
        destination: String,
        title: Option<String>,
        target: Option<TargetJson>,
        alt: String,
        attributes: Vec<AttrJson>,
    },
    Phrase {
        key: String,
    },
}

// ---------------------------------------------------------------------------
// Building it

struct Writer<'a, 'c> {
    cx: &'a PageContext<'c>,
}

impl Writer<'_, '_> {
    fn page(&self, page: &ResolvedPage) -> PageDoc {
        let emit = self.cx.emit;
        PageDoc {
            schema_version: JSON_SCHEMA_VERSION,
            format: "tessera-page",
            build: page.build.clone(),
            path: page.path.to_string(),
            route: page.route.clone(),
            site: emit.site_origin().map(str::to_owned),
            title: page.title.clone(),
            // A frontmatter mapping with a key that isn't a string can't be
            // JSON; it's left out (`tessera check` reports the key).
            frontmatter: page
                .frontmatter
                .as_ref()
                .and_then(|f| serde_json::to_value(f).ok()),
            availability: page.availability.as_deref().map(|a| self.availability(a)),
            headings: page
                .headings()
                .into_iter()
                .map(|(block, ids)| self.heading_entry(block, ids))
                .collect(),
            assets: page.assets.iter().map(|a| self.asset(page, a)).collect(),
            blocks: page.blocks.iter().map(|b| self.block(b)).collect(),
        }
    }

    fn source(&self, file: FileId, span: Span, via: &[IncludeSite]) -> SourceJson {
        let emit = self.cx.emit;
        SourceJson {
            file: emit.file_path(file).map(ToString::to_string),
            span: span.into(),
            lines: emit
                .range(file, span)
                .map(|(start, end)| [start.line, end.line]),
            via: via
                .iter()
                .map(|site| ViaJson {
                    file: emit.file_path(site.file).map(ToString::to_string),
                    span: site.span.into(),
                    line: emit.position(site.file, site.span.start()).map(|p| p.line),
                })
                .collect(),
        }
    }

    fn source_ref(&self, at: Location) -> Option<SourceRef> {
        Some(SourceRef {
            file: Some(self.cx.emit.file_path(at.file)?.to_string()),
            span: at.span.into(),
        })
    }

    fn heading_entry(&self, block: &ResolvedBlock, ids: &HeadingIds) -> HeadingJson {
        HeadingJson {
            level: ids.level,
            text: ids.text.clone(),
            id: ids.page_id.clone(),
            source_id: ids.source_id.clone(),
            explicit: ids.explicit,
            source: self.source(block.file, block.span, &block.via),
        }
    }

    fn asset(&self, _page: &ResolvedPage, asset: &tessera_resolve::PageAsset) -> AssetJson {
        let usage = match asset.kind {
            RefKind::Image => AssetUse::Image,
            RefKind::Link => AssetUse::Link,
        };
        let placement = self.cx.asset(&asset.path, usage);
        AssetJson {
            source: asset.path.to_string(),
            path: placement.copy_to.to_string(),
            reference: placement.reference,
            kind: kind_name(asset.kind),
            fragment: asset.fragment.clone(),
            written_in: Some(asset.written_in.to_string()),
            span: asset.location.span.into(),
        }
    }

    fn availability(&self, a: &Availability) -> AvailabilityJson {
        AvailabilityJson {
            text: a.text.clone(),
            display: availability_display(self.cx.emit.model, &a.spec),
            scope: match a.scope {
                Scope::Page => "page",
                Scope::Section => "section",
                Scope::Block => "block",
            },
            feature: a.feature.clone(),
            source: self.source_ref(a.written_at),
            enclosing: a
                .enclosing
                .as_deref()
                .filter(|e| e.scope != Scope::Page)
                .map(|e| Box::new(self.availability(e))),
        }
    }

    fn block(&self, block: &ResolvedBlock) -> BlockJson {
        BlockJson {
            body: self.body(block),
            source: self.source(block.file, block.span, &block.via),
            availability: block
                .availability
                .as_deref()
                .filter(|a| a.scope != Scope::Page)
                .map(|a| self.availability(a)),
            links: block.links.iter().map(|l| self.link(l)).collect(),
            glossary: block.glossary.iter().map(glossary).collect(),
            substitutions: block.substitutions.iter().map(substitution).collect(),
        }
    }

    fn body(&self, block: &ResolvedBlock) -> BlockBody {
        match &block.kind {
            ResolvedKind::Leaf(b) => match &b.kind {
                BlockKind::Heading(h) => BlockBody::Heading {
                    level: h.level,
                    id: block.heading.as_ref().map(|i| i.page_id.clone()),
                    source_id: block.heading.as_ref().map(|i| i.source_id.clone()),
                    explicit: block.heading.as_ref().is_some_and(|i| i.explicit),
                    text: block.heading.as_ref().map(|i| i.text.clone()),
                    inlines: self.inlines(block, &h.inlines),
                },
                BlockKind::Paragraph(p) => BlockBody::Paragraph {
                    inlines: self.inlines(block, &p.inlines),
                },
                BlockKind::CodeBlock(c) => BlockBody::Code {
                    fenced: c.fenced,
                    info: c.info.clone(),
                    literal: c.literal.clone(),
                },
                BlockKind::HtmlBlock(h) => BlockBody::Html {
                    literal: h.literal.clone(),
                },
                BlockKind::ThematicBreak => BlockBody::ThematicBreak,
                BlockKind::Table(t) => BlockBody::Table {
                    alignments: t
                        .alignments
                        .iter()
                        .map(|a| match a {
                            tessera_syntax::Alignment::None => "none",
                            tessera_syntax::Alignment::Left => "left",
                            tessera_syntax::Alignment::Center => "center",
                            tessera_syntax::Alignment::Right => "right",
                        })
                        .collect(),
                    rows: t
                        .rows
                        .iter()
                        .map(|row| RowJson {
                            header: row.header,
                            cells: row
                                .cells
                                .iter()
                                .map(|c| CellJson {
                                    inlines: self.inlines(block, &c.inlines),
                                })
                                .collect(),
                        })
                        .collect(),
                },
                BlockKind::Directive(line) => self.directive(block, line),
                // Never leaves, and end lines and titles aren't blocks of the
                // resolved tree; they are written out as an empty paragraph
                // rather than dropped from the list.
                BlockKind::BlockQuote(_)
                | BlockKind::List(_)
                | BlockKind::End(_)
                | BlockKind::Container(_)
                | BlockKind::Group(_)
                | BlockKind::Title(_) => BlockBody::Paragraph {
                    inlines: Vec::new(),
                },
            },
            ResolvedKind::BlockQuote { children } => BlockBody::BlockQuote {
                children: children.iter().map(|c| self.block(c)).collect(),
            },
            ResolvedKind::List {
                ordered,
                start,
                tight,
                items,
            } => BlockBody::List {
                ordered: *ordered,
                start: *start,
                tight: *tight,
                items: items
                    .iter()
                    .map(|i| ItemJson {
                        children: i.children.iter().map(|c| self.block(c)).collect(),
                    })
                    .collect(),
            },
            ResolvedKind::Container {
                opener, children, ..
            } => BlockBody::Container {
                name: opener.name.clone(),
                attributes: attributes(opener),
                title: self.title(block, opener),
                children: children.iter().map(|c| self.block(c)).collect(),
            },
            ResolvedKind::Group { name, arms, .. } => BlockBody::Group {
                name: name.clone(),
                arms: arms
                    .iter()
                    .map(|arm| ArmJson {
                        label: arm
                            .opener
                            .title
                            .as_ref()
                            .map(|t| plain_text(&t.inlines))
                            .or_else(|| dimensional_label(self.cx.emit.model, &arm.opener)),
                        attributes: attributes(&arm.opener),
                        title: self.title(block, &arm.opener),
                        span: arm.span.into(),
                        children: arm.children.iter().map(|c| self.block(c)).collect(),
                    })
                    .collect(),
            },
        }
    }

    fn directive(&self, block: &ResolvedBlock, line: &DirectiveLine) -> BlockBody {
        let model = self.cx.emit.model;
        BlockBody::Directive {
            name: line.name.clone(),
            attributes: attributes(line),
            title: self.title(block, line),
            primary: match &line.primary {
                Some(PrimaryValue::Text(t)) => Some(PrimaryJson::Text {
                    inlines: self.inlines(block, &t.inlines),
                }),
                Some(PrimaryValue::Identifier(i)) => Some(PrimaryJson::Identifier {
                    text: i.text.clone(),
                }),
                Some(PrimaryValue::Line(l)) => Some(PrimaryJson::Line {
                    text: l.text.clone(),
                }),
                Some(PrimaryValue::Unexpected(_)) | None => None,
            },
            binding: line.binding.map(|b| match b {
                tessera_syntax::Bound::Own => "own",
                tessera_syntax::Bound::Heading => "heading",
                tessera_syntax::Bound::FollowingBlock => "followingBlock",
                tessera_syntax::Bound::Unbound => "unbound",
            }),
            annotation: block
                .annotation
                .as_ref()
                .map(|a: &Annotation| AnnotationJson {
                    text: a.text.clone(),
                    display: parse_availability(&a.text, 0)
                        .ok()
                        .map(|spec| availability_display(model, &spec)),
                    feature: a.feature.clone(),
                }),
        }
    }

    fn title(&self, block: &ResolvedBlock, line: &DirectiveLine) -> Option<Vec<InlineJson>> {
        line.title.as_ref().map(|t| self.inlines(block, &t.inlines))
    }

    fn inlines(&self, block: &ResolvedBlock, list: &[Inline]) -> Vec<InlineJson> {
        list.iter().map(|i| self.inline(block, i)).collect()
    }

    fn inline(&self, block: &ResolvedBlock, node: &Inline) -> InlineJson {
        let body = match &node.kind {
            InlineKind::Text(v) => InlineBody::Text { value: v.clone() },
            InlineKind::Code(v) => InlineBody::Code { value: v.clone() },
            InlineKind::SoftBreak => InlineBody::SoftBreak,
            InlineKind::HardBreak => InlineBody::HardBreak,
            InlineKind::Html(v) => InlineBody::Html { value: v.clone() },
            InlineKind::Emphasis(c) => InlineBody::Emphasis {
                children: self.inlines(block, c),
            },
            InlineKind::Strong(c) => InlineBody::Strong {
                children: self.inlines(block, c),
            },
            InlineKind::Link(link) => {
                let resolved = block
                    .links
                    .iter()
                    .find(|l| l.span == node.span && l.kind == RefKind::Link);
                InlineBody::Link {
                    destination: link.destination.clone(),
                    title: link.title.clone(),
                    target: resolved.map(|l| self.target(&l.target, AssetUse::Link)),
                    children: self.inlines(block, &link.children),
                }
            }
            InlineKind::Image(image) => {
                let resolved = block
                    .links
                    .iter()
                    .find(|l| l.span == node.span && l.kind == RefKind::Image);
                InlineBody::Image {
                    destination: image.destination.clone(),
                    title: image.title.clone(),
                    target: resolved.map(|l| self.target(&l.target, AssetUse::Image)),
                    alt: plain_text(&image.children),
                    attributes: attribute_values(image.attributes.as_ref().map(|a| &a.block))
                        .into_iter()
                        .map(|(key, values)| AttrJson { key, values })
                        .collect(),
                }
            }
            InlineKind::Phrase(p) => InlineBody::Phrase { key: p.key.clone() },
        };
        InlineJson {
            body,
            span: node.span.into(),
        }
    }

    fn link(&self, link: &ResolvedLink) -> LinkJson {
        let usage = match link.kind {
            RefKind::Image => AssetUse::Image,
            RefKind::Link => AssetUse::Link,
        };
        LinkJson {
            span: link.span.into(),
            kind: kind_name(link.kind),
            destination: link.destination.clone(),
            target: self.target(&link.target, usage),
        }
    }

    fn target(&self, target: &LinkTarget, usage: AssetUse) -> TargetJson {
        match target {
            LinkTarget::External => TargetJson::External,
            LinkTarget::Page {
                page,
                id,
                url,
                text_filled,
            } => TargetJson::Page {
                page: page.to_string(),
                id: id.clone(),
                url: url.clone(),
                text_filled: *text_filled,
            },
            LinkTarget::Asset { path, fragment } => {
                let placement = self.cx.asset(path, usage);
                TargetJson::Asset {
                    source: path.to_string(),
                    path: placement.copy_to.to_string(),
                    reference: placement.reference,
                    url: placement.url,
                    fragment: fragment.clone(),
                }
            }
            LinkTarget::Unresolved => TargetJson::Unresolved,
        }
    }
}

fn kind_name(kind: RefKind) -> &'static str {
    match kind {
        RefKind::Image => "image",
        RefKind::Link => "link",
    }
}

fn attributes(line: &DirectiveLine) -> Vec<AttrJson> {
    attribute_values(line.attributes.as_ref())
        .into_iter()
        .map(|(key, values)| AttrJson { key, values })
        .collect()
}

fn glossary(g: &GlossaryUse) -> GlossaryJson {
    GlossaryJson {
        term: g.term.clone(),
        text: g.text.clone(),
        url: g.url.clone(),
    }
}

fn substitution(s: &Substitution) -> SubstitutionJson {
    SubstitutionJson {
        span: s.span.into(),
        key: s.key.clone(),
        value: s.value.clone(),
    }
}

/// Puts a short array of numbers or strings, such as a span, on one line.
/// `serde_json`'s pretty printer writes each element on its own line, which
/// makes a document with a span on every node several times longer to read.
fn compact_scalar_arrays(pretty: &str) -> String {
    const MAX_WIDTH: usize = 100;
    let lines: Vec<&str> = pretty.lines().collect();
    let mut out: Vec<String> = Vec::with_capacity(lines.len());
    let mut i = 0;
    while i < lines.len() {
        let line = lines[i];
        if line.trim_end().ends_with('[') {
            let mut items = Vec::new();
            let mut j = i + 1;
            while j < lines.len() {
                let item = lines[j].trim().trim_end_matches(',');
                let scalar = !item.is_empty()
                    && (item.bytes().all(|b| b.is_ascii_digit())
                        || (item.len() >= 2 && item.starts_with('"') && item.ends_with('"')));
                if !scalar {
                    break;
                }
                items.push(item);
                j += 1;
            }
            let closer = lines.get(j).map(|l| l.trim());
            let joined = format!("{}{}]", line.trim_end(), items.join(", "));
            if !items.is_empty() && matches!(closer, Some("]" | "],")) && joined.len() <= MAX_WIDTH
            {
                out.push(format!(
                    "{joined}{}",
                    closer.map_or("", |c| c.trim_start_matches(']'))
                ));
                i = j + 1;
                continue;
            }
        }
        out.push(line.to_owned());
        i += 1;
    }
    out.join("\n")
}
