//! `@snippet` at file level (SPEC §4.8): each problem is reported where the
//! address is, and one with a code file's tags points into the code file,
//! which the project can then show. Which variant each problem takes is
//! tested in `tessera-resolve`, where the rules are.

#![allow(clippy::expect_used, clippy::panic)]

use std::path::PathBuf;
use std::sync::Arc;

use tessera_check::{Diagnostic, Project, check_files};
use tessera_core::{FileId, RelPath};
use tessera_resolve::{Layout, MemoryFs};

const MODEL: &str = r#"
spec = "0.1"

[project]
content-root = "docs"

[sources.code]
path = "code"
include = ["**/*.py", "**/*.txt"]
ignore = ["private/**"]
"#;

fn check(page: &str, files: &[(&str, &str)]) -> (Project, Vec<Diagnostic>) {
    let model = tessera_model::load_str(MODEL, FileId::new(0)).expect("a model");
    let layout = Layout::from_model(&model);
    let mut fs = MemoryFs::new(&layout);
    for (path, text) in files {
        fs = fs.with_file(path, text);
    }
    let text = format!("---\ntitle: T\n---\n{page}");
    let sources = Project::from_sources([(RelPath::parse("index.md").expect("a path"), text)]);
    let project = Project::from_parts_with_fs(
        PathBuf::from("/no/such/project"),
        RelPath::parse("docs").expect("a path"),
        model,
        MODEL.to_owned(),
        sources,
        Arc::new(fs),
    );
    let found = check_files(&project);
    (project, found)
}

fn slugs(page: &str, files: &[(&str, &str)]) -> Vec<&'static str> {
    check(page, files)
        .1
        .iter()
        .map(|d| d.slug.as_str())
        .collect()
}

const APP: (&str, &str) = (
    "code/app.py",
    "# :snippet-start: setup\nsetup()\n# :snippet-end:\n",
);

#[test]
fn a_snippet_that_resolves_has_no_problem() {
    assert!(slugs("@snippet: code:app.py#setup\n", &[APP]).is_empty());
    assert!(slugs("@snippet: code:app.py\n", &[APP]).is_empty());
}

#[test]
fn each_problem_is_reported_at_the_address() {
    let files = [
        APP,
        ("code/data.txt", "a\0b\n"),
        ("code/private/key.py", "x\n"),
    ];
    let page = "@snippet: app.py\n\
                @snippet: cdoe:app.py\n\
                @snippet: code:gone.py\n\
                @snippet: code:private/key.py\n\
                @snippet: code:data.txt\n\
                @snippet {title=\"A\"}: code:app.py#setpu\n";
    let (project, found) = check(page, &files);
    let at: Vec<(&str, String)> = found
        .iter()
        .map(|d| {
            let text = &project.sources()[0].text;
            (d.slug.as_str(), text[d.location.span.range()].to_owned())
        })
        .collect();
    assert_eq!(
        at,
        [
            ("snippet-address", "app.py".to_owned()),
            ("snippet-source-unknown", "cdoe:app.py".to_owned()),
            ("snippet-file-missing", "code:gone.py".to_owned()),
            ("snippet-file-missing", "code:private/key.py".to_owned()),
            ("snippet-file-not-text", "code:data.txt".to_owned()),
            ("snippet-region-missing", "code:app.py#setpu".to_owned()),
        ]
    );
}

#[test]
fn unsound_tags_point_into_the_code_file() {
    let file = ("code/bad.py", "x = 1\n# :snippet-start: a\ny = 2\n");
    let (project, found) = check("@snippet: code:bad.py\n", &[file]);
    assert_eq!(found.len(), 1);
    let d = &found[0];
    assert_eq!(d.slug.as_str(), "snippet-tags");
    assert_eq!(d.related.len(), 1);
    let at = d.related[0].location;
    let code = project.code_file(at.file).expect("the code file");
    assert_eq!(code.path.as_str(), "code/bad.py");
    assert_eq!(&code.text[at.span.range()], "# :snippet-start: a");
    assert!(project.file(at.file).is_none());
}
