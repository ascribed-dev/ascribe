//! `corpora`: fetch, recognize, convert, and compare, by hand.

#![allow(clippy::print_stdout)]

use std::process::ExitCode;

use tessera_corpora::corpus::{self, Corpus};
use tessera_corpora::recognize::recognize;
use tessera_corpora::{convert, perf};

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("fetch") => {
            for c in Corpus::ALL {
                match corpus::fetch(c) {
                    Ok(f) => println!("{c}: {}", f.root.display()),
                    Err(e) => {
                        println!("{c}: {e}");
                        return ExitCode::FAILURE;
                    }
                }
            }
            ExitCode::SUCCESS
        }
        Some("recognize") => {
            let Some(name) = args.get(1).and_then(|n| Corpus::from_name(n)) else {
                println!("usage: corpora recognize <astro|elastic|docker> [class]");
                return ExitCode::FAILURE;
            };
            let Ok(fetched) = corpus::fetch(name) else {
                println!("can't fetch");
                return ExitCode::FAILURE;
            };
            let pages = corpus::pages(&fetched).unwrap_or_default();
            let report = recognize(&pages);
            println!("{} files, {} bytes", report.files, report.bytes);
            for (class, n) in report.counts() {
                println!("{n:>7}  {class}");
            }
            if args.get(2).map(String::as_str) == Some("--keys") {
                let mut keys: std::collections::BTreeMap<String, usize> = Default::default();
                for f in report
                    .findings
                    .iter()
                    .filter(|f| f.class.starts_with("phrase-candidate:prose"))
                {
                    *keys.entry(format!("{} {}", f.class, f.detail)).or_default() += 1;
                }
                let mut keys: Vec<_> = keys.into_iter().collect();
                keys.sort_by_key(|(_, n)| std::cmp::Reverse(*n));
                for (k, n) in keys
                    .into_iter()
                    .take(args.get(3).and_then(|n| n.parse().ok()).unwrap_or(40))
                {
                    println!("{n:>6} {k}");
                }
            } else if let Some(class) = args.get(2) {
                for f in report.of_class(class).take(40) {
                    println!("{}:{}:{}  {}", f.file, f.line, f.column, f.excerpt);
                }
            }
            ExitCode::SUCCESS
        }
        Some("convert") => {
            let Some(name) = args.get(1).and_then(|n| Corpus::from_name(n)) else {
                println!("usage: corpora convert <astro|elastic|docker> [--show <path>]");
                return ExitCode::FAILURE;
            };
            let Ok(fetched) = corpus::fetch(name) else {
                println!("can't fetch");
                return ExitCode::FAILURE;
            };
            let pages = corpus::pages(&fetched).unwrap_or_default();
            let extra = convert::Extra {
                docset: std::fs::read_to_string(fetched.root.join("docset.yml"))
                    .unwrap_or_default(),
            };
            let converted = convert::convert(name, &pages, &extra);
            if let Some(path) = args
                .get(3)
                .filter(|_| args.get(2).map(String::as_str) == Some("--show"))
            {
                println!(
                    "{}",
                    converted
                        .files
                        .get(path)
                        .map_or("(no such page)", String::as_str)
                );
                return ExitCode::SUCCESS;
            }
            let dir = corpus::cache_dir().join("converted").join(name.name());
            let _ = std::fs::remove_dir_all(&dir);
            let _ = std::fs::create_dir_all(&dir);
            if let Err(e) = converted.write_to(&dir) {
                println!("can't write: {e}");
                return ExitCode::FAILURE;
            }
            println!(
                "{} files, {} assets -> {}",
                converted.pages(),
                converted.assets.len(),
                dir.display()
            );
            for (k, v) in &converted.stats {
                println!("{v:>7}  {k}");
            }
            ExitCode::SUCCESS
        }
        Some("compare") => compare(&args[1..]),
        _ => {
            println!("usage: corpora <fetch|recognize|convert|compare>");
            ExitCode::FAILURE
        }
    }
}

/// `corpora compare <results.jsonl> [--baseline FILE] [--record]`: checks
/// benchmark results against the recorded baselines (see `perf`).
fn compare(args: &[String]) -> ExitCode {
    let Some(results) = args.first() else {
        println!("usage: corpora compare <results.jsonl> [--baseline FILE] [--record]");
        return ExitCode::FAILURE;
    };
    let baseline = args
        .iter()
        .position(|a| a == "--baseline")
        .and_then(|i| args.get(i + 1))
        .map_or_else(perf::default_baseline, std::path::PathBuf::from);
    let record = args.iter().any(|a| a == "--record");
    match perf::compare(std::path::Path::new(results), &baseline, record) {
        Ok(report) => {
            print!("{}", report.text);
            if report.ok {
                ExitCode::SUCCESS
            } else {
                ExitCode::FAILURE
            }
        }
        Err(e) => {
            println!("error: {e}");
            ExitCode::FAILURE
        }
    }
}
