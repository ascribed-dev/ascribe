//! Formatting is idempotent and preserves the outline over every input we
//! have: every conformance case input, the example projects, the SPEC and
//! project docs, and the CommonMark examples. Each input is also made ugly
//! in several ways first, so the rules have something to do. The inputs are
//! checked on every core at once.

#![allow(clippy::expect_used, clippy::panic)]

mod support;

use std::path::{Path, PathBuf};

use ascribe_core::{FileId, apply_edits};
use ascribe_fmt::format;
use ascribe_model::ContentModel;
use ascribe_syntax::ParseOptions;
use support::{markdown_files, model_at, options, outline, repo_root, shared_model};

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
            return ascribe_model::load_str(&text, FileId::new(0))
                .unwrap_or_else(|_| shared.clone());
        }
        dir = d.parent();
    }
    shared.clone()
}

fn inputs_under(dir: &Path) -> Vec<PathBuf> {
    markdown_files(dir)
}

/// `f` over every item, on as many threads as there are cores, with the
/// results in the items' order. A panic in one is the test's failure, with
/// its message.
fn parallel_map<T: Sync, R: Send>(items: &[T], f: impl Fn(&T) -> R + Sync) -> Vec<R> {
    let threads = std::thread::available_parallelism().map_or(1, usize::from);
    let per_thread = items.len().div_ceil(threads).max(1);
    std::thread::scope(|s| {
        let handles: Vec<_> = items
            .chunks(per_thread)
            .map(|chunk| s.spawn(|| chunk.iter().map(&f).collect::<Vec<R>>()))
            .collect();
        handles
            .into_iter()
            .flat_map(|handle| {
                handle
                    .join()
                    .unwrap_or_else(|panic| std::panic::resume_unwind(panic))
            })
            .collect()
    })
}

#[test]
fn every_conformance_input() {
    let cases = repo_root().join("tests/conformance/cases");
    let shared = shared_model();
    let inputs = inputs_under(&cases);
    assert!(inputs.len() > 200, "found only {} inputs", inputs.len());
    // How many an input changed by formatting (0 or 1), and how many of its
    // ugly variants did.
    let counts = parallel_map(&inputs, |path| {
        // `files/source-not-utf8` holds a file that isn't UTF-8 on purpose
        // (SPEC §2.1); it has no text to format.
        let source = match std::fs::read_to_string(path) {
            Ok(source) => source,
            Err(e) if e.kind() == std::io::ErrorKind::InvalidData => return (0, 0),
            Err(e) => panic!("{}: {e}", path.display()),
        };
        let model = model_for(path, &cases, &shared);
        let name = path
            .strip_prefix(&cases)
            .unwrap_or(path)
            .display()
            .to_string();
        let changed = usize::from(check(&name, &source, &model) != source);
        let mut ugly_changed = 0;
        for (how, ugly) in uglify(&source) {
            if check(&format!("{name} ({how})"), &ugly, &model) != ugly {
                ugly_changed += 1;
            }
        }
        (changed, ugly_changed)
    });
    let changed: usize = counts.iter().map(|(c, _)| c).sum();
    let ugly_changed: usize = counts.iter().map(|(_, u)| u).sum();
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
    let mut inputs = markdown_files(&root.join("examples"));
    let count = inputs.len();
    inputs.push(root.join("SPEC.md"));
    parallel_map(&inputs, |path| {
        let source = std::fs::read_to_string(path).expect("readable");
        let name = if path.ends_with("SPEC.md") {
            "SPEC.md".to_owned()
        } else {
            path.display().to_string()
        };
        check(&name, &source, &quill);
        if name == "SPEC.md" {
            return;
        }
        for (how, ugly) in uglify(&source) {
            check(&format!("{name} ({how})"), &ugly, &quill);
        }
    });
    assert!(count > 3, "found only {count} files");
}

#[test]
fn the_quill_example_is_already_canonical() {
    let root = repo_root();
    let quill = model_at(&root.join("examples/content-models/quill.toml"));
    parallel_map(&markdown_files(&root.join("examples/quill")), |path| {
        let source = std::fs::read_to_string(path).expect("readable");
        assert_eq!(
            check(&path.display().to_string(), &source, &quill),
            source,
            "{} isn't canonical",
            path.display()
        );
    });
}

#[test]
fn commonmark_examples_with_directives_around_them() {
    let examples = ascribe_commonmark_suite::load_bundled_examples().expect("spec.json loads");
    let model = shared_model();
    parallel_map(&examples, |ex| {
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
    });
}
