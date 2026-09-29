//! Recognition checks over the three corpora, unconverted.
//!
//! Nothing in the corpora is meant as Ascribe, so every recognition is either
//! a false positive or a consequence of the specification. Three checks:
//!
//! 1. every class of recognition has a recorded verdict in `recognition.toml`
//!    (a class with none is an unexplained finding and fails);
//! 2. the counts of each class equal `baselines/recognition-<corpus>.json`, so
//!    a change in the parser or a bumped commit is noticed (set
//!    `ASCRIBE_CORPORA_BLESS=1` to rewrite the baseline after reviewing);
//! 3. an independent scan of the raw lines finds every prose line starting
//!    with `@` or `.` outside code, and the parser recognizes none of them as
//!    a directive or a title.
//!
//! A test skips, with a message, when its corpus can't be fetched.

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::panic,
    clippy::print_stderr
)]

use std::collections::BTreeMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use tessera_corpora::corpus::{self, Corpus};
use tessera_corpora::recognize::{Report, recognize};

#[derive(Deserialize)]
struct Verdicts {
    class: BTreeMap<String, Verdict>,
}

#[derive(Deserialize)]
struct Verdict {
    verdict: String,
    why: String,
    finding: Option<String>,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
struct Baseline {
    files: usize,
    classes: BTreeMap<String, usize>,
}

fn dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn run(corpus: Corpus) {
    let Some(fetched) = corpus::corpus_or_skip(corpus) else {
        return;
    };
    let pages = corpus::pages(&fetched).expect("reads the corpus");
    let report = recognize(&pages);
    assert!(
        report.skipped.is_empty(),
        "pages with no valid path: {:?}",
        report.skipped
    );
    assert_eq!(report.files, pages.len());

    // 1. Every class has a verdict.
    let verdicts: Verdicts =
        toml::from_str(&std::fs::read_to_string(dir().join("recognition.toml")).expect("verdicts"))
            .expect("recognition.toml parses");
    let findings_doc = std::fs::read_to_string(dir().join("FINDINGS.md")).expect("FINDINGS.md");
    let counts = report.counts();
    let mut unexplained = Vec::new();
    for class in counts.keys() {
        match verdicts.class.get(class) {
            None => unexplained.push(class.clone()),
            Some(v) => {
                assert!(
                    !v.why.trim().is_empty(),
                    "{class}: a verdict needs a reason"
                );
                match v.verdict.as_str() {
                    "intended" => {}
                    "false-positive" => {
                        let id = v.finding.as_deref().unwrap_or("");
                        assert!(
                            !id.is_empty() && findings_doc.contains(id),
                            "{class}: a false positive must name a finding in FINDINGS.md"
                        );
                    }
                    other => panic!("{class}: unknown verdict {other}"),
                }
            }
        }
    }
    assert!(
        unexplained.is_empty(),
        "recognized with no verdict in tests/corpora/recognition.toml (a false positive, or intended by the spec? \
         give a minimal reproduction in FINDINGS.md): {unexplained:?}\n{}",
        unexplained
            .iter()
            .flat_map(|c| report.of_class(c).take(3))
            .map(|f| format!("  {}:{}:{}  {}", f.file, f.line, f.column, f.excerpt))
            .collect::<Vec<_>>()
            .join("\n")
    );

    // 2. The counts are the recorded ones.
    let current = Baseline {
        files: report.files,
        classes: counts,
    };
    let path = dir()
        .join("baselines")
        .join(format!("recognition-{corpus}.json"));
    if std::env::var_os("ASCRIBE_CORPORA_BLESS").is_some() {
        let mut text = serde_json::to_string_pretty(&current).expect("serializes");
        text.push('\n');
        std::fs::write(&path, text).expect("writes the baseline");
    } else {
        let recorded: Baseline = serde_json::from_str(
            &std::fs::read_to_string(&path)
                .expect("a baseline (ASCRIBE_CORPORA_BLESS=1 makes one)"),
        )
        .expect("the baseline parses");
        assert_eq!(
            current, recorded,
            "{corpus}: recognition changed since the recorded baseline; review the difference, \
             then ASCRIBE_CORPORA_BLESS=1 cargo test -p tessera-corpora --test recognition"
        );
    }

    // 3. Prose `@` and `.` lines, found without the parser.
    oracle(&pages, &report);
}

/// Lines starting with `@word` or `.word` outside fenced code, by a scan that
/// knows nothing about Ascribe.
fn oracle(pages: &[(String, String)], report: &Report) {
    let (mut at_lines, mut dot_lines, mut shaped) = (0, 0, 0);
    for (file, text) in pages {
        let mut fence: Option<(char, usize)> = None;
        let mut in_front = text.starts_with("---\n");
        for (n, line) in text.lines().enumerate() {
            let line_no = n as u32 + 1;
            if in_front {
                in_front = n == 0 || line != "---";
                continue;
            }
            let t = line.trim_start();
            let lead = line.len() - t.len();
            let ticks = t.chars().take_while(|c| *c == '`' || *c == '~').count();
            if let Some((c, len)) = fence {
                if ticks >= len && t.starts_with(c) && t.trim_matches(c).is_empty() {
                    fence = None;
                }
                continue;
            }
            if ticks >= 3 {
                fence = t.chars().next().map(|c| (c, ticks));
                continue;
            }
            if lead > 3 {
                continue;
            }
            let on_line = |class: &str| {
                report
                    .of_class(class)
                    .any(|f| f.file == *file && f.line == line_no)
            };
            if let Some(rest) = t.strip_prefix('@')
                && rest.starts_with(|c: char| c.is_ascii_alphabetic())
            {
                at_lines += 1;
                let name_end = rest
                    .find(|c: char| !(c.is_ascii_alphanumeric() || c == '-'))
                    .unwrap_or(rest.len());
                let after = rest[name_end..].trim_start_matches([' ', '\t']);
                let known = [
                    "id",
                    "include",
                    "variant",
                    "available",
                    "note",
                    "steps",
                    "details",
                    "end",
                ]
                .contains(&&rest[..name_end]);
                let directive_shaped = name_end == rest.len()
                    || rest[name_end..].starts_with([':', '{'])
                    || (after.is_empty());
                if known {
                    continue;
                }
                assert!(
                    !report.findings.iter().any(|f| f.file == *file
                        && f.line == line_no
                        && f.class.starts_with("directive:")),
                    "{file}:{line_no}: prose `@` recognized as a directive: {line}"
                );
                if directive_shaped {
                    shaped += 1;
                    assert!(
                        on_line("diagnostic:directive-unknown"),
                        "{file}:{line_no}: a directive-shaped line with an unknown name must warn: {line}"
                    );
                }
            } else if t.starts_with('.')
                && t[1..].starts_with(|c: char| !c.is_whitespace() && c != '.')
            {
                dot_lines += 1;
                assert!(
                    !on_line("title"),
                    "{file}:{line_no}: prose `.` recognized as a title: {line}"
                );
            }
        }
    }
    eprintln!(
        "  prose lines starting with `@`: {at_lines} ({shaped} directive-shaped); starting with `.`: {dot_lines}; none recognized"
    );
}

#[test]
fn astro() {
    run(Corpus::Astro);
}

#[test]
fn elastic() {
    run(Corpus::Elastic);
}

#[test]
fn docker() {
    run(Corpus::Docker);
}
