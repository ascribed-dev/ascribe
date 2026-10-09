//! The comparison's short path, differentially: for random pairs of
//! versions, [`compare_builds`], which skips the pages nothing they use
//! changed in, reports exactly what comparing every page one by one
//! ([`compare_page_in`]) does.
//!
//! The versions are made from the language server's incremental test's
//! vocabulary (`ascribe-resolve/tests/incremental_support`): includes,
//! links with and without text, headings, images, case twins, nested
//! projects, the glossary, and models that differ; plus snippets of two
//! code files whose regions change, and a glossary term on a page that
//! doesn't link to the glossary.

#![allow(clippy::expect_used, clippy::panic, clippy::unwrap_used)]

#[path = "../../ascribe-resolve/tests/incremental_support/mod.rs"]
mod incremental_support;

use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use ascribe_core::{FileId, RelPath};
use ascribe_diff::{BuildDiff, Side, compare_builds, compare_page_in};
use ascribe_resolve::{Change, Layout, Project};
use proptest::prelude::*;

use incremental_support::{BODY, FILES, FRONT, ModelSpec, SOURCES, World, path};

/// Lines after [`BODY`]'s: snippets, and a glossary term with no link to
/// its page.
const MORE: &[&str] = &[
    "Keep the API key safe.\n",
    "@snippet: code:app.py#main\n",
    "@snippet: code:app.py#other\n",
    "@snippet: code:lib.py\n",
    "@snippet: code:gone.py\n",
];

/// The code files, by project path, and the versions of each.
const CODE: &[&str] = &["code/app.py", "code/lib.py"];
const CODE_TEXT: &[&str] = &[
    "# :snippet-start: main\nrun()\n# :snippet-end:\n# :snippet-start: other\nstop()\n# :snippet-end:\n",
    "# :snippet-start: main\nrun(fast)\n# :snippet-end:\n# :snippet-start: other\nstop()\n# :snippet-end:\n",
    "# :snippet-start: main\nrun()\n# :snippet-end:\nrest()\n",
];

fn render(front: usize, body: &[usize]) -> String {
    let mut text = FRONT[front % FRONT.len()].to_owned();
    for at in body {
        let at = at % (BODY.len() + MORE.len());
        text.push_str(
            BODY.get(at)
                .copied()
                .unwrap_or_else(|| MORE[at - BODY.len()]),
        );
        text.push('\n');
    }
    text
}

#[derive(Clone, Debug)]
enum Op {
    Write {
        file: usize,
        front: usize,
        body: Vec<usize>,
    },
    Delete {
        file: usize,
    },
    Rename {
        from: usize,
        to: usize,
    },
    File {
        file: usize,
    },
    Code {
        file: usize,
        text: Option<usize>,
    },
    Model {
        phrase: bool,
    },
}

fn op() -> impl Strategy<Value = Op> {
    let lines = BODY.len() + MORE.len();
    prop_oneof![
        6 => (0..SOURCES.len(), 0..FRONT.len(), prop::collection::vec(0..lines, 0..6))
            .prop_map(|(file, front, body)| Op::Write { file, front, body }),
        2 => (0..SOURCES.len()).prop_map(|file| Op::Delete { file }),
        1 => (0..SOURCES.len(), 0..SOURCES.len()).prop_map(|(from, to)| Op::Rename { from, to }),
        2 => (0..FILES.len()).prop_map(|file| Op::File { file }),
        2 => (0..CODE.len(), prop::option::of(0..CODE_TEXT.len()))
            .prop_map(|(file, text)| Op::Code { file, text }),
        1 => any::<bool>().prop_map(|phrase| Op::Model { phrase }),
    ]
}

/// A version of the project: its files, its code files' texts, and its
/// model.
#[derive(Clone, Default)]
struct State {
    world: World,
    code: BTreeMap<&'static str, usize>,
    product: bool,
}

impl State {
    fn spec(&self) -> ModelSpec {
        ModelSpec {
            product: if self.product { "Quill Pro" } else { "Quill" },
            ..ModelSpec::base()
        }
    }

    fn apply(&mut self, op: &Op) {
        let layout = Layout::from_model(&self.spec().model());
        let change = match op {
            Op::Write { file, front, body } => Change::Edited {
                path: path(SOURCES[*file]),
                text: render(*front, body),
            },
            Op::Delete { file } => Change::Deleted {
                path: path(SOURCES[*file]),
            },
            Op::Rename { from, to } => Change::Renamed {
                from: path(SOURCES[*from]),
                to: path(SOURCES[*to]),
            },
            Op::File { file } => {
                let name = FILES[*file].trim_end_matches('/');
                // `docs/files/` is a folder a link names, never a file.
                if FILES[*file].ends_with('/') {
                    return;
                }
                let at = path(name);
                if self.world.files.contains(&at) {
                    Change::AssetDeleted { path: at }
                } else {
                    Change::AssetCreated { path: at }
                }
            }
            Op::Code { file, text } => {
                match text {
                    Some(text) => self.code.insert(CODE[*file], *text),
                    None => self.code.remove(CODE[*file]),
                };
                return;
            }
            Op::Model { phrase } => {
                self.product = *phrase;
                return;
            }
        };
        self.world.apply(&layout, &change);
    }

    fn load(&self) -> (String, Project) {
        let text = format!("{}\n[sources.code]\npath = \"code\"\n", self.spec().text());
        let model = ascribe_model::load_str(&text, FileId::new(0)).expect("a valid model");
        let layout = Layout::from_model(&model);
        let mut fs = self.world.fs(&layout);
        for (file, at) in &self.code {
            fs = fs.with_file(file, CODE_TEXT[*at]);
        }
        let project = Project::load(Arc::new(model), layout, &fs);
        (text, project)
    }
}

/// Every page of both versions compared one by one, as [`compare_builds`]
/// reports them.
fn every_page(base: Side<'_>, now: Side<'_>, builds: &[&str]) -> Vec<BuildDiff> {
    let paths: BTreeSet<&RelPath> = base
        .project
        .pages()
        .chain(now.project.pages())
        .map(|p| &p.path)
        .collect();
    builds
        .iter()
        .filter(|name| now.project.model().build(name).is_some())
        .map(|name| BuildDiff {
            build: (*name).to_owned(),
            pages: paths
                .iter()
                .filter_map(|path| compare_page_in(Some(base), now, name, path))
                .collect(),
        })
        .collect()
}

proptest! {
    #[test]
    fn skipping_pages_changes_nothing(
        start in prop::collection::vec(op(), 0..14),
        change in prop::collection::vec(op(), 1..5),
    ) {
        let mut before = State::default();
        for op in &start {
            before.apply(op);
        }
        let mut after = before.clone();
        for op in &change {
            after.apply(op);
        }
        let (base_model, base) = before.load();
        let (now_model, now) = after.load();
        let base = Side { project: &base, model_text: &base_model };
        let now = Side { project: &now, model_text: &now_model };
        let builds = ["site", "cloud"];
        prop_assert_eq!(compare_builds(Some(base), now, &builds), every_page(base, now, &builds));
    }
}
