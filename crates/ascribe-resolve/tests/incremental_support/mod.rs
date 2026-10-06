//! Helpers for the incremental-update tests: models that differ in the parts
//! each stage reads, a world of files that changes step by step, and dumps of
//! everything a project can answer, to compare exactly.

#![allow(dead_code, clippy::expect_used, clippy::panic, clippy::unwrap_used)]

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::sync::Arc;

use ascribe_core::{FileId, RelPath};
use ascribe_model::{ContentModel, load_str};
use ascribe_resolve::{
    Change, DefaultRouter, FileIds, FileSystem, IncrementalProject, Layout, MemoryFs, Project,
    in_nested_project, is_source_path,
};

/// What differs between the models the tests use. Each field is read by a
/// different stage: widgets by the parser, phrases and fragment patterns by
/// the index, the rest by resolution.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModelSpec {
    /// 0: no widgets; 1: `@callout`; 2: `@callout` and `@aside`.
    pub widgets: u8,
    pub product: &'static str,
    pub extra_phrase: bool,
    pub hybrid: bool,
    pub cloud_build: bool,
    pub shared_pattern: bool,
    pub glossary: bool,
    /// A comment, which changes nothing about the model but its warnings'
    /// spans in a real `ascribe.toml`.
    pub comment: bool,
}

impl ModelSpec {
    pub fn base() -> ModelSpec {
        ModelSpec {
            widgets: 0,
            product: "Quill",
            extra_phrase: false,
            hybrid: false,
            cloud_build: true,
            shared_pattern: false,
            glossary: true,
            comment: false,
        }
    }

    pub fn text(&self) -> String {
        let mut t = String::new();
        if self.comment {
            t.push_str("# a comment\n");
        }
        t.push_str("spec = \"0.1\"\n\n[project]\ncontent-root = \"docs\"\noutput-dir = \".ascribe/build\"\n\n");
        let _ = writeln!(
            t,
            "[phrases]\nproduct = \"{}\"\napi = \"https://api.quill.dev/v3/\"",
            self.product
        );
        if self.extra_phrase {
            t.push_str("extra = \"Extra\"\n");
        }
        t.push_str("\n[dimensions.deployment]\nvalues = [\"cloud\", \"self-managed\"");
        if self.hybrid {
            t.push_str(", \"hybrid\"");
        }
        t.push_str("]\nversionless = [\"cloud\"]\n\n[versions]\nscheme = \"numeric\"\n\n");
        if self.shared_pattern {
            t.push_str("[fragments]\npatterns = [\"shared/**\"]\n\n");
        }
        if self.widgets >= 1 {
            t.push_str("[widgets.quill-callout]\nforms = [\"line\"]\nprimary = \"text?\"\nbinding = \"block\"\n\n");
        }
        if self.widgets >= 2 {
            t.push_str("[widgets.quill-aside]\nforms = [\"line\", \"container\"]\nprimary = \"text?\"\nbinding = \"block\"\ntitle = \"accepted\"\n\n");
        }
        if self.glossary {
            t.push_str("[glossary]\nmatch = \"first\"\n\n[glossary.terms.api]\nterm = \"API key\"\ndefinition = \"A secret.\"\nlink = \"/glossary.md#api\"\n\n");
        }
        t.push_str("[consumer]\nprofile = \"astro\"\nbase-path = \"/docs/\"\n\n[builds.site]\nvariants = \"switch\"\navailability = \"badge\"\n");
        if self.cloud_build {
            t.push_str("\n[builds.cloud]\nvariants = { deployment = \"cloud\" }\navailability = { filter = \"cloud\" }\n");
        }
        t
    }

    pub fn model(&self) -> Arc<ContentModel> {
        match load_str(&self.text(), FileId::new(0)) {
            Ok(model) => Arc::new(model),
            Err(issues) => panic!("the test model doesn't load: {issues:?}\n{}", self.text()),
        }
    }
}

// The vocabulary of the random projects: source paths, other files, and
// the pieces a source file is made of. The comparison's differential test in
// `ascribe-diff` uses them too.

pub const SOURCES: &[&str] = &[
    "index.md",
    "guide.md",
    "_shared.md",
    "_deep/inner.md",
    "glossary.md",
    "Guide.md",
    "shared/thing.md",
    "sub/index.md",
    "nested/page.md",
    "nested/sub/deep.md",
];

/// Project paths of files that aren't sources.
pub const FILES: &[&str] = &[
    "docs/logo.png",
    "docs/img/pic.png",
    "docs/files/a.pdf",
    "README.md",
    "docs/.hidden/x.md",
    "docs/Logo.png",
    "docs/guide",
    "docs/files/",
    // Each makes its directory a nested project's folder.
    "docs/nested/ascribe.toml",
    "docs/nested/sub/ascribe.toml",
];

/// File names that differ from another only in case. The base file system and
/// the overlay may pick different twins for those, so the run that starts from
/// a non-empty base leaves them out.
pub const TWINS: &[&str] = &["docs/Logo.png", "Guide.md"];

/// Files that make a nested project, which loads the project again, from a
/// non-empty base: the run that has twins leaves them out.
pub const NESTED: &[&str] = &["docs/nested/ascribe.toml", "docs/nested/sub/ascribe.toml"];

pub const FRONT: &[&str] = &[
    "",
    "---\ntitle: Page\n---\n\n",
    "---\ntitle: '{product} guide'\n---\n\n",
    "---\ntitle: Cloud only\navailable: cloud\n---\n\n",
    "---\ntitle: Index\n---\n\n",
];

pub const BODY: &[&str] = &[
    "# Intro\n",
    "## Setup\n",
    "## Setup\n",
    "## {product} tips\n",
    "## Details\n@id: details\n",
    "@include: _shared.md\n",
    "@include: _shared.md#shared-setup\n",
    "@include: _deep/inner.md\n",
    "@include: missing.md\n",
    "@include: /guide.md\n",
    "[a](guide.md) and [b](guide.md#setup)\n",
    "[](guide.md)\n",
    "[](guide.md#setup)\n",
    "[c](index.md#intro) [x](_shared.md)\n",
    "[route](/guide) [r2](sub/)\n",
    "![l](logo.png) ![p](img/pic.png)\n",
    "[dl](files/a.pdf) [case](GUIDE.md) [o](../README.md)\n",
    "The API key is here. [g](glossary.md#api)\n",
    "[ref][r] and [p]({api}x)\n\n[r]: guide.md\n",
    "@variant {deployment=cloud}:\nCloud text\n@variant {deployment=self-managed}:\nSM text\n@end\n",
    "## Cloud feature\n@available: cloud\n\nText.\n",
    "@quill-callout: Heads up\n",
    "## Shared setup\n",
    "## API\n@id: api\n",
    "![l](Logo.png) [dir](files/)\n",
    "@include: shared/thing.md\n",
    "Some prose about {extra} and more.\n",
    "@include: nested/page.md\n",
    "[n](nested/sub/deep.md) ![i](nested/page.md)\n",
];

pub fn render(front: usize, body: &[usize]) -> String {
    let mut text = FRONT[front % FRONT.len()].to_owned();
    for at in body {
        text.push_str(BODY[at % BODY.len()]);
        text.push('\n');
    }
    text
}

pub fn path(text: &str) -> RelPath {
    RelPath::parse(text).expect("a valid path")
}

/// The files of a project, in memory, changing as the tests apply changes:
/// the reference the incremental result is compared with. Source paths are
/// content paths; other files are by project path.
#[derive(Clone, Debug, Default)]
pub struct World {
    pub sources: BTreeMap<RelPath, String>,
    pub files: BTreeSet<RelPath>,
}

impl World {
    pub fn fs(&self, layout: &Layout) -> MemoryFs {
        let mut fs = MemoryFs::new(layout);
        for (path, text) in &self.sources {
            fs = fs.with_source(path.as_str(), text);
        }
        for file in &self.files {
            fs = fs.with_file(file.as_str(), "");
        }
        fs
    }

    /// The folders of the projects nested in the content root: the
    /// directories below it that hold an `ascribe.toml`.
    pub fn nested(&self, layout: &Layout) -> Vec<RelPath> {
        self.fs(layout).sources().nested
    }

    /// Whether a content path names a source file, by the same rule as the
    /// project's: by its path, and not in a nested project's folder.
    pub fn is_source(&self, layout: &Layout, path: &RelPath) -> bool {
        is_source_path(path) && !in_nested_project(path, &self.nested(layout))
    }

    fn put_source(&mut self, layout: &Layout, path: &RelPath, text: String) {
        self.files.remove(&layout.project_path(path));
        self.sources.insert(path.clone(), text);
    }

    fn put_file(&mut self, layout: &Layout, path: &RelPath) {
        self.sources.remove(path);
        self.files.insert(layout.project_path(path));
    }

    fn remove(&mut self, layout: &Layout, path: &RelPath) {
        self.sources.remove(path);
        self.files.remove(&layout.project_path(path));
    }

    /// The same change to the world as to the project. A file in a nested
    /// project's folder is kept as a file that isn't a source, with no text,
    /// as the project keeps it.
    pub fn apply(&mut self, layout: &Layout, change: &Change) {
        match change {
            Change::Created { path, text } | Change::Edited { path, text } => {
                if self.is_source(layout, path) {
                    self.put_source(layout, path, text.clone());
                } else {
                    self.put_file(layout, path);
                }
            }
            Change::Deleted { path } => self.remove(layout, path),
            Change::Renamed { from, to } => {
                // A source kept as a file (it was made in a nested project's
                // folder that has since gone) reads as empty, as from `fs`.
                let text = if self.is_source(layout, from) {
                    self.sources.get(from).cloned().or_else(|| {
                        self.files
                            .contains(&layout.project_path(from))
                            .then(String::new)
                    })
                } else {
                    None
                };
                self.remove(layout, from);
                if self.is_source(layout, to) {
                    if let Some(text) = text {
                        self.put_source(layout, to, text);
                    }
                } else {
                    self.put_file(layout, to);
                }
            }
            // The world has no unreadable files: the index drops one as it
            // drops a deleted file, and the unit tests check the list.
            Change::Unreadable { path, .. } => {
                if self.is_source(layout, path) {
                    self.remove(layout, path);
                } else {
                    self.put_file(layout, path);
                }
            }
            Change::AssetCreated { path } => {
                self.files.insert(path.clone());
            }
            Change::AssetDeleted { path } => {
                self.files.remove(path);
            }
            Change::Model(_) => {}
        }
    }
}

pub fn router(project: &Project) -> DefaultRouter {
    DefaultRouter::from_consumer(&project.model().consumer)
}

/// Everything a project answers, as text, to compare exactly.
pub fn dump(project: &Project) -> Vec<String> {
    let mut out = Vec::new();
    let mut targets: BTreeSet<RelPath> = BTreeSet::new();
    for index in project.files() {
        assert_eq!(project.path_of(index.file), Some(&index.path));
        assert_eq!(
            project.file_by_id(index.file).map(|f| &f.path),
            Some(&index.path)
        );
        out.push(format!("file {} {:?}", index.path, index));
        out.push(format!(
            "resolutions {} {:?}",
            index.path,
            project.resolutions(&index.path)
        ));
        out.push(format!(
            "problems {} {:?}",
            index.path,
            project.problems(&index.path)
        ));
        out.push(format!(
            "expand {} {:?}",
            index.path,
            project.expand(&index.path)
        ));
        out.push(format!(
            "assets {} {:?}",
            index.path,
            project.assets(&index.path)
        ));
        targets.insert(index.path.clone());
        for resolution in project.resolutions(&index.path) {
            match resolution {
                ascribe_resolve::Resolution::Source { target, .. } => {
                    targets.insert(target.clone());
                }
                ascribe_resolve::Resolution::Asset { path, .. } => {
                    targets.insert(path.clone());
                }
                _ => {}
            }
        }
        for include in &index.includes {
            targets.extend(include.target.clone());
        }
    }
    for target in &targets {
        out.push(format!(
            "includers {target} {:?}",
            project.includers(target)
        ));
        out.push(format!(
            "including {target} {:?}",
            project.including_pages(target)
        ));
        out.push(format!("links {target} {:?}", project.links_to(target)));
        out.push(format!(
            "assets-of {target} {:?}",
            project.asset_users(target)
        ));
    }
    out.push(format!("unreadable {:?}", project.unreadable()));
    out.push(format!(
        "empty-slug {:?}",
        project.empty_slug_headings().len()
    ));
    out
}

/// What a consumer that re-checks only [`Affected::recheck`] would hold for a
/// file: everything file-level, plus the page-level problems in a page.
pub fn file_result(project: &Project, path: &RelPath) -> String {
    let Some(index) = project.file(path) else {
        return String::from("(no file)");
    };
    let mut out = format!(
        "{:?}\n{:?}\n{:?}\n",
        index,
        project.resolutions(path),
        project.problems(path)
    );
    if index.kind == ascribe_resolve::FileKind::Page {
        let router = router(project);
        for build in &project.model().builds {
            if let Some(page) = project.resolve_page(path, build, &router) {
                let _ = writeln!(out, "{} problems {:?}", build.name, page.problems);
            }
        }
    }
    out
}

/// What a consumer that re-resolves only [`Affected::re_resolve`] would hold
/// for a page in a build.
pub fn page_result(project: &Project, path: &RelPath, build: &ascribe_model::Build) -> String {
    format!("{:?}", project.resolve_page(path, build, &router(project)))
}

/// A consumer's kept results, updated only where an update says to.
#[derive(Default)]
pub struct Consumer {
    pub files: BTreeMap<RelPath, String>,
    pub pages: BTreeMap<(String, RelPath), String>,
}

impl Consumer {
    /// Recomputes exactly what `affected` lists, and drops the removed.
    pub fn update(&mut self, project: &Project, affected: &ascribe_resolve::Affected) {
        for path in &affected.removed {
            self.files.remove(path);
            self.pages.retain(|(_, p), _| p != path);
        }
        for path in &affected.recheck {
            self.files.insert(path.clone(), file_result(project, path));
        }
        for path in &affected.re_resolve {
            for build in &project.model().builds {
                self.pages.insert(
                    (build.name.clone(), path.clone()),
                    page_result(project, path, build),
                );
            }
        }
        // A page that stopped being a page (or a build that went away) has
        // nothing to keep.
        self.pages.retain(|(build, path), _| {
            project
                .file(path)
                .is_some_and(|f| f.kind == ascribe_resolve::FileKind::Page)
                && project.model().build(build).is_some()
        });
        self.files.retain(|path, _| project.file(path).is_some());
    }

    /// Everything, computed from scratch.
    pub fn from_scratch(project: &Project) -> Consumer {
        let mut consumer = Consumer::default();
        for index in project.files() {
            consumer
                .files
                .insert(index.path.clone(), file_result(project, &index.path));
            if index.kind == ascribe_resolve::FileKind::Page {
                for build in &project.model().builds {
                    consumer.pages.insert(
                        (build.name.clone(), index.path.clone()),
                        page_result(project, &index.path, build),
                    );
                }
            }
        }
        consumer
    }
}

/// A from-scratch load of the world, numbered as the incremental project is.
pub fn scratch(inc: &IncrementalProject, model: &Arc<ContentModel>, world: &World) -> Project {
    let layout = Layout::from_model(model);
    let mut ids: FileIds = inc.ids().clone();
    Project::load_with_ids(model.clone(), layout.clone(), &world.fs(&layout), &mut ids)
}

pub fn created(path: &str, text: &str) -> Change {
    Change::Created {
        path: self::path(path),
        text: text.to_owned(),
    }
}

pub fn edited(path: &str, text: &str) -> Change {
    Change::Edited {
        path: self::path(path),
        text: text.to_owned(),
    }
}

pub fn deleted(path: &str) -> Change {
    Change::Deleted {
        path: self::path(path),
    }
}

/// An incremental project over these sources (content paths), under a model.
pub fn load(spec: &ModelSpec, sources: &[(&str, &str)]) -> (IncrementalProject, World) {
    let model = spec.model();
    let layout = Layout::from_model(&model);
    let mut world = World::default();
    for (path, text) in sources {
        world.sources.insert(self::path(path), (*text).to_owned());
    }
    let inc = IncrementalProject::load(model, layout.clone(), world.fs(&layout));
    (inc, world)
}
