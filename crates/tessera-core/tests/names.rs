//! The names Ascribe puts on a page have one home, `tessera_core::names`.
//!
//! - Each package in `PACKAGES` has a generated `src/names.ts` with the same
//!   constants; run with `ASCRIBE_BLESS=1` to rewrite them after a change.
//! - No source outside the two homes writes one of the names as a literal.
//!   Tests and fixtures may: they're the independent check.
//! - Every `ascribe-` or `data-ascribe-` name a stylesheet selects, or an
//!   Astro template writes, is one of them, since neither can import.

#![allow(clippy::expect_used, clippy::panic)]

use std::fs;
use std::path::{Path, PathBuf};

use tessera_core::names::{ALL, Name};

const BLESS: &str = "ASCRIBE_BLESS=1 cargo test -p tessera-core --test names";

/// The packages that import the names, each from its own `src/names.ts`, so
/// none needs a dependency it doesn't have.
const PACKAGES: &[&str] = &["packages/astro", "packages/vscode"];

/// Source that still writes the names as literals, until it imports them
/// too.
const NOT_YET: &[&str] = &["packages/elements/src/", "packages/review/src/"];

/// Strings that are one of the names by coincidence: the file, and text on
/// the line that holds it.
const COINCIDENCES: &[(&str, &str)] = &[
    // The Astro plugin's own name, which Astro shows in its errors.
    (
        "packages/astro/src/satteri.ts",
        "name: \"ascribe-attributes\"",
    ),
];

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// `path` relative to the repository, with `/` between segments.
fn relative(path: &Path) -> String {
    path.strip_prefix(root())
        .expect("a path in the repository")
        .to_string_lossy()
        .replace('\\', "/")
}

/// The generated TypeScript module.
fn typescript() -> String {
    let mut out = String::from(
        "// The names Ascribe puts on a page: elements, attributes, classes, and ids.\n\
         // Generated from crates/tessera-core/src/names.rs by\n\
         // `ASCRIBE_BLESS=1 cargo test -p tessera-core --test names`. Don't edit it.\n",
    );
    for Name {
        constant,
        value,
        doc,
    } in ALL
    {
        out.push('\n');
        if let [line] = doc {
            out.push_str(&format!("/**{line} */\n"));
        } else {
            out.push_str("/**\n");
            for line in *doc {
                out.push_str(&format!(" *{line}\n"));
            }
            out.push_str(" */\n");
        }
        out.push_str(&format!("export const {constant} = \"{value}\";\n"));
    }
    out
}

#[test]
fn each_package_has_the_generated_names() {
    let expected = typescript();
    for package in PACKAGES {
        let path = root().join(package).join("src/names.ts");
        if std::env::var_os("ASCRIBE_BLESS").is_some() {
            fs::write(&path, &expected).expect("writes names.ts");
            continue;
        }
        let actual = fs::read_to_string(&path).unwrap_or_default();
        assert!(
            actual == expected,
            "{package}/src/names.ts is out of date: run `{BLESS}`"
        );
    }
}

#[test]
fn every_name_is_distinct_and_named_after_its_kind() {
    for (i, name) in ALL.iter().enumerate() {
        assert!(
            ALL[..i].iter().all(|other| other.value != name.value),
            "{} is declared twice",
            name.value
        );
        let kind_fits = match name.constant.split('_').next() {
            Some("DATA") => name.value.starts_with("data-ascribe-"),
            Some("ELEMENT" | "COMMENT" | "CLASS" | "ID") => name.value.starts_with("ascribe-"),
            _ => false,
        };
        assert!(
            kind_fits,
            "{} doesn't say what kind of name {} is",
            name.constant, name.value
        );
    }
}

/// Every file under `dir` with one of `extensions`, skipping dependencies and
/// build output.
fn files(dir: &Path, extensions: &[&str], out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let name = entry.file_name();
        if path.is_dir() {
            if !matches!(name.to_str(), Some("node_modules" | "dist" | "target")) {
                files(&path, extensions, out);
            }
        } else if path
            .extension()
            .and_then(|e| e.to_str())
            .is_some_and(|e| extensions.contains(&e))
        {
            out.push(path);
        }
    }
}

/// Whether `c` can be part of a name, so a match next to it is part of a
/// longer one.
fn word(c: u8) -> bool {
    c.is_ascii_alphanumeric() || c == b'_' || c == b'-'
}

/// Each `ascribe-…` or `data-ascribe-…` name in `line`, whole: not part of
/// a longer word or a custom property (`--ascribe-…`). `<!--ascribe-anchor`
/// counts, since `<!--` isn't part of a name.
fn names_in(line: &str) -> Vec<&str> {
    let bytes = line.as_bytes();
    let mut found = Vec::new();
    let mut from = 0;
    while let Some(at) = line[from..].find("ascribe-").map(|i| i + from) {
        let start = if line[..at].ends_with("data-") {
            at - 5
        } else {
            at
        };
        let mut end = at + "ascribe-".len();
        while end < bytes.len() && word(bytes[end]) {
            end += 1;
        }
        let before = &bytes[..start];
        let starts_word = match before {
            [.., b'<', b'!', b'-', b'-'] => true,
            [.., c] => !word(*c),
            [] => true,
        };
        if starts_word {
            found.push(&line[start..end]);
        }
        from = end;
    }
    found
}

/// The lines of a source file that are code: not comments, and for Rust,
/// not its `#[cfg(test)]` module.
fn code_lines(path: &Path, text: &str) -> Vec<(usize, String)> {
    let rust = path.extension().is_some_and(|e| e == "rs");
    let mut out = Vec::new();
    for (i, line) in text.lines().enumerate() {
        let trimmed = line.trim_start();
        if rust && trimmed.starts_with("#[cfg(test)]") {
            break;
        }
        if ["//", "/*", "* ", "*/"]
            .iter()
            .any(|c| trimmed.starts_with(c))
            || trimmed == "*"
        {
            continue;
        }
        out.push((i + 1, line.to_owned()));
    }
    out
}

#[test]
fn no_source_writes_a_name_outside_its_homes() {
    let root = root();
    let mut sources = Vec::new();
    files(&root.join("crates"), &["rs"], &mut sources);
    // An `.astro` template names an element as a tag, which a constant can't
    // be without changing how Astro renders it; it's checked as a stylesheet
    // is, below.
    files(
        &root.join("packages"),
        &["ts", "js", "mjs", "cjs"],
        &mut sources,
    );
    let mut stray = Vec::new();
    for path in sources {
        let rel = relative(&path);
        let is_source =
            rel.contains("/src/") && !rel.contains("/tests/") && !rel.contains("/test/");
        if !is_source
            || rel == "crates/tessera-core/src/names.rs"
            || rel.ends_with("/src/names.ts")
            || NOT_YET.iter().any(|dir| rel.starts_with(dir))
            // Generated from @ascribed/review and @ascribed/elements.
            || rel.starts_with("crates/tessera-diff/src/html/report.")
        {
            continue;
        }
        let text = fs::read_to_string(&path).expect("reads a source file");
        for (number, line) in code_lines(&path, &text) {
            if COINCIDENCES
                .iter()
                .any(|(file, text)| rel == *file && line.contains(text))
            {
                continue;
            }
            for name in names_in(&line) {
                if let Some(known) = ALL.iter().find(|n| n.value == name) {
                    stray.push(format!(
                        "{rel}:{number}: `{name}` is `names::{}`",
                        known.constant
                    ));
                }
            }
        }
    }
    assert!(
        stray.is_empty(),
        "use the constant, from tessera_core::names in Rust or ./names.js in TypeScript:\n{}",
        stray.join("\n")
    );
}

#[test]
fn every_name_a_stylesheet_or_template_uses_is_declared() {
    let root = root();
    let mut sheets = Vec::new();
    files(&root.join("packages"), &["css", "astro"], &mut sheets);
    files(&root.join("site/src"), &["css", "astro"], &mut sheets);
    assert!(sheets.len() > 5, "found the stylesheets");
    let mut unknown = Vec::new();
    for path in sheets {
        let rel = relative(&path);
        let text = fs::read_to_string(&path).expect("reads a stylesheet");
        for (i, line) in text.lines().enumerate() {
            for name in names_in(line) {
                if !ALL.iter().any(|n| n.value == name) {
                    unknown.push(format!("{rel}:{}: `{name}`", i + 1));
                }
            }
        }
    }
    assert!(
        unknown.is_empty(),
        "these aren't names Ascribe puts on a page; add them to crates/tessera-core/src/names.rs, or fix the selector:\n{}",
        unknown.join("\n")
    );
}
