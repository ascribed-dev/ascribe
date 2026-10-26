//! The differential property test: after every step of a random sequence of
//! edits, creations, deletions, renames, asset changes, sources that can't be
//! read, changes to the files outside the content root the project reads
//! (code a `@snippet` shows and its source's examples, a declared API's spec,
//! memo files, `ascribe.nav`), a base revision's specs, and model changes
//! (among them snippets' sources, the API, and release lines), the
//! incremental project equals a from-scratch load of the same files, and a
//! consumer that redoes only what [`Affected`] lists holds exactly what a
//! from-scratch consumer would. A step that adds or removes a nested project's
//! `ascribe.toml` is refused, exactly then, and the project is loaded again,
//! as the language server does.
//!
//! The second half is what makes `Affected` trustworthy: a file or page left
//! out of it must be unchanged.
//!
//! A kind of change the incremental project learns to handle joins the
//! generator here, with sources that use it, in the same change.

#![allow(clippy::expect_used, clippy::panic, clippy::unwrap_used)]

use crate::incremental_support;

use std::collections::BTreeSet;
use std::sync::{Arc, LazyLock};

use ascribe_core::RelPath;
use ascribe_model::ContentModel;
use ascribe_resolve::openapi::ApiSpecs;
use ascribe_resolve::side_file::Indexed;
use ascribe_resolve::{
    Affected, ApplyError, Change, Committed, IncrementalProject, Layout, MemoStore, ModelImpact,
    Project, is_memo_path, place,
};
use proptest::prelude::*;

use incremental_support::{
    BODY, Consumer, Disk, FILES, FRONT, ModelSpec, NESTED, SOURCES, TEXTS, TWINS, USES, World,
    dump, load, path, render_uses, scratch,
};

/// The text file whose changes the project hears as its side files'.
const NAV: &str = "ascribe.nav";

/// The folder of the snippets' source, deleted at once.
const CODE: &str = "code";

/// The declared API's spec.
const SPEC: &str = "api/pets.yaml";

/// [`models`], each loaded once for every run.
static LOADED: LazyLock<Vec<(ModelSpec, Arc<ContentModel>)>> = LazyLock::new(|| {
    models()
        .into_iter()
        .map(|spec| {
            let model = spec.model();
            (spec, model)
        })
        .collect()
});

fn models() -> Vec<ModelSpec> {
    let base = ModelSpec {
        code: true,
        api: true,
        ..ModelSpec::base()
    };
    vec![
        base.clone(),
        ModelSpec {
            widgets: 1,
            ..base.clone()
        },
        ModelSpec {
            widgets: 2,
            ..base.clone()
        },
        ModelSpec {
            product: "Quill Pro",
            ..base.clone()
        },
        ModelSpec {
            extra_phrase: true,
            ..base.clone()
        },
        ModelSpec {
            hybrid: true,
            ..base.clone()
        },
        ModelSpec {
            cloud_build: false,
            ..base.clone()
        },
        ModelSpec {
            shared_pattern: true,
            ..base.clone()
        },
        ModelSpec {
            glossary: false,
            ..base.clone()
        },
        ModelSpec {
            comment: true,
            ..base.clone()
        },
        ModelSpec {
            widgets: 1,
            shared_pattern: true,
            hybrid: true,
            ..base.clone()
        },
        ModelSpec {
            code: false,
            ..base.clone()
        },
        ModelSpec {
            api: false,
            ..base.clone()
        },
        ModelSpec {
            releases: 1,
            ..base.clone()
        },
        ModelSpec {
            releases: 2,
            ..base.clone()
        },
        ModelSpec {
            releases: 3,
            hybrid: true,
            ..base
        },
    ]
}

#[derive(Clone, Debug)]
enum Op {
    Write {
        file: usize,
        front: usize,
        body: Vec<usize>,
        create: bool,
    },
    Same {
        file: usize,
    },
    Delete {
        file: usize,
    },
    Rename {
        from: usize,
        to: usize,
    },
    Asset {
        file: usize,
        content_form: bool,
    },
    Model {
        spec: usize,
    },
    /// A file in [`TEXTS`] written with one of its texts, or deleted.
    Text {
        file: usize,
        text: Option<usize>,
    },
    /// The snippets' source's folder deleted.
    Folder,
    /// A base revision whose spec has one of [`TEXTS`]'s texts.
    Base {
        text: usize,
    },
    Unreadable {
        file: usize,
    },
}

fn op() -> impl Strategy<Value = Op> {
    prop_oneof![
        6 => (0..SOURCES.len(), 0..=FRONT.len(), prop::collection::vec(0..BODY.len() + USES.len(), 0..6), any::<bool>())
            .prop_map(|(file, front, body, create)| Op::Write { file, front, body, create }),
        1 => (0..SOURCES.len()).prop_map(|file| Op::Same { file }),
        2 => (0..SOURCES.len()).prop_map(|file| Op::Delete { file }),
        2 => (0..SOURCES.len(), 0..SOURCES.len()).prop_map(|(from, to)| Op::Rename { from, to }),
        3 => (0..FILES.len(), any::<bool>()).prop_map(|(file, content_form)| Op::Asset { file, content_form }),
        2 => (0..LOADED.len()).prop_map(|spec| Op::Model { spec }),
        3 => (0..TEXTS.len(), prop::option::weighted(0.8, 0..4usize)).prop_map(|(file, text)| Op::Text { file, text }),
        1 => Just(Op::Folder),
        1 => (0..4usize).prop_map(|text| Op::Base { text }),
        1 => (0..SOURCES.len()).prop_map(|file| Op::Unreadable { file }),
    ]
}

/// The file a change says appeared, changed, or went, by project path, when
/// it isn't a source.
fn reported(change: &Change) -> Option<&RelPath> {
    match change {
        Change::AssetCreated { path } | Change::AssetDeleted { path } => Some(path),
        _ => None,
    }
}

fn content_form(layout: &Layout, project_path: &str) -> RelPath {
    let project = path(project_path.trim_end_matches('/'));
    let up = "../".repeat(layout.content_root.segments().count());
    let mut out = String::new();
    if project.starts_with(&layout.content_root) {
        let rest: Vec<&str> = project
            .segments()
            .skip(layout.content_root.segments().count())
            .collect();
        out = rest.join("/");
    } else {
        out.push_str(&up);
        out.push_str(project.as_str());
    }
    path(&out)
}

struct Run {
    inc: IncrementalProject,
    world: World,
    /// The file system the incremental project reads, which follows the
    /// world.
    disk: Disk,
    spec: ModelSpec,
    model: Arc<ContentModel>,
    consumer: Consumer,
    layout: Layout,
    twins: bool,
    /// The specs as the project last read them: at load, and again whenever
    /// the model or the declared spec changes.
    apis: ApiSpecs,
    /// The readings before, oldest first, and the base revisions' specs:
    /// where each id was in them is what the project remembers, so a fresh
    /// load is given them too.
    readings: Vec<ApiSpecs>,
    bases: Vec<Arc<ApiSpecs>>,
}

impl Run {
    fn new(initial: bool) -> Run {
        let (spec, model) = LOADED[0].clone();
        let layout = Layout::from_model(&model);
        let mut world = World::default();
        if initial {
            for (at, text) in [
                (
                    "index.md",
                    "---\ntitle: Home\n---\n\n# Intro\n\n[g](guide.md#setup)\n",
                ),
                (
                    "guide.md",
                    "---\ntitle: Guide\n---\n\n## Setup\n\n@include: _shared.md\n",
                ),
                (
                    "_shared.md",
                    "## Shared setup\n\n![l](logo.png)\n\n@snippet: code:app.py#main\n",
                ),
                (
                    "glossary.md",
                    "---\ntitle: Glossary\n---\n\n## API key\n@id: api\n@term\n\nA secret.\n\n[list](../api/pets.yaml#listPets)\n",
                ),
            ] {
                world.sources.insert(path(at), text.to_owned());
            }
            world.files.insert(path("docs/logo.png"));
            // A nested project from the start, with a page in it.
            world.files.insert(path("docs/nested/ascribe.toml"));
            world
                .sources
                .insert(path("nested/page.md"), "---\ntitle: N\n---\n".to_owned());
            for (file, texts) in TEXTS.iter() {
                world.put_text(&path(file), Some(&texts[0]));
            }
        }
        let disk = world.disk(&layout);
        let inc = IncrementalProject::load(model.clone(), layout.clone(), disk.clone());
        let consumer = Consumer::from_scratch(&inc.snapshot());
        let apis = ApiSpecs::read(&model, &world.disk(&layout));
        Run {
            inc,
            world,
            disk,
            spec,
            model,
            consumer,
            layout,
            twins: !initial,
            apis,
            readings: Vec::new(),
            bases: Vec::new(),
        }
    }

    /// A from-scratch load of the world, remembering the specs as the
    /// project does: where each id was in the latest reading that had it,
    /// else in the first base that did.
    fn reference(&self, model: &Arc<ContentModel>) -> Project {
        let mut reference = scratch(&self.inc, model, &self.world);
        for reading in self.readings.iter().rev() {
            reference.remember_base_apis(reading);
        }
        for base in &self.bases {
            reference.remember_base_apis(base);
        }
        reference
    }

    fn change_for(&mut self, op: &Op, models: &[(ModelSpec, Arc<ContentModel>)]) -> Option<Change> {
        let source = |i: usize| path(SOURCES[i]);
        Some(match op {
            Op::Write {
                file,
                front,
                body,
                create,
            } => {
                if !self.twins && SOURCES[*file] == "Guide.md" {
                    return None;
                }
                let text = render_uses(*front, body);
                let path = source(*file);
                if *create {
                    Change::Created { path, text }
                } else {
                    Change::Edited { path, text }
                }
            }
            Op::Same { file } => {
                let path = source(*file);
                let text = self.world.sources.get(&path)?.clone();
                Change::Edited { path, text }
            }
            Op::Delete { file } => Change::Deleted {
                path: source(*file),
            },
            Op::Rename { from, to } => {
                if !self.twins && SOURCES[*to] == "Guide.md" {
                    return None;
                }
                Change::Renamed {
                    from: source(*from),
                    to: source(*to),
                }
            }
            Op::Asset {
                file,
                content_form: as_content,
            } => {
                let name = FILES[*file];
                if !self.twins && TWINS.contains(&name) || self.twins && NESTED.contains(&name) {
                    return None;
                }
                let project_path = path(name.trim_end_matches('/'));
                let present = self.world.files.contains(&project_path);
                // `docs/files/` is a directory name in a link, not a file:
                // only ever leave it out.
                if name.ends_with('/') {
                    return None;
                }
                if *as_content {
                    let content = content_form(&self.layout, name);
                    if present {
                        Change::Deleted { path: content }
                    } else {
                        Change::Created {
                            path: content,
                            text: String::new(),
                        }
                    }
                } else if present {
                    Change::AssetDeleted { path: project_path }
                } else {
                    Change::AssetCreated { path: project_path }
                }
            }
            Op::Model { spec } => {
                (self.spec, self.model) = models[*spec].clone();
                Change::Model(Arc::clone(&self.model))
            }
            // The world changes as the change is made, as the disk does
            // before the editor hears of it.
            Op::Text { file, text } => {
                let (name, texts) = &TEXTS[*file];
                let at = path(name);
                let text = text.map(|t| texts[t % texts.len()].as_str());
                self.world.put_text(&at, text);
                if *name == NAV {
                    Change::SideFiles(Indexed::read(&self.world.disk(&self.layout)))
                } else if text.is_some() {
                    Change::AssetCreated { path: at }
                } else {
                    Change::AssetDeleted { path: at }
                }
            }
            Op::Folder => {
                let at = path(CODE);
                self.world.remove_folder(&at);
                Change::AssetDeleted { path: at }
            }
            Op::Base { text } => {
                let texts = &TEXTS.iter().find(|(name, _)| *name == SPEC)?.1;
                let base = World {
                    files: BTreeSet::from([path(SPEC)]),
                    texts: [(path(SPEC), texts[text % texts.len()].clone())].into(),
                    ..World::default()
                };
                Change::BaseApis(Arc::new(ApiSpecs::read(
                    &models[0].1,
                    &base.fs(&self.layout),
                )))
            }
            Op::Unreadable { file } => Change::Unreadable {
                path: source(*file),
                reason: "permission denied".to_owned(),
            },
        })
    }

    /// The specs read as a fresh load reads them, with the same history.
    fn check_apis(&self, snapshot: &Project, reference: &Project, batch: &[Op]) {
        assert_eq!(
            snapshot.apis(),
            reference.apis(),
            "the specs differ after {batch:?}"
        );
    }

    /// The specs are read again when the model changes, or the declared
    /// spec does, and a base revision's are remembered.
    fn follow_apis(&mut self, changes: &[Change], spec_before: &ModelSpec, model: &ContentModel) {
        let spec_reported = self.spec.api
            && changes
                .iter()
                .any(|c| reported(c).is_some_and(|p| p.as_str() == SPEC));
        if self.spec != *spec_before || spec_reported {
            let reading = ApiSpecs::read(model, &self.world.disk(&self.layout));
            self.readings
                .push(std::mem::replace(&mut self.apis, reading));
        }
        if let Some(base) = changes.iter().rev().find_map(|c| match c {
            Change::BaseApis(base) => Some(base),
            _ => None,
        }) {
            self.bases.push(Arc::clone(base));
        }
    }

    /// After a step that changes a memo file, each memo file is placed as a
    /// fresh load places it. Placing reads only the index, which the dumps
    /// compare after every step. A memo file is no part of any page, so a
    /// step that changes only memo files changes nothing.
    fn check_memos(
        &self,
        changes: &[Change],
        affected: &Affected,
        snapshot: &Project,
        reference: &Project,
        batch: &[Op],
    ) {
        let memos: Vec<bool> = changes
            .iter()
            .map(|c| reported(c).is_some_and(|p| is_memo_path(p, &self.layout)))
            .collect();
        if memos.iter().all(|m| *m) {
            assert!(
                affected.is_empty(),
                "{batch:?} changed something: {affected:?}"
            );
        }
        if !memos.contains(&true) {
            return;
        }
        for memo in Committed::new(&self.disk, &self.layout).memos() {
            assert_eq!(
                place(&memo, snapshot),
                place(&memo, reference),
                "{} is placed differently after {batch:?}",
                memo.path
            );
        }
    }

    fn step(&mut self, batch: &[Op], models: &[(ModelSpec, Arc<ContentModel>)]) {
        let before = self.inc.snapshot();
        let spec_before = self.spec.clone();
        let changes: Vec<Change> = batch
            .iter()
            .filter_map(|op| self.change_for(op, models))
            .collect();
        let nested_before = self.world.nested(&self.layout);
        // The world sees the changes in order, as the project's batch does.
        // A rename of a file the batch itself created reads the world's text.
        let mut applied = Vec::new();
        for change in &changes {
            // Keep the world's view of a rename the same as the project's: the
            // project uses its own text unless an earlier change in the batch
            // set one, which the world has already applied.
            self.world.apply(&self.layout, change);
            applied.push(change.clone());
        }
        let nested_after = self.world.nested(&self.layout);
        let model = Arc::clone(&self.model);
        self.disk.update(&self.world, &self.layout);
        let affected = match self.inc.apply(applied) {
            Ok(affected) => {
                assert_eq!(
                    nested_before, nested_after,
                    "the nested projects changed, and {batch:?} was applied in place"
                );
                affected
            }
            Err(ApplyError::NestedProjectChanged) => {
                assert_ne!(
                    nested_before, nested_after,
                    "{batch:?} was refused, but the nested projects didn't change"
                );
                // Nothing was applied; the project is loaded again.
                assert_eq!(self.inc.version(), before.version());
                self.disk = self.world.disk(&self.layout);
                self.inc =
                    IncrementalProject::load(model.clone(), self.layout.clone(), self.disk.clone());
                self.apis = ApiSpecs::read(&model, &self.world.disk(&self.layout));
                self.readings.clear();
                self.bases.clear();
                let reference = scratch(&self.inc, &model, &self.world);
                assert_eq!(
                    dump(&self.inc.snapshot()),
                    dump(&reference),
                    "a fresh load differs after {batch:?}"
                );
                self.consumer = Consumer::from_scratch(&self.inc.snapshot());
                return;
            }
            Err(ApplyError::LayoutChanged) => panic!("no model moves the content root"),
        };
        let snapshot = self.inc.snapshot();

        self.follow_apis(&changes, &spec_before, &model);
        let reference = self.reference(&model);
        self.check_apis(&snapshot, &reference, batch);
        self.check_memos(&changes, &affected, &snapshot, &reference, batch);

        // 1. The state equals a from-scratch load.
        let ours = dump(&snapshot);
        let theirs = dump(&reference);
        assert_eq!(
            ours.len(),
            theirs.len(),
            "different numbers of answers after {batch:?}"
        );
        for (a, b) in ours.iter().zip(&theirs) {
            assert_eq!(a, b, "the incremental state differs after {batch:?}");
        }

        // 2. What Affected leaves out didn't change.
        self.consumer.update(&snapshot, &affected);
        let fresh = Consumer::from_scratch(&reference);
        assert_eq!(
            self.consumer.files.keys().collect::<Vec<_>>(),
            fresh.files.keys().collect::<Vec<_>>(),
            "the files a consumer holds differ after {batch:?}: {affected:?}"
        );
        for (path, result) in &fresh.files {
            assert_eq!(
                &self.consumer.files[path], result,
                "{path} changed but isn't in `recheck` after {batch:?}: {affected:?}"
            );
        }
        assert_eq!(
            self.consumer.pages.keys().collect::<Vec<_>>(),
            fresh.pages.keys().collect::<Vec<_>>(),
            "the pages a consumer holds differ after {batch:?}: {affected:?}"
        );
        for (key, result) in &fresh.pages {
            assert_eq!(
                &self.consumer.pages[key], result,
                "{key:?} changed but isn't in `re_resolve` after {batch:?}: {affected:?}"
            );
        }

        // 3. The bookkeeping is consistent.
        assert!(affected.parsed.is_subset(&affected.indexed));
        assert!(
            affected.recheck.iter().all(|p| snapshot.file(p).is_some()),
            "recheck lists a file that isn't there"
        );
        assert!(
            affected.removed.iter().all(|p| snapshot.file(p).is_none()),
            "removed lists a file that is there"
        );
        assert!(
            affected.re_resolve.iter().all(|p| snapshot
                .file(p)
                .is_some_and(|f| f.kind == ascribe_resolve::FileKind::Page)),
            "re_resolve lists something that isn't a page"
        );
        if affected.is_empty() {
            assert_eq!(snapshot.version(), before.version());
            assert!(before.is_current());
        } else {
            assert!(snapshot.version() > before.version());
            assert!(!before.is_current());
            assert!(snapshot.is_current());
        }
        // A file left out of `recheck` is current from the old snapshot.
        for file in snapshot.files() {
            let left_out = !affected.recheck.contains(&file.path)
                && !affected.removed.contains(&file.path)
                && affected.model.is_none_or(|m| m == ModelImpact::Warnings);
            if left_out {
                assert!(self.inc.is_file_current(&before, &file.path));
            }
        }
        // Without a model change, only files whose text changed are reparsed.
        if affected.model.is_none() {
            let texts: BTreeSet<&RelPath> = changes
                .iter()
                .filter_map(|c| match c {
                    Change::Created { path, .. }
                    | Change::Edited { path, .. }
                    | Change::Deleted { path }
                        if before.is_source(path) =>
                    {
                        Some(path)
                    }
                    Change::Renamed { to, .. } if before.is_source(to) => Some(to),
                    _ => None,
                })
                .collect();
            assert!(
                affected.parsed.iter().all(|p| texts.contains(p)),
                "reparsed a file no change named: {affected:?}"
            );
        }
        // Ids are as in the table: every file's id names its path.
        for file in snapshot.files() {
            assert_eq!(self.inc.ids().get(&file.path), Some(file.file));
        }
    }
}

fn steps() -> impl Strategy<Value = Vec<Vec<Op>>> {
    prop::collection::vec(prop::collection::vec(op(), 1..=3), 1..=22)
}

proptest! {
    #![proptest_config(ProptestConfig { max_shrink_iters: 2000, ..ProptestConfig::default() })]

    #[test]
    fn incremental_equals_from_scratch_starting_empty(batches in steps()) {
        let models = &LOADED;
        let mut run = Run::new(false);
        for batch in &batches {
            run.step(batch, models);
        }
    }

    #[test]
    fn incremental_equals_from_scratch_starting_from_disk(batches in steps()) {
        let models = &LOADED;
        let mut run = Run::new(true);
        for batch in &batches {
            run.step(batch, models);
        }
    }
}

/// The harness has to be able to fail: a consumer told less than the truth
/// ends up holding stale results.
#[test]
fn the_differential_check_notices_an_update_that_under_reports() {
    let spec = ModelSpec::base();
    let (mut inc, mut world) = load(
        &spec,
        &[
            ("index.md", "---\ntitle: Home\n---\n\n@include: _f.md\n"),
            ("_f.md", "## One\n"),
        ],
    );
    let layout = Layout::from_model(&spec.model());
    let mut consumer = Consumer::from_scratch(&inc.snapshot());
    let change = Change::Edited {
        path: path("_f.md"),
        text: "## Two\n".to_owned(),
    };
    world.apply(&layout, &change);
    let mut affected = inc.apply([change]).expect("applies");
    assert!(
        affected.re_resolve.contains(&path("index.md")),
        "the includer is re-resolved"
    );
    affected.re_resolve.clear();
    consumer.update(&inc.snapshot(), &affected);
    let fresh = Consumer::from_scratch(&scratch(&inc, &Arc::clone(&spec.model()), &world));
    assert_ne!(
        consumer.pages, fresh.pages,
        "the stale page result is detected"
    );
}
