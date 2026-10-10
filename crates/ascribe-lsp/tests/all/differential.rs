//! After any sequence of edits, file changes, and model changes, what the server
//! has published is what a fresh `check_project` over the same files reports.
//!
//! This is the server-level counterpart of `ascribe-resolve`'s differential
//! test (`incremental_differential.rs`): it holds the incremental bookkeeping
//! (which files to recompute, the layered file system, the currency checks)
//! to the one property that matters.

#![allow(clippy::expect_used, clippy::panic, clippy::unwrap_used)]

use crate::support;

use std::collections::BTreeMap;
use std::path::PathBuf;

use crate::support::{Client, Fixture};
use ascribe_check::{Project, SourceFile, check_project};
use ascribe_core::{FileId, LineIndex, RelPath, WideEncoding, WideLineCol};
use lsp_types::FileChangeType;

const PAGES: [&str; 12] = [
    "---\ntitle: X\n---\n# X\n\n@include: _frag.md#a\n",
    "---\ntitle: X\n---\n[l](guide.md#sec) [m](sub/other.md) ![i](pic.png)\n",
    "---\ntitle: X\n---\n# One\n@id: dup\n\n# Two\n@id: dup\n",
    "---\ntitle: X\n---\n😀 [broken](gone.md) {nope}\n",
    "---\ntitle: X\n---\n@note {type=tip}:\nunclosed\n",
    "---\ntitle: X\n---\n.T\n@variant {pm=npm}:\nA\n@variant {pm=pnpm}:\nB\n@end\n[l](sub/other.md)\n",
    "no frontmatter\n",
    "---\ntitle: X\navailable: cloud\n---\n# X\n\n[back](index.md)\n",
    "---\ntitle: X\n---\n# X\n\n@include: _frag.md\n@include: _frag.md\n",
    "---\ntitle: X\navailable: self-managed\n---\n# Only self-managed\n",
    "---\ntitle: X\n---\n# X\n\n[to guide](guide.md#setup) [to other](sub/other.md#part)\n",
    "---\ntitle: X\n---\n# Setup\n@id: setup\n\n# Part\n@id: part\n\n@include: _frag.md#b\n",
];

const FRAGMENTS: [&str; 6] = [
    "## Loop\n@include: _frag.md\n",
    "## B\n@id: b\n\n@include: sub/other.md\n",
    "## A\n@id: a\n\nText.\n",
    "## B\n@id: b\n",
    "## A\n@id: a\n[x](index.md#missing)\n",
    "![z](pic.png)\n",
];

fn model(build: &str, widget: bool) -> String {
    let mut text = String::from(
        "spec = \"0.1\"\n\n[project]\ncontent-root = \"docs\"\n\n[dimensions.pm]\nvalues = [\"npm\", \"pnpm\"]\n\n[dimensions.deployment]\nvalues = [\"cloud\", \"self-managed\"]\nversionless = [\"cloud\"]\n\n[builds.site]\nvariants = \"switch\"\navailability = \"badge\"\n\n[builds.cloud]\nvariants = { pm = \"npm\" }\navailability = { filter = \"cloud\" }\n",
    );
    if widget {
        text.push_str(
            "\n[widgets.my-note]\nforms = [\"line\"]\nprimary = \"text\"\nbinding = \"self\"\n",
        );
    }
    text.push_str(&format!("\n[editor]\nbuild = \"{build}\"\n"));
    // The content checks across the project run on save, which the steps
    // don't do, and the editor doesn't report the image checks; `across.rs`
    // covers them.
    text.push_str("\n[checks]\npage-orphan = \"off\"\nfragment-unused = \"off\"\ntitle-duplicate = \"off\"\nimage-unused = \"off\"\nimage-large = \"off\"\n");
    text
}

struct Rng(u64);

impl Rng {
    fn next(&mut self, n: usize) -> usize {
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        ((self.0 >> 33) as usize) % n
    }
}

/// The files of the world and which of them the editor has open.
struct World {
    fixture: Fixture,
    files: Vec<&'static str>,
    open: BTreeMap<&'static str, i32>,
    /// What an open buffer holds.
    buffers: BTreeMap<&'static str, String>,
    build: &'static str,
    widget: bool,
    model_open: Option<i32>,
    assets: bool,
}

impl World {
    fn path(&self, rel: &str) -> PathBuf {
        self.fixture.path(&format!("docs/{rel}"))
    }

    fn text_on_disk(&self, rel: &str) -> Option<String> {
        std::fs::read_to_string(self.path(rel)).ok()
    }

    /// What the project holds for a file: the buffer, else the disk.
    fn text(&self, rel: &str) -> Option<String> {
        self.buffers
            .get(rel)
            .cloned()
            .or_else(|| self.text_on_disk(rel))
    }

    /// A fresh `check_project` over what the files hold now, by file.
    fn expected(&self) -> BTreeMap<String, Vec<(String, usize, usize, String)>> {
        let model_text = model(self.build, self.widget);
        let root = self.fixture.root();
        let m = ascribe_model::load_str_in(&model_text, FileId::new(0), &root).expect("a model");
        let mut texts: Vec<(RelPath, String)> = Vec::new();
        for rel in &self.files {
            if let Some(text) = self.text(rel) {
                texts.push((RelPath::parse(rel).unwrap(), text));
            }
        }
        let sources: Vec<SourceFile> = Project::from_sources(texts);
        let project = Project::from_parts(
            root.clone(),
            RelPath::parse("docs").unwrap(),
            m.clone(),
            model_text,
            sources,
        );
        let build = m.editor_default_build().clone();
        let mut out: BTreeMap<String, Vec<(String, usize, usize, String)>> = BTreeMap::new();
        for d in check_project(&project, &build) {
            let Some(entry) = project.file(d.location.file) else {
                continue;
            };
            if entry.display_path == "ascribe.toml" {
                continue;
            }
            out.entry(entry.display_path.clone()).or_default().push((
                d.slug.as_str().to_owned(),
                d.location.span.start(),
                d.location.span.end(),
                d.message.clone(),
            ));
        }
        for list in out.values_mut() {
            list.sort();
        }
        out
    }

    /// What the server has published, by file.
    fn published(&self, client: &Client) -> BTreeMap<String, Vec<(String, usize, usize, String)>> {
        let mut out = BTreeMap::new();
        for rel in &self.files {
            let diagnostics = client.diagnostics(&self.path(rel));
            if diagnostics.is_empty() {
                continue;
            }
            let Some(text) = self.text(rel) else {
                panic!("diagnostics for {rel}, which doesn't exist");
            };
            let index = LineIndex::new(&text);
            let at = |p: lsp_types::Position| {
                index
                    .wide_offset(
                        WideEncoding::Utf16,
                        WideLineCol {
                            line: p.line,
                            col: p.character,
                        },
                    )
                    .expect("a position")
            };
            let mut list: Vec<_> = diagnostics
                .iter()
                .map(|d| {
                    (
                        support::slug(d),
                        at(d.range.start),
                        at(d.range.end),
                        d.message.clone(),
                    )
                })
                .collect();
            list.sort();
            out.insert(format!("docs/{rel}"), list);
        }
        out
    }
}

fn run(seed: u64, steps: usize) {
    let files = vec!["index.md", "guide.md", "_frag.md", "sub/other.md"];
    let fixture = Fixture::new(&model("site", false), &[]);
    for (i, rel) in files.iter().enumerate() {
        let text = if rel.starts_with('_') {
            FRAGMENTS[0]
        } else {
            PAGES[i]
        };
        fixture.write(&format!("docs/{rel}"), text);
    }
    fixture.write("docs/pic.png", "x");
    let mut world = World {
        fixture,
        files,
        open: BTreeMap::new(),
        buffers: BTreeMap::new(),
        build: "site",
        widget: false,
        model_open: None,
        assets: true,
    };
    let mut client = Client::start(&world.fixture.root());
    let mut rng = Rng(seed);
    let mut version = 10;
    client.settle();
    let mut trace: Vec<String> = Vec::new();
    for step in 0..steps {
        let rel = world.files[rng.next(world.files.len())];
        let pick = |rng: &mut Rng| -> String {
            if rel.starts_with('_') {
                FRAGMENTS[rng.next(FRAGMENTS.len())].to_owned()
            } else {
                PAGES[rng.next(PAGES.len())].to_owned()
            }
        };
        let what;
        match rng.next(9) {
            0 | 1 => {
                // Type into a buffer (opening it first when it isn't).
                what = format!("edit buffer {rel}");
                let text = pick(&mut rng);
                version += 1;
                if world.open.contains_key(rel) {
                    client.replace(&world.path(rel), version, &text);
                } else {
                    client.open(&world.path(rel), version, &text);
                }
                world.open.insert(rel, version);
                world.buffers.insert(rel, text);
            }
            2 => {
                what = format!("close {rel}");
                if world.open.remove(rel).is_some() {
                    client.close(&world.path(rel));
                    world.buffers.remove(rel);
                }
            }
            3 | 4 => {
                what = format!("write {rel} on disk");
                let text = pick(&mut rng);
                world.fixture.write(&format!("docs/{rel}"), &text);
                client.watched(&[(&world.path(rel), FileChangeType::CHANGED)]);
            }
            5 => {
                what = format!("delete {rel} on disk");
                if world.text_on_disk(rel).is_some() {
                    world.fixture.remove(&format!("docs/{rel}"));
                    client.watched(&[(&world.path(rel), FileChangeType::DELETED)]);
                }
            }
            6 => {
                what = format!("create {rel} on disk");
                if world.text_on_disk(rel).is_none() {
                    let text = pick(&mut rng);
                    world.fixture.write(&format!("docs/{rel}"), &text);
                    client.watched(&[(&world.path(rel), FileChangeType::CREATED)]);
                }
            }
            7 => {
                what = "toggle the image".to_owned();
                let pic = world.fixture.path("docs/pic.png");
                if world.assets {
                    world.fixture.remove("docs/pic.png");
                    client.watched(&[(&pic, FileChangeType::DELETED)]);
                } else {
                    world.fixture.write("docs/pic.png", "x");
                    client.watched(&[(&pic, FileChangeType::CREATED)]);
                }
                world.assets = !world.assets;
            }
            _ => {
                what = "change the model".to_owned();
                if rng.next(2) == 0 {
                    world.widget = !world.widget;
                } else {
                    world.build = if world.build == "site" {
                        "cloud"
                    } else {
                        "site"
                    };
                }
                let text = model(world.build, world.widget);
                let config = world.fixture.path("ascribe.toml");
                match world.model_open {
                    Some(v) => {
                        client.replace(&config, v + 1, &text);
                        world.model_open = Some(v + 1);
                    }
                    None => {
                        world.fixture.write("ascribe.toml", &text);
                        client.watched(&[(&config, FileChangeType::CHANGED)]);
                    }
                }
            }
        }
        trace.push(what.clone());
        client.settle();
        let expected = world.expected();
        let published = world.published(&client);
        assert_eq!(
            published, expected,
            "seed {seed}, step {step} ({what}): the server disagrees with a fresh check\nsteps: {trace:#?}"
        );
    }
}

#[test]
fn the_server_agrees_with_a_fresh_check_after_any_sequence() {
    // `ASCRIBE_LSP_SEEDS=200` for a longer soak.
    let seeds: u64 = std::env::var("ASCRIBE_LSP_SEEDS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(8);
    for seed in 1..=seeds {
        run(seed, 30);
    }
}
