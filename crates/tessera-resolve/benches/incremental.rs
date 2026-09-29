//! How long an incremental update takes on a project of 3,000 pages.
//!
//! Run with `cargo bench -p tessera-resolve --bench incremental`. It prints
//! the time to load the project from scratch, and the min, median, and 95th
//! percentile of applying one change, for several kinds of change. A plain
//! `main` rather than a benchmark framework: nothing here needs statistics
//! beyond those.
//!
//! The project: 3,000 pages in 30 sections, 100 fragments, and 60 images. Every
//! page has a title, a few headings, an include of one of the fragments, a
//! few links to other pages (a third to a heading), and an image; one page in
//! ten also has an availability marker.

#![allow(clippy::expect_used, clippy::unwrap_used, clippy::print_stdout)]

use std::fmt::Write as _;
use std::sync::Arc;
use std::time::{Duration, Instant};

use tessera_core::{FileId, RelPath};
use tessera_resolve::{Change, IncrementalProject, Layout, MemoryFs, Project};

const PAGES: usize = 3000;
const FRAGMENTS: usize = 100;
const IMAGES: usize = 60;
const RUNS: usize = 200;

const MODEL: &str = r#"spec = "0.1"

[project]
content-root = "docs"

[phrases]
product = "Quill"

[dimensions.deployment]
values = ["cloud", "self-managed"]
versionless = ["cloud"]

[builds.site]
variants = "switch"
availability = "badge"
"#;

fn page_path(i: usize) -> String {
    format!("s{}/p{i}.md", i % 30)
}

fn page_text(i: usize, edit: usize) -> String {
    let mut t = String::new();
    let _ = write!(
        t,
        "---\ntitle: Page {i}\n---\n\n# Page {i}\n\nIntro for {{product}}, revision {edit}.\n\n"
    );
    let _ = write!(
        t,
        "## Setup\n\nSteps for page {i}.\n\n## Usage\n\nMore.\n\n"
    );
    let _ = write!(t, "@include: /_f/f{}.md\n\n", i % FRAGMENTS);
    for k in 1..=3 {
        let target = (i * 7 + k * 131) % PAGES;
        if k == 1 {
            let _ = writeln!(t, "See [the setup](/{}#setup).", page_path(target));
        } else {
            let _ = writeln!(t, "See [page {target}](/{}).", page_path(target));
        }
    }
    let _ = write!(t, "\n![diagram](/img/i{}.png)\n", i % IMAGES);
    if i.is_multiple_of(10) {
        t.push_str("\n## Cloud only\n@available: cloud\n\nCloud text.\n");
    }
    t
}

fn fragment_text(i: usize, edit: usize) -> String {
    format!("## Shared {i}\n\nShared text, revision {edit}.\n")
}

fn summarize(name: &str, mut times: Vec<Duration>, note: &str) {
    times.sort();
    let at = |q: f64| times[((times.len() as f64 - 1.0) * q).round() as usize];
    println!(
        "{name:<44} min {:>9.3?}  median {:>9.3?}  p95 {:>9.3?}   {note}",
        times[0],
        at(0.5),
        at(0.95),
    );
}

fn main() {
    let model = Arc::new(tessera_model::load_str(MODEL, FileId::new(0)).expect("a model"));
    let layout = Layout::from_model(&model);
    let mut fs = MemoryFs::new(&layout);
    for i in 0..PAGES {
        fs = fs.with_source(&page_path(i), &page_text(i, 0));
    }
    for i in 0..FRAGMENTS {
        fs = fs.with_source(&format!("_f/f{i}.md"), &fragment_text(i, 0));
    }
    for i in 0..IMAGES {
        fs = fs.with_file(&format!("docs/img/i{i}.png"), "");
    }

    let start = Instant::now();
    let scratch = Project::load(model.clone(), layout.clone(), &fs);
    println!(
        "from-scratch load of {PAGES} pages, {FRAGMENTS} fragments: {:.3?} ({} files)",
        start.elapsed(),
        scratch.files().count()
    );
    drop(scratch);
    let start = Instant::now();
    let mut inc = IncrementalProject::load(model.clone(), layout, fs);
    println!(
        "incremental load (same, plus caches): {:.3?}\n",
        start.elapsed()
    );

    // Warm the expansion cache the way a language server would have: every
    // page expanded once.
    let warm = inc.snapshot();
    let start = Instant::now();
    for page in warm.pages() {
        let _ = warm.expansion(&page.path);
    }
    println!("expanding every page once: {:.3?}\n", start.elapsed());
    drop(warm);

    let p = |s: &str| RelPath::parse(s).expect("a path");
    let mut run =
        |name: &str, note: &str, hold: bool, mut change: Box<dyn FnMut(usize) -> Vec<Change>>| {
            let mut times = Vec::with_capacity(RUNS);
            let mut last = None;
            for n in 1..=RUNS {
                let changes = change(n);
                // A language server usually has a snapshot in use while the next
                // update arrives, which makes the update copy the project's maps.
                let held = hold.then(|| inc.snapshot());
                let start = Instant::now();
                let affected = inc.apply(changes).expect("applies");
                times.push(start.elapsed());
                drop(held);
                last = Some(affected);
            }
            let affected = last.expect("ran");
            summarize(
                name,
                times,
                &format!(
                    "{note}: {} rechecked, {} re-resolved",
                    affected.recheck.len(),
                    affected.re_resolve.len()
                ),
            );
        };

    run(
        "edit a paragraph of a page",
        "a page",
        false,
        Box::new(move |n| {
            vec![Change::Edited {
                path: p(&page_path(1234)),
                text: page_text(1234, n),
            }]
        }),
    );
    run(
        "edit a paragraph, a snapshot held",
        "worst case: copies the maps",
        true,
        Box::new(move |n| {
            vec![Change::Edited {
                path: p(&page_path(1234)),
                text: page_text(1234, n),
            }]
        }),
    );
    run(
        "edit a fragment's text (30 pages include it)",
        "",
        false,
        Box::new(move |n| {
            vec![Change::Edited {
                path: p("_f/f7.md"),
                text: fragment_text(7, n),
            }]
        }),
    );
    run(
        "rename a heading in a fragment",
        "",
        false,
        Box::new(move |n| {
            vec![Change::Edited {
                path: p("_f/f7.md"),
                text: format!("## Shared {n}\n\nShared text.\n"),
            }]
        }),
    );
    run(
        "create then delete a page (two updates)",
        "",
        false,
        Box::new(move |n| {
            if n % 2 == 1 {
                vec![Change::Created {
                    path: p("s0/new.md"),
                    text: page_text(0, n),
                }]
            } else {
                vec![Change::Deleted {
                    path: p("s0/new.md"),
                }]
            }
        }),
    );
    run(
        "add then remove an image",
        "",
        false,
        Box::new(move |n| {
            let path = RelPath::parse("docs/img/i0.png").expect("a path");
            if n % 2 == 1 {
                vec![Change::AssetDeleted { path }]
            } else {
                vec![Change::AssetCreated { path }]
            }
        }),
    );
    println!();
    println!("model change (phrase value): reindexes every file");
    let start = Instant::now();
    let changed = MODEL.replace("Quill", "Quill Pro");
    let model = Arc::new(tessera_model::load_str(&changed, FileId::new(0)).expect("a model"));
    let affected = inc.apply([Change::Model(model)]).expect("applies");
    println!(
        "{:<44} {:>9.3?}   {} indexed, {} parsed",
        "apply",
        start.elapsed(),
        affected.indexed.len(),
        affected.parsed.len()
    );
}
