//! The site output (SPEC §9.4): markdown plus web components, for a consumer
//! that renders CommonMark with raw HTML. Spec 0.1's consumer is Astro.
//!
//! What a page becomes:
//!
//! | Source | Site output |
//! |---|---|
//! | Frontmatter | Passed through; `available` as a list of targets (Q142) |
//! | Heading | An ATX heading ending in a `<tessera-attributes>` marker with its page id |
//! | Image with attributes | The image, then a marker with the attributes |
//! | `@note` | `<tessera-note type label heading>` wrapping the content |
//! | `@steps` | `<tessera-steps>` wrapping the list |
//! | `@variant` group | `<tessera-tabs sync>` of `<tessera-tab value label>`, when several arms survive |
//! | `@details` | `<details>` with a `<summary>` |
//! | `@available` | `<tessera-availability scope>` of `<tessera-availability-target>` |
//! | Project widget | An element named after the widget, and `<tessera-group>` for groups |
//! | Glossary term | An ordinary link, with the definition as its title |
//! | Raw HTML | Passed through unchanged |
//!
//! The element contract (`packages/elements/CONTRACT.md`) is the interface
//! with the element library; the site-render contract
//! (`project-docs/contracts/site-render.md`) is the interface with the
//! consumer's markdown pipeline, and [`crate::render_site_html`] implements
//! it. Links are the consumer's routes, and assets are placed as the `astro`
//! profile says ([`AstroProfile`]).
//!
//! Alongside the pages the emitter writes `_ascribe/schema.ts`, the Zod
//! schema of every content type ([`crate::zod`]).

mod blocks;
mod element;
mod frontmatter;
mod inline;
mod profile;

pub use profile::{ASTRO_VERSION, AstroProfile};

use tessera_core::{AssetPlacement, AssetUse, ConsumerProfile, RelPath};
use tessera_model::ContentModel;
use tessera_resolve::{ResolvedBuild, ResolvedPage};

use crate::assets::{Placement, mirrored_path, relative_reference};
use crate::emitter::{EmitContext, Emitter, PageContext};
use crate::error::EmitError;
use crate::store::{Contents, EmittedFile, FileKind};

/// The path of the generated Zod schema in the emitter root.
pub const SCHEMA_PATH: &str = "_ascribe/schema.ts";

/// The directory in the emitter root that holds link targets, which the
/// consumer serves as static files (asset contract §3.2).
pub const FILES_DIR: &str = "_ascribe/files";

/// The site emitter.
#[derive(Clone, Debug)]
pub struct SiteEmitter {
    profile: AstroProfile,
}

impl SiteEmitter {
    /// An emitter for the content model's consumer profile.
    pub fn new(model: &ContentModel) -> SiteEmitter {
        SiteEmitter {
            profile: AstroProfile::from_consumer(&model.consumer),
        }
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
        // Resolved Q143: two pages with one route can't both be
        // published.
        let collisions = self
            .profile
            .astro_router()
            .collisions(build.pages.iter().map(|p| &p.path));
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
                "the site output can't publish two pages at one route: {}. Rename one of the files",
                lines.join("; ")
            ),
        })
    }

    fn render_page(&self, cx: &PageContext<'_>, page: &ResolvedPage) -> Result<String, EmitError> {
        let renderer = blocks::Renderer {
            page: cx,
            model: cx.emit.model,
        };
        let mut out = frontmatter::render(cx, page)?;
        let body = renderer.blocks(&page.blocks).join("\n\n");
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
