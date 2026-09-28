//! The file-level checks, on small projects built in memory (and, where the
//! file system matters, in a temporary directory).

#![allow(clippy::expect_used, clippy::panic)]

use std::fs;
use std::path::PathBuf;

use tessera_check::{Diagnostic, Project, Severity, check_files};
use tessera_core::{FileId, RelPath, apply_edits};

const MODEL: &str = r#"
spec = "0.1"

[project]
content-root = "docs"
output-dir = ".out"

[types.page]
default = true

[types.page.frontmatter]
title = "string"

[dimensions.pm]
values = ["npm", "pnpm"]

[dimensions.deployment]
values = ["cloud", "self-managed"]
versionless = ["cloud"]

[phrases]
product = "Quill"
api = "https://api.quill.dev/"
"#;

fn model() -> tessera_model::ContentModel {
    tessera_model::load_str(MODEL, FileId::new(0)).expect("the model loads")
}

/// A project of in-memory pages under `docs/`, whose root is `root`.
fn project_in(root: PathBuf, files: &[(&str, &str)]) -> Project {
    let sources = Project::from_sources(files.iter().map(|(path, text)| {
        (
            RelPath::parse(path).expect("a relative path"),
            (*text).to_owned(),
        )
    }));
    Project::from_parts(
        root,
        RelPath::parse("docs").expect("a relative path"),
        model(),
        MODEL.to_owned(),
        sources,
    )
}

fn project(files: &[(&str, &str)]) -> Project {
    project_in(PathBuf::from("/nonexistent-tessera-project"), files)
}

fn page(body: &str) -> String {
    format!("---\ntitle: T\n---\n\n{body}\n")
}

/// `(slug, line)` of every diagnostic, in the order reported.
fn found(project: &Project) -> Vec<(String, usize)> {
    check_files(project)
        .iter()
        .map(|d| {
            let file = project.file(d.location.file).expect("a known file");
            let line = file.text[..d.location.span.start()].matches('\n').count() + 1;
            (d.slug.to_string(), line)
        })
        .collect()
}

fn slugs(project: &Project) -> Vec<String> {
    found(project).into_iter().map(|(s, _)| s).collect()
}

fn one(project: &Project) -> Diagnostic {
    let mut all = check_files(project);
    assert_eq!(all.len(), 1, "{all:#?}");
    all.remove(0)
}

#[test]
fn a_clean_page_has_no_diagnostics() {
    let p = project(&[("index.md", &page("Hello {product}."))]);
    assert!(check_files(&p).is_empty());
}

#[test]
fn parser_issues_are_reported_once_with_registry_data() {
    let p = project(&[("index.md", &page("@note {type=tip}:\nNever closed."))]);
    let d = one(&p);
    assert_eq!(d.slug.as_str(), "container-unclosed");
    assert_eq!(d.code, "TSR007");
    assert_eq!(d.severity, Severity::Error);
    assert!(d.message.contains("`@note`"), "{}", d.message);
}

#[test]
fn diagnostics_come_back_in_file_then_source_order() {
    let p = project(&[
        ("b.md", &page("[x](gone.md)")),
        ("a.md", &page("[x](gone.md)\n\n[y](gone2.md)")),
    ]);
    let files: Vec<_> = check_files(&p)
        .iter()
        .map(|d| p.file(d.location.file).expect("a file").display_path)
        .collect();
    assert_eq!(files, ["docs/a.md", "docs/a.md", "docs/b.md"]);
}

#[test]
fn variant_dimensions_and_values() {
    let p = project(&[(
        "index.md",
        &page("@variant {pm=npm|pip, colour=red}:\nx\n@end"),
    )]);
    assert_eq!(slugs(&p), ["variant-unknown", "variant-unknown"]);
    let messages: Vec<_> = check_files(&p).into_iter().map(|d| d.message).collect();
    assert!(
        messages[0].contains("`pip` isn't a value of `pm`"),
        "{messages:?}"
    );
    assert!(
        messages[1].contains("`colour` isn't a declared dimension"),
        "{messages:?}"
    );
}

#[test]
fn available_specs_are_checked_against_the_model() {
    let cases = [
        ("cloud, self-managed preview 3.4", vec![]),
        ("deployment beta", vec![]),
        ("deployment 3.4", vec!["available-versionless"]),
        ("cloud 3.3", vec!["available-versionless"]),
        ("edge", vec!["available-unknown"]),
        ("cloud (preview", vec!["available-syntax"]),
        (
            "self-managed (ga 3.5, preview 3.3)",
            vec!["available-history-order"],
        ),
        ("self-managed frozen 3.3", vec!["available-unknown"]),
    ];
    for (spec, expected) in cases {
        let p = project(&[(
            "index.md",
            &page(&format!("## S\n@available: {spec}\n\nText.")),
        )]);
        assert_eq!(slugs(&p), expected, "{spec}");
    }
}

#[test]
fn the_dimension_message_suggests_a_value() {
    let p = project(&[(
        "index.md",
        &page("## S\n@available: deployment 3.4\n\nText."),
    )]);
    let d = one(&p);
    assert!(
        d.message.contains("`deployment` is a dimension name"),
        "{}",
        d.message
    );
    assert!(d.message.contains("`self-managed`"), "{}", d.message);
}

#[test]
fn availability_in_frontmatter_is_located_at_the_spec() {
    let text = "---\ntitle: T\navailable: cloud 3.3\n---\n\nText.\n";
    let p = project(&[("index.md", text)]);
    let d = one(&p);
    assert_eq!(d.slug.as_str(), "available-versionless");
    assert_eq!(&text[d.location.span.range()], "3.3");

    let quoted = "---\ntitle: T\navailable: \"cloud 3.3\"\n---\n\nText.\n";
    let p = project(&[("index.md", quoted)]);
    assert_eq!(&quoted[one(&p).location.span.range()], "3.3");
}

#[test]
fn variant_frontmatter_names_and_values() {
    let text = "---\ntitle: T\nvariant:\n  pm: [npm, yarn]\n  colour: red\n  deployment: 3\n---\n\nText.\n";
    let p = project(&[("index.md", text)]);
    assert_eq!(
        found(&p),
        [
            ("variant-unknown".to_owned(), 4),
            ("variant-unknown".to_owned(), 5),
            ("frontmatter-type-mismatch".to_owned(), 6),
        ]
    );
}

#[test]
fn frontmatter_problems_are_reported_at_the_key() {
    let text = "---\ntitle: 3.10\nowner: x\n---\n\nText.\n";
    let p = project(&[("index.md", text)]);
    let all = check_files(&p);
    assert_eq!(all.len(), 2);
    assert_eq!(all[0].slug.as_str(), "frontmatter-type-mismatch");
    assert_eq!(
        all[0].message,
        "`title` must be a string, but YAML reads 3.10 as a number; quote it: title: \"3.10\""
    );
    assert_eq!(&text[all[0].location.span.range()], "3.10");
    assert_eq!(all[1].slug.as_str(), "frontmatter-unknown-key");
    assert_eq!(&text[all[1].location.span.range()], "owner");
}

#[test]
fn a_missing_field_is_reported_on_the_first_line() {
    let p = project(&[("index.md", "Just text.\n")]);
    assert_eq!(found(&p), [("frontmatter-missing-field".to_owned(), 1)]);
}

#[test]
fn frontmatter_that_is_not_yaml_is_a_type_mismatch() {
    // SPEC-QUESTION(Q51): no registry entry covers invalid YAML.
    let p = project(&[("index.md", "---\ntitle: [oops\n---\n\nText.\n")]);
    let d = one(&p);
    assert_eq!(d.slug.as_str(), "frontmatter-type-mismatch");
    assert!(d.message.contains("invalid YAML"), "{}", d.message);
}

#[test]
fn fragments_use_the_fragment_schema() {
    let p = project(&[
        ("_f.md", "---\ntitle: no\navailable: cloud\n---\n\nText.\n"),
        ("g/_h.md", "No frontmatter needed.\n"),
    ]);
    assert_eq!(
        slugs(&p),
        [
            "frontmatter-unknown-key",
            "frontmatter-reserved-in-fragment"
        ]
    );
}

#[test]
fn undeclared_phrases_and_headings_with_phrases() {
    let p = project(&[(
        "index.md",
        &page(
            "{nope} and \\{escaped} and `{code}`.\n\n## About {product}\n\n## Stable {product}\n@id: stable",
        ),
    )]);
    assert_eq!(
        slugs(&p),
        ["phrase-undeclared", "heading-phrase-without-id"]
    );
    assert!(
        check_files(&p)
            .iter()
            .all(|d| d.severity == Severity::Warning)
    );
}

#[test]
fn ids_use_letters_digits_and_hyphens() {
    let p = project(&[("index.md", &page("## A\n@id: ok-1\n\n## B\n@id: not_ok"))]);
    assert_eq!(slugs(&p), ["id-invalid"]);
}

#[test]
fn attribute_types_and_required_attributes() {
    let p = project(&[(
        "index.md",
        &page("@include {heading=maybe}: _x.md\n\n@note {type=hint}: t\n\n@steps {n=1}\n1. a"),
    )]);
    let mut got = slugs(&p);
    got.sort();
    assert_eq!(
        got,
        [
            "attribute-type-mismatch",
            "attribute-type-mismatch",
            "attribute-unknown-key",
            "include-target-missing"
        ]
    );
}

#[test]
fn links_files_pages_and_fragments() {
    let p = project(&[
        (
            "index.md",
            &page(
                "[a](keys.md) [b](keys.md#x) [c](#top) [d](https://x.dev/y) [e](nope.md) [f](_f.md) [g](mailto:a@b.c)",
            ),
        ),
        ("keys.md", &page("Text.")),
        ("_f.md", "Text.\n"),
    ]);
    assert_eq!(slugs(&p), ["link-target-missing", "link-to-fragment"]);
}

#[test]
fn a_route_gets_link_route_and_a_fix_that_applies() {
    let p = project(&[
        (
            "guides/index.md",
            &page("[x](/keys/#rotate) [y](../keys) [z](/nowhere/)"),
        ),
        ("keys.md", &page("Text.")),
    ]);
    let all = check_files(&p);
    assert_eq!(slugs(&p), ["link-route", "link-route", "link-route"]);
    let file = p
        .source_at(&RelPath::parse("guides/index.md").expect("path"))
        .expect("a source");
    let fix = &all[0].fixes[0];
    assert_eq!(
        apply_edits(&file.text, &fix.edits).expect("edits apply"),
        page("[x](/keys.md#rotate) [y](../keys) [z](/nowhere/)")
    );
    let relative = &all[1].fixes[0].edits[0].new_text;
    assert_eq!(relative, "../keys.md");
    // No page exists for `/nowhere/`, so there's nothing to offer.
    assert!(all[2].fixes.is_empty());
    assert!(
        all[2].message.contains("`nowhere.md`"),
        "{}",
        all[2].message
    );
}

#[test]
fn phrases_in_destinations_are_substituted_before_checking() {
    let p = project(&[(
        "index.md",
        &page("[ref]({api}streaming) [ref2]({nope}x.md)\n\n[def]: {api}y"),
    )]);
    assert_eq!(slugs(&p), ["link-target-missing"]);
}

#[test]
fn images_alt_source_and_attributes() {
    let p = project(&[(
        "index.md",
        &page("![](gone.png)\n\n![A](gone.png){width=1}\n\n![B](https://x.dev/a.png)"),
    )]);
    assert_eq!(
        slugs(&p),
        [
            "image-alt-missing",
            "image-source-missing",
            "image-source-missing",
            "attribute-unknown-key",
        ]
    );
}

#[test]
fn files_are_found_with_exact_names_inside_the_boundary() {
    let dir = tempfile::tempdir().expect("a temporary directory");
    let root = dir.path();
    fs::create_dir_all(root.join("docs/img")).expect("mkdir");
    fs::create_dir_all(root.join(".out/site")).expect("mkdir");
    fs::create_dir_all(root.join("shared")).expect("mkdir");
    for f in [
        "docs/img/Logo.png",
        "docs/Guide.md.bak",
        ".out/site/p.png",
        "shared/s.png",
    ] {
        fs::write(root.join(f), "x").expect("write");
    }
    let outside = root.parent().expect("a parent").join("tessera-outside.png");
    let text = page(
        "![a](img/Logo.png) ![b](img/logo.png) ![c](../.out/site/p.png) ![d](../shared/s.png) \
         ![e](../../tessera-outside.png) ![f](img)",
    );
    let p = project_in(root.to_owned(), &[("index.md", &text)]);
    let all = check_files(&p);
    let messages: Vec<&str> = all.iter().map(|d| d.message.as_str()).collect();
    assert_eq!(all.len(), 4, "{messages:#?}");
    assert!(
        messages[0].contains("`img/logo.png` doesn't exist; `img/Logo.png` differs only in case")
    );
    assert!(messages[1].contains("`../.out/site/p.png` is outside the project"));
    assert!(messages[2].contains("`../../tessera-outside.png` is outside the project"));
    assert!(
        messages[3].contains("`img` doesn't exist"),
        "a directory isn't a file: {}",
        messages[3]
    );
    drop(outside);
}

#[test]
fn case_differences_in_a_directory_name_are_found_too() {
    let dir = tempfile::tempdir().expect("a temporary directory");
    fs::create_dir_all(dir.path().join("docs/Img")).expect("mkdir");
    fs::write(dir.path().join("docs/Img/a.png"), "x").expect("write");
    let p = project_in(
        dir.path().to_owned(),
        &[("index.md", &page("![a](img/a.png)"))],
    );
    let d = one(&p);
    assert!(
        d.message.contains("`Img/a.png` differs only in case"),
        "{}",
        d.message
    );
}

#[test]
fn includes_resolve_beside_the_including_file() {
    let p = project(&[
        (
            "guides/page.md",
            &page("@include: _f.md\n\n@include: /_f.md#x\n\n@include: ../_f.md"),
        ),
        ("_f.md", "Text.\n"),
    ]);
    // Beside the page there is no `_f.md`; the root-relative and `..` paths find it.
    assert_eq!(found(&p), [("include-target-missing".to_owned(), 5)]);
}

#[test]
fn model_warnings_are_part_of_the_list() {
    let text = MODEL.replace(
        "[phrases]",
        "[dimensions.mixed]\nvalues = [\"Nightly\", \"nightly\"]\n\n[phrases]",
    );
    let model = tessera_model::load_str(&text, FileId::new(0)).expect("loads with a warning");
    assert!(!model.warnings.is_empty());
    let p = Project::from_parts(
        PathBuf::from("/x"),
        RelPath::parse("docs").expect("path"),
        model,
        text,
        Vec::new(),
    );
    let all = check_files(&p);
    assert!(!all.is_empty());
    assert!(
        all.iter()
            .all(|d| d.location.file == FileId::new(0) && d.severity == Severity::Warning)
    );
}

mod robustness {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(400))]

        /// Nor does frontmatter, whatever is in it.
        #[test]
        fn frontmatter_never_panics(body in proptest::collection::vec(
            prop_oneof![
                Just("title: "), Just("a:"), Just("  - "), Just("["), Just("]"), Just("{"),
                Just("}"), Just("\""), Just("'"), Just("é"), Just("\n"), Just(": "),
                Just("available: "), Just("variant:\n  "), Just("cloud"), Just("3.10"),
                Just("pm: "), Just("|\n  "), Just("&x "), Just("*x"), Just("# c"), Just("\t"),
            ],
            0..40,
        ).prop_map(|parts| parts.concat()), fragment in any::<bool>()) {
            let text = format!("---\n{body}\n---\n\nText.\n");
            let path = if fragment { "_f.md" } else { "index.md" };
            let p = project(&[(path, &text)]);
            for d in check_files(&p) {
                let file = p.file(d.location.file).expect("a known file");
                prop_assert!(d.location.span.end() <= file.text.len());
                prop_assert!(file.text.is_char_boundary(d.location.span.start()));
                prop_assert!(file.text.is_char_boundary(d.location.span.end()));
            }
        }

        /// No input makes the checks panic, and every location is a valid range.
        #[test]
        fn never_panics(text in proptest::collection::vec(
            prop_oneof![
                Just("@"), Just("{"), Just("}"), Just("["), Just("]"), Just("("), Just(")"),
                Just("!"), Just(":"), Just("."), Just("#"), Just("\n"), Just(" "), Just("---\n"),
                Just("@note"), Just("@variant"), Just("@available"), Just("@include"), Just("@id"),
                Just("cloud"), Just("x.md"), Just("{product}"), Just("=") , Just("\\"), Just("\""),
                Just("title: "), Just("é"),
            ],
            0..60,
        ).prop_map(|parts| parts.concat())) {
            let p = project(&[("index.md", &text)]);
            for d in check_files(&p) {
                let file = p.file(d.location.file).expect("a known file");
                prop_assert!(d.location.span.end() <= file.text.len());
                prop_assert!(file.text.is_char_boundary(d.location.span.start()));
                prop_assert!(file.text.is_char_boundary(d.location.span.end()));
            }
        }
    }
}
