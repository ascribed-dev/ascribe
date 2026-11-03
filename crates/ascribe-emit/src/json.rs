//! The JSON output (SPEC §9.4): the resolved tree, for custom consumers.
//!
//! One document per page, at the page's path with `.json` in place of `.md`,
//! and the build's navigation at `_ascribe/nav.json`, as the site output has
//! it, with each formatted label in pieces. The schema is versioned by
//! `schemaVersion` and documented in the crate's README.

use ascribe_core::availability::parse_availability;
use ascribe_core::{AssetUse, FileId, Location, RelPath, Span};
use ascribe_model::Segment;
use ascribe_resolve::{
    Annotation, Availability, GlossaryUse, HeadingIds, IncludeSite, LinkTarget, RefKind,
    ResolvedBlock, ResolvedKind, ResolvedLink, ResolvedPage, Scope, SectionSummary, Substitution,
};
use ascribe_syntax::{BlockKind, DirectiveLine, Inline, InlineKind, PrimaryValue};
use serde::Serialize;

use crate::emitter::{EmitContext, Emitter, PageContext};
use crate::error::EmitError;
use crate::labels::{attribute_values, availability_display, dimensional_label, plain_text};
use crate::site::blocks::availability_target;
use crate::site::{NAV_PATH, nav};
use crate::store::{Contents, EmittedFile, FileKind};

// SPEC §9.4 fixes only "the resolved tree, for custom consumers"; the shape
// below is Ascribe's own, versioned by `JSON_SCHEMA_VERSION`.

/// The version of the JSON schema. It changes only when a field is removed or
/// changes meaning; fields can be added without a new version.
pub const JSON_SCHEMA_VERSION: u32 = 2;

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

    fn generated(
        &self,
        cx: &EmitContext<'_>,
        build: &ascribe_resolve::ResolvedBuild,
    ) -> Result<Vec<EmittedFile>, EmitError> {
        let text = nav::list(cx.project(), &cx.project().navigation(), build, &|s| {
            formatted_value(s)
        });
        Ok(vec![EmittedFile {
            path: RelPath::parse(NAV_PATH).map_err(|e| EmitError::Invalid {
                message: format!("bad path for a generated file: {e}"),
            })?,
            kind: FileKind::Generated,
            source: None,
            url: None,
            contents: Contents::Text(text),
        }])
    }

    fn render_page(&self, cx: &PageContext<'_>, page: &ResolvedPage) -> Result<String, EmitError> {
        let doc = Writer {
            cx,
            summaries: &page.summaries,
        }
        .page(page);
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

/// A page of the JSON output: the page as the build resolved it, with every
/// node's place in its source (the JSON output contract). Spans are UTF-8
/// byte offsets, `[start, end]`, from the start of the file, frontmatter
/// included; lines count from 1.
#[derive(Serialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
struct PageDoc {
    /// The format's version, `2`. It changes only when a field is removed or
    /// changes meaning.
    schema_version: u32,
    /// `"ascribe-page"`.
    format: &'static str,
    /// The build's name.
    build: String,
    /// The page's source path, relative to the content root.
    path: String,
    /// The page's root-relative URL, as the router made it.
    route: String,
    /// `[consumer] site`: the origin to put before a root-relative URL.
    site: Option<String>,
    /// The frontmatter `title`, phrases substituted, as plain text.
    title: Option<String>,
    /// The frontmatter, with phrases substituted in the fields that ask for
    /// it, and each field that sets `inline = "code"` as plain text.
    frontmatter: Option<serde_json::Value>,
    /// Each field that sets `inline = "code"`, as a list of text and code
    /// pieces (`{type, value}`); its plain text is in `frontmatter`.
    #[serde(skip_serializing_if = "serde_json::Map::is_empty")]
    formatted: serde_json::Map<String, serde_json::Value>,
    /// The page's own availability (frontmatter `available`).
    availability: Option<AvailabilityJson>,
    /// Every heading that survived the build, in order.
    headings: Vec<HeadingJson>,
    /// Every asset reference on the page that survived the build.
    assets: Vec<AssetJson>,
    /// The page's blocks.
    blocks: Vec<BlockJson>,
}

/// A heading of the page.
#[derive(Serialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "json-schema", schemars(rename = "Heading"))]
struct HeadingJson {
    /// Its level, 1 to 6.
    level: u8,
    /// Its text, as plain text.
    text: String,
    /// Its page id: the id it has on the page.
    id: String,
    /// Its source id: the id it has in its source file.
    source_id: String,
    /// Whether an `@id` gave it its id.
    explicit: bool,
    /// Where it's written.
    source: SourceJson,
}

/// An asset reference on the page.
#[derive(Serialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "json-schema", schemars(rename = "Asset"))]
struct AssetJson {
    /// The asset's source path, relative to the content root.
    source: String,
    /// The copy's path in this output.
    path: String,
    /// How this page refers to the copy.
    reference: String,
    /// `image` or `link`.
    kind: &'static str,
    /// The reference's fragment, without its `#`.
    fragment: Option<String>,
    /// The file the reference is written in, relative to the content root.
    written_in: Option<String>,
    /// The reference's span in that file.
    span: SpanJson,
}

/// A byte range in a source file, `[start, end]`, counted in UTF-8 bytes from
/// the start of the file (frontmatter included).
#[derive(Clone, Copy, Serialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "json-schema", schemars(rename = "Span"))]
struct SpanJson(usize, usize);

impl From<Span> for SpanJson {
    fn from(span: Span) -> SpanJson {
        SpanJson(span.start(), span.end())
    }
}

/// Where a node is written.
#[derive(Serialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "json-schema", schemars(rename = "Source"))]
struct SourceJson {
    /// The file the node is written in, relative to the content root.
    file: Option<String>,
    /// Its span in that file.
    span: SpanJson,
    /// First and last line, counted from 1.
    lines: Option<[u32; 2]>,
    /// The includes it came through, outermost first; absent for a page's
    /// own nodes.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    via: Vec<ViaJson>,
}

/// An include a node came through.
#[derive(Serialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "json-schema", schemars(rename = "Via"))]
struct ViaJson {
    /// The file the `@include` is written in.
    file: Option<String>,
    /// The `@include`'s span in that file.
    span: SpanJson,
    /// The `@include`'s line, counted from 1.
    line: Option<u32>,
}

/// An availability: the page's own, or a block's effective one.
#[derive(Serialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "json-schema", schemars(rename = "Availability"))]
struct AvailabilityJson {
    /// The spec as written (for a feature key, the spec it stands for).
    text: String,
    /// The spec with the content model's display labels.
    display: String,
    /// Each target as the availability element gives it, when the project
    /// lists release lines in `[releases]`.
    #[serde(skip_serializing_if = "Option::is_none")]
    targets: Option<Vec<AvailabilityTargetJson>>,
    /// What it applies to: `page`, `section`, `block`, or `row`.
    scope: &'static str,
    /// The feature key it was written as, when it was one.
    feature: Option<String>,
    /// Where it's written.
    source: Option<SourceRef>,
    /// The scope it sits in, whose availability it inherits too. The chain
    /// stops before the page's availability, which is in the page's own
    /// `availability` and applies to every block.
    #[serde(skip_serializing_if = "Option::is_none")]
    enclosing: Option<Box<AvailabilityJson>>,
}

/// One target of an availability spec: the element's attributes and its
/// text (element contract §4).
#[derive(Serialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "json-schema", schemars(rename = "AvailabilityTarget"))]
struct AvailabilityTargetJson {
    /// The target's name.
    target: String,
    /// The dimension it's a value of.
    dimension: String,
    /// Its lifecycle states.
    states: Vec<String>,
    /// The versions it names; absent when it names none.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    versions: Vec<String>,
    /// `true` when the target's last version is in a planned line; absent
    /// otherwise.
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    planned: bool,
    /// The target as text.
    text: String,
}

/// A place in a source file.
#[derive(Serialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
struct SourceRef {
    /// The file, relative to the content root.
    file: Option<String>,
    /// The span in that file.
    span: SpanJson,
}

/// A directive's attribute.
#[derive(Serialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "json-schema", schemars(rename = "Attribute"))]
struct AttrJson {
    /// Its key.
    key: String,
    /// Its values: several for a set, none for a bare key.
    values: Vec<String>,
}

// Blocks.

/// A block: its `type` and the fields of that type, and what every block
/// has.
#[derive(Serialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "json-schema", schemars(rename = "Block"))]
struct BlockJson {
    #[serde(flatten)]
    body: BlockBody,
    /// Where it's written.
    source: SourceJson,
    /// Its effective availability below page level: the innermost spec, with
    /// `enclosing` chaining outward. Absent when nothing but the page's
    /// availability applies.
    #[serde(skip_serializing_if = "Option::is_none")]
    availability: Option<AvailabilityJson>,
    /// The block's own links and images, resolved; absent when it has none.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    links: Vec<LinkJson>,
    /// Glossary terms linked in its prose, and its links to a term's heading;
    /// absent when it has none.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    glossary: Vec<GlossaryJson>,
    /// Phrases replaced in its text; absent when it has none.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    substitutions: Vec<SubstitutionJson>,
    /// For a `code` block a `@snippet` became, where its code comes from;
    /// absent on every other block.
    #[serde(skip_serializing_if = "Option::is_none")]
    snippet: Option<SnippetJson>,
}

/// Where a code block that a `@snippet` became takes its code from.
#[derive(Serialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "json-schema", schemars(rename = "Snippet"))]
struct SnippetJson {
    /// The address, as written.
    address: String,
    /// The code file, from the project root.
    path: String,
    /// The lines of the code file it covers, first and last.
    lines: Option<[u32; 2]>,
    /// Whether the file was tested, when it's a tested example
    /// (decision #458c); absent on any other file.
    #[serde(skip_serializing_if = "Option::is_none")]
    tested: Option<TestedJson>,
}

/// A tested example's status, and the observations current for it.
#[derive(Serialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "json-schema", schemars(rename = "Tested"))]
struct TestedJson {
    /// `current`, `stale`, or `unobserved`.
    status: &'static str,
    /// The observations current for it, in name order: empty unless it's
    /// `current`.
    observations: Vec<ObservationJson>,
}

/// An observation current for an example.
#[derive(Serialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "json-schema", schemars(rename = "Observation"))]
struct ObservationJson {
    /// Its name: `default`, or the name `ascribe observe --name` gave it.
    name: String,
    /// Its environment as it has it, in its order: `os`, `arch`, then each
    /// entry's kept line, such as `"node": "v24.9.0"`.
    environment: Environment,
}

/// Name and value pairs, written as an object in their order.
struct Environment(Vec<(String, String)>);

impl Serialize for Environment {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeMap;
        let mut map = serializer.serialize_map(Some(self.0.len()))?;
        for (key, value) in &self.0 {
            map.serialize_entry(key, value)?;
        }
        map.end()
    }
}

#[cfg(feature = "json-schema")]
impl schemars::JsonSchema for Environment {
    fn schema_name() -> std::borrow::Cow<'static, str> {
        "Environment".into()
    }

    fn json_schema(_: &mut schemars::SchemaGenerator) -> schemars::Schema {
        schemars::json_schema!({
            "description": "An environment's names and values, in its order.",
            "type": "object",
            "additionalProperties": { "type": "string" }
        })
    }
}

/// A block's type, and the fields of that type.
#[derive(Serialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(tag = "type", rename_all = "snake_case")]
enum BlockBody {
    /// A heading.
    #[serde(rename_all = "snake_case")]
    Heading {
        /// Its level, 1 to 6.
        level: u8,
        /// Its page id.
        id: Option<String>,
        /// Its source id.
        source_id: Option<String>,
        /// Whether an `@id` gave it its id.
        explicit: bool,
        /// Its text, as plain text.
        text: Option<String>,
        /// Its text.
        inlines: Vec<InlineJson>,
        /// The section's summary (SPEC §4.12), when it has one.
        #[serde(skip_serializing_if = "Option::is_none")]
        summary: Option<SummaryJson>,
    },
    /// A paragraph.
    Paragraph {
        /// Its text.
        inlines: Vec<InlineJson>,
    },
    /// A code block.
    Code {
        /// Whether it's fenced, rather than indented.
        fenced: bool,
        /// Its info string.
        info: String,
        /// Its code.
        literal: String,
    },
    /// A block quote.
    BlockQuote {
        /// Its blocks.
        children: Vec<BlockJson>,
    },
    /// A list.
    List {
        /// Whether it's numbered.
        ordered: bool,
        /// The first item's number, for a numbered list.
        start: Option<u64>,
        /// Whether it's tight: no blank line between its items.
        tight: bool,
        /// Its items.
        items: Vec<ItemJson>,
    },
    /// A block of HTML.
    Html {
        /// The HTML.
        literal: String,
    },
    /// A thematic break.
    ThematicBreak,
    /// A table.
    Table {
        /// Each column's alignment: `none`, `left`, `center`, or `right`.
        alignments: Vec<&'static str>,
        /// Its rows.
        rows: Vec<RowJson>,
    },
    /// A line-form directive that survived: `@available` with its annotation,
    /// or a `@note`, `@steps`, `@details`, or widget that binds the block
    /// that follows it.
    Directive {
        /// Its name.
        name: String,
        /// Its attributes.
        attributes: Vec<AttrJson>,
        /// Its title.
        title: Option<Vec<InlineJson>>,
        /// Its primary value.
        primary: Option<PrimaryJson>,
        /// What it binds: `own`, `heading`, `following_block`, or `unbound`.
        binding: Option<&'static str>,
        /// An `@available`'s spec.
        annotation: Option<AnnotationJson>,
    },
    /// An `@operation` that names an operation the spec has (SPEC §4.13).
    #[serde(rename_all = "snake_case")]
    Operation {
        /// Its API, the name of its `[apis.<name>]`.
        api: String,
        /// The operation's id, when it has one.
        #[serde(skip_serializing_if = "Option::is_none")]
        operation_id: Option<String>,
        /// The operation's HTTP method.
        method: String,
        /// The operation's path.
        path: String,
        /// The operation's summary, when it has one.
        #[serde(skip_serializing_if = "Option::is_none")]
        summary: Option<String>,
        /// Whether the spec marks it deprecated.
        deprecated: bool,
        /// The operation's address in the published reference.
        address: String,
    },
    /// A container directive.
    Container {
        /// Its name.
        name: String,
        /// Its attributes.
        attributes: Vec<AttrJson>,
        /// Its title.
        title: Option<Vec<InlineJson>>,
        /// Its blocks.
        children: Vec<BlockJson>,
    },
    /// A group, such as `@variant`, with its surviving arms.
    Group {
        /// Its name.
        name: String,
        /// Its surviving arms.
        arms: Vec<ArmJson>,
    },
}

/// A list item.
#[derive(Serialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "json-schema", schemars(rename = "Item"))]
struct ItemJson {
    /// Its blocks.
    children: Vec<BlockJson>,
}

/// A table row.
#[derive(Serialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "json-schema", schemars(rename = "Row"))]
struct RowJson {
    /// Whether it's the header row.
    header: bool,
    /// The row's own availability, in its table's (SPEC §4.4).
    #[serde(skip_serializing_if = "Option::is_none")]
    availability: Option<AvailabilityJson>,
    /// Its cells.
    cells: Vec<CellJson>,
}

/// A table cell.
#[derive(Serialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "json-schema", schemars(rename = "Cell"))]
struct CellJson {
    /// Its text.
    inlines: Vec<InlineJson>,
}

/// A directive's primary value.
#[derive(Serialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(tag = "kind", rename_all = "snake_case")]
#[cfg_attr(feature = "json-schema", schemars(rename = "Primary"))]
enum PrimaryJson {
    /// Text.
    Text {
        /// The text.
        inlines: Vec<InlineJson>,
    },
    /// An identifier.
    Identifier {
        /// The identifier.
        text: String,
    },
    /// The rest of the line, as written.
    Line {
        /// The line.
        text: String,
    },
}

/// A section's summary: its text, and the form that gives it, `text` for
/// `@summary: text` and `paragraph` for `@summary` above a paragraph.
#[derive(Serialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "json-schema", schemars(rename = "Summary"))]
struct SummaryJson {
    /// `text` or `paragraph`.
    form: &'static str,
    /// The summary's text.
    inlines: Vec<InlineJson>,
    /// The `@summary` line for the text form, and the paragraph for the
    /// other.
    source: SourceJson,
}

/// An `@available`'s spec.
#[derive(Serialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "json-schema", schemars(rename = "Annotation"))]
struct AnnotationJson {
    /// The spec as shown; for a feature key, the spec it stands for.
    text: String,
    /// The spec with the content model's display labels.
    display: Option<String>,
    /// The feature key it was written as, when it was one.
    feature: Option<String>,
}

/// A surviving arm of a group.
#[derive(Serialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "json-schema", schemars(rename = "Arm"))]
struct ArmJson {
    /// The arm's label: its title as plain text, or the display labels of the
    /// values it names.
    label: Option<String>,
    /// Its attributes.
    attributes: Vec<AttrJson>,
    /// Its title.
    title: Option<Vec<InlineJson>>,
    /// Its span.
    span: SpanJson,
    /// Its blocks.
    children: Vec<BlockJson>,
}

// Links, glossary, substitutions.

/// A link or image of a block, resolved.
#[derive(Serialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "json-schema", schemars(rename = "Link"))]
struct LinkJson {
    /// Its span.
    span: SpanJson,
    /// `link` or `image`.
    kind: &'static str,
    /// Its destination, as written.
    destination: String,
    /// What it resolved to.
    target: TargetJson,
}

/// What a link or image resolved to.
#[derive(Serialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(tag = "type", rename_all = "snake_case")]
#[cfg_attr(feature = "json-schema", schemars(rename = "Target"))]
enum TargetJson {
    /// An external link.
    External,
    /// A page the build publishes.
    #[serde(rename_all = "snake_case")]
    Page {
        /// The page's source path.
        page: String,
        /// The page id it names, if any.
        id: Option<String>,
        /// Its URL.
        url: String,
        /// Whether the link had no text and took the page's title.
        text_filled: bool,
        /// The line a link out of a line leads to; absent for any other
        /// link.
        #[serde(skip_serializing_if = "Option::is_none")]
        line: Option<String>,
    },
    /// An asset.
    #[serde(rename_all = "snake_case")]
    Asset {
        /// The asset's source path.
        source: String,
        /// The copy's path in this output.
        path: String,
        /// How this page refers to the copy.
        reference: String,
        /// The copy's URL.
        url: Option<String>,
        /// The fragment, without its `#`.
        fragment: Option<String>,
    },
    /// An operation of an API spec.
    #[serde(rename_all = "snake_case")]
    Operation {
        /// Its API, the name of its `[apis.<name>]`.
        api: String,
        // An operation's id, or, when it has none, its method and path.
        /// The operation's id, when it has one.
        #[serde(skip_serializing_if = "Option::is_none")]
        operation_id: Option<String>,
        /// The operation's HTTP method, for an operation with no id.
        #[serde(skip_serializing_if = "Option::is_none")]
        method: Option<String>,
        /// The operation's path, for an operation with no id.
        #[serde(skip_serializing_if = "Option::is_none")]
        path: Option<String>,
        /// The operation's address in the published reference.
        address: String,
    },
    /// The build couldn't resolve it; the checks report why.
    Unresolved,
}

/// A glossary term linked in a block.
#[derive(Serialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "json-schema", schemars(rename = "GlossaryTerm"))]
struct GlossaryJson {
    /// The term.
    term: String,
    /// The text that links it.
    text: String,
    /// The term's URL.
    url: String,
    /// The term's definition, as plain text.
    definition: String,
}

/// A phrase replaced in a block's text.
#[derive(Serialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "json-schema", schemars(rename = "Substitution"))]
struct SubstitutionJson {
    /// Its span over the source.
    span: SpanJson,
    /// The phrase's key.
    key: String,
    /// The text that replaced it.
    value: String,
}

// Inlines.

/// An inline: its `type` and the fields of that type, and its span in the
/// block's file.
#[derive(Serialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "json-schema", schemars(rename = "Inline"))]
struct InlineJson {
    #[serde(flatten)]
    body: InlineBody,
    /// Its span in the block's file.
    span: SpanJson,
}

/// An inline's type, and the fields of that type.
#[derive(Serialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(tag = "type", rename_all = "snake_case")]
enum InlineBody {
    /// Text.
    Text {
        /// The text, decoded.
        value: String,
    },
    /// A code span.
    Code {
        /// Its content.
        value: String,
    },
    /// A soft line break.
    SoftBreak,
    /// A hard line break.
    HardBreak,
    /// Inline HTML.
    Html {
        /// The HTML.
        value: String,
    },
    /// Emphasis.
    Emphasis {
        /// What it emphasizes.
        children: Vec<InlineJson>,
    },
    /// Strong emphasis.
    Strong {
        /// What it emphasizes.
        children: Vec<InlineJson>,
    },
    /// A link.
    Link {
        /// A page link's route and id, or an external link's destination as
        /// written.
        destination: String,
        /// Its title.
        title: Option<String>,
        /// The block's resolved link at the same span, or `null` for a
        /// glossary link.
        target: Option<TargetJson>,
        /// Its text.
        children: Vec<InlineJson>,
    },
    /// An image.
    Image {
        /// Its destination.
        destination: String,
        /// Its title.
        title: Option<String>,
        /// The block's resolved image at the same span.
        target: Option<TargetJson>,
        /// Its alternative text, as plain text.
        alt: String,
        /// Its attributes, such as `width`.
        attributes: Vec<AttrJson>,
    },
    /// A phrase the resolution left, which it doesn't.
    Phrase {
        /// The phrase's key.
        key: String,
    },
}

/// The schema of a page of the JSON output, for `schemas/`.
#[cfg(feature = "json-schema")]
pub fn page_schema(generator: &mut schemars::SchemaGenerator) -> schemars::Schema {
    generator.root_schema_for::<PageDoc>()
}

// ---------------------------------------------------------------------------
// Building it

/// A piece of a formatted value, as the JSON output writes it:
/// `{ "type": "text" | "code", "value": … }`. Other JSON that shows a
/// formatted title, such as the HTML preview's, writes it the same way.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
pub struct FormattedPiece {
    /// What the piece is.
    #[serde(rename = "type")]
    pub kind: PieceKind,
    /// Its text: a code span's content without its backticks.
    pub value: String,
}

/// What a piece of a formatted value is.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "lowercase")]
pub enum PieceKind {
    /// Text.
    Text,
    /// A code span.
    Code,
}

/// A formatted value's pieces, as inlines are written but without spans,
/// since the value comes from the frontmatter.
pub fn formatted_pieces(segments: &[Segment]) -> Vec<FormattedPiece> {
    segments
        .iter()
        .map(|segment| {
            let (kind, value) = match segment {
                Segment::Text(v) => (PieceKind::Text, v),
                Segment::Code(v) => (PieceKind::Code, v),
            };
            FormattedPiece {
                kind,
                value: value.clone(),
            }
        })
        .collect()
}

/// The page's title, formatted, when its field sets `inline`.
pub fn formatted_title(page: &ResolvedPage) -> Option<Vec<FormattedPiece>> {
    page.formatted_title().map(formatted_pieces)
}

/// A formatted field's value: [`formatted_pieces`], as JSON.
fn formatted_value(segments: &[Segment]) -> serde_json::Value {
    serde_json::to_value(formatted_pieces(segments)).unwrap_or_default()
}

struct Writer<'a, 'c> {
    cx: &'a PageContext<'c>,
    /// The page's section summaries, for each heading's `summary`.
    summaries: &'a [SectionSummary],
}

impl Writer<'_, '_> {
    fn page(&self, page: &ResolvedPage) -> PageDoc {
        let emit = self.cx.emit;
        PageDoc {
            schema_version: JSON_SCHEMA_VERSION,
            format: "ascribe-page",
            build: page.build.clone(),
            path: page.path.to_string(),
            route: page.route.clone(),
            site: emit.site_origin().map(str::to_owned),
            title: page.title.clone(),
            // A frontmatter mapping with a key that isn't a string can't be
            // JSON; it's left out (`ascribe check` reports the key).
            frontmatter: page
                .frontmatter
                .as_ref()
                .and_then(|f| serde_json::to_value(f).ok()),
            formatted: page
                .formatted
                .iter()
                .map(|field| (field.name.clone(), formatted_value(&field.segments)))
                .collect(),
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

    fn asset(&self, _page: &ResolvedPage, asset: &ascribe_resolve::PageAsset) -> AssetJson {
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
            targets: (!self.cx.emit.model.releases.is_empty()).then(|| {
                a.spec
                    .entries
                    .iter()
                    .map(|entry| {
                        let view = availability_target(self.cx.emit.model, entry);
                        AvailabilityTargetJson {
                            target: view.target,
                            dimension: view.dimension,
                            states: view.states,
                            versions: view.versions,
                            planned: view.planned,
                            text: view.text,
                        }
                    })
                    .collect()
            }),
            scope: match a.scope {
                Scope::Page => "page",
                Scope::Section => "section",
                Scope::Block => "block",
                Scope::Row => "row",
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
            snippet: block.snippet.as_deref().map(|s| SnippetJson {
                address: s.address.clone(),
                path: s.path.to_string(),
                lines: s.lines.map(|(first, last)| [first, last]),
                tested: s.example.as_ref().map(|e| TestedJson {
                    status: e.status.as_str(),
                    observations: e
                        .current
                        .iter()
                        .map(|o| ObservationJson {
                            name: o.name.clone(),
                            environment: Environment(o.environment.clone()),
                        })
                        .collect(),
                }),
            }),
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
                    summary: block.heading.as_ref().and_then(|ids| {
                        let summary = self
                            .summaries
                            .iter()
                            .find(|s| s.anchor.as_deref() == Some(ids.page_id.as_str()))?;
                        Some(SummaryJson {
                            form: summary.form.name(),
                            inlines: self.inlines(&summary.block, summary.inlines()),
                            source: self.source(
                                summary.block.file,
                                summary.block.span,
                                &summary.block.via,
                            ),
                        })
                    }),
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
                            ascribe_syntax::Alignment::None => "none",
                            ascribe_syntax::Alignment::Left => "left",
                            ascribe_syntax::Alignment::Center => "center",
                            ascribe_syntax::Alignment::Right => "right",
                        })
                        .collect(),
                    rows: t
                        .rows
                        .iter()
                        .map(|row| RowJson {
                            header: row.header,
                            availability: block
                                .rows
                                .iter()
                                .find(|r| r.span == row.span)
                                .map(|r| self.availability(&r.availability)),
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
                BlockKind::Directive(_) if let Some(o) = block.operation.as_deref() => {
                    BlockBody::Operation {
                        api: o.api.clone(),
                        operation_id: o.operation_id.clone(),
                        method: o.method.clone(),
                        path: o.path.clone(),
                        summary: o.summary.clone(),
                        deprecated: o.deprecated,
                        address: o.address.clone(),
                    }
                }
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
                ascribe_syntax::Bound::Own => "own",
                ascribe_syntax::Bound::Heading => "heading",
                ascribe_syntax::Bound::FollowingBlock => "following_block",
                ascribe_syntax::Bound::Unbound => "unbound",
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
                line,
            } => TargetJson::Page {
                page: page.to_string(),
                id: id.clone(),
                url: url.clone(),
                text_filled: *text_filled,
                line: line.clone(),
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
            LinkTarget::Operation {
                api,
                operation_id,
                method,
                path,
                address,
            } => {
                let by_path = operation_id.is_none();
                TargetJson::Operation {
                    api: api.clone(),
                    operation_id: operation_id.clone(),
                    method: by_path.then(|| method.clone()),
                    path: by_path.then(|| path.clone()),
                    address: address.clone(),
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
        definition: g.definition.clone(),
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
