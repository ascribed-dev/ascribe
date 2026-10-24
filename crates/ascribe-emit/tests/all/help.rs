//! `_ascribe/help.json`: each permalink's section of the docs, in the site
//! output (output-layout contract §1.7).
//!
//! An entry's text is rendered by the plain renderer, not cut from the
//! page's plain output, so the two must agree: for every project under
//! `examples/` and `tests/conformance/cases/`, in every build, each entry's
//! text is found in its page's plain output, images read as their alt text.
//! An empty text would pass that, so quill's `prerequisites` and `connect`
//! are also held the other way: the page's output under the heading is the
//! entry's text.

#![allow(clippy::expect_used, clippy::panic)]

use crate::support;

use std::path::Path;

use ascribe_emit::site::HELP_PATH;
use ascribe_emit::{Contents, EmitContext, Emitter, PlainEmitter, SiteEmitter, emit_page};
use ascribe_resolve::{AstroRouter, Project};
use serde_json::{Value, json};
use support::{cases, emit_build, example, memory_project, models, repository};

const MODEL: &str = "spec = \"0.1\"\n\
    [types.page.frontmatter]\ntitle = \"string\"\n\
    description = { type = \"string?\", role = \"description\" }\n\
    [dimensions.deployment]\nvalues = [\"cloud\", \"self-managed\"]\n\
    [phrases]\nproduct = \"Quill\"\n\
    [builds.site]\navailability = \"badge\"\n\
    [builds.cloud]\nvariants = { deployment = \"cloud\" }\navailability = \"badge\"\n\
    [consumer]\nsite = \"https://docs.example.com\"\nbase-path = \"/docs/\"\n";

fn file(model: &str, files: &[(&str, &str)], build: &str) -> Value {
    let project = memory_project(model, files);
    let emitter = SiteEmitter::new(project.model());
    let text = emit_build(Path::new("/nowhere"), &project, build, &emitter)
        .remove(HELP_PATH)
        .expect("the site output has the file");
    serde_json::from_str(&text).expect("the file is JSON")
}

fn entry(files: &[(&str, &str)], name: &str) -> Value {
    file(MODEL, files, "site")["help"][name].clone()
}

#[test]
fn a_heading_s_entry_is_its_section() {
    let files = [(
        "keys.md",
        "---\ntitle: Keys\n---\n\n## Rotate `keys`\n@id {permalink=true}: rotate-keys\n\n\
         @summary\nRotate them **often**.\n\n### Why\n\nThey leak.\n\n## After\n\nDone.\n",
    )];
    assert_eq!(
        file(MODEL, &files, "site"),
        json!({
            "version": 1,
            "build": "site",
            "help": {
                "rotate-keys": {
                    "title": "Rotate keys",
                    "summary": "Rotate them **often**.",
                    "text": "Rotate them **often**.\n\n### Why\n\nThey leak.",
                    "address": "https://docs.example.com/docs/go/rotate-keys/",
                    "source": "keys.md",
                },
            },
        })
    );
}

#[test]
fn a_page_s_entry_is_its_opening_with_its_description_or_its_summary() {
    let files = [
        (
            "keys.md",
            "---\ntype: page\ntitle: Keys\npermalink: keys\ndescription: Keys and how to rotate them.\n---\n\n\
             A key signs requests.\n\n## Rotate\n\nRotate them.\n",
        ),
        (
            "plain.md",
            "---\ntitle: Plain\npermalink: plain\n---\n\nThe first paragraph.\n\nThe second.\n",
        ),
        (
            "empty.md",
            "---\ntype: page\ntitle: Empty\npermalink: empty\ndescription: Nothing first.\n---\n\n## Start\n",
        ),
        (
            "marked.md",
            "---\ntitle: Marked\npermalink: marked\n---\n\nThe first paragraph.\n\n@summary\nThe second.\n",
        ),
    ];
    let keys = entry(&files, "keys");
    assert_eq!(keys["title"], "Keys");
    assert_eq!(keys["summary"], "Keys and how to rotate them.");
    assert_eq!(keys["text"], "A key signs requests.");
    // Without a description or a `@summary`, none: nothing is taken from
    // the prose.
    let plain = entry(&files, "plain");
    assert!(plain.get("summary").is_none(), "{plain}");
    assert_eq!(plain["text"], "The first paragraph.\n\nThe second.");
    // Without a description, the opening's `@summary`.
    let marked = entry(&files, "marked");
    assert_eq!(marked["summary"], "The second.");
    assert_eq!(marked["text"], "The first paragraph.\n\nThe second.");
    // An empty opening has an empty text and the summary alone.
    let empty = entry(&files, "empty");
    assert_eq!(empty["text"], "");
    assert_eq!(empty["summary"], "Nothing first.");
}

#[test]
fn a_link_to_the_same_page_is_made_absolute() {
    let files = [(
        "keys.md",
        "---\ntitle: Keys\n---\n\n## Rotate\n@id {permalink=true}: rotate\n\n\
         See [why](#why).\n\n## Why\n\nThey leak.\n",
    )];
    assert_eq!(
        entry(&files, "rotate")["text"],
        "See [why](https://docs.example.com/docs/keys/#why)."
    );
}

#[test]
fn without_a_site_links_stay_routes_and_there_s_no_address() {
    let model = MODEL.replace("site = \"https://docs.example.com\"\n", "");
    let files = [(
        "keys.md",
        "---\ntitle: Keys\n---\n\n## Rotate\n@id {permalink=true}: rotate\n\n\
         See [why](#why).\n\n## Why\n",
    )];
    let help = &file(&model, &files, "site")["help"]["rotate"];
    assert_eq!(help["text"], "See [why](/docs/keys/#why).");
    assert!(help.get("address").is_none(), "{help}");
}

#[test]
fn an_include_its_phrases_and_its_image_are_resolved() {
    // A summary in a fragment is the summary of the section it lands in.
    let files = [
        (
            "install.md",
            "---\ntitle: Install\n---\n\n## Prerequisites\n@id {permalink=true}: prerequisites\n\n\
             @include: _prerequisites.md\n",
        ),
        (
            "_prerequisites.md",
            "@summary\nYou need {product} 3.\n\n![A checklist](checklist.png)\n",
        ),
    ];
    let help = entry(&files, "prerequisites");
    assert_eq!(help["summary"], "You need Quill 3.");
    assert_eq!(help["text"], "You need Quill 3.\n\nA checklist");
}

#[test]
fn a_summary_s_own_text_isn_t_in_the_section_s_text() {
    let files = [(
        "sync.md",
        "---\ntitle: Sync\n---\n\n## Streaming\n@id {permalink=true}: streaming\n\
         @summary: It streams to [{product} Cloud](#bound), `fast`.\n@available: cloud\n\n\
         It streams.\n\n## Bound\n@id {permalink=true}: bound\n\n@available: self-managed\n\
         @summary\nIt binds.\n",
    )];
    let streaming = entry(&files, "streaming");
    assert_eq!(
        streaming["summary"],
        "It streams to [Quill Cloud](https://docs.example.com/docs/sync/#bound), `fast`."
    );
    assert_eq!(streaming["text"], "Available: cloud (GA)\n\nIt streams.");
    // The paragraph a `@summary` marks is the summary, without the line
    // an `@available` above it adds.
    let bound = entry(&files, "bound");
    assert_eq!(bound["summary"], "It binds.");
    assert_eq!(bound["text"], "Available: self-managed (GA)\n\nIt binds.");
}

#[test]
fn a_section_without_a_summary_has_none() {
    let files = [(
        "cli.md",
        "---\ntitle: CLI\n---\n\n## Run\n@id {permalink=true}: run\n\nIt runs.\n\n\
         ### Options\n\n@summary\nA subsection's summary is its own.\n",
    )];
    let help = entry(&files, "run");
    assert!(help.get("summary").is_none(), "{help}");
    assert_eq!(
        help["text"],
        "It runs.\n\n### Options\n\nA subsection's summary is its own."
    );
}

#[test]
fn a_summary_an_include_brings_into_an_arm_is_none() {
    // In a build that keeps the arms and in one that picks the arm, the
    // paragraph it marks stays, as content.
    let files = [
        (
            "install.md",
            "---\ntitle: Install\n---\n\n## Connect\n@id {permalink=true}: connect\n\n\
             @variant {deployment=cloud}:\n@include: _cloud.md\n\
             @variant {deployment=self-managed}:\nPoint it at your server.\n@end\n",
        ),
        ("_cloud.md", "@summary\nSign in to the cloud.\n"),
    ];
    for build in ["site", "cloud"] {
        let help = file(MODEL, &files, build)["help"]["connect"].clone();
        assert!(help.get("summary").is_none(), "{build}: {help}");
        let text = help["text"].as_str().unwrap_or_default();
        assert!(text.contains("Sign in to the cloud."), "{build}: {text}");
    }
}

#[test]
fn an_alias_has_its_permalink_s_entry_and_a_retired_permalink_none() {
    let model = format!(
        "{MODEL}[permalinks.aliases]\nrotate = \"rotate-keys\"\n\
         [permalinks.retired]\n\
         old-rotate = {{ to = \"keys.md#rotate-keys\", reason = \"Renamed\" }}\n\
         legacy = {{ reason = \"Removed\" }}\n"
    );
    let files = [(
        "keys.md",
        "---\ntitle: Keys\n---\n\n## Rotate\n@id {permalink=true}: rotate-keys\n\nRotate them.\n",
    )];
    let help = &file(&model, &files, "site")["help"];
    let names: Vec<&str> = help
        .as_object()
        .expect("a table")
        .keys()
        .map(String::as_str)
        .collect();
    assert_eq!(names, ["rotate", "rotate-keys"]);
    assert_eq!(help["rotate"]["text"], help["rotate-keys"]["text"]);
    assert_eq!(
        help["rotate"]["address"],
        "https://docs.example.com/docs/go/rotate/"
    );
}

#[test]
fn a_fragment_s_heading_is_the_entry_of_the_page_its_permalink_names() {
    let files = [
        (
            "a.md",
            "---\ntitle: A\n---\n\n@include: _rotate.md\n\n## After\n\nDone.\n",
        ),
        ("b.md", "---\ntitle: B\n---\n\n@include: _rotate.md\n"),
        (
            "_rotate.md",
            "## Rotate\n@id {permalink=true}: rotate\n@summary: Rotate them often.\n\nRotate them.\n",
        ),
    ];
    let help = entry(&files, "rotate");
    assert_eq!(help["source"], "a.md");
    assert_eq!(help["summary"], "Rotate them often.");
    assert_eq!(help["text"], "Rotate them.");
}

#[test]
fn each_line_of_a_versioned_build_says_which_it_is() {
    let model = format!(
        "{MODEL}[releases.self-managed]\nlive = [\"3.4\", \"3.5\"]\n\
         [builds.versioned]\navailability = \"badge\"\nversions = \"self-managed\"\n"
    );
    let files = [("index.md", "---\ntitle: Home\n---\n")];
    for (build, line) in [("versioned", "3.5"), ("versioned@3.4", "3.4")] {
        let help = file(&model, &files, build);
        assert_eq!(help["build"], "versioned", "{build}");
        assert_eq!(help["line"], line, "{build}");
    }
    assert!(file(&model, &files, "site").get("line").is_none());
}

#[test]
fn a_build_with_no_permalinks_has_an_empty_table() {
    let files = [("index.md", "---\ntitle: Home\n---\n\nHello.\n")];
    assert_eq!(
        file(MODEL, &files, "cloud"),
        json!({ "version": 1, "build": "cloud", "help": {} })
    );
}

/// A page's plain output with each image read as its alt text, as an
/// entry's text writes it.
fn images_as_alt_text(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(start) = rest.find("![") {
        out.push_str(&rest[..start]);
        let after = &rest[start + 2..];
        let image = after.find("](").and_then(|close| {
            let destination = &after[close + 2..];
            let end = destination.find(')')?;
            Some((&after[..close], close + 2 + end + 1))
        });
        match image {
            Some((alt, length)) => {
                out.push_str(alt);
                rest = &after[length..];
            }
            None => {
                out.push_str("![");
                rest = after;
            }
        }
    }
    out.push_str(rest);
    out
}

/// Each build of `project`'s entries, each with its page's plain output,
/// images read as their alt text: `(build, name, entry, page output)`.
fn entries_with_pages(root: &Path, project: &Project) -> Vec<(String, String, Value, String)> {
    let mut out = Vec::new();
    for build in &project.model().builds {
        let project = project.for_build(build);
        let project = project.as_ref();
        let router = AstroRouter::from_consumer(&project.model().consumer);
        let resolved = project.resolve_build(build, &router);
        let cx = EmitContext::new(project, root, build);
        let Ok(generated) = SiteEmitter::new(project.model()).generated(&cx, &resolved) else {
            continue;
        };
        let help = generated
            .into_iter()
            .find(|f| f.path.as_str() == HELP_PATH)
            .expect("the site output has the file");
        let Contents::Text(text) = help.contents else {
            panic!("the file is text");
        };
        let file: Value = serde_json::from_str(&text).expect("the file is JSON");
        let plain = PlainEmitter::new(project.model());
        for (name, entry) in file["help"].as_object().expect("a table") {
            let source = entry["source"].as_str().expect("a source");
            let page = resolved
                .pages
                .iter()
                .find(|p| p.path.as_str() == source)
                .expect("the entry's page is in the build");
            let emitted = emit_page(&plain, &cx, page).expect("the page renders");
            out.push((
                build.name.clone(),
                name.clone(),
                entry.clone(),
                images_as_alt_text(&emitted.text),
            ));
        }
    }
    out
}

/// Every entry's text that isn't found in its page's plain output.
fn disagreements(root: &Path, name: &str, project: &Project) -> Vec<String> {
    entries_with_pages(root, project)
        .into_iter()
        .filter_map(|(build, permalink, entry, page)| {
            let text = entry["text"].as_str().expect("a text");
            (!page.contains(text)).then(|| {
                format!(
                    "{name}, build {build}, {permalink}: its text isn't in its page's plain output:\n{text}\n--- the page:\n{page}"
                )
            })
        })
        .collect()
}

#[test]
fn every_example_s_entries_agree_with_their_pages() {
    let mut found = Vec::new();
    models(&repository().join("examples"), &mut found);
    assert!(found.len() > 5, "the examples weren't found: {found:?}");
    let mut problems = Vec::new();
    for model in found {
        let (root, project) = example(&model).expect("an example loads");
        problems.extend(disagreements(&root, &model.display().to_string(), &project));
    }
    assert!(problems.is_empty(), "{}", problems.join("\n\n"));
}

#[test]
fn every_conformance_case_s_entries_agree_with_their_pages() {
    let conformance = repository().join("tests/conformance");
    let mut found = Vec::new();
    cases(
        &conformance.join("cases"),
        &conformance.join("_model/ascribe.toml"),
        &mut found,
    );
    assert!(
        found.len() > 100,
        "the cases weren't found: {}",
        found.len()
    );
    let mut problems = Vec::new();
    for (dir, project) in found {
        problems.extend(disagreements(&dir, &dir.display().to_string(), &project));
    }
    assert!(problems.is_empty(), "{}", problems.join("\n\n"));
}

/// The part of a page's plain output from after the heading line `heading`
/// to the next heading at its level or above.
fn after_heading(page: &str, heading: &str) -> String {
    let level = heading.chars().take_while(|c| *c == '#').count();
    let mut lines = page.lines().skip_while(|l| *l != heading);
    assert!(lines.next().is_some(), "the page has {heading:?}:\n{page}");
    let section: Vec<&str> = lines
        .take_while(|l| {
            let marks = l.chars().take_while(|c| *c == '#').count();
            !(marks > 0 && marks <= level && l[marks..].starts_with(' '))
        })
        .collect();
    section.join("\n").trim().to_owned()
}

#[test]
fn quill_s_sections_are_their_pages_text_both_ways() {
    let root = support::quill();
    let project = support::load(&root);
    let entries = entries_with_pages(&root, &project);
    let mut checked = 0;
    for (permalink, heading) in [
        ("prerequisites", "## Prerequisites"),
        ("connect", "## Connect to Quill"),
    ] {
        for (build, name, entry, page) in &entries {
            if name != permalink {
                continue;
            }
            assert_eq!(
                entry["text"].as_str().expect("a text"),
                after_heading(page, heading),
                "{build}, {permalink}"
            );
            checked += 1;
        }
    }
    assert!(checked >= 2, "quill's two permalinks weren't found");
}
