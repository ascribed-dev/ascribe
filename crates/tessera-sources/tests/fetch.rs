//! `fetch`: copying the files snippets name, at each source's pin, from a
//! real repository reached by a `file://` URL.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod common;
#[path = "../../../tests/support/links.rs"]
mod links;

use common::{Code, Docs};
use tessera_sources::{CopyState, SourcesError, fetch, status};

const AUTH: &str = "// :snippet-start: login\nfn login() {}\n// :snippet-end:\n";

fn api(code: &Code) -> String {
    format!(
        "[sources.api]\ngit = \"{}\"\nbranch = \"main\"\ninclude = [\"src/**\", \"examples/**\"]\n",
        code.url()
    )
}

#[test]
fn a_first_fetch_pins_the_branch_and_copies_what_snippets_use() {
    let code = Code::new();
    code.write("src/auth.rs", AUTH)
        .write("src/other.rs", "fn other() {}\n")
        .write("README.md", "# API\n");
    let commit = code.commit("first");
    let docs = Docs::new(&api(&code));
    docs.page(
        "index.md",
        "---\ntitle: Home\n---\n\n@snippet: api:src/auth.rs#login\n",
    );

    let report = fetch(&docs.workspace(), &[], &docs.options()).unwrap();
    let api = &report.sources[0];
    assert_eq!(api.commit.as_deref(), Some(commit.as_str()));
    assert!(api.pinned && api.first_copy);
    assert_eq!(api.copied, ["src/auth.rs"]);
    assert!(report.lock_written);
    // Only the file a snippet uses: not the repository, and not everything
    // `include` matches.
    assert_eq!(docs.read("sources/api/src/auth.rs").as_deref(), Some(AUTH));
    assert!(!docs.exists("sources/api/src/other.rs"));
    let lock = docs.read("ascribe.lock").unwrap();
    assert!(lock.contains(&format!("commit = \"{commit}\"")), "{lock}");
    assert!(lock.contains("\"src/auth.rs\" = \"sha256:"), "{lock}");
    assert_eq!(docs.check(), Vec::<String>::new());

    // Run again, nothing changes and nothing is fetched.
    let again = fetch(&docs.workspace(), &[], &docs.options()).unwrap();
    assert!(again.sources[0].copied.is_empty() && !again.sources[0].pinned);
    assert!(!again.lock_written);
}

#[test]
fn fetch_never_moves_a_pin_and_reads_the_cache_the_second_time() {
    let code = Code::new();
    code.write("src/auth.rs", AUTH);
    let first = code.commit("first");
    let docs = Docs::new(&api(&code));
    docs.page(
        "index.md",
        "---\ntitle: Home\n---\n\n@snippet: api:src/auth.rs\n",
    );
    fetch(&docs.workspace(), &[], &docs.options()).unwrap();

    // The branch moves on; fetch keeps the pin.
    code.write("src/auth.rs", "fn login() { changed() }\n");
    code.commit("second");
    // A copy deleted, and one edited by hand, are put back as pinned.
    std::fs::remove_file(common::path(docs.root(), "sources/api/src/auth.rs")).unwrap();
    let report = fetch(&docs.workspace(), &[], &docs.options()).unwrap();
    assert_eq!(report.sources[0].commit.as_deref(), Some(first.as_str()));
    assert_eq!(docs.read("sources/api/src/auth.rs").as_deref(), Some(AUTH));

    // With the repository gone, the cache still has the pinned commit's
    // files.
    common::write(docs.root(), "sources/api/src/auth.rs", "edited\n");
    let moved = code.root().with_extension("moved");
    std::fs::rename(code.root(), &moved).unwrap();
    let report = fetch(&docs.workspace(), &[], &docs.options());
    std::fs::rename(&moved, code.root()).unwrap();
    assert_eq!(report.unwrap().sources[0].copied, ["src/auth.rs"]);
    assert_eq!(docs.read("sources/api/src/auth.rs").as_deref(), Some(AUTH));
}

#[test]
fn files_that_cant_be_copied_are_reported_and_left_out() {
    let code = Code::new();
    code.write("src/auth.rs", AUTH)
        .write("src/big.txt", "x".repeat(tessera_sources::SIZE_LIMIT + 1))
        .write("src/data.bin", b"\0\x01\x02".as_slice())
        .write("private/key.rs", "secret\n");
    code.commit("first");
    let docs = Docs::new(&api(&code));
    docs.page(
        "index.md",
        "---\ntitle: Home\n---\n\n@snippet: api:src/auth.rs\n\n@snippet: api:src/big.txt\n\n@snippet: api:src/data.bin\n\n@snippet: api:src/gone.rs\n\n@snippet: api:private/key.rs\n",
    );
    let report = fetch(&docs.workspace(), &[], &docs.options()).unwrap();
    assert!(report.has_failures());
    let failed: Vec<(&str, &str)> = report.sources[0]
        .failed
        .iter()
        .map(|f| (f.path.as_str(), f.reason.as_str()))
        .collect();
    assert_eq!(failed.len(), 3, "{failed:?}");
    assert_eq!(failed[0].0, "src/gone.rs");
    assert!(failed[0].1.starts_with("it isn't in the repository at "));
    assert_eq!(
        failed[1..],
        [
            (
                "src/big.txt",
                "it's larger than 1024 KB, the most Ascribe copies"
            ),
            ("src/data.bin", "it isn't text: it has a NUL character"),
        ]
    );
    // A file outside `include` isn't asked for at all.
    assert!(!docs.exists("sources/api/private/key.rs"));
    assert!(!docs.exists("sources/api/src/big.txt"));
    assert!(!docs.exists("sources/api/src/data.bin"));
    assert_eq!(report.sources[0].copied, ["src/auth.rs"]);
    // `check` reports each snippet that has no copy, and the one outside
    // `include` as such.
    let problems = docs.check();
    assert_eq!(problems.len(), 4, "{problems:?}");
    assert!(
        problems
            .iter()
            .all(|p| p.starts_with("snippet-file-missing"))
    );
    assert!(
        problems[3].contains("patterns don't make it readable"),
        "{problems:?}"
    );
}

#[test]
fn a_copy_no_snippet_uses_is_removed() {
    let code = Code::new();
    code.write("src/auth.rs", AUTH)
        .write("src/other.rs", "fn other() {}\n");
    code.commit("first");
    let docs = Docs::new(&api(&code));
    docs.page(
        "index.md",
        "---\ntitle: Home\n---\n\n@snippet: api:src/auth.rs\n\n@snippet: api:src/other.rs\n",
    );
    fetch(&docs.workspace(), &[], &docs.options()).unwrap();
    assert!(docs.exists("sources/api/src/other.rs"));

    docs.page(
        "index.md",
        "---\ntitle: Home\n---\n\n@snippet: api:src/auth.rs\n",
    );
    let st = status(&docs.workspace());
    let states: Vec<_> = st.sources[0].files.iter().map(|f| f.state).collect();
    assert_eq!(states, [CopyState::Current, CopyState::Unused]);
    assert!(docs.check()[0].starts_with("source-copy-unused"));

    let report = fetch(&docs.workspace(), &[], &docs.options()).unwrap();
    assert_eq!(report.sources[0].removed, ["src/other.rs"]);
    assert!(!docs.exists("sources/api/src/other.rs"));
    assert!(
        docs.read("ascribe.lock")
            .is_some_and(|l| !l.contains("other.rs"))
    );

    // With no snippet left, the copies' folders go too, and the pin stays.
    docs.page("index.md", "---\ntitle: Home\n---\n\nNo code.\n");
    fetch(&docs.workspace(), &[], &docs.options()).unwrap();
    assert!(!docs.exists("sources"));
    assert!(
        docs.read("ascribe.lock")
            .is_some_and(|l| l.contains("name = \"api\""))
    );

    // A source that's gone loses its pin.
    common::write(
        docs.root(),
        "ascribe.toml",
        "spec = \"0.1\"\n[project]\ncontent-root = \"docs\"\n",
    );
    let report = fetch(&docs.workspace(), &[], &docs.options()).unwrap();
    assert_eq!(report.unpinned, ["api"]);
    assert!(
        docs.read("ascribe.lock")
            .is_some_and(|l| !l.contains("[[source]]"))
    );
}

#[test]
fn two_sources_and_a_name() {
    let api_code = Code::new();
    api_code.write("src/auth.rs", AUTH);
    api_code.commit("first");
    let cli_code = Code::new();
    cli_code.write("cmd/main.go", "package main\n");
    cli_code.commit("first");
    let docs = Docs::new(&format!(
        "{}[sources.cli]\ngit = \"{}\"\n",
        api(&api_code),
        cli_code.url()
    ));
    docs.page(
        "index.md",
        "---\ntitle: Home\n---\n\n@snippet: api:src/auth.rs\n\n@snippet: cli:cmd/main.go\n",
    );
    // Only the source named.
    let report = fetch(&docs.workspace(), &["cli".into()], &docs.options()).unwrap();
    assert_eq!(report.sources.len(), 1);
    assert!(docs.exists("sources/cli/cmd/main.go") && !docs.exists("sources/api"));
    // Then every one; the one without a branch follows the default.
    fetch(&docs.workspace(), &[], &docs.options()).unwrap();
    assert!(docs.exists("sources/api/src/auth.rs"));
    assert_eq!(docs.check(), Vec::<String>::new());

    assert!(matches!(
        fetch(&docs.workspace(), &["web".into()], &docs.options()),
        Err(SourcesError::UnknownSource(name)) if name == "web"
    ));
}

#[test]
fn no_access_names_the_source_and_repeats_git() {
    let missing = tempfile::tempdir().unwrap();
    let url = common::file_url(&missing.path().join("nothing-here"));
    let docs = Docs::new(&format!("[sources.api]\ngit = \"{url}\"\n"));
    docs.page(
        "index.md",
        "---\ntitle: Home\n---\n\n@snippet: api:src/auth.rs\n",
    );
    let error = fetch(&docs.workspace(), &[], &docs.options()).unwrap_err();
    let SourcesError::Git { name, message } = &error else {
        panic!("{error}");
    };
    assert_eq!(name, "api");
    assert!(!message.is_empty(), "{error}");
    assert!(!docs.exists("ascribe.lock") && !docs.exists("sources"));
}

#[test]
fn a_copy_cant_be_written_through_a_link_out_of_its_folder() {
    let code = Code::new();
    code.write("src/auth.rs", AUTH);
    code.commit("first");
    let docs = Docs::new(&api(&code));
    docs.page(
        "index.md",
        "---\ntitle: Home\n---\n\n@snippet: api:src/auth.rs\n",
    );
    let outside = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(docs.root().join("sources").join("api")).unwrap();
    links::dir(outside.path(), docs.root().join("sources/api/src"));
    let error = fetch(&docs.workspace(), &[], &docs.options()).unwrap_err();
    assert!(matches!(error, SourcesError::Write { .. }), "{error}");
    assert!(!outside.path().join("auth.rs").exists());
}
