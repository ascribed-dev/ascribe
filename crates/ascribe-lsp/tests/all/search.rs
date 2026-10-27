//! `ascribe/search`, over an in-memory connection, on a copy of
//! examples/quill: the places `ascribe search` finds, from the server's
//! snapshot, unsaved text included.

#![allow(clippy::expect_used, clippy::panic, clippy::unwrap_used)]

use std::path::Path;

use ascribe_query::{Readings, Search};
use serde_json::{Value, json};

use crate::support::{self, Client, Fixture};

fn copy(from: &Path, to: &Path) {
    for entry in std::fs::read_dir(from).expect("a directory") {
        let entry = entry.expect("an entry");
        let target = to.join(entry.file_name());
        if entry.file_type().expect("a type").is_dir() {
            std::fs::create_dir_all(&target).expect("mkdir");
            copy(&entry.path(), &target);
        } else {
            std::fs::copy(entry.path(), &target).expect("copy");
        }
    }
}

fn quill() -> Fixture {
    let fixture = Fixture::new("", &[]);
    let source = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/quill");
    copy(&source, &fixture.root());
    fixture
}

fn search(client: &mut Client, through: &Path, query: &str, build: Option<&str>) -> Value {
    let mut params = json!({
        "textDocument": { "uri": support::uri(through).as_str() },
        "query": query,
    });
    if let Some(build) = build {
        params["build"] = json!(build);
    }
    client
        .request("ascribe/search", params)
        .response_result
        .expect("the request succeeds")
}

/// Each place as `file:line`, its line from 1.
fn lines(result: &Value) -> Vec<String> {
    result["places"]
        .as_array()
        .expect("a list")
        .iter()
        .map(|p| {
            format!(
                "{}:{}",
                p["file"].as_str().unwrap(),
                p["range"]["start"]["line"].as_u64().unwrap() + 1
            )
        })
        .collect()
}

fn started(f: &Fixture) -> Client {
    let mut client = Client::start(&f.root());
    client.settle();
    client
}

#[test]
fn the_thread_query_opens_line_62_through_the_phrase() {
    let f = quill();
    let mut client = started(&f);
    let result = search(
        &mut client,
        &f.path("ascribe.toml"),
        "\"sign in to quill cloud\"",
        None,
    );
    assert_eq!(result["problem"], Value::Null);
    let first = &result["places"][0];
    assert_eq!(first["file"], "install-agent.md");
    assert_eq!(
        first["uri"],
        support::uri(&f.path("docs/install-agent.md")).as_str()
    );
    assert_eq!(
        first["range"]["start"],
        json!({ "line": 61, "character": 0 })
    );
    assert_eq!(first["in"], "text");
    assert_eq!(first["through"], json!([{ "phrase": "cloud" }]));
    assert_eq!(first["arm"], "deployment=cloud");
    assert_eq!(first["match"], json!([0, 22]));
    assert_eq!(first["pages"][0]["heading"], "Connect to Quill");
    assert_eq!(first["pagesTotal"], 1);
    assert_eq!(first["builds"], json!([]), "every build has it");
    // The range runs to the end of the line it starts on.
    let text = std::fs::read_to_string(f.path("docs/install-agent.md")).unwrap();
    let line = text.lines().nth(61).unwrap();
    assert_eq!(
        first["range"]["end"],
        json!({ "line": 61, "character": line.encode_utf16().count() })
    );
}

#[test]
fn a_match_wholly_in_a_value_opens_the_phrase_in_ascribe_toml() {
    let f = quill();
    let mut client = started(&f);
    let result = search(
        &mut client,
        &f.path("docs/index.md"),
        "\"quill cloud\"",
        None,
    );
    let phrase = result["places"]
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["in"] == "phrase")
        .expect("the phrase is a place");
    assert_eq!(phrase["file"], "ascribe.toml");
    assert_eq!(
        phrase["uri"],
        support::uri(&f.path("ascribe.toml")).as_str()
    );
    assert_eq!(phrase["range"]["start"]["line"], 33);
    assert_eq!(phrase["text"], "Quill Cloud");
    assert!(phrase["pagesTotal"].as_u64().unwrap() > 1);
}

#[test]
fn the_places_are_the_commands() {
    let f = quill();
    let mut client = started(&f);
    let project = ascribe_check::load(&f.path("ascribe.toml"))
        .map(|w| ascribe_check::Project::new(w.snapshot()))
        .unwrap();
    for query in ["quill", "\"sign in\"", "streaming sync", "key"] {
        for build in [None, Some("cloud")] {
            let builds: Vec<String> = build.into_iter().map(str::to_owned).collect();
            let command = ascribe_query::search(
                &project,
                project.index(),
                &Readings::default(),
                &Search {
                    query,
                    builds: &builds,
                    scope: None,
                    page_type: None,
                    case: false,
                    word: false,
                    summary: false,
                    limit: 50,
                },
            )
            .unwrap();
            let result = search(&mut client, &f.path("ascribe.toml"), query, build);
            let places = result["places"].as_array().unwrap();
            assert_eq!(result["total"], command.total, "{query} {build:?}");
            assert_eq!(places.len(), command.places.len(), "{query} {build:?}");
            for (place, expected) in places.iter().zip(&command.places) {
                assert_eq!(place["in"], expected.kind, "{query}");
                assert_eq!(place["text"], expected.text, "{query}");
                assert_eq!(place["match"], json!(expected.matched), "{query}");
                assert_eq!(
                    place["range"]["start"]["line"],
                    expected.line - 1,
                    "{query}: {}",
                    expected.file
                );
                assert!(
                    expected.file.ends_with(place["file"].as_str().unwrap()),
                    "{} names {}",
                    expected.file,
                    place["file"]
                );
            }
        }
    }
}

#[test]
fn unsaved_text_is_searched_and_read_again_after_each_edit() {
    let f = quill();
    let mut client = started(&f);
    let page = f.path("docs/keys.md");
    let text = std::fs::read_to_string(&page).unwrap();
    // Read every build before the edit, so what's kept must be made again.
    let before = search(&mut client, &page, "zebrafinch", None);
    assert_eq!(before["total"], 0);
    assert_eq!(before["nothing"], "The words aren't in any page.");

    client.open(&page, 1, &text);
    client.replace(&page, 2, &format!("{text}\nA zebrafinch sings.\n"));
    client.settle();
    let after = search(&mut client, &page, "zebrafinch", None);
    assert_eq!(after["total"], 1, "{after}");
    assert_eq!(after["places"][0]["file"], "keys.md");
    assert_eq!(after["places"][0]["text"], "A zebrafinch sings.");

    client.replace(&page, 3, &format!("{text}\nA kingfisher dives.\n"));
    client.settle();
    let edited = search(&mut client, &page, "zebrafinch", None);
    assert_eq!(edited["total"], 0, "{edited}");
    let found = search(&mut client, &page, "kingfisher", Some("site"));
    assert_eq!(lines(&found).len(), 1, "{found}");
}

#[test]
fn a_build_other_than_the_editors_and_every_build() {
    let f = quill();
    let mut client = started(&f);
    let query = "\"point the agent at your server\"";
    let cloud = search(&mut client, &f.path("ascribe.toml"), query, Some("cloud"));
    assert_eq!(cloud["builds"], json!(["cloud"]));
    assert_eq!(cloud["total"], 0);
    assert_eq!(
        cloud["nothing"],
        "Builds not searched publish them: `site`, `site@3.4`."
    );
    let every = search(&mut client, &f.path("ascribe.toml"), query, None);
    assert_eq!(lines(&every), ["install-agent.md:70"]);
    assert_eq!(every["places"][0]["builds"], json!(["site", "site@3.4"]));
    let site = search(&mut client, &f.path("ascribe.toml"), query, Some("site"));
    assert_eq!(site["builds"], json!(["site"]));
    assert_eq!(lines(&site), ["install-agent.md:70"]);
}

#[test]
fn words_written_but_published_by_no_build_say_where() {
    let f = quill();
    let mut client = started(&f);
    // The phrase's key, as the source spells it, which no reader reads.
    let result = search(&mut client, &f.path("ascribe.toml"), "\"{cloud}\"", None);
    assert_eq!(result["total"], 0);
    let nothing = result["nothing"].as_str().unwrap();
    assert!(
        nothing.starts_with("No build searched publishes them. They're written at `docs/"),
        "{nothing}"
    );
}

#[test]
fn a_query_or_a_build_that_cant_be_used_is_a_problem() {
    let f = quill();
    let mut client = started(&f);
    let open = search(&mut client, &f.path("ascribe.toml"), "\"sign in", None);
    assert_eq!(
        open["problem"],
        "The query can't be read: a quote in `\"sign in` isn't closed."
    );
    assert_eq!(open["places"], json!([]));
    let unknown = search(&mut client, &f.path("ascribe.toml"), "quill", Some("nope"));
    assert!(
        unknown["problem"]
            .as_str()
            .unwrap()
            .starts_with("The content model has no build named nope."),
        "{unknown}"
    );
}
