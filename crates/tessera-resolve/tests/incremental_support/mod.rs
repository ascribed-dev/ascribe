//! Helpers for the incremental-update tests: models that differ in the parts
//! each stage reads, a world of files that changes step by step, and dumps of
//! everything a project can answer, to compare exactly.

#![allow(dead_code, clippy::expect_used, clippy::panic, clippy::unwrap_used)]

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::sync::Arc;

use tessera_core::{FileId, RelPath};
use tessera_model::{ContentModel, load_str};
use tessera_resolve::{
    Change, DefaultRouter, FileIds, IncrementalProject, Layout, MemoryFs, Project, is_source_path,
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
    /// spans in a real `tessera.toml`.
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
        t.push_str("spec = \"0.1\"\n\n[project]\ncontent-root = \"docs\"\noutput-dir = \".tessera/build\"\n\n");
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

    /// The same change to the world as to the project.
    pub fn apply(&mut self, layout: &Layout, change: &Change) {
        match change {
            Change::Created { path, text } | Change::Edited { path, text } => {
                if is_source_path(path) {
                    self.sources.insert(path.clone(), text.clone());
                } else {
                    self.files.insert(layout.project_path(path));
                }
            }
            Change::Deleted { path } => {
                if is_source_path(path) {
                    self.sources.remove(path);
                } else {
                    self.files.remove(&layout.project_path(path));
                }
            }
            Change::Renamed { from, to } => {
                let text = if is_source_path(from) {
                    self.sources.remove(from)
                } else {
                    self.files.remove(&layout.project_path(from));
                    None
                };
                if is_source_path(to) {
                    if let Some(text) = text {
                        self.sources.insert(to.clone(), text);
                    }
                } else {
                    self.files.insert(layout.project_path(to));
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
                tessera_resolve::Resolution::Source { target, .. } => {
                    targets.insert(target.clone());
                }
                tessera_resolve::Resolution::Asset { path, .. } => {
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
    if index.kind == tessera_resolve::FileKind::Page {
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
pub fn page_result(project: &Project, path: &RelPath, build: &tessera_model::Build) -> String {
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
    pub fn update(&mut self, project: &Project, affected: &tessera_resolve::Affected) {
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
                .is_some_and(|f| f.kind == tessera_resolve::FileKind::Page)
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
            if index.kind == tessera_resolve::FileKind::Page {
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
