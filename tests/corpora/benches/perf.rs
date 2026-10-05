//! Times `ascribe check`, `ascribe build`, `ascribe diff`, and `ascribe
//! drift` on the 3,000-page synthetic project and on the converted Elastic corpus.
//!
//! Run with `cargo bench -p tessera-corpora --bench perf` (build the CLI first
//! with `cargo build --release -p tessera-cli`; the bench finds `ascribe` next
//! to the other release binaries, or in `ASCRIBE_BIN`). For each project it
//! prints:
//!
//! - **the command**: wall time of `ascribe check` (and, for the synthetic
//!   project, `ascribe build`, `ascribe diff`, and `ascribe drift`), process
//!   start to exit,
//!   output to /dev/null, on the project as it is and with a snippet on every
//!   page;
//! - **the library phases** the command is made of, in-process: loading the
//!   project, the file-level checks, and the page-level checks of every build;
//!   and, for a build, resolving and emitting plain markdown.
//!
//! The Elastic corpus is fetched at the pinned commit (skipped, with a
//! message, when the network isn't there). With `ASCRIBE_BENCH_OUT` set, the
//! command timings are also appended as JSON lines for `corpora compare`.

#![allow(clippy::expect_used, clippy::unwrap_used, clippy::print_stdout)]

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::Arc;
use std::time::{Duration, Instant};

use tessera_check::{Project, check_all_builds, check_files};
use tessera_corpora::convert::{self, Extra};
use tessera_corpora::corpus::{self, Corpus};
use tessera_emit::{EmitContext, Emitter, PlainEmitter, emit};
use tessera_resolve::AstroRouter;
use tessera_synthetic::{CONTENT_ROOT, Synthetic, report::record};

fn median(times: &mut [Duration]) -> Duration {
    times.sort();
    times[times.len() / 2]
}

fn ascribe_bin() -> PathBuf {
    if let Some(p) = std::env::var_os("ASCRIBE_BIN") {
        return PathBuf::from(p);
    }
    let exe = std::env::current_exe().expect("the bench's path");
    // target/release/deps/perf-xxxx -> target/release/ascribe
    let release = exe.ancestors().nth(2).expect("target dir").to_path_buf();
    let bin = release.join("ascribe");
    assert!(
        bin.exists(),
        "{} not found: run `cargo build --release -p tessera-cli` first",
        bin.display()
    );
    bin
}

fn run_cli(bin: &Path, dir: &Path, args: &[&str]) -> Duration {
    let start = Instant::now();
    let status = Command::new(bin)
        .args(args)
        .current_dir(dir)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .expect("runs ascribe");
    let elapsed = start.elapsed();
    // Exit code 1 is "the project has errors" (the converted corpora do); 2 is
    // a failure to run at all.
    assert!(
        status.code().is_some_and(|c| c <= 1),
        "ascribe failed: {status}"
    );
    elapsed
}

fn time_cli(name: &str, metric: &str, bin: &Path, dir: &Path, args: &[&str], runs: usize) {
    let mut times: Vec<Duration> = (0..runs).map(|_| run_cli(bin, dir, args)).collect();
    println!(
        "{name:<52} median {:>9.3?}  min {:>9.3?}  max {:>9.3?}  ({runs} runs)",
        median(&mut times),
        times[0],
        times[times.len() - 1]
    );
    record(metric, &mut times);
}

/// The phases `ascribe check` and `ascribe build` are made of, in-process.
fn phases(name: &str, dir: &Path) {
    let config = dir.join("ascribe.toml");
    let start = Instant::now();
    let project = Project::load(&config).expect("loads");
    let load = start.elapsed();
    let start = Instant::now();
    let file_level = check_files(&project);
    let files = start.elapsed();
    let start = Instant::now();
    let all = check_all_builds(&project);
    let pages = start.elapsed();
    println!(
        "{name:<52} load {load:>9.3?}  file-level {files:>9.3?} ({} diagnostics)  file+page-level, all builds {pages:>9.3?} ({})",
        file_level.len(),
        all.len()
    );

    // A build without the checks in front of it: resolve every page of the
    // first build and render plain markdown.
    let model = Arc::new(project.model().clone());
    let start = Instant::now();
    let resolved = tessera_resolve::Project::load(
        model.clone(),
        project.layout().clone(),
        project.file_system(),
    );
    let index = start.elapsed();
    let Some(build) = model.builds.first() else {
        return;
    };
    let router = AstroRouter::from_consumer(&model.consumer);
    let start = Instant::now();
    let tree = resolved.resolve_build(build, &router);
    let resolve = start.elapsed();
    let start = Instant::now();
    let cx = EmitContext::new(&resolved, project.root(), build);
    let plain: &dyn Emitter = &PlainEmitter;
    let emission = emit(plain, &cx, &tree).expect("emits");
    let emitted = start.elapsed();
    println!(
        "{:<52} index {index:>9.3?}  resolve build `{}` {resolve:>9.3?}  emit plain {emitted:>9.3?} ({} files)",
        "",
        build.name,
        emission.files.len()
    );
}

/// `ascribe diff` on the synthetic project in a git repository: with nothing
/// changed, with one page changed, with a fragment that 100 pages include
/// changed, and with a phrase every page uses changed. Skipped when `git`
/// isn't there.
fn diff(bin: &Path, dir: &Path) {
    let root = dir.join("synthetic-git");
    std::fs::create_dir_all(&root).expect("creates");
    let project = Synthetic::standard();
    project.write_to(&root).expect("writes");
    // One fragment that the first 100 pages include.
    let docs = root.join(CONTENT_ROOT);
    std::fs::write(
        docs.join("_f").join("wide.md"),
        "Shared by a hundred pages.\n",
    )
    .expect("writes");
    for i in 0..100 {
        let path = docs.join(project.page_path(i));
        let text = std::fs::read_to_string(&path).expect("reads");
        std::fs::write(&path, format!("{text}\n@include: /_f/wide.md\n")).expect("writes");
    }
    let git = |args: &[&str]| {
        Command::new("git")
            .current_dir(&root)
            .args([
                "-c",
                "user.name=bench",
                "-c",
                "user.email=bench@example.com",
            ])
            .args(["-c", "commit.gpgsign=false"])
            .args(args)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .is_ok_and(|s| s.success())
    };
    if !(git(&["init", "-q", "-b", "main"])
        && git(&["add", "-A"])
        && git(&["commit", "-q", "-m", "base"]))
    {
        println!("ascribe diff: skipped (git isn't available)");
        println!();
        return;
    }
    println!("synthetic project in git: 3,000 pages, 100 of them including one more fragment");
    let args = ["diff", "--base", "main", "--format", "json"];
    time_cli(
        "  ascribe diff (nothing changed)",
        "diff/synthetic-3000-unchanged",
        bin,
        &root,
        &args,
        3,
    );

    let one = docs.join(project.page_path(1234));
    let text = std::fs::read_to_string(&one).expect("reads");
    std::fs::write(&one, text.replace("welcome.", "welcome back.")).expect("writes");
    time_cli(
        "  ascribe diff (one page changed)",
        "diff/synthetic-3000-one-page",
        bin,
        &root,
        &args,
        3,
    );
    std::fs::write(&one, text).expect("writes");

    let wide = docs.join("_f").join("wide.md");
    std::fs::write(&wide, "Shared by a hundred pages, and changed.\n").expect("writes");
    time_cli(
        "  ascribe diff (fragment used by 100 pages changed)",
        "diff/synthetic-3000-fragment",
        bin,
        &root,
        &args,
        3,
    );
    std::fs::write(&wide, "Shared by a hundred pages.\n").expect("writes");

    let model = root.join("ascribe.toml");
    let text = std::fs::read_to_string(&model).expect("reads");
    std::fs::write(
        &model,
        text.replace("product = \"Quill\"", "product = \"Quill Cloud\""),
    )
    .expect("writes");
    time_cli(
        "  ascribe diff (phrase used by every page changed)",
        "diff/synthetic-3000-phrase",
        bin,
        &root,
        &args,
        3,
    );
    std::fs::write(&model, text).expect("writes");
    println!();
}

/// `ascribe diff` and `ascribe drift` on the synthetic project with
/// snippets, in a git repository: with nothing changed, and with one code
/// file changed in the region ten pages show. Skipped when `git` isn't there.
fn diff_snippets(bin: &Path, dir: &Path) {
    let root = dir.join("synthetic-snippets-git");
    std::fs::create_dir_all(&root).expect("creates");
    let project = Synthetic::standard().with_snippets();
    project.write_to(&root).expect("writes");
    let git = |args: &[&str]| {
        Command::new("git")
            .current_dir(&root)
            .args([
                "-c",
                "user.name=bench",
                "-c",
                "user.email=bench@example.com",
            ])
            .args(["-c", "commit.gpgsign=false"])
            .args(args)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .is_ok_and(|s| s.success())
    };
    if !(git(&["init", "-q", "-b", "main"])
        && git(&["add", "-A"])
        && git(&["commit", "-q", "-m", "base"]))
    {
        println!("ascribe diff with snippets: skipped (git isn't available)");
        println!();
        return;
    }
    println!("synthetic project with snippets in git");
    let args = ["diff", "--base", "main", "--format", "json"];
    time_cli(
        "  ascribe diff (nothing changed)",
        "diff/synthetic-3000-snippets-unchanged",
        bin,
        &root,
        &args,
        3,
    );
    let drift = ["drift", "--base", "main", "--format", "json"];
    time_cli(
        "  ascribe drift (nothing changed)",
        "drift/synthetic-3000-snippets-unchanged",
        bin,
        &root,
        &drift,
        3,
    );
    let code = root.join("code").join("m34.py");
    std::fs::write(&code, project.code_text(34, "return value * 2")).expect("writes");
    time_cli(
        "  ascribe diff (a region ten pages show changed)",
        "diff/synthetic-3000-snippet",
        bin,
        &root,
        &args,
        3,
    );
    time_cli(
        "  ascribe drift (a region ten pages show changed)",
        "drift/synthetic-3000-snippet",
        bin,
        &root,
        &drift,
        3,
    );
    println!();
}

fn machine() {
    let cpu = std::fs::read_to_string("/proc/cpuinfo")
        .ok()
        .and_then(|t| {
            t.lines()
                .find(|l| l.starts_with("model name"))
                .and_then(|l| l.split(':').nth(1).map(|s| s.trim().to_owned()))
        })
        .unwrap_or_else(|| "unknown CPU".into());
    let cores = std::thread::available_parallelism().map_or(0, usize::from);
    println!(
        "machine: {cpu}, {cores} logical cores, {}",
        std::env::consts::OS
    );
}

fn main() {
    machine();
    let bin = ascribe_bin();
    println!();

    let dir = tempfile::tempdir().expect("a temp dir");
    let root = dir.path().join("synthetic");
    std::fs::create_dir_all(&root).expect("creates");
    Synthetic::standard().write_to(&root).expect("writes");
    println!("synthetic project: 3,000 pages, 100 fragments, 60 images");
    phases("  library, in-process", &root);
    time_cli(
        "  ascribe check",
        "check/synthetic-3000",
        &bin,
        &root,
        &["check"],
        5,
    );
    time_cli(
        "  ascribe check --format json",
        "check/synthetic-3000-json",
        &bin,
        &root,
        &["check", "--format", "json"],
        5,
    );
    time_cli(
        "  ascribe build (first: writes every output)",
        "build/synthetic-3000-first",
        &bin,
        &root,
        &["build", "--emit", "plain,json"],
        1,
    );
    time_cli(
        "  ascribe build (again: nothing changed)",
        "build/synthetic-3000-again",
        &bin,
        &root,
        &["build", "--emit", "plain,json"],
        3,
    );
    println!();

    // The same project with a snippet on every page, read from 100 code
    // files: what reading and extracting them adds.
    let snippets = dir.path().join("synthetic-snippets");
    std::fs::create_dir_all(&snippets).expect("creates");
    Synthetic::standard()
        .with_snippets()
        .write_to(&snippets)
        .expect("writes");
    println!(
        "synthetic project with snippets: the same, and a snippet on every page from 100 code files"
    );
    phases("  library, in-process", &snippets);
    time_cli(
        "  ascribe check",
        "check/synthetic-3000-snippets",
        &bin,
        &snippets,
        &["check"],
        5,
    );
    time_cli(
        "  ascribe build (first: writes every output)",
        "build/synthetic-3000-snippets-first",
        &bin,
        &snippets,
        &["build", "--emit", "plain,json"],
        1,
    );
    println!();

    diff(&bin, dir.path());
    diff_snippets(&bin, dir.path());

    // The same command on a project with a diagnostic on every page: the text
    // report is the default, and FINDINGS.md P1 is about what it costs.
    let noisy = dir.path().join("noisy");
    std::fs::create_dir_all(&noisy).expect("creates");
    let small = Synthetic::new(1000);
    small.write_to(&noisy).expect("writes");
    for (path, text) in small.pages_text() {
        let file = noisy.join(CONTENT_ROOT).join(path);
        std::fs::write(file, format!("{text}\nAn undeclared {{key}} in prose.\n")).expect("writes");
    }
    println!("noisy project: 1,000 pages, one warning on each");
    time_cli(
        "  ascribe check --format json",
        "check/noisy-1000-json",
        &bin,
        &noisy,
        &["check", "--format", "json"],
        3,
    );
    time_cli(
        "  ascribe check (text, the default)",
        "check/noisy-1000-text",
        &bin,
        &noisy,
        &["check"],
        1,
    );
    println!();

    let Some(fetched) = corpus::corpus_or_skip(Corpus::Elastic) else {
        println!("elastic corpus: skipped (not available)");
        return;
    };
    let pages = corpus::pages(&fetched).expect("reads the corpus");
    let extra = Extra {
        docset: std::fs::read_to_string(fetched.root.join("docset.yml")).unwrap_or_default(),
    };
    let converted = convert::convert(Corpus::Elastic, &pages, &extra);
    let root = dir.path().join("elastic");
    std::fs::create_dir_all(&root).expect("creates");
    converted.write_to(&root).expect("writes");
    println!(
        "converted Elastic sample: {} pages and fragments, {} images (empty placeholders)",
        converted.pages(),
        converted.assets.len()
    );
    phases("  library, in-process", &root);
    time_cli(
        "  ascribe check --format json",
        "check/elastic-converted-json",
        &bin,
        &root,
        &["check", "--format", "json"],
        3,
    );
}
