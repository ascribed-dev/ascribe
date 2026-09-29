//! Shared by the emitter tests: load a project from disk, resolve a build, and
//! emit it in memory.

#![allow(dead_code, clippy::expect_used, clippy::panic)]

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use tessera_core::FileId;
use tessera_emit::{Contents, EmitContext, Emitter, emit};
use tessera_resolve::{AstroRouter, DiskFs, Layout, Project};

/// `examples/quill`, from a test in this crate.
pub fn quill() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/quill")
}

/// Loads the project whose `ascribe.toml` is in `root`.
pub fn load(root: &Path) -> Project {
    let model = tessera_model::load_with_file(root.join("ascribe.toml"), FileId::new(0))
        .unwrap_or_else(|issues| panic!("the model has errors: {issues:?}"));
    let layout = Layout::from_model(&model);
    let fs = DiskFs::new(root, &layout);
    Project::load(Arc::new(model), layout, &fs)
}

/// The files one emitter writes for one build: path to text (a copied asset
/// shows as `<copy of SOURCE>`).
pub fn emit_build(
    root: &Path,
    project: &Project,
    build: &str,
    emitter: &dyn Emitter,
) -> BTreeMap<String, String> {
    let model_build = project.model().build(build).expect("a build of that name");
    let router = AstroRouter::from_consumer(&project.model().consumer);
    let resolved = project.resolve_build(model_build, &router);
    let cx = EmitContext::new(project, root, model_build);
    let emission = emit(emitter, &cx, &resolved).expect("the build emits");
    emission
        .files
        .into_iter()
        .map(|f| {
            let text = match f.contents {
                Contents::Text(text) => text,
                Contents::Copy(_) => format!(
                    "<copy of {}>",
                    f.source.map(|s| s.to_string()).unwrap_or_default()
                ),
            };
            (f.path.to_string(), text)
        })
        .collect()
}

/// The content model with every feature (`examples/content-models/full.toml`).
pub const FULL_MODEL: &str = include_str!("../../../../examples/content-models/full.toml");

/// A project held in memory: the content model text, and `(path, text)` for
/// each source file under the content root.
pub fn memory_project(model: &str, files: &[(&str, &str)]) -> Project {
    let model = tessera_model::load_str(model, FileId::new(0))
        .unwrap_or_else(|issues| panic!("the model has errors: {issues:?}"));
    let layout = Layout::from_model(&model);
    let mut fs = tessera_resolve::MemoryFs::new(&layout);
    for (path, text) in files {
        fs = fs.with_source(path, text);
    }
    Project::load(Arc::new(model), layout, &fs)
}

/// One page of a build, as plain markdown.
pub fn plain(project: &Project, build: &str, page: &str) -> String {
    emit_build(
        Path::new("/nowhere"),
        project,
        build,
        &tessera_emit::PlainEmitter,
    )
    .remove(page)
    .unwrap_or_else(|| panic!("the build has no page {page}"))
}

/// A page with frontmatter `title: Test` and this body.
pub fn page(body: &str) -> String {
    format!("---\ntitle: Test\n---\n\n{body}")
}

/// One page of a build, as site markdown.
pub fn site(project: &Project, build: &str, page: &str) -> String {
    let emitter = tessera_emit::SiteEmitter::new(project.model());
    emit_build(Path::new("/nowhere"), project, build, &emitter)
        .remove(page)
        .unwrap_or_else(|| panic!("the build has no page {page}"))
}

/// Like [`memory_project`], with files that aren't sources (assets), as
/// `(project path, text)`.
pub fn memory_project_with_files(
    model: &str,
    files: &[(&str, &str)],
    assets: &[(&str, &str)],
) -> Project {
    let model = tessera_model::load_str(model, FileId::new(0))
        .unwrap_or_else(|issues| panic!("the model has errors: {issues:?}"));
    let layout = Layout::from_model(&model);
    let mut fs = tessera_resolve::MemoryFs::new(&layout);
    for (path, text) in files {
        fs = fs.with_source(path, text);
    }
    for (path, text) in assets {
        fs = fs.with_file(path, text);
    }
    Project::load(Arc::new(model), layout, &fs)
}
