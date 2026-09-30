//! How long an incremental update takes on a project of 3,000 pages.
//!
//! Run with `cargo bench -p tessera-resolve --bench incremental`. It prints
//! the time to load the project from scratch, and the min, median, and 95th
//! percentile of applying one change, for several kinds of change. A plain
//! `main` rather than a benchmark framework: nothing here needs statistics
//! beyond those.
//!
//! The project is the standard synthetic one (`tessera-synthetic`, shared with
//! the language server's benchmarks): 3,000 pages in 30 sections, 100
//! fragments, and 60 images. Every page has a title, a few headings, an include
//! of one of the fragments, a few links to other pages (a third to a heading),
//! and an image; one page in ten also has an availability marker.

#![allow(clippy::expect_used, clippy::unwrap_used, clippy::print_stdout)]

use std::sync::Arc;
use std::time::{Duration, Instant};

use tessera_core::{FileId, RelPath};
use tessera_resolve::{Change, IncrementalProject, Layout, MemoryFs, Project};
use tessera_synthetic::{CONTENT_ROOT, FRAGMENTS, IMAGES, MODEL, Synthetic};

const RUNS: usize = 200;

fn page_path(i: usize) -> String {
    Synthetic::standard().page_path(i)
}

fn page_text(i: usize, edit: usize) -> String {
    Synthetic::standard().page_text(i, &format!("revision {edit}."))
}

fn fragment_text(i: usize, edit: usize) -> String {
    Synthetic::standard().fragment_text(i, &format!("Shared text, revision {edit}."))
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
    let synthetic = Synthetic::standard();
    let pages = synthetic.pages;
    let model = Arc::new(tessera_model::load_str(MODEL, FileId::new(0)).expect("a model"));
    let layout = Layout::from_model(&model);
    let mut fs = MemoryFs::new(&layout);
    for i in 0..pages {
        fs = fs.with_source(&page_path(i), &page_text(i, 0));
    }
    for i in 0..FRAGMENTS {
        fs = fs.with_source(&synthetic.fragment_path(i), &fragment_text(i, 0));
    }
    for i in 0..IMAGES {
        fs = fs.with_file(&format!("{CONTENT_ROOT}/{}", synthetic.image_path(i)), "");
    }

    let start = Instant::now();
    let scratch = Project::load(model.clone(), layout.clone(), &fs);
    println!(
        "from-scratch load of {pages} pages, {FRAGMENTS} fragments: {:.3?} ({} files)",
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
            tessera_synthetic::report::record(&format!("incremental/{name}"), &mut times.clone());
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
