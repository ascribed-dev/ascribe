//! A compact index of one build's docs, for the `AGENTS.md` of the product
//! the docs are about ([`ascribe_resolve::llms::compact_index`]), and, when
//! asked, the Markdown version of each page it offers, to copy into a folder
//! the index points into.

use std::path::Path;

use ascribe_emit::{EmitContext, PlainEmitter, emit_page};
use ascribe_model::Build;
use ascribe_resolve::{AstroRouter, Project, llms, pages};
use serde::Serialize;

use crate::{ASCRIBE_VERSION, QueryError, SCHEMA_VERSION};

/// What `ascribe agents index --format json` answers.
#[derive(Debug, Serialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
pub struct AgentsIndex {
    /// The version of this schema. It changes only when a field is removed
    /// or changes meaning.
    pub schema_version: u32,
    /// The version of Ascribe that wrote it.
    pub ascribe_version: &'static str,
    /// The build.
    pub build: String,
    /// The most bytes the index holds.
    pub limit: usize,
    /// How many bytes it holds.
    pub bytes: usize,
    /// How many pages the build offers: those it publishes, apart from the
    /// `unlisted`.
    pub offered: usize,
    /// The pages the index lists, as paths from the content root, in its
    /// order.
    pub listed: Vec<String>,
    /// The offered pages the index leaves out to stay under `limit`, as paths
    /// from the content root.
    pub left_out: Vec<String>,
    /// The pages the index lists without their description, to stay under
    /// `limit`, as paths from the content root.
    pub without_description: Vec<String>,
    /// The file the index was written into, between its markers, from the
    /// folder the command ran in; `null` when it was printed.
    pub file: Option<String>,
    /// Whether that file changed.
    pub changed: bool,
    /// The folder the pages were copied into, from the folder the command
    /// ran in; `null` without `--pages`.
    pub folder: Option<String>,
    /// The pages copied into the folder: every offered page, with
    /// `--pages`.
    pub pages: Vec<IndexPage>,
    /// The index's Markdown.
    pub text: String,
}

/// A page copied into the folder an index points into.
#[derive(Debug, Serialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
pub struct IndexPage {
    /// The page, as a path from the content root.
    pub source: String,
    /// Its Markdown version's path in the folder.
    pub path: String,
    /// Its Markdown version: what the build's `plain` output writes for it.
    #[serde(skip)]
    pub text: String,
}

/// The compact index of the pages `build` offers, at most `limit` bytes;
/// with `folder`, its addresses are paths in that folder, as the index
/// reads it (`docs` for an index beside a folder `docs/`), and the answer
/// carries each offered page's Markdown version, to write there.
/// `project_root` is the directory of `ascribe.toml`. Writes nothing.
///
/// # Errors
///
/// A page can't be rendered.
pub fn agents_index(
    project: &Project,
    project_root: &Path,
    build: &Build,
    folder: Option<&str>,
    limit: usize,
) -> Result<AgentsIndex, QueryError> {
    // An older line's pages are routed under its folder.
    let project = &*project.for_build(build);
    let model = project.model();
    let router = AstroRouter::from_consumer(&model.consumer);
    let resolved = project.resolve_build(build, &router);
    let records = pages::records(model, &resolved.pages);
    let index = llms::compact_index(model, &records, folder, limit);
    let mut copied = Vec::new();
    if folder.is_some() {
        let cx = EmitContext::new(project, project_root, build);
        let emitter = PlainEmitter::new(model);
        for (page, record) in resolved.pages.iter().zip(&records.pages) {
            if !record.offered() {
                continue;
            }
            copied.push(IndexPage {
                source: page.path.to_string(),
                path: router.markdown_path(&page.path),
                text: emit_page(&emitter, &cx, page)?.text,
            });
        }
    }
    let strings = |pages: &[ascribe_core::RelPath]| pages.iter().map(|p| p.to_string()).collect();
    Ok(AgentsIndex {
        schema_version: SCHEMA_VERSION,
        ascribe_version: ASCRIBE_VERSION,
        build: build.name.clone(),
        limit,
        bytes: index.text.len(),
        offered: records.pages.iter().filter(|r| r.offered()).count(),
        listed: strings(&index.listed),
        left_out: strings(&index.left_out),
        without_description: strings(&index.without_description),
        file: None,
        changed: false,
        folder: None,
        pages: copied,
        text: index.text,
    })
}
