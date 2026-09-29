//! Helpers for building a project in memory.

#![allow(dead_code)]

use std::sync::Arc;

use tessera_core::{FileId, RelPath};
use tessera_model::{ContentModel, load_str};
use tessera_resolve::{Layout, MemoryFs, Project};

pub const MODEL: &str = r#"
spec = "0.1"

[project]
content-root = "docs"
output-dir = ".ascribe/build"

[phrases]
product = "Quill"
cloud = "Quill Cloud"
api = "https://api.quill.dev/v3/"

[builds.site]
variants = "switch"
availability = "badge"
"#;

pub fn model() -> Arc<ContentModel> {
    match load_str(MODEL, FileId::new(0)) {
        Ok(model) => Arc::new(model),
        Err(issues) => panic!("the test model doesn't load: {issues:?}"),
    }
}

pub fn path(text: &str) -> RelPath {
    RelPath::parse(text).expect("a valid path")
}

/// A project with these files, at paths relative to the project root.
/// `docs/…` files are in the content root.
pub fn project(files: &[(&str, &str)]) -> Project {
    let model = model();
    let layout = Layout::from_model(&model);
    let mut fs = MemoryFs::new(&layout);
    for (name, text) in files {
        fs = fs.with_file(name, text);
    }
    Project::load(model, layout, &fs)
}

/// The slugs of a list of issues, with the line each is on in `file`.
pub fn slugs(project: &Project, file: &str, issues: &[tessera_core::Issue]) -> Vec<(String, u32)> {
    let index = project
        .file(&path(file))
        .expect("the file is in the project");
    let lines = tessera_core::LineIndex::new(&index.source);
    let mut out: Vec<(String, u32)> = issues
        .iter()
        .filter(|i| project.path_of(i.location.file) == Some(&path(file)))
        .map(|i| {
            let at = lines.line_col(i.location.span.start()).expect("in range");
            (i.slug.to_string(), at.line + 1)
        })
        .collect();
    out.sort();
    out
}

pub fn has_slug(issues: &[tessera_core::Issue], slug: tessera_core::DiagnosticSlug) -> bool {
    issues.iter().any(|i| i.slug == slug)
}
