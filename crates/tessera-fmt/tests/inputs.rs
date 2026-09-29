//! Formatting is idempotent and preserves the outline over every input we
//! have: every conformance case input, the example projects, the SPEC and
//! project docs, and the CommonMark examples. Each input is also made ugly
//! in several ways first, so the rules have something to do.

#![allow(clippy::expect_used, clippy::panic)]

mod support;

use std::path::{Path, PathBuf};

use support::{markdown_files, model_at, options, outline, repo_root, shared_model};
use tessera_core::{FileId, apply_edits};
use tessera_fmt::format;
use tessera_model::ContentModel;
use tessera_syntax::ParseOptions;

/// Checks one input: formatting applies cleanly, is idempotent, and keeps the
/// outline. Returns the formatted text.
#[track_caller]
fn check(name: &str, source: &str, model: &ContentModel) -> String {
    let options = options(model);
    let edits = format(source, &options, model);
    let once = apply_edits(source, &edits)
        .unwrap_or_else(|e| panic!("{name}: the edits don't apply: {e}\n{edits:?}"));
    let again = format(&once, &options, model);
    assert!(
        again.is_empty(),
        "{name}: not idempotent, a second pass wants {again:?}\nfirst pass:\n{once}"
    );
    assert_same_outline(name, source, &once, &options);
    once
}

#[track_caller]
fn assert_same_outline(name: &str, before: &str, after: &str, options: &ParseOptions) {
    let (a, b) = (outline(before, options), outline(after, options));
    if a != b {
        let first = a
            .lines()
            .zip(b.lines())
            .position(|(x, y)| x != y)
            .unwrap_or(0);
        panic!(
            "{name}: the outline changed at line {first}:\n--- before\n{}\n--- after\n{}\n\ninput:\n{before}\nformatted:\n{after}",
            a.lines()
                .skip(first.saturating_sub(3))
                .take(8)
                .collect::<Vec<_>>()
                .join("\n"),
            b.lines()
                .skip(first.saturating_sub(3))
                .take(8)
                .collect::<Vec<_>>()
                .join("\n"),
        );
    }
}

/// Ways to make a document less canonical, without a parser: they act on
/// lines that look like directive lines and may change what a document means,
/// which is fine, because the check compares a document with its own
/// formatting.
fn uglify(source: &str) -> Vec<(&'static str, String)> {
    let directive = |line: &str| line.trim_start().starts_with('@');
    let map = |f: &dyn Fn(&str) -> String| -> String {
        let mut out = String::new();
        for line in source.split_inclusive('\n') {
            let (body, eol) = match line.strip_suffix('\n') {
                Some(body) => (body, "\n"),
                None => (line, ""),
            };
            if directive(body) {
                out.push_str(&f(body));
            } else {
                out.push_str(body);
            }
            out.push_str(eol);
        }
        out
    };
    vec![
        (
            "spread out",
            map(&|l| {
                l.replacen(": ", " :   ", 1)
                    .replace('{', "   {  ")
                    .replace('=', " = ")
                    .replace(',', "  ,")
                    .replace('|', " | ")
                    .replace('}', "  }")
            }),
        ),
        (
            "squashed",
            map(&|l| l.replace(": ", ":").replace(" {", "{").replace(", ", ",")),
        ),
        ("indented", map(&|l| format!("  {l}"))),
        ("indented by three", map(&|l| format!("   {l}"))),
        ("trailing space", map(&|l| format!("{l}   "))),
        ("blank lines", map(&|l| format!("{l}\n"))),
        (
            "reversed attributes",
            map(&|l| match (l.find('{'), l.find('}')) {
                (Some(open), Some(close)) if open < close => {
                    let mut pairs: Vec<&str> = l[open + 1..close].split(", ").collect();
                    pairs.reverse();
                    format!("{}{{{}}}{}", &l[..open], pairs.join(", "), &l[close + 1..])
                }
                _ => l.to_owned(),
            }),
        ),
        (
            "quoted",
            map(&|l| {
                // Quote each value that looks like a plain word.
                let mut out = String::new();
                let mut rest = l;
                while let Some(eq) = rest.find('=') {
                    let (head, tail) = rest.split_at(eq + 1);
                    let end = tail
                        .find(|c: char| !c.is_alphanumeric() && c != '-' && c != '.')
                        .unwrap_or(tail.len());
                    out.push_str(head);
                    if end > 0 {
                        out.push_str(&format!("\"{}\"", &tail[..end]));
                    }
                    rest = &tail[end..];
                }
                out.push_str(rest);
                out
            }),
        ),
    ]
}

/// The model a case uses: its own `ascribe.toml` (looking up from the input's
/// directory to `cases/`), or the shared model.
fn model_for(input: &Path, cases: &Path, shared: &ContentModel) -> ContentModel {
    let mut dir = input.parent();
    while let Some(d) = dir {
        if !d.starts_with(cases) {
            break;
        }
        let candidate = d.join("ascribe.toml");
        if candidate.is_file() {
            let text = std::fs::read_to_string(&candidate).expect("readable");
            // A case about a broken model has a model that doesn't load.
            return tessera_model::load_str(&text, FileId::new(0))
                .unwrap_or_else(|_| shared.clone());
        }
        dir = d.parent();
    }
    shared.clone()
}

fn inputs_under(dir: &Path) -> Vec<PathBuf> {
    markdown_files(dir)
}

#[test]
fn every_conformance_input() {
    let cases = repo_root().join("tests/conformance/cases");
    let shared = shared_model();
    let inputs = inputs_under(&cases);
    assert!(inputs.len() > 200, "found only {} inputs", inputs.len());
    let mut changed = 0;
    let mut ugly_changed = 0;
    for path in &inputs {
        // `files/source-not-utf8` holds a file that isn't UTF-8 on purpose
        // (SPEC §2.1); it has no text to format.
        let source = match std::fs::read_to_string(path) {
            Ok(source) => source,
            Err(e) if e.kind() == std::io::ErrorKind::InvalidData => continue,
            Err(e) => panic!("{}: {e}", path.display()),
        };
        let model = model_for(path, &cases, &shared);
        let name = path
            .strip_prefix(&cases)
            .unwrap_or(path)
            .display()
            .to_string();
        if check(&name, &source, &model) != source {
            changed += 1;
        }
        for (how, ugly) in uglify(&source) {
            if check(&format!("{name} ({how})"), &ugly, &model) != ugly {
                ugly_changed += 1;
            }
        }
    }
    println!(
        "{} inputs, {changed} changed by formatting, {ugly_changed} ugly variants changed",
        inputs.len()
    );
    // The check has something to check: the rules fire on ugly input.
    assert!(
        ugly_changed > 500,
        "only {ugly_changed} ugly variants changed"
    );
}

#[test]
fn examples_and_docs() {
    let root = repo_root();
    let quill = model_at(&root.join("examples/content-models/quill.toml"));
    let mut count = 0;
    for dir in [root.join("examples"), root.join("project-docs")] {
        for path in markdown_files(&dir) {
            let source = std::fs::read_to_string(&path).expect("readable");
            let name = path.display().to_string();
            check(&name, &source, &quill);
            for (how, ugly) in uglify(&source) {
                check(&format!("{name} ({how})"), &ugly, &quill);
            }
            count += 1;
        }
    }
    let spec = std::fs::read_to_string(root.join("SPEC.md")).expect("SPEC.md");
    check("SPEC.md", &spec, &quill);
    assert!(count > 3, "found only {count} files");
}

#[test]
fn the_quill_example_is_already_canonical() {
    let root = repo_root();
    let quill = model_at(&root.join("examples/content-models/quill.toml"));
    for path in markdown_files(&root.join("examples/quill")) {
        let source = std::fs::read_to_string(&path).expect("readable");
        assert_eq!(
            check(&path.display().to_string(), &source, &quill),
            source,
            "{} isn't canonical",
            path.display()
        );
    }
}

#[test]
fn commonmark_examples_with_directives_around_them() {
    let examples = tessera_commonmark_suite::load_bundled_examples().expect("spec.json loads");
    let model = shared_model();
    for ex in &examples {
        let md = &ex.markdown;
        let name = format!("commonmark example {}", ex.example);
        // Plain CommonMark has nothing to format.
        assert_eq!(check(&name, md, &model), *md, "{name}");
        for source in [
            format!("@note{{type = tip}}:  \n{md}\n@end\n"),
            format!("@id : x\n\n{md}@steps\n\n{md}"),
            format!("> @note :  quoted\n> {}", md.replace('\n', "\n> ")),
            format!("- item\n   @note:Text.\n   {}", md.replace('\n', "\n   ")),
        ] {
            check(&name, &source, &model);
        }
    }
}
