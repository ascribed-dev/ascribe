//! The site output (SPEC §9.4): markdown plus web components, for a consumer
//! that renders CommonMark with raw HTML. Spec 0.1's consumer is Astro.
//!
//! What a page becomes:
//!
//! | Source | Site output |
//! |---|---|
//! | Frontmatter | Passed through; `available` as a list of targets; `inline` fields as plain text, and as HTML under `formatted` |
//! | Heading | An ATX heading ending in a `<ascribe-attributes>` marker with its page id |
//! | Image with attributes | The image, then a marker with the attributes |
//! | `@note` | `<ascribe-note type label heading>` wrapping the content |
//! | `@steps` | `<ascribe-steps>` wrapping the list |
//! | `@variant` group | `<ascribe-tabs sync>` of `<ascribe-tab value label>`, when several arms survive |
//! | `@details` | `<details>` with a `<summary>` |
//! | `@available` | `<ascribe-availability scope>` of `<ascribe-availability-target>` |
//! | Project widget | An element named after the widget, and `<ascribe-group>` for groups |
//! | Glossary term | An ordinary link, with the definition as its title |
//! | Raw HTML | Passed through unchanged |
//!
//! The element contract (`packages/elements/CONTRACT.md`) is the interface
//! with the element library; the `<ascribe-attributes>` markers are the
//! interface with the consumer's markdown pipeline, and
//! [`crate::render_site_html`] applies them. Links are the consumer's routes, and assets are placed as the `astro`
//! profile says ([`AstroProfile`]).
//!
//! Alongside the pages the emitter writes `_ascribe/schema.ts`, the Zod
//! schema of every content type ([`crate::zod`]).

mod anchor;
mod blocks;
mod element;
mod frontmatter;
mod inline;
mod profile;

pub(crate) use anchor::starts_anchor;
pub use profile::{ASTRO_VERSION, AstroProfile};

use ascribe_core::{AssetPlacement, AssetUse, ConsumerProfile, RelPath, names};
use ascribe_model::ContentModel;
use ascribe_resolve::{AstroRouter, ResolvedBuild, ResolvedPage};

use crate::assets::{Placement, mirrored_path, relative_reference};
use crate::emitter::{EmitContext, Emitter, PageContext};
use crate::error::EmitError;
use crate::store::{Contents, EmittedFile, FileKind};

/// The path of the generated Zod schema in the emitter root.
pub const SCHEMA_PATH: &str = "_ascribe/schema.ts";

/// The directory in the emitter root that holds link targets, which the
/// consumer serves as static files.
pub const FILES_DIR: &str = "_ascribe/files";

/// The site emitter.
#[derive(Clone, Debug)]
pub struct SiteEmitter {
    profile: AstroProfile,
    anchors: bool,
}

impl SiteEmitter {
    /// An emitter for the content model's consumer profile, without source
    /// anchors.
    pub fn new(model: &ContentModel) -> SiteEmitter {
        SiteEmitter {
            profile: AstroProfile::from_consumer(&model.consumer),
            anchors: false,
        }
    }

    /// The emitter with source anchors on or off (site-render contract §7):
    /// with them on, every block says which source lines it came from. Off,
    /// the output is exactly what it is without this call.
    pub fn with_anchors(mut self, anchors: bool) -> SiteEmitter {
        self.anchors = anchors;
        self
    }

    /// Whether the emitter writes source anchors.
    pub fn anchors(&self) -> bool {
        self.anchors
    }

    /// The consumer profile the emitter writes for.
    pub fn profile(&self) -> &AstroProfile {
        &self.profile
    }
}

impl Emitter for SiteEmitter {
    fn name(&self) -> &'static str {
        "site"
    }

    fn page_path(&self, page: &RelPath) -> RelPath {
        page.clone()
    }

    fn prepare(&self, _cx: &EmitContext<'_>, build: &ResolvedBuild) -> Result<(), EmitError> {
        route_collisions(self.profile.astro_router(), build, "the site output")
    }

    fn render_page(&self, cx: &PageContext<'_>, page: &ResolvedPage) -> Result<String, EmitError> {
        let renderer = blocks::Renderer {
            page: cx,
            model: cx.emit.model,
            anchors: self.anchors,
        };
        let mut out = frontmatter::render(cx, page)?;
        let mut chunks = Vec::new();
        if let Some(pointer) = agents_pointer(cx, page) {
            chunks.push(pointer);
        }
        chunks.extend(renderer.blocks(&page.blocks));
        let body = chunks.join("\n\n");
        out.push_str(&body);
        if !body.is_empty() {
            out.push('\n');
        }
        Ok(out)
    }

    fn place_asset(&self, page_output: &RelPath, asset: &RelPath, usage: AssetUse) -> Placement {
        let mirrored = mirrored_path(asset);
        match self.profile.asset_placement(usage) {
            AssetPlacement::Mirror => Placement {
                reference: relative_reference(page_output, &mirrored),
                copy_to: mirrored,
                url: None,
            },
            AssetPlacement::Published => {
                let copy_to = RelPath::parse(&format!("{FILES_DIR}/{mirrored}"))
                    .unwrap_or_else(|_| mirrored.clone());
                let mut url = self.profile.astro_router().base().to_owned();
                let encoded: Vec<String> = copy_to.segments().map(encode_segment).collect();
                url.push_str(&encoded.join("/"));
                Placement {
                    reference: url.clone(),
                    copy_to,
                    url: Some(url),
                }
            }
        }
    }

    fn generated(
        &self,
        cx: &EmitContext<'_>,
        _build: &ResolvedBuild,
    ) -> Result<Vec<EmittedFile>, EmitError> {
        let path = RelPath::parse(SCHEMA_PATH).map_err(|e| EmitError::Invalid {
            message: format!("bad schema path: {e}"),
        })?;
        Ok(vec![EmittedFile {
            path,
            kind: FileKind::Generated,
            source: None,
            url: None,
            contents: Contents::Text(crate::zod::generate(cx.model)),
        }])
    }
}

/// The pointer to `llms.txt` at the top of a page, with `[consumer] agents
/// = true` (element contract §8): an `<ascribe-for-agents>` element, which the
/// element library hides from sight but not from an agent reading the page,
/// linking to the index and to the page's Markdown version.
fn agents_pointer(cx: &PageContext<'_>, page: &ResolvedPage) -> Option<String> {
    let consumer = &cx.emit.model.consumer;
    if !consumer.agents {
        return None;
    }
    // Root-relative, as every link in the site output is: a page in HTML
    // carries its URL, so a reader resolves them (the delivery spec asks
    // for absolute URLs in Markdown only).
    let router = AstroRouter::from_consumer(consumer);
    let index = router.url_of("llms.txt");
    let markdown = router.markdown_url(&page.path);
    Some(format!(
        "<{tag}>\n\nFor AI agents: the documentation index is at [llms.txt]({index}), and this page is available as [Markdown]({markdown}).\n\n</{tag}>",
        tag = names::ELEMENT_FOR_AGENTS
    ))
}

/// Refuses a build with two pages at one route, which `output` (an output's
/// name in a sentence) can't publish both of.
pub(crate) fn route_collisions(
    router: &AstroRouter,
    build: &ResolvedBuild,
    output: &str,
) -> Result<(), EmitError> {
    let collisions = router.collisions(build.pages.iter().map(|p| &p.path));
    if collisions.is_empty() {
        return Ok(());
    }
    let lines: Vec<String> = collisions
        .iter()
        .map(|(route, pages)| {
            let names: Vec<String> = pages.iter().map(|p| p.to_string()).collect();
            format!("{route} is the route of {}", names.join(" and "))
        })
        .collect();
    Err(EmitError::Invalid {
        message: format!(
            "{output} can't publish two pages at one route: {}. Rename one of the files",
            lines.join("; ")
        ),
    })
}

/// Percent-encodes a URL path segment: everything but unreserved characters.
fn encode_segment(segment: &str) -> String {
    let mut out = String::with_capacity(segment.len());
    for byte in segment.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_' | b'~') {
            out.push(char::from(byte));
        } else {
            out.push_str(&format!("%{byte:02X}"));
        }
    }
    out
}
