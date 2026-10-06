//! Every output is the same from one run to the next.
//!
//! The example projects and the docs are copied into two temporary
//! directories, and every command that writes something a reader or a tool
//! sees runs in each: `check`, `build` (the site, plain, and JSON outputs,
//! and the site again with anchors), `diff`, and `drift`, in each of their formats. The two runs must
//! agree to the byte, once the directory each ran in is replaced with a
//! placeholder. Each run is a new process, so a hash map's order, which
//! changes from one process to the next, shows up here as a difference.
//!
//! For `diff` and `drift`, each copy is a git repository whose one commit is
//! the projects with a word changed throughout: in every code file, and in
//! every second page, so that some pages' examples change while their words
//! don't. The working tree is the projects as they are.
//!
//! `scripts/compare/outputs.ts` makes the same comparison between two
//! binaries, for a pull request that means to change nothing, in a copy set
//! up the same way: a change to one belongs in the other.

#![allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use tempfile::TempDir;

/// What's copied: the projects, and what `docs/ascribe.toml`'s
/// `[sources.code]` reads besides them.
const COPIED: &[&str] = &[
    "examples",
    "docs",
    ".github/workflows",
    "crates/ascribe-cli/tests/output",
];

/// Projects whose build stops with errors, by design: their `check` and
/// `build` reports are compared, and there's no output.
const FAILING: &[&str] = &[
    // A broken link, on purpose.
    "examples/getting-started",
    // Its examples come from another repository, with `ascribe sources fetch`.
    "examples/docs-repository",
];

/// Stands for the directory a run happened in.
const ROOT: &str = "<root>";

fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// The files to copy, as `/`-separated paths, from git so that nothing a
/// build left behind comes along.
fn tracked_files() -> Vec<String> {
    let out = Command::new("git")
        .current_dir(repository_root())
        .args(["ls-files", "-z", "--"])
        .args(COPIED)
        .output()
        .expect("run git");
    assert!(
        out.status.success(),
        "git ls-files failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let files: Vec<String> = String::from_utf8(out.stdout)
        .unwrap()
        .split('\0')
        .filter(|f| !f.is_empty())
        .map(str::to_owned)
        .collect();
    assert!(!files.is_empty(), "git lists no files under {COPIED:?}");
    files
}

fn join(root: &Path, rel: &str) -> PathBuf {
    rel.split('/').fold(root.to_path_buf(), |p, s| p.join(s))
}

/// `text` with every whole word "the" made "a": a change that keeps every
/// directive, region, and link working.
fn reword(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let bytes = text.as_bytes();
    let word = |i: usize| {
        bytes
            .get(i)
            .is_some_and(|b| b.is_ascii_alphanumeric() || *b == b'_')
    };
    let mut i = 0;
    while i < text.len() {
        if text[i..].starts_with("the") && (i == 0 || !word(i - 1)) && !word(i + 3) {
            out.push('a');
            i += 3;
        } else {
            let c = text[i..].chars().next().unwrap();
            out.push(c);
            i += c.len_utf8();
        }
    }
    out
}

fn git(dir: &Path, args: &[&str]) {
    let out = Command::new("git")
        .current_dir(dir)
        .args([
            "-c",
            "user.name=Mira Okafor",
            "-c",
            "user.email=mira@example.com",
            "-c",
            "commit.gpgsign=false",
            "-c",
            "core.autocrlf=false",
            "-c",
            "init.defaultBranch=main",
        ])
        .args(args)
        .env("GIT_AUTHOR_DATE", "2026-10-01T12:00:00+00:00")
        .env("GIT_COMMITTER_DATE", "2026-10-01T12:00:00+00:00")
        .output()
        .expect("run git");
    assert!(
        out.status.success(),
        "git {args:?} failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

/// A copy of the projects: a repository whose commit has the words changed,
/// with the projects as they are in its working tree.
fn copy(files: &[String]) -> TempDir {
    let dir = tempfile::tempdir().expect("a temporary directory");
    let root = repository_root();
    let mut pages = 0;
    let mut originals = Vec::new();
    for rel in files {
        let bytes = fs::read(join(&root, rel)).unwrap();
        let name = rel.rsplit('/').next().unwrap();
        let reworded = match std::str::from_utf8(&bytes) {
            Ok(_) if name == "ascribe.toml" || name == "ascribe.lock" => None,
            Ok(text) if name.ends_with(".md") => {
                pages += 1;
                (pages % 2 == 0).then(|| reword(text))
            }
            Ok(text) => Some(reword(text)),
            Err(_) => None,
        };
        let target = join(dir.path(), rel);
        fs::create_dir_all(target.parent().unwrap()).unwrap();
        match reworded {
            Some(text) if text.as_bytes() != bytes.as_slice() => {
                fs::write(&target, text).unwrap();
                originals.push((target, bytes));
            }
            _ => fs::write(&target, &bytes).unwrap(),
        }
    }
    git(dir.path(), &["init", "-q"]);
    git(dir.path(), &["add", "-A"]);
    git(dir.path(), &["commit", "-q", "-m", "The base"]);
    for (target, bytes) in originals {
        fs::write(target, bytes).unwrap();
    }
    dir
}

/// The projects: each folder with an `ascribe.toml`, as a `/`-separated path.
fn projects(files: &[String]) -> Vec<String> {
    let mut projects: Vec<String> = files
        .iter()
        .filter_map(|f| f.strip_suffix("/ascribe.toml"))
        .map(str::to_owned)
        .collect();
    projects.sort();
    projects
}

/// The ways `dir` can appear in what a command writes: as given, as the
/// operating system resolves it, with either separator, and escaped for JSON.
fn spellings(dir: &Path) -> Vec<String> {
    let mut forms = vec![dir.display().to_string()];
    if let Ok(canonical) = dir.canonicalize() {
        let canonical = canonical.display().to_string();
        forms.push(canonical.trim_start_matches(r"\\?\").to_owned());
        forms.push(canonical);
    }
    for form in forms.clone() {
        forms.push(form.replace('\\', "/"));
        forms.push(form.replace('\\', r"\\"));
    }
    // The longest first, so a shorter one doesn't replace part of it.
    forms.sort_by_key(|f| std::cmp::Reverse(f.len()));
    forms.dedup();
    forms
}

fn normalize(bytes: &[u8], spellings: &[String]) -> Vec<u8> {
    match std::str::from_utf8(bytes) {
        Ok(text) => spellings
            .iter()
            .fold(text.to_owned(), |text, form| text.replace(form, ROOT))
            .into_bytes(),
        Err(_) => bytes.to_vec(),
    }
}

/// Every file under `dir`, by its `/`-separated path relative to it.
fn files_under(dir: &Path, prefix: &str, into: &mut BTreeMap<String, PathBuf>) {
    for entry in fs::read_dir(dir).unwrap() {
        let entry = entry.unwrap();
        let name = entry.file_name().to_string_lossy().into_owned();
        let rel = format!("{prefix}{name}");
        if entry.file_type().unwrap().is_dir() {
            files_under(&entry.path(), &format!("{rel}/"), into);
        } else {
            into.insert(rel, entry.path());
        }
    }
}

/// The project's `[project] output-dir`, or the default.
fn output_dir(project: &Path) -> PathBuf {
    let toml = fs::read_to_string(project.join("ascribe.toml")).unwrap();
    let dir = toml
        .lines()
        .find_map(|l| {
            let (key, value) = l.split_once('=')?;
            let value = value.split('#').next()?.trim();
            (key.trim() == "output-dir").then(|| value.trim_matches(['"', '\'']).to_owned())
        })
        .unwrap_or_else(|| ".ascribe/build".to_owned());
    join(project, &dir)
}

/// Everything one copy's commands wrote: each command's exit code, standard
/// output, and standard error, and each output file, by a name that says
/// where it came from.
fn run_all(dir: &Path, projects: &[String]) -> BTreeMap<String, Vec<u8>> {
    let spellings = spellings(dir);
    let mut found = BTreeMap::new();
    let commands: &[&[&str]] = &[
        &["check"],
        &["check", "--format", "json"],
        &["diff", "--base", "HEAD"],
        &["diff", "--base", "HEAD", "--format", "json"],
        &["diff", "--base", "HEAD", "--format", "html"],
        &["drift", "--base", "HEAD"],
        &["drift", "--base", "HEAD", "--format", "json"],
        &["drift", "--base", "HEAD", "--format", "summary"],
        &["build"],
        &["build", "--emit", "site", "--anchors"],
    ];
    for project in projects {
        let at = join(dir, project);
        let outputs = output_dir(&at);
        for args in commands {
            let out = Command::new(env!("CARGO_BIN_EXE_ascribe"))
                .current_dir(&at)
                .args(*args)
                .env("NO_COLOR", "1")
                .output()
                .expect("run ascribe");
            let code = out.status.code();
            let expected = match args[0] {
                "build" if FAILING.contains(&project.as_str()) => 1,
                "check" if FAILING.contains(&project.as_str()) => 1,
                _ => 0,
            };
            assert_eq!(
                code,
                Some(expected),
                "{project}: ascribe {}\n{}",
                args.join(" "),
                String::from_utf8_lossy(&out.stderr)
            );
            let name = format!("{project}: ascribe {}", args.join(" "));
            let mut report = format!("exit {}\n--- stdout\n", code.unwrap()).into_bytes();
            report.extend(normalize(&out.stdout, &spellings));
            report.extend(b"\n--- stderr\n");
            report.extend(normalize(&out.stderr, &spellings));
            found.insert(name.clone(), report);

            if args[0] == "build" && outputs.is_dir() {
                let mut files = BTreeMap::new();
                files_under(&outputs, "", &mut files);
                for (rel, path) in files {
                    if rel == ".lock" {
                        continue;
                    }
                    let bytes = fs::read(path).unwrap();
                    found.insert(format!("{name}: {rel}"), normalize(&bytes, &spellings));
                }
                fs::remove_dir_all(&outputs).unwrap();
            }
        }
    }
    found
}

/// The first lines where `a` and `b` differ, for the failure message.
fn first_difference(a: &[u8], b: &[u8]) -> String {
    let (a, b) = (String::from_utf8_lossy(a), String::from_utf8_lossy(b));
    match a.lines().zip(b.lines()).position(|(x, y)| x != y) {
        Some(line) => format!(
            "line {}:\n  first:  {}\n  second: {}",
            line + 1,
            a.lines().nth(line).unwrap(),
            b.lines().nth(line).unwrap()
        ),
        None => format!(
            "one is longer: {} and {} lines",
            a.lines().count(),
            b.lines().count()
        ),
    }
}

#[test]
fn every_output_is_the_same_twice() {
    let files = tracked_files();
    let projects = projects(&files);
    assert!(
        projects.iter().any(|p| p == "docs") && projects.len() > 2,
        "too few projects: {projects:?}"
    );
    let (first, second) = (copy(&files), copy(&files));
    let a = run_all(first.path(), &projects);
    let b = run_all(second.path(), &projects);

    let mut differences = Vec::new();
    for name in a.keys().chain(b.keys().filter(|k| !a.contains_key(*k))) {
        match (a.get(name), b.get(name)) {
            (Some(x), Some(y)) if x == y => {}
            (Some(x), Some(y)) => differences.push(format!("{name}: {}", first_difference(x, y))),
            (Some(_), None) => differences.push(format!("{name}: only in the first run")),
            (None, _) => differences.push(format!("{name}: only in the second run")),
        }
    }
    assert!(
        differences.is_empty(),
        "{} of {} outputs differ between two runs:\n\n{}",
        differences.len(),
        a.len(),
        differences.join("\n\n")
    );
    // The base really is a different revision: diff found pages that changed.
    let diffs = a
        .iter()
        .filter(|(k, v)| k.ends_with("ascribe diff --base HEAD") && v.len() > 40)
        .count();
    assert!(diffs > 0, "diff found no changes in any project");
}
