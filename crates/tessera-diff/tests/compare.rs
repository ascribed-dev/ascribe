//! The comparison, on projects held in memory: each kind of change, nesting,
//! moves, the size limits, and pages that change only through something they
//! use.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::sync::Arc;

use tessera_core::FileId;
use tessera_diff::{BuildDiff, Change, ChangeKind, PageDiff, PageStatus, Side, compare_builds};
use tessera_resolve::{Layout, MemoryFs, Project};

const MODEL: &str = "spec = \"0.1\"\n[phrases]\nproduct = \"Quill\"\n[builds.site]\n";

struct Version {
    model: String,
    project: Project,
}

fn version(model: &str, files: &[(&str, &str)]) -> Version {
    let loaded = tessera_model::load_str(model, FileId::new(0)).expect("a valid model");
    let layout = Layout::from_model(&loaded);
    let mut fs = MemoryFs::new(&layout);
    for (path, text) in files {
        fs = fs.with_source(path, text);
    }
    Version {
        model: model.to_owned(),
        project: Project::load(Arc::new(loaded), layout, &fs),
    }
}

fn side(v: &Version) -> Side<'_> {
    Side {
        project: &v.project,
        model_text: &v.model,
    }
}

fn diff(before: &Version, after: &Version) -> Vec<BuildDiff> {
    let names: Vec<String> = after
        .project
        .model()
        .builds
        .iter()
        .map(|b| b.name.clone())
        .collect();
    let names: Vec<&str> = names.iter().map(String::as_str).collect();
    compare_builds(Some(side(before)), side(after), &names)
}

/// The one changed page of the one build.
fn page(before: &Version, after: &Version) -> PageDiff {
    let builds = diff(before, after);
    assert_eq!(builds.len(), 1);
    assert_eq!(builds[0].pages.len(), 1, "{:#?}", builds[0].pages);
    builds[0].pages[0].clone()
}

fn kinds(page: &PageDiff) -> Vec<(ChangeKind, Option<String>, Option<String>)> {
    page.changes
        .iter()
        .map(|c| {
            (
                c.kind,
                c.was.as_ref().map(|a| a.source.clone()),
                c.now.as_ref().map(|a| a.source.clone()),
            )
        })
        .collect()
}

fn only_change(page: &PageDiff) -> &Change {
    assert_eq!(page.changes.len(), 1, "{:#?}", page.changes);
    &page.changes[0]
}

fn slice(text: &str, [a, b]: [usize; 2]) -> String {
    text.chars().skip(a).take(b - a).collect()
}

const PAGE: &str = "---\ntitle: Install\n---\n\n# Install\n\nFirst paragraph about installing.\n\nSecond paragraph with more words.\n\nThird paragraph at the end.\n";

#[test]
fn an_unchanged_project_has_no_changes() {
    let a = version(MODEL, &[("install.md", PAGE)]);
    let b = version(MODEL, &[("install.md", PAGE)]);
    let builds = diff(&a, &b);
    assert_eq!(builds.len(), 1);
    assert!(builds[0].pages.is_empty());
}

#[test]
fn rewrapping_and_moving_lines_down_is_no_change() {
    let a = version(MODEL, &[("install.md", PAGE)]);
    let rewrapped = "---\ntitle: Install\n---\n\n\n\n#   Install\n\nFirst paragraph\nabout   installing.\n\nSecond paragraph\nwith more words.\n\nThird paragraph at the end.\n";
    let b = version(MODEL, &[("install.md", rewrapped)]);
    assert!(diff(&a, &b)[0].pages.is_empty());
}

#[test]
fn a_changed_word_marks_the_block_and_the_words() {
    let a = version(MODEL, &[("install.md", PAGE)]);
    let b = version(
        MODEL,
        &[(
            "install.md",
            &PAGE.replace("with more words", "with fewer words"),
        )],
    );
    let page = page(&a, &b);
    assert_eq!(page.status, PageStatus::Changed);
    assert!(page.own_file_changed);
    assert!(page.because.is_empty());
    assert_eq!(page.route, "/install/");
    let change = only_change(&page);
    assert_eq!(change.kind, ChangeKind::Changed);
    let now = change.now.as_ref().unwrap();
    assert_eq!(now.source, "install.md:9-9");
    assert!(now.via.is_empty());
    let words = change.words.as_ref().unwrap();
    assert_eq!(words.now_text, "Second paragraph with fewer words.");
    let now_words: Vec<String> = words
        .now
        .iter()
        .map(|r| slice(&words.now_text, *r))
        .collect();
    let was_words: Vec<String> = words
        .was
        .iter()
        .map(|r| slice(&words.was_text, *r))
        .collect();
    assert_eq!(now_words, ["fewer"]);
    assert_eq!(was_words, ["more"]);
    assert_eq!(page.counts.changed, 1);
}

#[test]
fn added_and_removed_blocks() {
    let a = version(MODEL, &[("install.md", PAGE)]);
    let b = version(
        MODEL,
        &[(
            "install.md",
            &PAGE
                .replace("Second paragraph with more words.\n\n", "")
                .replace(
                    "Third paragraph at the end.\n",
                    "Third paragraph at the end.\n\nA brand new closing remark here.\n",
                ),
        )],
    );
    let page = page(&a, &b);
    assert_eq!(
        kinds(&page),
        vec![
            (ChangeKind::Removed, Some("install.md:9-9".into()), None),
            (ChangeKind::Added, None, Some("install.md:11-11".into())),
        ]
    );
    let removed = &page.changes[0];
    // It came after the first paragraph, at the top of the page.
    assert_eq!(removed.after.as_ref().unwrap().source, "install.md:7-7");
    assert!(removed.parent.is_none());
    assert_eq!(
        removed.text.as_deref(),
        Some("Second paragraph with more words.")
    );
    assert_eq!(page.counts.added, 1);
    assert_eq!(page.counts.removed, 1);
}

#[test]
fn a_moved_block_links_its_two_ends() {
    let a = version(MODEL, &[("install.md", PAGE)]);
    let moved = "---\ntitle: Install\n---\n\n# Install\n\nSecond paragraph with more words.\n\nFirst paragraph about installing.\n\nThird paragraph at the end.\n";
    let b = version(MODEL, &[("install.md", moved)]);
    let page = page(&a, &b);
    let change = only_change(&page);
    // Of two blocks that swapped places, one stayed and the other moved.
    assert_eq!(change.kind, ChangeKind::Moved);
    assert_eq!(change.was.as_ref().unwrap().source, "install.md:9-9");
    assert_eq!(change.now.as_ref().unwrap().source, "install.md:7-7");
    // Its old place: after the first paragraph, which is now on line 9.
    assert_eq!(change.after.as_ref().unwrap().source, "install.md:9-9");
    assert_eq!(page.counts.moved, 1);
}

#[test]
fn a_block_moved_into_a_container_is_a_move() {
    let a = version(
        MODEL,
        &[(
            "install.md",
            "# Install\n\nA paragraph that moves.\n\n@note:\nStay here.\n@end\n",
        )],
    );
    let b = version(
        MODEL,
        &[(
            "install.md",
            "# Install\n\n@note:\nStay here.\n\nA paragraph that moves.\n@end\n",
        )],
    );
    let page = page(&a, &b);
    let moved: Vec<_> = page
        .changes
        .iter()
        .filter(|c| c.kind == ChangeKind::Moved)
        .collect();
    assert_eq!(moved.len(), 1, "{:#?}", page.changes);
    assert_eq!(moved[0].now.as_ref().unwrap().source, "install.md:6-6");
}

#[test]
fn a_line_form_note_is_one_block_with_its_paragraph() {
    let without = "# Install\n\nFirst paragraph about installing.\n";
    let with = "# Install\n\nFirst paragraph about installing.\n\n@note\nKeep the key safe.\n";
    let a = version(MODEL, &[("install.md", with)]);
    let b = version(MODEL, &[("install.md", without)]);
    // The page renders one element from the directive through the
    // paragraph, so its removal is one change with that element's anchor.
    let removed = page(&a, &b);
    let change = only_change(&removed);
    assert_eq!(change.kind, ChangeKind::Removed);
    assert_eq!(change.was.as_ref().unwrap().source, "install.md:5-6");
    assert_eq!(change.text.as_deref(), Some("Keep the key safe."));
    assert_eq!(removed.counts.removed, 1);
    let added = page(&b, &a);
    assert_eq!(
        kinds(&added),
        vec![(ChangeKind::Added, None, Some("install.md:5-6".into()))]
    );
    // A change to the paragraph is the paragraph's.
    let c = version(MODEL, &[("install.md", &with.replace("safe", "secret"))]);
    let changed = page(&a, &c);
    assert_eq!(
        kinds(&changed),
        vec![(
            ChangeKind::Changed,
            Some("install.md:6-6".into()),
            Some("install.md:6-6".into())
        )]
    );
}

#[test]
fn a_change_inside_a_list_item_marks_the_paragraph_not_the_list() {
    let steps = |second: &str| {
        format!("# Install\n\n@steps\n1. Download the package.\n2. {second}\n3. Start the agent.\n")
    };
    let a = version(MODEL, &[("install.md", &steps("Unpack it somewhere."))]);
    let b = version(MODEL, &[("install.md", &steps("Unpack it anywhere."))]);
    let page = page(&a, &b);
    let change = only_change(&page);
    assert_eq!(change.kind, ChangeKind::Changed);
    assert_eq!(change.now.as_ref().unwrap().source, "install.md:5-5");
    assert!(change.words.is_some());
}

#[test]
fn a_change_inside_a_variant_arm() {
    let model = "spec = \"0.1\"\n[dimensions.deployment]\nvalues = [\"cloud\", \"self-managed\"]\n[builds.site]\nvariants = \"switch\"\n";
    let text = |cloud: &str| {
        format!(
            "# Deploy\n\n@variant {{deployment=cloud}}:\n{cloud}\n@variant {{deployment=self-managed}}:\nRun the server yourself.\n@end\n"
        )
    };
    let a = version(model, &[("deploy.md", &text("Sign in to the console."))]);
    let b = version(
        model,
        &[("deploy.md", &text("Sign in to the web console."))],
    );
    let page = page(&a, &b);
    let change = only_change(&page);
    assert_eq!(change.kind, ChangeKind::Changed);
    assert_eq!(change.now.as_ref().unwrap().source, "deploy.md:4-4");
}

#[test]
fn a_page_changed_only_through_a_fragment_names_it() {
    let page_text = "# Install\n\nIntro.\n\n@include: _fragments/prereqs.md\n\nOutro.\n";
    let a = version(
        MODEL,
        &[
            ("install.md", page_text),
            ("_fragments/prereqs.md", "You need agent 2.2 or later.\n"),
        ],
    );
    let b = version(
        MODEL,
        &[
            ("install.md", page_text),
            ("_fragments/prereqs.md", "You need agent 2.4 or later.\n"),
        ],
    );
    let page = page(&a, &b);
    assert!(!page.own_file_changed);
    assert_eq!(page.because, ["_fragments/prereqs.md"]);
    let change = only_change(&page);
    let now = change.now.as_ref().unwrap();
    assert_eq!(now.source, "_fragments/prereqs.md:1-1");
    assert_eq!(now.via, ["install.md:5"]);
}

/// A version whose `code/` folder is a source, with these code files (paths
/// from the project root) beside its pages.
fn with_code(pages: &[(&str, &str)], code: &[(&str, &str)]) -> Version {
    let model = format!("{MODEL}[sources.code]\npath = \"code\"\n");
    let loaded = tessera_model::load_str(&model, FileId::new(0)).expect("a valid model");
    let layout = Layout::from_model(&loaded);
    let mut fs = MemoryFs::new(&layout);
    for (path, text) in pages {
        fs = fs.with_source(path, text);
    }
    for (path, text) in code {
        fs = fs.with_file(path, text);
    }
    Version {
        model,
        project: Project::load(Arc::new(loaded), layout, &fs),
    }
}

#[test]
fn a_page_changed_only_through_a_snippet_names_its_address() {
    let page_text = "# Run\n\n@snippet: code:app.py#main\n\nThen go on.\n";
    let code = |body: &str| {
        format!("import os\n# :snippet-start: main\n{body}\n# :snippet-end:\nrest()\n")
    };
    let a = with_code(
        &[("run.md", page_text)],
        &[("code/app.py", &code("connect(host, port)\nmain(verbose)"))],
    );
    // A change outside the region changes nothing.
    let same = with_code(
        &[("run.md", page_text)],
        &[(
            "code/app.py",
            &code("connect(host, port)\nmain(verbose)").replace("rest()", "other()"),
        )],
    );
    assert!(diff(&a, &same)[0].pages.is_empty());
    let b = with_code(
        &[("run.md", page_text)],
        &[("code/app.py", &code("connect(host, port)\nmain(quiet)"))],
    );
    let page = page(&a, &b);
    assert!(!page.own_file_changed);
    assert_eq!(page.because, ["code:app.py#main"]);
    let change = only_change(&page);
    assert_eq!(change.kind, ChangeKind::Changed);
    assert_eq!(change.now.as_ref().unwrap().source, "run.md:3-3");
}

#[test]
fn a_snippet_added_to_a_page_is_the_pages_own_change() {
    let code = [("code/app.py", "main()\n")];
    let a = with_code(&[("run.md", "# Run\n")], &code);
    let b = with_code(&[("run.md", "# Run\n\n@snippet: code:app.py\n")], &code);
    let page = page(&a, &b);
    assert!(page.own_file_changed);
    assert!(page.because.is_empty(), "{:?}", page.because);
}

#[test]
fn a_page_changed_only_through_a_phrase_names_the_model() {
    let text = "# About\n\nWelcome to {product}.\n";
    let a = version(MODEL, &[("about.md", text)]);
    let b = version(
        &MODEL.replace("\"Quill\"", "\"Quill Cloud\""),
        &[("about.md", text)],
    );
    let page = page(&a, &b);
    assert!(!page.own_file_changed);
    assert_eq!(page.because, ["ascribe.toml"]);
    let change = only_change(&page);
    let words = change.words.as_ref().unwrap();
    assert_eq!(words.now_text, "Welcome to Quill Cloud.");
}

#[test]
fn a_page_changed_only_through_a_builds_settings_names_the_model() {
    let model = |selected: &str| {
        format!(
            "spec = \"0.1\"\n[dimensions.deployment]\nvalues = [\"cloud\", \"self-managed\"]\n[builds.site]\nvariants = {{ deployment = \"{selected}\" }}\n"
        )
    };
    let text = "# Deploy\n\n@variant {deployment=cloud}:\nSign in to the console.\n@variant {deployment=self-managed}:\nRun the server yourself.\n@end\n";
    let a = version(&model("cloud"), &[("deploy.md", text)]);
    let b = version(&model("self-managed"), &[("deploy.md", text)]);
    let page = page(&a, &b);
    assert!(!page.own_file_changed);
    assert_eq!(page.because, ["ascribe.toml"]);
    assert_eq!(
        kinds(&page),
        vec![
            (ChangeKind::Removed, Some("deploy.md:4-4".into()), None),
            (ChangeKind::Added, None, Some("deploy.md:6-6".into())),
        ]
    );
}

#[test]
fn pages_a_build_starts_or_stops_publishing() {
    let a = version(MODEL, &[("install.md", PAGE), ("old.md", "# Old\n")]);
    let b = version(MODEL, &[("install.md", PAGE), ("new.md", "# New\n")]);
    let builds = diff(&a, &b);
    let pages: Vec<(&str, PageStatus, bool)> = builds[0]
        .pages
        .iter()
        .map(|p| (p.path.as_str(), p.status, p.own_file_changed))
        .collect();
    assert_eq!(
        pages,
        vec![
            ("new.md", PageStatus::Added, true),
            ("old.md", PageStatus::Removed, true)
        ]
    );
    assert!(builds[0].pages.iter().all(|p| p.changes.is_empty()));
}

#[test]
fn without_a_base_every_page_is_added() {
    let b = version(MODEL, &[("install.md", PAGE)]);
    let builds = compare_builds(None, side(&b), &["site"]);
    assert_eq!(builds[0].pages.len(), 1);
    assert_eq!(builds[0].pages[0].status, PageStatus::Added);
}

#[test]
fn a_title_change_changes_the_page_with_no_block_changes() {
    let a = version(MODEL, &[("install.md", PAGE)]);
    let b = version(
        MODEL,
        &[(
            "install.md",
            &PAGE.replace("title: Install", "title: Set up"),
        )],
    );
    let page = page(&a, &b);
    assert_eq!(page.status, PageStatus::Changed);
    assert!(page.changes.is_empty());
    assert_eq!(page.page_changed, ["title"]);
}

#[test]
fn other_frontmatter_and_availability_are_named_apart_from_the_title() {
    let model = "spec = \"0.1\"\n[dimensions.deployment]\nvalues = [\"cloud\", \"self-managed\"]\n[builds.site]\n";
    let a = version(model, &[("install.md", PAGE)]);
    let b = version(
        model,
        &[(
            "install.md",
            &PAGE.replace(
                "title: Install",
                "title: Install\nsidebar: 2\navailable: cloud",
            ),
        )],
    );
    let page = page(&a, &b);
    assert!(page.changes.is_empty());
    assert_eq!(page.page_changed, ["frontmatter", "availability"]);
}

#[test]
fn a_page_with_block_changes_only_has_no_page_level_changes() {
    let a = version(MODEL, &[("install.md", PAGE)]);
    let b = version(
        MODEL,
        &[("install.md", &PAGE.replace("more words", "fewer words"))],
    );
    assert!(page(&a, &b).page_changed.is_empty());
}

#[test]
fn crlf_line_endings_are_no_change() {
    let page_text =
        "# Install\n\nRun this:\n\n```sh\nquill install\nquill start\n```\n\n<div>\nraw\n</div>\n";
    let a = version(MODEL, &[("install.md", page_text)]);
    let b = version(MODEL, &[("install.md", &page_text.replace('\n', "\r\n"))]);
    let builds = diff(&a, &b);
    assert!(builds[0].pages.is_empty(), "{:#?}", builds[0].pages);
}

#[test]
fn a_linked_page_that_changed_isnt_a_cause_unless_the_link_shows_it() {
    let before = [
        ("index.md", "# Home\n\nSee [the guide](guide.md).\n"),
        ("guide.md", "# Guide\n\nGuide text.\n"),
    ];
    let a = version(MODEL, &before);
    // The guide's body changed, and so did the paragraph linking to it, but
    // the link shows nothing of the guide, so the guide isn't a cause.
    let b = version(
        MODEL,
        &[
            ("index.md", "# Home\n\nDo see [the guide](guide.md).\n"),
            ("guide.md", "# Guide\n\nOther guide text.\n"),
        ],
    );
    let builds = diff(&a, &b);
    let home = builds[0]
        .pages
        .iter()
        .find(|p| p.path == "index.md")
        .unwrap();
    assert!(home.because.is_empty(), "{:?}", home.because);

    // A link that takes its text from the guide's title does show it.
    let a = version(
        MODEL,
        &[
            ("index.md", "# Home\n\nSee [](guide.md).\n"),
            ("guide.md", "---\ntitle: Guide\n---\n\n# Guide\n"),
        ],
    );
    let b = version(
        MODEL,
        &[
            ("index.md", "# Home\n\nSee [](guide.md).\n"),
            ("guide.md", "---\ntitle: Handbook\n---\n\n# Guide\n"),
        ],
    );
    let builds = diff(&a, &b);
    let home = builds[0]
        .pages
        .iter()
        .find(|p| p.path == "index.md")
        .unwrap();
    assert_eq!(home.because, ["guide.md"]);
}

#[test]
fn a_changed_link_target_is_a_change_even_with_the_same_text() {
    let a = version(
        MODEL,
        &[
            ("index.md", "# Home\n\nSee [the guide](a.md).\n"),
            ("a.md", "# A\n"),
            ("b.md", "# B\n"),
        ],
    );
    let b = version(
        MODEL,
        &[
            ("index.md", "# Home\n\nSee [the guide](b.md).\n"),
            ("a.md", "# A\n"),
            ("b.md", "# B\n"),
        ],
    );
    let page = page(&a, &b);
    let change = only_change(&page);
    assert_eq!(change.kind, ChangeKind::Changed);
    // The text is the same, so there are no words to mark.
    assert!(change.words.is_none());
}

#[test]
fn a_long_paragraph_is_changed_without_a_word_diff() {
    let long = "word ".repeat(6000);
    let a = version(MODEL, &[("p.md", &format!("# P\n\n{long}end.\n"))]);
    let b = version(MODEL, &[("p.md", &format!("# P\n\n{long}finish.\n"))]);
    let page = page(&a, &b);
    let change = only_change(&page);
    assert_eq!(change.kind, ChangeKind::Changed);
    assert!(change.words.is_none());
}

#[test]
fn a_very_long_list_is_changed_as_a_whole() {
    let list = |last: &str| {
        let mut t = String::from("# P\n\n");
        for i in 0..1100 {
            t.push_str(&format!("- item {i}\n"));
        }
        t.push_str(&format!("- {last}\n"));
        t
    };
    let a = version(MODEL, &[("p.md", &list("the last one"))]);
    let b = version(MODEL, &[("p.md", &list("the final one"))]);
    let page = page(&a, &b);
    let change = only_change(&page);
    assert_eq!(change.kind, ChangeKind::Changed);
    assert_eq!(change.now.as_ref().unwrap().source, "p.md:3-1103");
}

#[test]
fn a_rewritten_block_is_removed_and_added() {
    let a = version(
        MODEL,
        &[("p.md", "# P\n\nThe old sentence about apples.\n")],
    );
    let b = version(
        MODEL,
        &[("p.md", "# P\n\nSomething else entirely, on oranges.\n")],
    );
    let page = page(&a, &b);
    assert_eq!(
        page.changes.iter().map(|c| c.kind).collect::<Vec<_>>(),
        [ChangeKind::Removed, ChangeKind::Added]
    );
}

#[test]
fn the_json_has_the_documented_shape() {
    let a = version(MODEL, &[("install.md", PAGE)]);
    let b = version(
        MODEL,
        &[(
            "install.md",
            &PAGE.replace("with more words", "with fewer words"),
        )],
    );
    let json = serde_json::to_value(diff(&a, &b)).unwrap();
    let page = &json[0]["pages"][0];
    assert_eq!(page["status"], "changed");
    assert_eq!(page["own_file_changed"], true);
    assert_eq!(page["page_changed"], serde_json::json!([]));
    assert_eq!(page["counts"]["changed"], 1);
    let change = &page["changes"][0];
    assert_eq!(change["kind"], "changed");
    assert_eq!(change["now"]["source"], "install.md:9-9");
    assert_eq!(change["now"]["via"], serde_json::json!([]));
    assert!(change["words"]["now"].is_array());
    assert!(change.get("after").is_none());
}
