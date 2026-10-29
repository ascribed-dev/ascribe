//! The differential property test over a `Workspace`: after every step of a
//! random sequence of changes to sources, files that aren't sources, side
//! files, memo files, code files, and `ascribe.toml`, the snapshot equals a
//! fresh `Workspace::load` of the same files: its source index (numbered as
//! the workspace's is), its sources, its model and side files, and what it
//! reads on demand (memo files, code files, images). A step that adds or
//! removes a nested project's `ascribe.toml` is refused, exactly then, and
//! the project is loaded again. `ascribe-resolve`'s differential test holds
//! the source index's own update to the same; this holds what the
//! `Workspace` adds, with the vocabulary of that test
//! (`ascribe-resolve/tests/incremental_support`). The files are read as
//! they are at each step, probes too, as a project on disk is read when
//! it's told of every change.

#![allow(clippy::expect_used, clippy::panic, clippy::unwrap_used)]

use std::path::PathBuf;
use std::sync::{Arc, LazyLock};

use ascribe_core::RelPath;
use ascribe_model::ContentModel;
use ascribe_resolve::side_file::{Addresses, Entry, Lock, Nav, Permalinks};
use ascribe_resolve::{ApplyError, Change, Layout, Project};
use ascribe_workspace::{Changed, ModelFile, Workspace, WorkspaceSnapshot};
use proptest::prelude::*;

use crate::incremental_support::{
    BODY, Disk, FILES, FRONT, ModelSpec, NESTED, SOURCES, TEXTS, TWINS, USES, World, dump, path,
    render_uses,
};

/// The side files a project here has.
static SIDE_FILES: [Entry; 4] = [
    Entry::of::<Lock>(),
    Entry::of::<Permalinks>(),
    Entry::of::<Nav>(),
    Entry::of::<Addresses>(),
];

/// Side files beyond `ascribe.nav`, which [`TEXTS`] has, and their texts.
const SIDE_TEXTS: &[(&str, &[&str])] = &[
    (
        "ascribe.permalinks",
        &["version = 1\n", "version = 1\n\n[permalinks]\n", "[[\n"],
    ),
    ("ascribe.addresses", &["version = 1\n", "[[\n"]),
    ("ascribe.lock", &["version = 1\n", "[[\n"]),
];

/// The code files the checks read on demand.
const CODE_FILES: &[&str] = &["code/app.py", "code/lib.py"];

/// The models, each loaded once for every run. No API is declared: a
/// spec's readings are remembered across updates, which the source index's
/// own test follows.
static LOADED: LazyLock<Vec<ModelSpec>> = LazyLock::new(|| {
    let base = ModelSpec {
        code: true,
        ..ModelSpec::base()
    };
    vec![
        base.clone(),
        ModelSpec {
            widgets: 1,
            ..base.clone()
        },
        ModelSpec {
            product: "Quill Pro",
            ..base.clone()
        },
        ModelSpec {
            hybrid: true,
            ..base.clone()
        },
        ModelSpec {
            comment: true,
            ..base.clone()
        },
        ModelSpec {
            code: false,
            ..base.clone()
        },
        ModelSpec {
            releases: 1,
            ..base
        },
    ]
});

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
    },
    Model {
        spec: usize,
    },
    /// A file in [`TEXTS`] written with one of its texts, or deleted.
    Text {
        file: usize,
        text: Option<usize>,
    },
    /// A side file in [`SIDE_TEXTS`] written with one of its texts, or
    /// deleted.
    Side {
        file: usize,
        text: Option<usize>,
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
        2 => (0..FILES.len()).prop_map(|file| Op::Asset { file }),
        2 => (0..LOADED.len()).prop_map(|spec| Op::Model { spec }),
        3 => (0..TEXTS.len(), prop::option::weighted(0.8, 0..4usize)).prop_map(|(file, text)| Op::Text { file, text }),
        2 => (0..SIDE_TEXTS.len(), prop::option::weighted(0.7, 0..3usize)).prop_map(|(file, text)| Op::Side { file, text }),
        1 => (0..SOURCES.len()).prop_map(|file| Op::Unreadable { file }),
    ]
}

fn model_file(spec: &ModelSpec, model: &ContentModel) -> ModelFile {
    ModelFile {
        root: PathBuf::from("/project"),
        text: spec.text(),
        model: model.clone(),
    }
}

/// The workspace of `world`, loaded fresh.
fn fresh(spec: &ModelSpec, model: &ContentModel, world: &World, layout: &Layout) -> Workspace {
    Workspace::load_with(
        &SIDE_FILES,
        model_file(spec, model),
        layout.content_root.clone(),
        Arc::new(world.live_disk(layout)),
    )
}

/// Asks `snapshot` for what it reads on demand, so a later snapshot that
/// keeps it, stale, is caught.
fn prime(snapshot: &WorkspaceSnapshot) {
    let _ = snapshot.memos();
    let _ = snapshot.all_memos();
    for file in CODE_FILES {
        let _ = snapshot
            .code_files()
            .read(snapshot.file_system(), &path(file));
    }
    let _ = snapshot.images_or(|| images(snapshot));
}

/// The images under the content root, as the image checks list them.
fn images(snapshot: &WorkspaceSnapshot) -> Vec<(RelPath, RelPath)> {
    let content = snapshot.content_root().clone();
    snapshot
        .file_system()
        .files_in(&content)
        .into_iter()
        .filter(|p| p.extension() == Some("png"))
        .map(|p| (p.clone(), p))
        .collect()
}

/// Everything a snapshot holds beyond its source index, as text, with what
/// it reads on demand read as `snapshot` reads it now.
fn holds(snapshot: &WorkspaceSnapshot) -> Vec<String> {
    let mut sources: Vec<String> = snapshot
        .sources()
        .iter()
        .map(|s| format!("{} {:?} {:?}", s.path, s.text, s.unreadable))
        .collect();
    sources.sort();
    let mut out = sources;
    out.push(format!("model {:?}", snapshot.model_text()));
    for held in snapshot.side_files() {
        out.push(format!("side {} {:?}", held.path(), held.text()));
    }
    out.push(format!("memos {:?}", snapshot.memos()));
    out.push(format!("all memos {:?}", snapshot.all_memos()));
    for file in CODE_FILES {
        let read = snapshot
            .code_files()
            .read(snapshot.file_system(), &path(file))
            .map(|f| f.text.clone());
        out.push(format!("code {file} {read:?}"));
    }
    out.push(format!(
        "images {:?}",
        snapshot.images_or(|| images(snapshot))
    ));
    out
}

struct Run {
    ws: Workspace,
    world: World,
    disk: Disk,
    spec: ModelSpec,
    model: Arc<ContentModel>,
    layout: Layout,
    twins: bool,
}

impl Run {
    fn new(initial: bool) -> Run {
        let spec = LOADED[0].clone();
        let model = spec.model();
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
                    "---\ntitle: Glossary\n---\n\n## API key\n@id: api\n@term\n\nA secret.\n",
                ),
            ] {
                world.sources.insert(path(at), text.to_owned());
            }
            world.files.insert(path("docs/logo.png"));
            world.files.insert(path("docs/nested/ascribe.toml"));
            world
                .sources
                .insert(path("nested/page.md"), "---\ntitle: N\n---\n".to_owned());
            for (file, texts) in TEXTS.iter() {
                world.put_text(&path(file), Some(&texts[0]));
            }
            for (file, texts) in SIDE_TEXTS {
                world.put_text(&path(file), Some(texts[0]));
            }
        }
        let disk = world.live_disk(&layout);
        let ws = Workspace::load_with(
            &SIDE_FILES,
            model_file(&spec, &model),
            layout.content_root.clone(),
            Arc::new(disk.clone()),
        );
        Run {
            ws,
            world,
            disk,
            spec,
            model,
            layout,
            twins: !initial,
        }
    }

    fn change_for(&mut self, op: &Op) -> Option<Changed> {
        let source = |i: usize| path(SOURCES[i]);
        let change = match op {
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
            Op::Asset { file } => {
                let name = FILES[*file];
                if !self.twins && TWINS.contains(&name)
                    || self.twins && NESTED.contains(&name)
                    || name.ends_with('/')
                {
                    return None;
                }
                let project_path = path(name);
                if self.world.files.contains(&project_path) {
                    Change::AssetDeleted { path: project_path }
                } else {
                    Change::AssetCreated { path: project_path }
                }
            }
            Op::Model { spec } => {
                self.spec = LOADED[*spec].clone();
                self.model = self.spec.model();
                return Some(Changed::Model(Box::new(model_file(
                    &self.spec,
                    &self.model,
                ))));
            }
            // The world changes as the change is made, as the disk does
            // before the listing sees it.
            Op::Text { file, text } => {
                let (name, texts) = &TEXTS[*file];
                let at = path(name);
                let text = text.map(|t| texts[t % texts.len()].as_str());
                self.world.put_text(&at, text);
                if *name == "ascribe.nav" {
                    return Some(Changed::SideFile((*name).to_owned()));
                } else if text.is_some() {
                    Change::AssetCreated { path: at }
                } else {
                    Change::AssetDeleted { path: at }
                }
            }
            Op::Side { file, text } => {
                let (name, texts) = SIDE_TEXTS[*file];
                let text = text.map(|t| texts[t % texts.len()]);
                self.world.put_text(&path(name), text);
                return Some(Changed::SideFile(name.to_owned()));
            }
            Op::Unreadable { file } => Change::Unreadable {
                path: source(*file),
                reason: "permission denied".to_owned(),
            },
        };
        Some(Changed::Index(change))
    }

    fn step(&mut self, batch: &[Op]) {
        let before = self.ws.snapshot();
        prime(&before);
        let changes: Vec<Changed> = batch.iter().filter_map(|op| self.change_for(op)).collect();
        let nested_before = self.world.nested(&self.layout);
        for change in &changes {
            if let Changed::Index(change) = change {
                self.world.apply(&self.layout, change);
            }
        }
        let nested_after = self.world.nested(&self.layout);
        self.disk.update(&self.world, &self.layout);
        let updated = match self.ws.apply(changes) {
            Ok(updated) => {
                assert_eq!(
                    nested_before, nested_after,
                    "the nested projects changed, and {batch:?} was applied in place"
                );
                updated
            }
            Err(ApplyError::NestedProjectChanged) => {
                assert_ne!(
                    nested_before, nested_after,
                    "{batch:?} was refused, but the nested projects didn't change"
                );
                assert!(self.ws.snapshot().same_as(&before), "nothing was applied");
                self.disk = self.world.live_disk(&self.layout);
                self.ws = Workspace::load_with(
                    &SIDE_FILES,
                    model_file(&self.spec, &self.model),
                    self.layout.content_root.clone(),
                    Arc::new(self.disk.clone()),
                );
                return;
            }
            Err(ApplyError::LayoutChanged) => panic!("no model moves the content root"),
        };
        let snapshot = self.ws.snapshot();
        if !updated.changed {
            assert!(
                snapshot.same_as(&before),
                "{batch:?} changed nothing but made a new snapshot"
            );
        }

        // The source index equals one loaded from scratch, numbered as the
        // workspace's is.
        let reference = Project::load_with_ids(
            self.model.clone(),
            self.layout.clone(),
            &self.world.live_disk(&self.layout),
            &mut self.ws.ids().clone(),
        );
        let ours = dump(snapshot.index());
        let theirs = dump(&reference);
        assert_eq!(ours.len(), theirs.len(), "after {batch:?}");
        for (a, b) in ours.iter().zip(&theirs) {
            assert_eq!(a, b, "the index differs after {batch:?}");
        }

        // Everything else equals a fresh load's.
        let fresh = fresh(&self.spec, &self.model, &self.world, &self.layout).snapshot();
        let ours = holds(&snapshot);
        let theirs = holds(&fresh);
        assert_eq!(ours.len(), theirs.len(), "after {batch:?}");
        for (a, b) in ours.iter().zip(&theirs) {
            assert_eq!(a, b, "the workspace differs after {batch:?}");
        }
    }
}

fn steps() -> impl Strategy<Value = Vec<Vec<Op>>> {
    prop::collection::vec(prop::collection::vec(op(), 1..=3), 1..=16)
}

proptest! {
    #![proptest_config(ProptestConfig { cases: 64, max_shrink_iters: 2000, ..ProptestConfig::default() })]

    #[test]
    fn a_workspace_updated_equals_a_fresh_load_starting_empty(batches in steps()) {
        let mut run = Run::new(false);
        for batch in &batches {
            run.step(batch);
        }
    }

    #[test]
    fn a_workspace_updated_equals_a_fresh_load_starting_from_disk(batches in steps()) {
        let mut run = Run::new(true);
        for batch in &batches {
            run.step(batch);
        }
    }
}

/// The harness has to be able to fail: a snapshot that keeps what it read
/// of a memo file after the file changed is caught.
#[test]
fn the_comparison_notices_a_stale_memo() {
    let mut run = Run::new(true);
    let before = run.ws.snapshot();
    prime(&before);
    let (name, texts) = TEXTS
        .iter()
        .find(|(name, _)| name.starts_with("memos/"))
        .unwrap();
    run.world.put_text(&path(name), Some(&texts[1]));
    run.disk.update(&run.world, &run.layout);
    let fresh = fresh(&run.spec, &run.model, &run.world, &run.layout).snapshot();
    assert_ne!(holds(&before), holds(&fresh), "the old snapshot is stale");
    run.ws
        .apply([Changed::Index(Change::AssetCreated { path: path(name) })])
        .unwrap();
    assert_eq!(holds(&run.ws.snapshot()), holds(&fresh));
}
