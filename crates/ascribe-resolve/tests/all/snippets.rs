//! Snippets (SPEC §4.8): the code block a `@snippet` becomes when a page is
//! expanded, and which variant of each problem `snippet_issues` reports.

#![allow(clippy::expect_used, clippy::panic)]

use std::sync::Arc;

use crate::links;

use ascribe_core::{FileId, RelPath};
use ascribe_model::load_str;
use ascribe_resolve::{
    CodeFiles, ExpandedKind, Layout, MemoryFs, Project, snippet_issues, source_files,
};
use ascribe_syntax::BlockKind;

const MODEL: &str = r#"
spec = "0.1"

[project]
content-root = "docs"

[sources.code]
path = "../code"
include = ["**/*.py", "**/*.txt"]
ignore = ["private/**"]

[phrases]
product = "Quill"
"#;

const APP: (&str, &str) = (
    "../code/app.py",
    "import os\n\n# :snippet-start: main\ndef main():\n    print(\"{product}\")  # :remove:\n    print(\"hi {product}\")\n# :snippet-end:\n",
);

fn load(page: &str, files: &[(&str, &str)]) -> (Project, MemoryFs) {
    let model = Arc::new(load_str(MODEL, FileId::new(0)).expect("a model"));
    let layout = Layout::from_model(&model);
    let mut fs =
        MemoryFs::new(&layout).with_file("docs/index.md", &format!("---\ntitle: T\n---\n{page}"));
    for (path, text) in files {
        fs = fs.with_file(path, text);
    }
    (Project::load(model, layout, &fs), fs)
}

fn index() -> RelPath {
    RelPath::parse("index.md").expect("a path")
}

/// The slug and variant of the first problem with the page's only snippet.
fn problem(page: &str, files: &[(&str, &str)]) -> Option<(String, Option<&'static str>)> {
    let (project, fs) = load(page, files);
    let file = project.file(&index()).expect("the page");
    let snippet = file.snippets.first().expect("a snippet");
    let issues = snippet_issues(snippet, project.model(), &fs, &CodeFiles::new(), file.file);
    issues
        .first()
        .map(|i| (i.slug.as_str().to_owned(), i.variant))
}

fn is(slug: &str, variant: Option<&'static str>) -> Option<(String, Option<&'static str>)> {
    Some((slug.to_owned(), variant))
}

#[test]
fn a_snippet_becomes_a_code_block_on_the_expanded_page() {
    let (project, _) = load(
        "@snippet {title=\"Run\", phrases=true}: code:app.py#main\n\n@snippet: code:nothing.py\n",
        &[APP],
    );
    let page = project.expand(&index()).expect("the page");
    let mut seen = Vec::new();
    page.visit(&mut |block| {
        let ExpandedKind::Leaf(leaf) = &block.kind else {
            return;
        };
        match (&leaf.kind, &block.snippet) {
            (BlockKind::CodeBlock(code), Some(snippet)) => {
                assert_eq!(code.info, "py title=\"Run\" phrases=true");
                assert_eq!(code.literal, "def main():\n    print(\"hi {product}\")\n");
                assert!(code.phrases.as_ref().is_some_and(|p| p.len() == 1));
                assert_eq!(snippet.address, "code:app.py#main");
                assert_eq!(snippet.path.as_str(), "../code/app.py");
                assert_eq!(snippet.lines, Some((4, 6)));
                seen.push("code");
            }
            // One that doesn't resolve stays as its directive.
            (BlockKind::Directive(line), None) if line.name == "snippet" => seen.push("directive"),
            _ => {}
        }
    });
    assert_eq!(seen, ["code", "directive"]);
}

#[test]
fn an_address_names_a_source() {
    assert_eq!(problem("@snippet: code:app.py#main\n", &[APP]), None);
    assert_eq!(
        problem("@snippet: app.py\n", &[APP]),
        is("snippet-address", Some("no-source"))
    );
    assert_eq!(
        problem("@snippet: code:./app.py\n", &[APP]),
        is("snippet-address", None)
    );
}

#[test]
fn an_unknown_source_suggests_a_declared_one() {
    assert_eq!(
        problem("@snippet: cdoe:app.py\n", &[APP]),
        is("snippet-source-unknown", Some("suggestion"))
    );
    assert_eq!(
        problem("@snippet: examples:app.py\n", &[APP]),
        is("snippet-source-unknown", None)
    );
}

#[test]
fn a_file_must_exist_by_its_exact_name_and_be_included() {
    let files = [
        APP,
        ("../code/private/key.py", "x\n"),
        ("../code/notes.md", "x\n"),
    ];
    assert_eq!(
        problem("@snippet: code:gone.py\n", &files),
        is("snippet-file-missing", None)
    );
    assert_eq!(
        problem("@snippet: code:App.py\n", &files),
        is("snippet-file-missing", Some("case"))
    );
    assert_eq!(
        problem("@snippet: code:private/key.py\n", &files),
        is("snippet-file-missing", Some("not-included"))
    );
    assert_eq!(
        problem("@snippet: code:notes.md\n", &files),
        is("snippet-file-missing", Some("not-included"))
    );
}

#[test]
fn a_file_must_be_text() {
    assert_eq!(
        problem(
            "@snippet: code:data.txt\n",
            &[("../code/data.txt", "a\0b\n")]
        ),
        is("snippet-file-not-text", None)
    );
}

#[test]
fn a_missing_region_suggests_one_or_lists_them() {
    assert_eq!(
        problem("@snippet: code:app.py#mian\n", &[APP]),
        is("snippet-region-missing", Some("suggestion"))
    );
    assert_eq!(
        problem("@snippet: code:app.py#teardown\n", &[APP]),
        is("snippet-region-missing", None)
    );
    assert_eq!(
        problem(
            "@snippet: code:bare.py#a\n",
            &[("../code/bare.py", "x = 1\n")]
        ),
        is("snippet-region-missing", Some("none"))
    );
    assert_eq!(
        problem(
            "@snippet: code:plain.txt#a\n",
            &[("../code/plain.txt", "x\n")]
        ),
        is("snippet-region-missing", Some("no-comments"))
    );
}

#[test]
fn each_kind_of_unsound_tag_has_its_variant() {
    let cases = [
        ("# :snippet-start: a\nx = 1\n", None),
        ("x = 1\n# :snippet-end:\n", Some("unmatched")),
        (
            "# :snippet-start: a\n# :snippet-end:\n# :snippet-start: a\n# :snippet-end:\n",
            Some("duplicate"),
        ),
        ("# :snippet-start: a b\n# :snippet-end:\n", Some("name")),
        ("# :state-start: x\n# :state-end:\n", Some("reserved")),
        ("x = 1  # :emphasize:\n", Some("reserved")),
    ];
    for (text, variant) in cases {
        assert_eq!(
            problem("@snippet: code:tags.py\n", &[("../code/tags.py", text)]),
            is("snippet-tags", variant),
            "{text:?}"
        );
    }
}

/// A snippet is read where its links lead, and a link out of the source's
/// folder, or to a file its patterns don't include, is refused.
#[test]
fn a_link_out_of_the_source_is_refused() {
    use ascribe_resolve::DiskFs;
    use std::fs;

    let dir = tempfile::tempdir().expect("a temp dir");
    let root = dir.path();
    fs::create_dir_all(root.join("project/docs")).expect("docs");
    fs::create_dir_all(root.join("code/private")).expect("code");
    fs::create_dir_all(root.join("elsewhere")).expect("elsewhere");
    fs::write(root.join("code/app.py"), APP.1).expect("app");
    fs::write(root.join("code/private/key.py"), APP.1).expect("key");
    fs::write(root.join("secret.py"), APP.1).expect("secret");
    fs::write(root.join("elsewhere/far.py"), APP.1).expect("far");
    links::dir("../elsewhere", root.join("code/linked"));
    // Without file links, the folder link is still tested on its own.
    let files_ok = [
        ("app.py", "code/alias.py"),
        ("../secret.py", "code/out.py"),
        ("private/key.py", "code/hidden.py"),
    ]
    .iter()
    .all(|(target, link)| links::file(target, root.join(link)));

    let problem = |page: &str| {
        fs::write(
            root.join("project/docs/index.md"),
            format!("---\ntitle: T\n---\n{page}"),
        )
        .expect("the page");
        let model = Arc::new(load_str(MODEL, FileId::new(0)).expect("a model"));
        let layout = Layout::from_model(&model);
        let fs = DiskFs::new(root.join("project"), &layout);
        let project = Project::load(model, layout, &fs);
        let file = project.file(&index()).expect("the page");
        let snippet = file.snippets.first().expect("a snippet");
        snippet_issues(snippet, project.model(), &fs, &CodeFiles::new(), file.file)
            .first()
            .map(|i| (i.slug.as_str().to_owned(), i.variant))
    };

    let link = is("snippet-file-missing", Some("link"));
    assert_eq!(problem("@snippet: code:app.py#main\n"), None);
    assert_eq!(problem("@snippet: code:linked/far.py#main\n"), link);
    if files_ok {
        assert_eq!(problem("@snippet: code:alias.py#main\n"), None);
        assert_eq!(problem("@snippet: code:out.py#main\n"), link);
        assert_eq!(problem("@snippet: code:hidden.py#main\n"), link);
    }
}

/// A source whose folder holds the project reaches the project's own files:
/// their real paths are compared from the same folder as the source's.
#[test]
fn a_source_above_the_project_reads_its_files() {
    use ascribe_resolve::DiskFs;
    use std::fs;

    let dir = tempfile::tempdir().expect("a temp dir");
    let root = dir.path();
    let model = "\
spec = \"0.1\"

[project]
content-root = \"content\"

[sources.code]
path = \"..\"
include = [\"docs/ascribe.toml\", \"examples/**\"]
";
    fs::create_dir_all(root.join("docs/content")).expect("docs");
    fs::create_dir_all(root.join("examples")).expect("examples");
    fs::write(
        root.join("docs/ascribe.toml"),
        format!("{model}# :snippet-start: sources\n[sources]\n# :snippet-end:\n"),
    )
    .expect("the model");
    fs::write(root.join("examples/a.py"), APP.1).expect("an example");

    let model = Arc::new(load_str(model, FileId::new(0)).expect("a model"));
    let layout = Layout::from_model(&model);
    for address in ["code:docs/ascribe.toml#sources", "code:examples/a.py#main"] {
        fs::write(
            root.join("docs/content/index.md"),
            format!("---\ntitle: T\n---\n@snippet: {address}\n"),
        )
        .expect("the page");
        let fs = DiskFs::new(root.join("docs"), &layout);
        let project = Project::load(model.clone(), layout.clone(), &fs);
        let file = project.file(&index()).expect("the page");
        let snippet = file.snippets.first().expect("a snippet");
        let issues = snippet_issues(snippet, project.model(), &fs, &CodeFiles::new(), file.file);
        assert!(issues.is_empty(), "{address}: {issues:?}");
    }
}

/// A link that leads above a source's folder is refused, however wide its
/// patterns: `../above.py` isn't a path in the folder.
#[test]
fn a_link_above_a_source_at_the_parent_is_refused() {
    use ascribe_resolve::DiskFs;
    use std::fs;

    let dir = tempfile::tempdir().expect("a temp dir");
    let root = dir.path();
    fs::create_dir_all(root.join("repo/docs/content")).expect("docs");
    fs::create_dir_all(root.join("repo/examples")).expect("examples");
    fs::write(root.join("above.py"), APP.1).expect("above");
    if !links::file("../../above.py", root.join("repo/examples/up.py")) {
        return;
    }
    fs::write(
        root.join("repo/docs/content/index.md"),
        "---\ntitle: T\n---\n@snippet: code:examples/up.py#main\n",
    )
    .expect("the page");
    for include in ["", "include = [\"**/*.py\"]\n"] {
        let model = format!(
            "spec = \"0.1\"\n\n[project]\ncontent-root = \"content\"\n\n[sources.code]\npath = \"..\"\n{include}"
        );
        let model = Arc::new(load_str(&model, FileId::new(0)).expect("a model"));
        let layout = Layout::from_model(&model);
        let fs = DiskFs::new(root.join("repo/docs"), &layout);
        let project = Project::load(model, layout, &fs);
        let file = project.file(&index()).expect("the page");
        let snippet = file.snippets.first().expect("a snippet");
        let issues = snippet_issues(snippet, project.model(), &fs, &CodeFiles::new(), file.file);
        assert_eq!(
            issues
                .first()
                .map(|i| (i.slug.as_str().to_owned(), i.variant)),
            is("snippet-file-missing", Some("link")),
            "{include:?}"
        );
    }
}

#[test]
fn a_sources_files_are_listed_with_their_regions_and_each_address_resolves() {
    let (project, fs) = load(
        "",
        &[
            APP,
            ("../code/notes/read me.txt", "plain\n"),
            ("../code/private/secret.py", "x = 1\n"),
            ("../code/skipped.rs", "fn main() {}\n"),
            ("../code/broken.py", "# :snippet-start: a\nx = 1\n"),
            ("../code/binary.py", "\0"),
        ],
    );
    let source = project.model().source("code").expect("the source");
    let code = CodeFiles::new();
    let files = source_files(source, &fs, &code);
    let listed: Vec<(&str, &str, Vec<&str>)> = files
        .iter()
        .map(|f| {
            (
                f.path.as_str(),
                f.address.as_str(),
                f.regions.iter().map(String::as_str).collect(),
            )
        })
        .collect();
    // Only what `include` and `ignore` take in, and what gives a snippet.
    assert_eq!(
        listed,
        [
            ("app.py", "code:app.py", vec!["main"]),
            ("notes/read me.txt", "code:notes/read%20me.txt", vec![]),
        ]
    );
    for file in &files {
        let addresses = std::iter::once(file.address.clone())
            .chain(file.regions.iter().map(|r| format!("{}#{r}", file.address)));
        for address in addresses {
            let page = format!("@snippet: {address}\n");
            assert_eq!(
                problem(&page, &[APP, ("../code/notes/read me.txt", "plain\n")]),
                None,
                "{address}"
            );
        }
    }
}
