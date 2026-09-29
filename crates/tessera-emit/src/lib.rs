//! Emitters for Ascribe's outputs: site markdown, plain markdown, and JSON, plus Zod schema generation.
//!
//! Phase 18 adds the plain-markdown and JSON emitters and the shared output
//! handling; phase 20 adds site output.
//!
//! # What an emitter does
//!
//! An [`Emitter`] renders pages from phase 12's **resolved tree**
//! ([`tessera_resolve::ResolvedPage`]). It writes what survived the build and
//! never works out what a build mode would keep (SPEC §9.2), so a selection
//! build that keeps several arms of a group emits all of them, and a filter
//! build emits the availability annotations still attached.
//!
//! [`emit`] walks a [`tessera_resolve::ResolvedBuild`] and returns an
//! [`Emission`]: every page, and one copy of every asset the surviving pages
//! use, at the paths and with the references the
//! [asset contract](../../../project-docs/contracts/assets.md) gives.
//!
//! # Writing the output
//!
//! [`OutputDir`] implements the
//! [output-layout contract](../../../project-docs/contracts/output-layout.md):
//! it locks the output directory, stages the files, replaces the previous
//! output only when the new one is complete, removes files the previous
//! manifest listed that this build no longer produces, and never touches a
//! file a manifest didn't list.
//!
//! ```no_run
//! use std::path::Path;
//! use tessera_emit::{EmitContext, Emitter, OutputDir, PlainEmitter, emit};
//! # fn demo(project: &tessera_resolve::Project, build: &tessera_model::Build, resolved: &tessera_resolve::ResolvedBuild) -> Result<(), Box<dyn std::error::Error>> {
//! let root = Path::new(".");
//! let cx = EmitContext::new(project, root, build);
//! let emission = emit(&PlainEmitter, &cx, resolved)?;
//! let output = OutputDir::lock(&root.join(".ascribe/build"))?;
//! output.replace(&build.name, PlainEmitter.name(), &emission.files)?;
//! # Ok(()) }
//! ```
//!
//! The JSON output's schema is in the crate's README.

pub mod assets;
mod emitter;
mod error;
mod json;
pub mod labels;
mod plain;
pub mod render;
pub mod site;
mod store;
pub mod zod;

pub use assets::Placement;
pub use emitter::{
    Emission, EmitContext, EmittedPage, Emitter, PageContext, PlacedAsset, Position, emit,
    emit_page,
};
pub use error::EmitError;
pub use json::{JSON_SCHEMA_VERSION, JsonEmitter};
pub use plain::PlainEmitter;
pub use render::render_site_html;
pub use site::{AstroProfile, SiteEmitter};
pub use store::{Contents, EmittedFile, FileKind, OutputDir, Replaced, StoreError};
