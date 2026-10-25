//! Reads every spec in a folder with the OpenAPI reader, as `[apis.<name>]`
//! reads a declared one, and says what it couldn't read: the corpus run of
//! OpenAPI's phase 1, kept out of the test suite since the corpus is
//! downloaded (`scripts/openapi-corpus/sample.ts`).
//!
//! cargo run --release -p ascribe-resolve --example openapi_corpus -- <folder>

#![allow(clippy::print_stdout, clippy::print_stderr, clippy::expect_used)]

use std::path::PathBuf;
use std::process::ExitCode;
use std::time::{Duration, Instant};

use ascribe_resolve::openapi::read_spec;

fn main() -> ExitCode {
    let Some(folder) = std::env::args_os().nth(1).map(PathBuf::from) else {
        eprintln!("usage: openapi_corpus <folder>");
        return ExitCode::from(2);
    };
    let mut files: Vec<PathBuf> = std::fs::read_dir(&folder)
        .expect("the folder")
        .filter_map(|entry| Some(entry.ok()?.path()))
        .filter(|path| {
            path.extension()
                .is_some_and(|e| e == "json" || e == "yaml" || e == "yml")
        })
        .collect();
    files.sort();
    let (mut specs, mut operations, mut without_id, mut unread_parts) = (0, 0, 0, 0);
    let (mut every, mut none, mut some) = (0, 0, 0);
    let mut failed = Vec::new();
    let mut slowest: Option<(Duration, PathBuf, usize)> = None;
    for path in &files {
        let text = std::fs::read_to_string(path).expect("a spec");
        let json = path.extension().is_some_and(|e| e == "json");
        let start = Instant::now();
        let read = read_spec(&text, json);
        let took = start.elapsed();
        let spec = match read {
            Ok(spec) => spec,
            Err(why) => {
                failed.push(format!("{}: {why}", path.display()));
                continue;
            }
        };
        specs += 1;
        let ids = spec.ids().count();
        operations += spec.operations.len();
        without_id += spec.operations.len() - ids;
        unread_parts += spec.unread.len();
        for part in &spec.unread {
            println!("unread {}: {part}", path.display());
        }
        match (ids, spec.operations.len()) {
            (_, 0) => println!("no operations {}", path.display()),
            (i, n) if i == n => every += 1,
            (0, _) => none += 1,
            _ => some += 1,
        }
        if slowest.as_ref().is_none_or(|(t, _, _)| took > *t) {
            slowest = Some((took, path.clone(), text.len()));
        }
    }
    for failure in &failed {
        println!("failed {failure}");
    }
    println!(
        "{} files: {specs} read, {} not; {operations} operations, {without_id} with no operationId; \
         ids on every operation in {every} specs, on none in {none}, on some in {some}; \
         {unread_parts} parts of `paths` not read",
        files.len(),
        failed.len()
    );
    if let Some((took, path, bytes)) = slowest {
        println!(
            "slowest: {} ({} KB) in {} ms",
            path.display(),
            bytes / 1024,
            took.as_millis()
        );
    }
    if failed.is_empty() {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}
