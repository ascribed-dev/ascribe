//! `_ascribe/help.json`: each permalink's section of the docs, in the site
//! output (output-layout contract §1.7).
//!
//! An entry's text is rendered by the plain renderer, not cut from the
//! page's plain output, so the two must agree: for every project under
//! `examples/` and `tests/conformance/cases/`, in every build, each entry's
//! text is found in its page's plain output, images read as their alt text.
//! An empty text would pass that, so quill's `prerequisites` and `connect`
//! are also held the other way: the page's output under the heading is the
//! entry's text. Each of an entry's choices is held the same way, to the
//! page's plain output with that value selected, and to a build's that
//! selects it where the project has one.

#![allow(clippy::expect_used, clippy::panic)]

use crate::support;

use std::path::Path;

use ascribe_emit::site::HELP_PATH;
use ascribe_emit::{Contents, EmitContext, Emitter, PlainEmitter, SiteEmitter, emit_page};
use ascribe_model::VariantMode;
use ascribe_resolve::{AstroRouter, Project, ResolvedPage, select};
use serde_json::{Value, json};
use support::{cases, emit_build, example, memory_project, models, repository};

const MODEL: &str = "spec = \"0.1\"\n\
    [types.page.frontmatter]\ntitle = \"string\"\n\
    description = { type = \"string?\", role = \"description\" }\n\
    [dimensions.deployment]\nvalues = [\"cloud\", \"self-managed\"]\n\
    [dimensions.pm]\nlabel = \"Package manager\"\nvalues = [\"npm\", \"pnpm\", \"yarn\"]\n\
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

/// The goal: a product that knows its reader uses pnpm shows the section
/// with the pnpm arm alone, as a build that selects it writes it.
#[test]
fn a_switcher_s_section_has_its_text_with_each_arm_chosen() {
    let files = [(
        "install.md",
        "---\ntitle: Install\n---\n\n## Install\n@id {permalink=true}: install\n\n\
         @summary\nInstall the agent.\n\n\
         @variant {pm=npm}:\n```sh\nnpm install quill\n```\n\
         @variant {pm=pnpm|yarn}:\nUse your package manager's `add`.\n\n\
         ### Pinning\n\nPin the version.\n\
         @end\n",
    )];
    let help = entry(&files, "install");
    assert_eq!(help["summary"], "Install the agent.");
    assert_eq!(
        help["choices"],
        json!({
            "pm": {
                "npm": "Install the agent.\n\n```sh\nnpm install quill\n```",
                "pnpm": "Install the agent.\n\nUse your package manager's `add`.\n\n### Pinning\n\nPin the version.",
                "yarn": "Install the agent.\n\nUse your package manager's `add`.\n\n### Pinning\n\nPin the version.",
            },
        })
    );
    // The values are in the dimension's order.
    let values: Vec<&str> = help["choices"]["pm"]
        .as_object()
        .expect("a table")
        .keys()
        .map(String::as_str)
        .collect();
    assert_eq!(values, ["npm", "pnpm", "yarn"]);
    // `text` is the switcher, as before.
    assert!(
        help["text"].as_str().expect("a text").contains(
            "A reader sees one of the next 2 sections at a time, chosen by package manager."
        ),
        "{help}"
    );
}

#[test]
fn each_dimension_varies_its_own_switcher_and_leaves_the_other() {
    let files = [(
        "setup.md",
        "---\ntitle: Setup\n---\n\n## Setup\n@id {permalink=true}: setup\n\n\
         @variant {pm=npm}:\nWith npm.\n@variant {pm=pnpm}:\nWith pnpm.\n@end\n\n\
         @variant {deployment=cloud}:\nOn cloud.\n@variant {deployment=self-managed}:\nOn your server.\n@end\n",
    )];
    let choices = &entry(&files, "setup")["choices"];
    let dimensions: Vec<&str> = choices
        .as_object()
        .expect("a table")
        .keys()
        .map(String::as_str)
        .collect();
    assert_eq!(dimensions, ["deployment", "pm"]);
    let deployment = "A reader sees one of the next 2 sections at a time, chosen by deployment.\n\n\
        ### deployment: cloud\n\nOn cloud.\n\n### deployment: self-managed\n\nOn your server.";
    let pm = "A reader sees one of the next 2 sections at a time, chosen by package manager.\n\n\
        ### Package manager: npm\n\nWith npm.\n\n### Package manager: pnpm\n\nWith pnpm.";
    assert_eq!(choices["pm"]["pnpm"], format!("With pnpm.\n\n{deployment}"));
    assert_eq!(choices["deployment"]["cloud"], format!("{pm}\n\nOn cloud."));
}

#[test]
fn a_labeled_group_is_no_choice_and_stays_in_each() {
    let labeled = "A reader sees one of the next 2 sections at a time.\n\n\
        ### Hardened images\n\nHardened.\n\n### Other images\n\nOther.";
    let files = [(
        "images.md",
        "---\ntitle: Images\n---\n\n## Images\n@id {permalink=true}: images\n\n\
         .Hardened images\n@variant:\nHardened.\n\n.Other images\n@variant:\nOther.\n@end\n\n\
         ## Run\n@id {permalink=true}: run\n\n\
         .Hardened images\n@variant:\nHardened.\n\n.Other images\n@variant:\nOther.\n@end\n\n\
         @variant {pm=npm}:\nWith npm.\n@variant {pm=pnpm}:\nWith pnpm.\n@end\n",
    )];
    let images = entry(&files, "images");
    assert_eq!(images["text"], labeled);
    assert!(images.get("choices").is_none(), "{images}");
    let run = entry(&files, "run");
    assert_eq!(
        run["choices"]["pm"]["npm"],
        format!("{labeled}\n\nWith npm.")
    );
}

#[test]
fn a_section_with_no_switcher_has_no_choices() {
    let files = [(
        "keys.md",
        "---\ntitle: Keys\n---\n\n## Rotate\n@id {permalink=true}: rotate\n\nRotate them.\n\n\
         ## Cloud\n@id {permalink=true}: cloud\n\n@variant {deployment=cloud}:\nOn cloud.\n@end\n",
    )];
    let help = file(MODEL, &files, "site");
    for name in ["rotate", "cloud"] {
        let entry = &help["help"][name];
        assert!(entry.get("choices").is_none(), "{name}: {entry}");
    }
    // A build that selects the dimension has reduced the switcher already.
    let files = [(
        "setup.md",
        "---\ntitle: Setup\n---\n\n## Setup\n@id {permalink=true}: setup\n\n\
         @variant {deployment=cloud}:\nOn cloud.\n@variant {deployment=self-managed}:\nOn your server.\n@end\n",
    )];
    let setup = &file(MODEL, &files, "cloud")["help"]["setup"];
    assert_eq!(setup["text"], "On cloud.");
    assert!(setup.get("choices").is_none(), "{setup}");
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

/// A text of an entry and the plain output it must be found in, images
/// read as their alt text.
struct Held {
    build: String,
    /// The permalink's name.
    name: String,
    /// What the text is: `text`, or a choice, as `choices.pm.pnpm`.
    what: String,
    text: String,
    page: String,
}

/// Each build of `project`'s entries with the plain output each text must be
/// found in: `text` in its page's plain output, and each choice in the
/// page's output in a build that selects that value too and is otherwise
/// the same as this one, where the project has one, and in any case in the
/// page's plain output with its blocks selected so.
fn entries_with_pages(root: &Path, project: &Project) -> Vec<Held> {
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
        let render = |cx: &EmitContext<'_>, page: &ResolvedPage| {
            let emitted = emit_page(&plain, cx, page).expect("the page renders");
            images_as_alt_text(&emitted.text)
        };
        for (name, entry) in file["help"].as_object().expect("a table") {
            let source = entry["source"].as_str().expect("a source");
            let page = resolved
                .pages
                .iter()
                .find(|p| p.path.as_str() == source)
                .expect("the entry's page is in the build");
            let held = |what: String, text: &Value, page: String| Held {
                build: build.name.clone(),
                name: name.clone(),
                what,
                text: text.as_str().expect("a text").to_owned(),
                page,
            };
            out.push(held("text".to_owned(), &entry["text"], render(&cx, page)));
            let Some(choices) = entry.get("choices") else {
                continue;
            };
            for (dimension, values) in choices.as_object().expect("a table") {
                for (value, text) in values.as_object().expect("a table") {
                    let what = format!("choices.{dimension}.{value}");
                    let selection = vec![(dimension.clone(), vec![value.clone()])];
                    let mut chosen = page.clone();
                    chosen.blocks = select(&page.blocks, &selection, project.model());
                    out.push(held(what.clone(), text, render(&cx, &chosen)));
                    // This build's selection, with the value added.
                    let mut wanted = match &build.variants {
                        VariantMode::Select(selected) => selected.clone(),
                        VariantMode::Switch => Vec::new(),
                    };
                    wanted.retain(|(d, _)| d != dimension);
                    wanted.extend(selection.iter().cloned());
                    wanted.sort();
                    let selecting = project.model().builds.iter().find(|b| {
                        let VariantMode::Select(selected) = &b.variants else {
                            return false;
                        };
                        let mut selected = selected.clone();
                        selected.sort();
                        selected == wanted
                            && b.availability == build.availability
                            && b.versions == build.versions
                            && b.line == build.line
                    });
                    if let Some(selecting) = selecting {
                        let resolved = project.resolve_build(selecting, &router);
                        let cx = EmitContext::new(project, root, selecting);
                        let page = resolved
                            .pages
                            .iter()
                            .find(|p| p.path.as_str() == source)
                            .expect("a build that selects a value keeps a page that has it");
                        out.push(held(
                            format!("{what}, in build {}", selecting.name),
                            text,
                            render(&cx, page),
                        ));
                    }
                }
            }
        }
    }
    out
}

/// Every entry's text that isn't found in the plain output it must be.
fn disagreements(root: &Path, name: &str, project: &Project) -> Vec<String> {
    entries_with_pages(root, project)
        .into_iter()
        .filter(|held| !held.page.contains(&held.text))
        .map(|Held { build, name: permalink, what, text, page }| {
            format!(
                "{name}, build {build}, {permalink}: its {what} isn't in its page's plain output:\n{text}\n--- the page:\n{page}"
            )
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
        for held in &entries {
            if held.name != permalink || held.what != "text" {
                continue;
            }
            assert_eq!(
                held.text,
                after_heading(&held.page, heading),
                "{}, {permalink}",
                held.build
            );
            checked += 1;
        }
    }
    assert!(checked >= 2, "quill's two permalinks weren't found");
}
