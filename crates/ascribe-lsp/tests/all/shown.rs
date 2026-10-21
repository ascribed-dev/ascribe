//! "Shown in N pages", in a code file a `@snippet` shows, against a copy of
//! `examples/tested`: the registration for the sources' files, the lens on
//! a region and on a whole file, the places it lists, and the lenses asked
//! for again when a file changes on disk.

#![allow(clippy::expect_used, clippy::panic, clippy::unwrap_used)]

use std::path::Path;

use crate::support::{Client, Fixture, Setup, uri};
use lsp_types::FileChangeType;
use serde_json::{Value, json};

fn copy_dir(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).expect("mkdir");
    for entry in std::fs::read_dir(from).expect("read_dir").flatten() {
        let target = to.join(entry.file_name());
        if entry.path().is_dir() {
            copy_dir(&entry.path(), &target);
        } else {
            std::fs::copy(entry.path(), target).expect("copy");
        }
    }
}

/// A copy of `examples/tested`.
fn tested() -> Fixture {
    let source = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/tested");
    let f = Fixture::new("", &[]);
    copy_dir(&source, &f.root());
    f
}

fn start(f: &Fixture) -> Client {
    start_in(&f.root())
}

fn start_in(folder: &Path) -> Client {
    let mut client = Client::start_with(
        folder,
        Setup {
            lenses: true,
            ..Setup::default()
        },
    );
    client.settle();
    client
}

fn lenses(client: &mut Client, path: &Path) -> Value {
    client
        .request(
            "textDocument/codeLens",
            json!({ "textDocument": { "uri": uri(path).as_str() } }),
        )
        .response_result
        .expect("a result")
}

/// Each lens's line and title.
fn shown(lenses: &Value) -> Vec<(u64, String)> {
    lenses
        .as_array()
        .unwrap_or_else(|| panic!("lenses: {lenses}"))
        .iter()
        .map(|lens| {
            (
                lens["range"]["start"]["line"].as_u64().unwrap(),
                lens["command"]["title"].as_str().unwrap().to_owned(),
            )
        })
        .collect()
}

/// The places a lens lists: each file's name and line.
fn places(lens: &Value) -> Vec<(String, u64)> {
    lens["command"]["arguments"][2]
        .as_array()
        .unwrap()
        .iter()
        .map(|location| {
            let uri = location["uri"].as_str().unwrap();
            (
                uri.rsplit('/').next().unwrap().to_owned(),
                location["range"]["start"]["line"].as_u64().unwrap(),
            )
        })
        .collect()
}

/// The patterns of each lens registration the server asked for.
fn registrations(client: &Client) -> Vec<Vec<String>> {
    client
        .requests_from_server
        .iter()
        .filter(|r| r.method == "client/registerCapability")
        .flat_map(|r| r.params["registrations"].as_array().unwrap().clone())
        .filter(|r| r["method"] == "textDocument/codeLens")
        .map(|r| {
            r["registerOptions"]["documentSelector"]
                .as_array()
                .unwrap()
                .iter()
                .map(|filter| filter["pattern"].as_str().unwrap().to_owned())
                .collect()
        })
        .collect()
}

fn glob(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

#[test]
fn the_lenses_are_registered_for_the_pages_and_the_sources_files() {
    let f = tested();
    let mut client = start(&f);
    // A client that registers lenses isn't told the server has them.
    assert!(client.initialize_result["capabilities"]["codeLensProvider"].is_null());
    let root = glob(&f.root());
    assert_eq!(
        registrations(&client),
        [vec![format!("{root}/**"), format!("{root}/code/**")]]
    );

    // A source moved is registered again, the old registration dropped.
    let model = std::fs::read_to_string(f.path("ascribe.toml")).unwrap();
    client.open(
        &f.path("ascribe.toml"),
        1,
        &model.replace(
            "path = \"code\"",
            "path = \"code\"\ninclude = [\"examples/**\"]",
        ),
    );
    client.settle();
    assert_eq!(
        registrations(&client).last().unwrap(),
        &vec![format!("{root}/**"), format!("{root}/code/examples/**")]
    );
    assert!(
        client
            .requests_from_server
            .iter()
            .any(|r| r.method == "client/unregisterCapability")
    );
}

#[test]
fn a_client_that_registers_nothing_gets_lenses_in_the_pages_it_sends() {
    let f = tested();
    let mut client = Client::start(&f.root());
    client.settle();
    assert!(client.initialize_result["capabilities"]["codeLensProvider"].is_object());
    assert!(registrations(&client).is_empty());
}

#[test]
fn a_code_file_in_a_source_gets_a_lens_and_one_outside_gets_none() {
    let f = tested();
    f.write(
        "other/greet.ts",
        "// :snippet-start: greet\n// :snippet-end:\n",
    );
    let mut client = start(&f);
    let found = lenses(&mut client, &f.path("code/examples/greet.ts"));
    assert_eq!(shown(&found), [(0, "Shown in 1 page".to_owned())]);
    assert!(lenses(&mut client, &f.path("other/greet.ts")).is_null());
}

#[test]
fn a_lens_on_a_region_lists_the_snippet_that_shows_it() {
    let f = tested();
    let mut client = start(&f);
    let path = f.path("code/examples/greet.ts");
    let found = lenses(&mut client, &path);
    let lens = &found[0];
    assert_eq!(lens["command"]["command"], "ascribe.showReferences");
    assert_eq!(lens["command"]["arguments"][0], uri(&path).as_str());
    assert_eq!(
        lens["command"]["arguments"][1],
        json!({ "line": 0, "character": 0 })
    );
    // index.md's `@snippet: code:examples/greet.ts#greet`, on its line 6.
    assert_eq!(places(lens), [("index.md".to_owned(), 6)]);
}

#[test]
fn a_whole_file_gets_a_lens_on_its_first_line() {
    let f = tested();
    let mut client = start(&f);
    let found = lenses(&mut client, &f.path("code/examples/greet.out.txt"));
    assert_eq!(shown(&found), [(0, "Shown in 1 page".to_owned())]);
    assert_eq!(places(&found[0]), [("index.md".to_owned(), 10)]);
}

#[test]
fn a_fragments_snippet_counts_each_page_that_includes_it() {
    let f = tested();
    f.write(
        "docs/_greet.md",
        "## Greet\n\n@snippet: code:examples/greet.ts#greet\n",
    );
    for page in ["a", "b"] {
        f.write(
            &format!("docs/{page}.md"),
            &format!("---\ntitle: {page}\n---\n\n@include: _greet.md\n"),
        );
    }
    let mut client = start(&f);
    let found = lenses(&mut client, &f.path("code/examples/greet.ts"));
    assert_eq!(shown(&found), [(0, "Shown in 3 pages".to_owned())]);
    assert_eq!(
        places(&found[0]),
        [("_greet.md".to_owned(), 2), ("index.md".to_owned(), 6)]
    );
}

#[test]
fn a_file_two_pages_show_is_shown_in_two_pages() {
    let f = tested();
    f.write(
        "docs/again.md",
        "---\ntitle: Again\n---\n\n@snippet {lang=text}: code:examples/greet.out.txt\n",
    );
    let mut client = start(&f);
    let found = lenses(&mut client, &f.path("code/examples/greet.out.txt"));
    assert_eq!(shown(&found), [(0, "Shown in 2 pages".to_owned())]);
}

#[test]
fn a_lens_is_placed_from_the_saved_file_and_asked_for_again_when_it_changes() {
    let f = tested();
    let mut client = start(&f);
    let path = f.path("code/examples/greet.ts");
    // What the editor holds isn't read: the client doesn't send code files,
    // and the lens stays where the saved file has the region.
    let text = std::fs::read_to_string(&path).unwrap();
    client.open(&path, 1, &format!("// one\n// two\n{text}"));
    assert_eq!(shown(&lenses(&mut client, &path))[0].0, 0);

    let before = client.requests_from_server.len();
    f.write("code/examples/greet.ts", &format!("// one\n// two\n{text}"));
    client.watched(&[(&path, FileChangeType::CHANGED)]);
    client.settle();
    assert!(
        client.requests_from_server[before..]
            .iter()
            .any(|r| r.method == "workspace/codeLens/refresh")
    );
    assert_eq!(shown(&lenses(&mut client, &path))[0].0, 2);
}

#[test]
fn a_file_its_source_doesnt_include_asks_for_no_lens_again() {
    // The docs in docs/, with the repository around them as their source,
    // as Ascribe's own are: a build writes target/ in the source's folder.
    let f = Fixture::new("spec = \"0.1\"\n", &[]);
    std::fs::create_dir_all(f.path(".git")).unwrap();
    f.write(
        "docs/ascribe.toml",
        "spec = \"0.1\"\n\n[project]\ncontent-root = \"docs\"\n\n[sources.code]\npath = \"..\"\ninclude = [\"code/**\"]\n",
    );
    f.write(
        "docs/docs/index.md",
        "---\ntitle: Greet\n---\n\n@snippet {lang=text}: code:code/greet.txt\n",
    );
    f.write("code/greet.txt", "Hello\n");
    let mut client = start_in(&f.path("docs"));
    let refreshes = |client: &Client, from: usize| {
        client.requests_from_server[from..]
            .iter()
            .filter(|r| r.method == "workspace/codeLens/refresh")
            .count()
    };

    let before = client.requests_from_server.len();
    f.write("target/debug/build/a.d", "x\n");
    f.write("code-old/greet.txt", "Hello\n");
    client.watched(&[
        (&f.path("target"), FileChangeType::CREATED),
        (&f.path("target/debug/build/a.d"), FileChangeType::CHANGED),
        (&f.path("code-old/greet.txt"), FileChangeType::CREATED),
    ]);
    client.settle();
    assert_eq!(refreshes(&client, before), 0);

    // The file the page shows, and the source's examples, ask again.
    for file in ["code/greet.txt", "ascribe.examples.toml"] {
        let before = client.requests_from_server.len();
        f.write(file, "Hello\n");
        client.watched(&[(&f.path(file), FileChangeType::CHANGED)]);
        client.settle();
        assert_eq!(refreshes(&client, before), 1, "{file}");
    }
}

#[test]
fn a_page_in_a_source_gets_its_include_lenses_once() {
    let f = tested();
    let model = std::fs::read_to_string(f.path("ascribe.toml")).unwrap();
    f.write(
        "ascribe.toml",
        &model.replace(
            "[sources.code]\npath = \"code\"",
            "[sources.code]\npath = \".\"",
        ),
    );
    f.write("docs/_part.md", "Part.\n");
    f.write(
        "docs/whole.md",
        "---\ntitle: Whole\n---\n\n@include: _part.md\n",
    );
    let mut client = start(&f);
    let found = lenses(&mut client, &f.path("docs/whole.md"));
    assert_eq!(shown(&found), [(4, "Includes _part.md".to_owned())]);
}
