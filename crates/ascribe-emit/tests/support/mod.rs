//! Shared by the emitter tests: load a project from disk, resolve a build, and
//! emit it in memory.

#![allow(dead_code, clippy::expect_used, clippy::panic)]

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use ascribe_core::FileId;
use ascribe_emit::{Contents, EmitContext, Emitter, emit};
use ascribe_resolve::{AstroRouter, DiskFs, Layout, Project};
use html5ever::tendril::TendrilSink;
use html5ever::{ParseOpts, QualName, local_name, ns, parse_fragment};
use markup5ever_rcdom::{Handle, NodeData, RcDom};

/// `examples/quill`, from a test in this crate.
pub fn quill() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/quill")
}

/// Loads the project whose `ascribe.toml` is in `root`.
pub fn load(root: &Path) -> Project {
    let model = ascribe_model::load_with_file(root.join("ascribe.toml"), FileId::new(0))
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
    let model = ascribe_model::load_str(model, FileId::new(0))
        .unwrap_or_else(|issues| panic!("the model has errors: {issues:?}"));
    let layout = Layout::from_model(&model);
    let mut fs = ascribe_resolve::MemoryFs::new(&layout);
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
        &ascribe_emit::PlainEmitter::default(),
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
    let emitter = ascribe_emit::SiteEmitter::new(project.model());
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
    let model = ascribe_model::load_str(model, FileId::new(0))
        .unwrap_or_else(|issues| panic!("the model has errors: {issues:?}"));
    let layout = Layout::from_model(&model);
    let mut fs = ascribe_resolve::MemoryFs::new(&layout);
    for (path, text) in files {
        fs = fs.with_source(path, text);
    }
    for (path, text) in assets {
        fs = fs.with_file(path, text);
    }
    Project::load(Arc::new(model), layout, &fs)
}

/// Parses an HTML fragment in a `<body>` context and writes it as a
/// normalized tree, one node per line, as `tests/render/README.md` says:
/// element names, attributes as a sorted set, text exactly, comments exactly;
/// whitespace-only text is dropped except inside `<pre>`. With `drop_anchors`,
/// `data-ascribe-source` and `data-ascribe-via` are left out.
pub fn html_tree(html: &str, drop_anchors: bool) -> Vec<String> {
    let dom = parse_fragment(
        RcDom::default(),
        ParseOpts::default(),
        QualName::new(None, ns!(html), local_name!("body")),
        Vec::new(),
        false,
    )
    .one(html);
    let mut lines = Vec::new();
    // The fragment's nodes are the children of the one root element.
    for root in dom.document.children.borrow().iter() {
        for child in root.children.borrow().iter() {
            walk(child, 0, false, drop_anchors, &mut lines);
        }
    }
    lines
}

fn walk(node: &Handle, depth: usize, in_pre: bool, drop_anchors: bool, out: &mut Vec<String>) {
    let pad = "  ".repeat(depth);
    match &node.data {
        NodeData::Element { name, attrs, .. } => {
            let mut attributes: Vec<String> = attrs
                .borrow()
                .iter()
                .filter(|a| {
                    !(drop_anchors
                        && matches!(&*a.name.local, "data-ascribe-source" | "data-ascribe-via"))
                })
                .map(|a| format!("{}={:?}", a.name.local, a.value.to_string()))
                .collect();
            attributes.sort();
            let tag = name.local.to_string();
            out.push(format!("{pad}<{tag} {}>", attributes.join(" ")));
            let pre = in_pre || tag == "pre";
            for child in node.children.borrow().iter() {
                walk(child, depth + 1, pre, drop_anchors, out);
            }
        }
        NodeData::Text { contents } => {
            let text = contents.borrow().to_string();
            if in_pre || !text.trim().is_empty() {
                out.push(format!("{pad}{text:?}"));
            }
        }
        NodeData::Comment { contents } => out.push(format!("{pad}<!--{contents}-->")),
        _ => {}
    }
}

/// The first difference between two trees from [`html_tree`]: its index and
/// the two nodes there.
pub fn first_difference(want: &[String], got: &[String]) -> Option<(usize, String, String)> {
    if want == got {
        return None;
    }
    let at = want
        .iter()
        .zip(got)
        .position(|(w, g)| w != g)
        .unwrap_or(want.len().min(got.len()));
    Some((
        at,
        want.get(at).cloned().unwrap_or_default(),
        got.get(at).cloned().unwrap_or_default(),
    ))
}
