//! The differential property test: after every step of a random sequence of
//! edits, creations, deletions, renames, asset changes, and model changes,
//! the incremental project equals a from-scratch load of the same files, and a
//! consumer that redoes only what [`Affected`] lists holds exactly what a
//! from-scratch consumer would. A step that adds or removes a nested project's
//! `ascribe.toml` is refused, exactly then, and the project is loaded again,
//! as the language server does.
//!
//! The second half is what makes `Affected` trustworthy: a file or page left
//! out of it must be unchanged.

#![allow(clippy::expect_used, clippy::panic, clippy::unwrap_used)]

use crate::incremental_support;

use std::collections::BTreeSet;
use std::sync::Arc;

use ascribe_core::RelPath;
use ascribe_resolve::{ApplyError, Change, IncrementalProject, Layout, ModelImpact};
use proptest::prelude::*;

use incremental_support::{
    BODY, Consumer, FILES, FRONT, ModelSpec, NESTED, SOURCES, TWINS, World, dump, load, path,
    render, scratch,
};

fn models() -> Vec<ModelSpec> {
    let base = ModelSpec::base();
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
}

fn op() -> impl Strategy<Value = Op> {
    prop_oneof![
        6 => (0..SOURCES.len(), 0..FRONT.len(), prop::collection::vec(0..BODY.len(), 0..6), any::<bool>())
            .prop_map(|(file, front, body, create)| Op::Write { file, front, body, create }),
        1 => (0..SOURCES.len()).prop_map(|file| Op::Same { file }),
        2 => (0..SOURCES.len()).prop_map(|file| Op::Delete { file }),
        2 => (0..SOURCES.len(), 0..SOURCES.len()).prop_map(|(from, to)| Op::Rename { from, to }),
        3 => (0..FILES.len(), any::<bool>()).prop_map(|(file, content_form)| Op::Asset { file, content_form }),
        2 => (0..models().len()).prop_map(|spec| Op::Model { spec }),
    ]
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
    spec: ModelSpec,
    consumer: Consumer,
    layout: Layout,
    twins: bool,
}

impl Run {
    fn new(initial: bool) -> Run {
        let spec = ModelSpec::base();
        let (inc, world) = if initial {
            load(
                &spec,
                &[
                    (
                        "index.md",
                        "---\ntitle: Home\n---\n\n# Intro\n\n[g](guide.md#setup)\n",
                    ),
                    (
                        "guide.md",
                        "---\ntitle: Guide\n---\n\n## Setup\n\n@include: _shared.md\n",
                    ),
                    ("_shared.md", "## Shared setup\n\n![l](logo.png)\n"),
                ],
            )
        } else {
            load(&spec, &[])
        };
        let mut world = world;
        if initial {
            world.files.insert(path("docs/logo.png"));
            // A nested project from the start, with a page in it.
            world.files.insert(path("docs/nested/ascribe.toml"));
            world
                .sources
                .insert(path("nested/page.md"), "---\ntitle: N\n---\n".to_owned());
        }
        let layout = Layout::from_model(&spec.model());
        // The incremental project reads its initial state from a file system
        // that must have the asset too.
        let model = spec.model();
        let inc = if initial {
            IncrementalProject::load(model, layout.clone(), world.fs(&layout))
        } else {
            inc
        };
        let consumer = Consumer::from_scratch(&inc.snapshot());
        Run {
            inc,
            world,
            spec,
            consumer,
            layout,
            twins: !initial,
        }
    }

    fn change_for(&mut self, op: &Op, models: &[ModelSpec]) -> Option<Change> {
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
                let text = render(*front, body);
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
                self.spec = models[*spec].clone();
                Change::Model(self.spec.model())
            }
        })
    }

    fn step(&mut self, batch: &[Op], models: &[ModelSpec]) {
        let before = self.inc.snapshot();
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
        let model = self.spec.model();
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
                self.inc = IncrementalProject::load(
                    model.clone(),
                    self.layout.clone(),
                    self.world.fs(&self.layout),
                );
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
        let reference = scratch(&self.inc, &model, &self.world);

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
        let models = models();
        let mut run = Run::new(false);
        for batch in &batches {
            run.step(batch, &models);
        }
    }

    #[test]
    fn incremental_equals_from_scratch_starting_from_disk(batches in steps()) {
        let models = models();
        let mut run = Run::new(true);
        for batch in &batches {
            run.step(batch, &models);
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
