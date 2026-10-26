//! The file-level checks, on small projects built in memory (and, where the
//! file system matters, in a temporary directory).

#![allow(clippy::expect_used, clippy::panic)]

use std::fs;
use std::path::PathBuf;

use ascribe_check::{Diagnostic, Project, Severity, check_files};
use ascribe_core::{Applicability, FileId, RelPath, apply_edits};

const MODEL: &str = r#"
spec = "0.1"

[project]
content-root = "docs"
output-dir = ".out"

[dimensions.pm]
values = ["npm", "pnpm"]

[dimensions.deployment]
values = ["cloud", "self-managed"]
versionless = ["cloud"]

[releases.self-managed]
live = ["3.4", "3.5"]
planned = ["3.6"]

[releases.self-managed.archived]
"3.2" = {}

[phrases]
product = "Quill"
api = "https://api.quill.dev/"

# The content checks across the project are tested in across.rs.
[checks]
page-orphan = "off"
fragment-unused = "off"
phrase-unused = "off"
feature-unused = "off"
glossary-term-unused = "off"
image-unused = "off"
image-large = "off"
title-duplicate = "off"
"#;

fn model() -> ascribe_model::ContentModel {
    ascribe_model::load_str(MODEL, FileId::new(0)).expect("the model loads")
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
    project_in(PathBuf::from("/nonexistent-ascribe-project"), files)
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
    assert_eq!(d.code, "ASC007");
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
        ("cloud, self-managed preview 3.5", vec![]),
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
fn a_rows_available_is_checked_like_a_directives() {
    let cases = [
        ("{available=\"cloud, self-managed preview 3.5\"}", vec![]),
        ("{available=cloud}", vec![]),
        ("{available=edge}", vec!["available-unknown"]),
        ("{available=\"cloud 3.3\"}", vec!["available-versionless"]),
        ("{available=\"cloud (preview\"}", vec!["available-syntax"]),
        ("{since=3.4}", vec!["attribute-unknown-key"]),
    ];
    for (block, expected) in cases {
        let p = project(&[("index.md", &page(&format!("| a |\n|---|\n| x {block} |")))]);
        assert_eq!(slugs(&p), expected, "{block}");
    }
    // A problem in the spec is located in it, inside the quotes.
    let text = page("| a |\n|---|\n| x {available=\"cloud, edge\"} |");
    let p = project(&[("index.md", &text)]);
    let d = one(&p);
    assert_eq!(&text[d.location.span.range()], "edge");
    let p = project(&[("index.md", &page("| a |\n|---|\n| x {since=3.4} |"))]);
    let d = one(&p);
    assert!(
        d.message.contains("only attribute is `available`"),
        "{}",
        d.message
    );
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
fn frontmatter_that_is_not_yaml_is_a_syntax_error() {
    // SPEC §8.2.
    let p = project(&[("index.md", "---\ntitle: [oops\n---\n\nText.\n")]);
    let d = one(&p);
    assert_eq!(d.slug.as_str(), "frontmatter-syntax");
    assert!(d.message.contains("isn't valid YAML"), "{}", d.message);
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
fn ids_use_letters_digits_hyphens_underscores_and_periods() {
    let p = project(&[(
        "index.md",
        &page("## A\n@id: ok-1\n\n## B\n@id: ok_2.0\n\n## C\n@id: not:ok"),
    )]);
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

/// The route of a page whose file name Astro slugs is found by the router,
/// so the fix is offered.
#[test]
fn a_route_whose_page_has_a_slugged_file_name_gets_a_fix() {
    let p = project(&[
        ("index.md", &page("[x](/guides/my-guide/)")),
        ("Guides/My Guide.md", &page("Text.")),
    ]);
    let all = check_files(&p);
    assert_eq!(slugs(&p), ["link-route"]);
    assert_eq!(all[0].fixes.len(), 1);
    // The space in the file name puts the destination in angle brackets.
    assert_eq!(all[0].fixes[0].edits[0].new_text, "</Guides/My Guide.md>");
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

/// A broken link is offered the targets `ascribe link` lists as closest,
/// as fixes only the author can confirm. A page renamed far from its old
/// name is found by the heading the link names.
#[test]
fn a_broken_link_is_offered_the_page_with_its_heading() {
    let p = project(&[
        ("guides/install.md", &page("[keys](keys.md#rotate-keys)")),
        ("guides/api-keys.md", &page("## Rotate keys\n\nText.")),
        ("guides/other.md", &page("Text.")),
    ]);
    let all = check_files(&p);
    assert_eq!(slugs(&p), ["link-target-missing"]);
    let fixes = &all[0].fixes;
    assert_eq!(fixes.len(), 1, "{fixes:#?}");
    assert_eq!(fixes[0].applicability, Applicability::Unsafe);
    let file = p
        .source_at(&RelPath::parse("guides/install.md").expect("path"))
        .expect("a source");
    assert_eq!(
        apply_edits(&file.text, &fixes[0].edits).expect("edits apply"),
        page("[keys](api-keys.md#rotate-keys)")
    );
}

/// A version the list can't place is offered the declared lines: the
/// newest, for one newer than every line, and those either side, for one
/// between two.
#[test]
fn a_version_the_list_cant_place_is_offered_the_declared_lines() {
    let p = project(&[
        (
            "ahead.md",
            &page(
                "@available: self-managed preview 3.45
Text.",
            ),
        ),
        (
            "between.md",
            &page(
                "@available: self-managed 3.3.1
Text.",
            ),
        ),
    ]);
    let all = check_files(&p);
    let offered = |path: &str| -> Vec<String> {
        let file = p
            .source_at(&RelPath::parse(path).expect("path"))
            .expect("a source");
        let found = all
            .iter()
            .find(|d| d.location.file == file.id)
            .expect("a diagnostic");
        found
            .fixes
            .iter()
            .map(|f| {
                assert_eq!(f.applicability, Applicability::Unsafe);
                apply_edits(&file.text, &f.edits).expect("edits apply")
            })
            .collect()
    };
    assert_eq!(
        slugs(&p),
        ["available-version-ahead", "available-line-undeclared"]
    );
    assert_eq!(
        offered("ahead.md"),
        [page("@available: self-managed preview 3.6\nText.")]
    );
    assert_eq!(
        offered("between.md"),
        [
            page("@available: self-managed 3.2\nText."),
            page("@available: self-managed 3.4\nText.")
        ]
    );
}

/// A link to a fragment is offered the pages that include it.
#[test]
fn a_link_to_a_fragment_is_offered_the_pages_that_include_it() {
    let p = project(&[
        ("index.md", &page("[f](_f.md)")),
        ("setup.md", &page("@include: _f.md")),
        ("_f.md", "Text.\n"),
    ]);
    let all = check_files(&p);
    assert_eq!(slugs(&p), ["link-to-fragment"]);
    let hrefs: Vec<&str> = all[0]
        .fixes
        .iter()
        .map(|f| f.edits[0].new_text.as_str())
        .collect();
    assert_eq!(hrefs, ["setup.md"]);
}

/// A reference form's destination is in its definition, which may serve
/// other links, so it's offered no fix.
#[test]
fn a_broken_reference_link_is_offered_no_fix() {
    let p = project(&[
        ("index.md", &page("[keys][k]\n\n[k]: key.md")),
        ("keys.md", &page("Text.")),
    ]);
    let all = check_files(&p);
    assert_eq!(slugs(&p), ["link-target-missing"]);
    assert!(all[0].fixes.is_empty(), "{:#?}", all[0].fixes);
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
    let outside = root.parent().expect("a parent").join("ascribe-outside.png");
    let text = page(
        "![a](img/Logo.png) ![b](img/logo.png) ![c](../.out/site/p.png) ![d](../shared/s.png) \
         ![e](../../ascribe-outside.png) ![f](img)",
    );
    let p = project_in(root.to_owned(), &[("index.md", &text)]);
    let all = check_files(&p);
    let messages: Vec<&str> = all.iter().map(|d| d.message.as_str()).collect();
    assert_eq!(all.len(), 4, "{messages:#?}");
    assert!(
        messages[0].contains("`img/logo.png` doesn't exist; `img/Logo.png` differs only in case")
    );
    assert!(messages[1].contains("`../.out/site/p.png` is outside the project"));
    assert!(messages[2].contains("`../../ascribe-outside.png` is outside the project"));
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
    let model = ascribe_model::load_str(&text, FileId::new(0)).expect("loads with a warning");
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

/// Applies the first fix of the first diagnostic with `slug` to page `path`,
/// and returns the new text.
fn fixed(path: &str, text: &str, slug: &str) -> String {
    let p = project(&[(path, text)]);
    let d = check_files(&p)
        .into_iter()
        .find(|d| d.slug.as_str() == slug)
        .unwrap_or_else(|| panic!("no {slug}"));
    assert_eq!(d.fixes.len(), 1, "{slug} carries one fix");
    apply_edits(text, &d.fixes[0].edits).expect("edits apply")
}

#[test]
fn certain_replacements_come_with_fixes_that_clear_the_diagnostic() {
    let cases = [
        (
            "attribute-unknown-key",
            page("@note {typ=tip}: t"),
            page("@note {type=tip}: t"),
        ),
        (
            "frontmatter-unknown-key",
            "---\ntitel: x\n---\n\nText.\n".to_owned(),
            "---\ntitle: x\n---\n\nText.\n".to_owned(),
        ),
        (
            "directive-unknown",
            page("@warning: Careful."),
            page("@note {type=warning}: Careful."),
        ),
        (
            "directive-unknown",
            page("@avaliable: cloud\n\nText."),
            page("@available: cloud\n\nText."),
        ),
        (
            "directive-extra-text",
            page("@include: _f.md extra words"),
            page("@include: _f.md"),
        ),
        (
            "directive-extra-text",
            page("@steps foo\n1. a"),
            page("@steps\n1. a"),
        ),
        (
            "directive-extra-text",
            page("@note type=warning: You need Node 20."),
            page("@note {type=warning}: You need Node 20."),
        ),
        (
            "directive-extra-text",
            page("@note type=warning:\nInside.\n@end"),
            page("@note {type=warning}:\nInside.\n@end"),
        ),
        (
            "directive-extra-text",
            page(r#"@snippet lang=py title="A: b": code:x.py"#),
            page(r#"@snippet {lang=py, title="A: b"}: code:x.py"#),
        ),
    ];
    for (slug, before, after) in cases {
        let now = fixed("index.md", &before, slug);
        assert_eq!(now, after, "{slug}");
        // The diagnostic is gone (other diagnostics may remain).
        let p = project(&[("index.md", &now)]);
        assert!(
            !slugs(&p).iter().any(|s| s == slug),
            "{slug} remains in {now:?}"
        );
    }
}

#[test]
fn attributes_without_braces_are_braced_safely_only_before_a_primary() {
    let fix = |body: &str| {
        let p = project(&[("index.md", &page(body))]);
        let d = check_files(&p)
            .into_iter()
            .find(|d| d.slug.as_str() == "directive-extra-text")
            .expect("directive-extra-text");
        let fix = d.fixes.first().expect("a fix");
        (d.message.clone(), fix.title.clone(), fix.applicability)
    };
    let (message, title, applicability) = fix("@note type=warning: You need Node 20.");
    assert!(message.contains("`{type=warning}`"), "{message}");
    assert_eq!(title, "Put `type=warning` in braces");
    assert_eq!(applicability, Applicability::Safe);
    // Without a primary, or with more text after the pairs, the braces may
    // not be all that's meant.
    for body in [
        "@note type=warning:\nInside.\n@end",
        "@note type=warning\nBound.",
        "@note type=warning and more: text",
    ] {
        let (message, _, applicability) = fix(body);
        assert!(message.contains("go in braces"), "{body}: {message}");
        assert_eq!(applicability, Applicability::Unsafe, "{body}");
    }
    // A key the directive doesn't take isn't an attribute: the text is
    // removed, as any other extra text is, and not safely.
    let (message, title, applicability) = fix("@note colour=red: Careful.");
    assert!(message.contains("isn't part of `@note`"), "{message}");
    assert_eq!(title, "Remove `colour=red: Careful.`");
    assert_eq!(applicability, Applicability::Unsafe);
}

#[test]
fn guesses_carry_no_fix() {
    // A missing value, an unknown key with no close match, and a wrong type
    // need the author's judgment.
    let p = project(&[(
        "index.md",
        &page("@note {colour=red}: t\n\n@note {type}: t\n\n@note {type=hint}: t"),
    )]);
    assert!(check_files(&p).iter().all(|d| d.fixes.is_empty()));
}

#[test]
fn a_frontmatter_path_the_index_misses_is_reported_on_the_first_line() {
    // A key that isn't a string is in the values but not in the position
    // index, so the diagnostic falls back to the file's first line.
    let text = "---\ntitle: T\n1: x\n---\n\nText.\n";
    let p = project(&[("index.md", text)]);
    let d = one(&p);
    assert_eq!(d.slug.as_str(), "frontmatter-type-mismatch");
    assert_eq!(&text[d.location.span.range()], "---");

    // Even when the first line is blank, the span isn't empty.
    let p = project(&[("index.md", "\n\nJust text.\n")]);
    let d = one(&p);
    assert_eq!(d.slug.as_str(), "frontmatter-missing-field");
    assert!(!d.location.span.is_empty());
}

#[test]
fn an_image_with_no_path_is_a_missing_source_and_a_link_is_not() {
    let p = project(&[(
        "index.md",
        &page("![A]()\n\n![B](#top)\n\n[Self](#top) []()"),
    )]);
    assert_eq!(slugs(&p), ["image-source-missing", "image-source-missing"]);
}

mod robustness {
    use super::*;
    use proptest::prelude::*;

    proptest! {
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

/// A project of in-memory pages under the model `MODEL` with a description
/// field, and the outputs for agents on or off.
fn described(agents: bool, files: &[(&str, &str)]) -> Project {
    let text = format!(
        "{MODEL}\n[consumer]\nsite = \"https://docs.example.com\"\nagents = {agents}\n\n[types.page.frontmatter]\ntitle = \"string\"\ndescription = {{ type = \"string?\", role = \"description\" }}\n"
    );
    let model = ascribe_model::load_str(&text, FileId::new(0)).expect("the model loads");
    let sources = Project::from_sources(files.iter().map(|(path, text)| {
        (
            RelPath::parse(path).expect("a relative path"),
            (*text).to_owned(),
        )
    }));
    Project::from_parts(
        PathBuf::from("/nonexistent-ascribe-project"),
        RelPath::parse("docs").expect("a relative path"),
        model,
        text,
        sources,
    )
}

#[test]
fn a_description_is_measured_as_its_line_in_llms_txt_with_agents_on() {
    let long = format!(
        "---\ntype: page\ntitle: T\ndescription: {}\n---\n",
        "word ".repeat(70)
    );
    assert_eq!(
        found(&described(true, &[("a.md", &long)])),
        [("description-too-long".to_owned(), 4)]
    );
    // Without llms.txt there's no line to keep short.
    assert!(found(&described(false, &[("a.md", &long)])).is_empty());
    // A link counts as its text, which is all llms.txt shows of it.
    let linked = format!(
        "---\ntype: page\ntitle: T\ndescription: See [the guide](https://example.com/{}).\n---\n",
        "x".repeat(400)
    );
    assert_eq!(
        found(&described(true, &[("a.md", &linked)])),
        [("description-link".to_owned(), 4)]
    );
}

#[test]
fn a_page_looser_than_its_rule_quotes_the_rule_and_its_line() {
    let text = format!("{MODEL}\n[listing.folders]\n\"internal/\" = \"unlisted\"\n");
    let line = text
        .lines()
        .position(|l| l == "\"internal/\" = \"unlisted\"")
        .expect("the rule")
        + 1;
    let model = ascribe_model::load_str(&text, FileId::new(0)).expect("the model loads");
    let sources = Project::from_sources([(
        RelPath::parse("internal/plan.md").expect("a relative path"),
        "---\ntitle: Plan\nlisting: listed\n---\n".to_owned(),
    )]);
    let project = Project::from_parts(
        PathBuf::from("/nonexistent-ascribe-project"),
        RelPath::parse("docs").expect("a relative path"),
        model,
        text.clone(),
        sources,
    );
    let d = one(&project);
    assert_eq!(d.slug.as_str(), "listing-loosened");
    assert_eq!(
        d.message,
        format!(
            "this page is `listed`, looser than `\"internal/\" = \"unlisted\"` on line {line} of \
             ascribe.toml, which it's under; a page can only be as strict as its rule, or stricter"
        )
    );
    // The fix removes the page's own setting, and the rule holds.
    let fix = &d.fixes[0];
    assert_eq!(fix.applicability, Applicability::Safe);
    let file = project.file(d.location.file).expect("a known file");
    assert_eq!(
        apply_edits(file.text, &fix.edits).expect("the edits apply"),
        "---\ntitle: Plan\n---\n"
    );
}

#[test]
fn a_link_in_a_description_is_found_at_the_link_and_fixed_to_its_text() {
    let text = "---\ntype: page\ntitle: T\ndescription: Set up **[the server](server.md)** and <https://example.com>.\n---\n";
    let p = described(false, &[("a.md", text)]);
    let links: Vec<_> = check_files(&p)
        .into_iter()
        .filter(|d| d.slug.as_str() == "description-link")
        .collect();
    assert_eq!(links.len(), 2, "{links:#?}");
    let file = p.file(links[0].location.file).expect("a known file");
    assert_eq!(
        &file.text[links[0].location.span.range()],
        "[the server](server.md)"
    );
    let mut fixed = text.to_owned();
    for d in links.iter().rev() {
        assert_eq!(d.fixes[0].applicability, Applicability::Unsafe);
        fixed = apply_edits(&fixed, &d.fixes[0].edits).expect("edits apply");
    }
    assert_eq!(
        fixed,
        "---\ntype: page\ntitle: T\ndescription: Set up **the server** and https://example.com.\n---\n"
    );
    assert!(found(&described(false, &[("a.md", &fixed)])).is_empty());

    // A quoted description is reported at its value, with no fix to apply.
    let quoted = "---\ntype: page\ntitle: T\ndescription: \"See [the guide](g.md).\"\n---\n";
    let d = check_files(&described(false, &[("a.md", quoted)]))
        .into_iter()
        .find(|d| d.slug.as_str() == "description-link")
        .expect("description-link");
    assert!(d.fixes.is_empty());
    // A fix that would end the value early in YAML isn't offered.
    let colon = "---\ntype: page\ntitle: T\ndescription: See [the guide:](g.md) it.\n---\n";
    let d = check_files(&described(false, &[("a.md", colon)]))
        .into_iter()
        .find(|d| d.slug.as_str() == "description-link")
        .expect("description-link");
    assert!(d.fixes.is_empty());
}

/// The text of `body`'s page after its one diagnostic's one fix, which must
/// be `memo-empty`'s and safe; `None` when it has no fix.
fn empty_memo_removed(body: &str) -> Option<String> {
    let text = page(body);
    let p = project(&[("a.md", &text)]);
    let d = one(&p);
    assert_eq!(d.slug.as_str(), "memo-empty");
    let fix = d.fixes.first()?;
    assert_eq!(fix.applicability, Applicability::Safe);
    Some(apply_edits(&text, &fix.edits).expect("edits apply"))
}

/// An empty container memo's fix removes its lines, and a blank line after
/// them when one is before them too; one that starts a list item has none.
#[test]
fn an_empty_memo_s_fix_removes_it() {
    assert_eq!(
        empty_memo_removed("Text.\n\n@memo:\n@end\nThe block.").as_deref(),
        Some(page("Text.\n\nThe block.").as_str())
    );
    assert_eq!(
        empty_memo_removed("## A\n\n@memo:\n\n@end\n\nText.").as_deref(),
        Some(page("## A\n\nText.").as_str())
    );
    assert_eq!(
        empty_memo_removed("> Text.\n>\n> @memo:\n> @end\n> The block.").as_deref(),
        Some(page("> Text.\n>\n> The block.").as_str())
    );
    assert_eq!(empty_memo_removed("- @memo:\n  @end\n  The block."), None);
}

/// The messages of each problem the memo files in `memos` have, beside a
/// page `a.md` with one heading, `#setup`.
fn memo_messages(memos: &[(&str, &str)]) -> Vec<String> {
    let dir = tempfile::tempdir().expect("a temporary directory");
    fs::create_dir_all(dir.path().join("memos")).expect("memos/");
    for (name, text) in memos {
        fs::write(dir.path().join("memos").join(name), text).expect("a memo file");
    }
    let text = page("## Setup\n@id: setup\n\nRun it.");
    let p = project_in(dir.path().to_owned(), &[("a.md", &text)]);
    check_files(&p)
        .iter()
        .map(|d| {
            let file = p.file(d.location.file).expect("a known file");
            assert!(file.display_path.starts_with("memos/"), "{d:?}");
            d.message.clone()
        })
        .collect()
}

/// A detached memo names the heading's page's ids, or the closest page.
#[test]
fn a_detached_memo_says_what_it_could_be_about() {
    assert_eq!(
        memo_messages(&[
            ("heading.md", "---\nabout: a.md#install\n---\nWhy.\n"),
            ("page.md", "---\nabout: b.md\n---\nWhy.\n"),
            ("phrase.md", "---\nabout: phrase:prodcut\n---\nWhy.\n"),
        ]),
        [
            "`a.md` has no heading `#install`, so this memo is about nothing; its ids are: `setup`",
            "`b.md` isn't a page or fragment of the project, so this memo is about nothing; did you mean `a.md`?",
            "`phrase:prodcut` isn't declared in ascribe.toml, so this memo is about nothing; did you mean `phrase:product`?",
        ]
    );
}

/// A memo that's placed, quoting words of the section it's about, is no
/// problem; one whose words are gone is outdated, on its section.
#[test]
fn a_memo_is_outdated_when_its_words_are_gone() {
    assert!(
        memo_messages(&[(
            "ok.md",
            "---\nabout: a.md#setup\nquote: Run it\n---\nWhy.\n"
        )])
        .is_empty()
    );
    assert_eq!(
        memo_messages(&[(
            "old.md",
            "---\nabout: a.md#setup\nquote: Run them\n---\nWhy.\n"
        )]),
        [
            "no block of `a.md#setup` has the words this memo quotes, so it's on the section `#setup` and no block"
        ]
    );
}

/// An include of a file that isn't beside the including file is offered
/// the source file with its name, written from the including file, keeping
/// the `#id`.
#[test]
fn a_missing_include_is_offered_the_file_with_its_name() {
    let p = project(&[
        (
            "guides/setup.md",
            &page("@include: _shared/restart.md#intro"),
        ),
        ("_shared/restart.md", "## Intro\n\nRestart it.\n"),
        ("_shared/other.md", "Text.\n"),
    ]);
    let all = check_files(&p);
    assert_eq!(slugs(&p), ["include-target-missing"]);
    let fixes = &all[0].fixes;
    assert_eq!(fixes.len(), 1, "{fixes:#?}");
    assert_eq!(fixes[0].title, "Include `../_shared/restart.md`");
    assert_eq!(fixes[0].applicability, Applicability::Unsafe);
    let file = p
        .source_at(&RelPath::parse("guides/setup.md").expect("path"))
        .expect("a source");
    assert_eq!(
        apply_edits(&file.text, &fixes[0].edits).expect("edits apply"),
        page("@include: ../_shared/restart.md#intro")
    );
}

/// In a project that declares no dimensions, an arm's dimension gets the
/// table to add, with the values every page gives it.
#[test]
fn a_dimension_with_none_declared_gets_the_table_to_add() {
    const BARE: &str = "spec = \"0.1\"\n\n[project]\ncontent-root = \"docs\"\n";
    let sources = Project::from_sources([
        (
            RelPath::parse("a.md").expect("a path"),
            page("@variant {os=mac}:\nMac.\n@variant {os=linux|windows}:\nOther.\n@end"),
        ),
        (
            RelPath::parse("b.md").expect("a path"),
            page("@variant {os=bsd}:\nBSD.\n@end"),
        ),
    ]);
    let p = Project::from_parts(
        PathBuf::from("/nonexistent-ascribe-project"),
        RelPath::parse("docs").expect("a relative path"),
        ascribe_model::load_str(BARE, FileId::new(0)).expect("the model loads"),
        BARE.to_owned(),
        sources,
    );
    let all: Vec<String> = check_files(&p)
        .into_iter()
        .filter(|d| d.slug.as_str() == "variant-unknown")
        .map(|d| d.message)
        .collect();
    assert_eq!(all.len(), 3, "{all:#?}");
    for message in all {
        assert_eq!(
            message,
            "`os` isn't a declared dimension, and ascribe.toml declares none: add \
             `[dimensions.os]` with `values = [\"mac\", \"linux\", \"windows\", \"bsd\"]`"
        );
    }
}
