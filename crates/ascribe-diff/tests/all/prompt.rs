//! Agent prompts about changes (`ascribe_diff::prompt`), on projects held in
//! memory: a page changed through a fragment, through a phrase, an added
//! page, a fragment's reach, every changed page, and the caps.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::sync::Arc;

use ascribe_check::prompt::{Base, Builds, Context, LIMIT};
use ascribe_core::{FileId, RelPath};
use ascribe_diff::prompt::{self, Review};
use ascribe_diff::{BaseInfo, BuildDiff, PageDiff, Renames, Side, compare_builds};
use ascribe_resolve::{Layout, MemoryFs, Project};

const MODEL: &str = "spec = \"0.1\"\n[phrases]\nproduct = \"Quill\"\n[builds.site]\n";

const PAGE: &str = "---\ntitle: Install\n---\n\n# Install\n\nIntro for {product}.\n\n@include: _fragments/prereqs.md\n\nThe end.\n";

struct Version {
    model: String,
    project: Project,
}

fn version(model: &str, files: &[(&str, &str)]) -> Version {
    let loaded = ascribe_model::load_str(model, FileId::new(0)).expect("a valid model");
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
    compare_builds(Some(side(before)), side(after), &["site"], &Renames::new())
}

fn pairs(files: &[(String, String)]) -> Vec<(&str, &str)> {
    files
        .iter()
        .map(|(p, t)| (p.as_str(), t.as_str()))
        .collect()
}

fn base() -> BaseInfo {
    BaseInfo {
        requested: "origin/main".to_owned(),
        commit: "1a2b3c4d5e6f".to_owned(),
        merge_base: Some("9f8e7d6c5b4a".to_owned()),
    }
}

/// A project in `site/` of its repository, with an `AGENTS.md`.
fn context() -> Context {
    Context {
        base: Base::Repository,
        folder: Some("site".to_owned()),
        agents: Some("AGENTS.md".to_owned()),
        builds: Builds::Named(vec!["site".to_owned()]),
        unsaved: Vec::new(),
        program: None,
        codes: Vec::new(),
    }
}

fn content_root() -> RelPath {
    RelPath::parse("content").unwrap()
}

fn page_prompt(context: &Context, page: &PageDiff) -> String {
    let base = base();
    let root = content_root();
    let review = Review {
        base: &base,
        content_root: &root,
        memos: &[],
    };
    prompt::page(context, &review, "site", page)
}

fn find<'a>(builds: &'a [BuildDiff], path: &str) -> &'a PageDiff {
    builds[0]
        .pages
        .iter()
        .find(|p| p.path == path)
        .expect("the page changed")
}

#[test]
fn a_page_changed_through_a_fragment_names_it() {
    let fragment = "You need agent 2.2 or later.\n";
    let before = version(
        MODEL,
        &[("install.md", PAGE), ("_fragments/prereqs.md", fragment)],
    );
    let after = version(
        MODEL,
        &[
            ("install.md", PAGE),
            ("_fragments/prereqs.md", "You need agent 2.4 or later.\n"),
        ],
    );
    let builds = diff(&before, &after);
    insta::assert_snapshot!(page_prompt(&context(), find(&builds, "install.md")));
}

#[test]
fn a_page_changed_through_a_phrase_names_the_content_model() {
    let files = [
        ("install.md", PAGE),
        ("_fragments/prereqs.md", "Prereqs.\n"),
    ];
    let before = version(MODEL, &files);
    let after = version(&MODEL.replace("Quill", "Quill Pro"), &files);
    let builds = diff(&before, &after);
    let text = page_prompt(&context(), find(&builds, "install.md"));
    assert!(
        text.contains("\nChanged through: `site/ascribe.toml`\n"),
        "{text}"
    );
    assert!(
        text.contains("\n- changed: site/content/install.md:7\n"),
        "{text}"
    );
}

#[test]
fn an_added_page_says_it_is_new() {
    let before = version(MODEL, &[("about.md", "# About\n")]);
    let after = version(
        MODEL,
        &[
            ("about.md", "# About\n"),
            ("my guide.md", "# Guide\n\nHello.\n"),
        ],
    );
    let builds = diff(&before, &after);
    insta::assert_snapshot!(page_prompt(&context(), find(&builds, "my guide.md")));
}

#[test]
fn a_prompt_about_unsaved_text_says_to_save_it() {
    let before = version(MODEL, &[("about.md", "# About\n\nOld.\n")]);
    let after = version(MODEL, &[("about.md", "# About\n\nNew.\n")]);
    let builds = diff(&before, &after);
    let mut context = context();
    context.unsaved = vec!["content/about.md".to_owned()];
    let text = page_prompt(&context, find(&builds, "about.md"));
    assert!(
        text.starts_with("Review what this change does to `site/content/about.md`, as a reader of build `site` sees it.\n\nWhere: site/content/about.md\nThe file has unsaved changes; save it before you start.\nPaths and commands here are from the repository's root.\nProject: site/\n"),
        "{text}"
    );
}

#[test]
fn a_pages_changes_are_capped() {
    let paragraphs = |word: &str| -> String {
        (0..25)
            .map(|i| format!("Paragraph {i} {word}.\n\n"))
            .collect()
    };
    let before = version(MODEL, &[("long.md", &paragraphs("before"))]);
    let after = version(MODEL, &[("long.md", &paragraphs("after"))]);
    let builds = diff(&before, &after);
    let page = find(&builds, "long.md");
    assert_eq!(page.changes.len(), 25);
    let text = page_prompt(&context(), page);
    assert_eq!(text.matches("\n- changed: ").count(), 20, "{text}");
    assert!(
        text.contains("\nand 5 more: `ascribe diff --config site/ascribe.toml --base origin/main --build site --format json`\n"),
        "{text}"
    );
    assert!(text.chars().count() <= LIMIT);
}

#[test]
fn a_fragments_reach_lists_the_pages_that_show_it_up_to_the_cap() {
    let mut before_files = vec![(
        "_fragments/prereqs.md".to_owned(),
        "You need agent 2.2.\n".to_owned(),
    )];
    for i in 0..23 {
        before_files.push((
            format!("page-{i:02}.md"),
            format!("# Page {i}\n\n@include: _fragments/prereqs.md\n"),
        ));
    }
    let mut after_files = before_files.clone();
    after_files[0].1 = "You need agent 2.4.\n".to_owned();
    let before = version(MODEL, &pairs(&before_files));
    let after = version(MODEL, &pairs(&after_files));
    let builds = diff(&before, &after);
    let base = base();
    let root = content_root();
    let review = Review {
        base: &base,
        content_root: &root,
        memos: &[],
    };
    let fragment = RelPath::parse("_fragments/prereqs.md").unwrap();
    let text =
        prompt::fragment_reach(&context(), &review, &builds[0], &fragment).expect("a prompt");
    insta::assert_snapshot!(text);
    let unchanged = RelPath::parse("page-00.md").unwrap();
    assert_eq!(
        prompt::fragment_reach(&context(), &review, &builds[0], &unchanged),
        None
    );
}

#[test]
fn every_changed_page_is_a_line() {
    let before = version(
        MODEL,
        &[
            ("install.md", PAGE),
            ("about.md", "# About\n\nOld.\n"),
            ("_fragments/prereqs.md", "Prereqs.\n"),
        ],
    );
    let after = version(
        MODEL,
        &[
            ("install.md", PAGE),
            ("about.md", "# About\n\nNew.\n"),
            ("_fragments/prereqs.md", "Prereqs, changed.\n"),
            ("new.md", "# New\n"),
        ],
    );
    let builds = diff(&before, &after);
    let base = base();
    let root = content_root();
    let review = Review {
        base: &base,
        content_root: &root,
        memos: &[],
    };
    let text = prompt::pages(&context(), &review, &builds).expect("a prompt");
    insta::assert_snapshot!(text);
    let unchanged = diff(&before, &before);
    assert_eq!(prompt::pages(&context(), &review, &unchanged), None);
}

#[test]
fn a_change_names_the_memos_on_its_page_its_section_and_its_block() {
    let before = "---\ntitle: Keys\n---\n\n## Rotate\n\nOld step.\n\nGone soon.\n\n## Revoke\n\nRevoke it now.\n\nAnother line.\n";
    let after = "---\ntitle: Keys\n---\n\n## Rotate\n\nOld step.\n\n## Revoke\n\nRevoke it now.\n\nAnother changed line.\n";
    let old = version(
        MODEL,
        &[("keys.md", before), ("other.md", "# Other\n\nText.\n")],
    );
    let new = version(
        MODEL,
        &[("keys.md", after), ("other.md", "# Other\n\nText.\n")],
    );
    let memo = |name: &str, front: &str| {
        ascribe_resolve::MemoFile::parse(
            RelPath::parse(&format!("memos/{name}.md")).unwrap(),
            format!("---\n{front}\n---\n\nWhy.\n"),
        )
    };
    let memos = [
        memo("page", "about: keys.md"),
        memo("rotate", "about: keys.md#rotate"),
        memo(
            "revoke-first",
            "about: keys.md#revoke\nquote: Revoke it now",
        ),
        memo("revoke-second", "about: keys.md#revoke\nquote: Another"),
        memo("gone", "about: keys.md#rotate\nquote: Gone soon"),
        memo("other", "about: other.md"),
    ];
    let places = ascribe_check::prompt::MemoPlace::all(&new.project, &memos);
    let builds = diff(&old, &new);
    let base = base();
    let root = content_root();
    let review = Review {
        base: &base,
        content_root: &root,
        memos: &places,
    };
    let text = prompt::page(&context(), &review, "site", find(&builds, "keys.md"));
    let ask = ": check the change still agrees with it.";
    assert!(
        text.contains(&format!(
            "Memos about what changed, to read before judging it:\n\
             - site/memos/gone.md, about keys.md#rotate (outdated: no block holds its quote now){ask}\n\
             - site/memos/page.md, about keys.md{ask}\n\
             - site/memos/revoke-second.md, about keys.md#revoke{ask}\n\
             - site/memos/rotate.md, about keys.md#rotate{ask}\n\n"
        )),
        "{text}"
    );
}

#[test]
fn a_moved_page_and_a_link_to_it_say_so() {
    let configure = "---\ntitle: Configure\n---\n\n# Configure\n\nSet the key.\n";
    let install = "# Install\n\nThen [](configure.md).\n";
    let before = version(
        MODEL,
        &[
            ("guides/configure.md", configure),
            ("guides/install.md", install),
        ],
    );
    let after = version(
        MODEL,
        &[
            ("guides/setup.md", configure),
            ("guides/install.md", install),
        ],
    );
    let renames = Renames::from([(
        "guides/configure.md".to_owned(),
        "guides/setup.md".to_owned(),
    )]);
    let builds = compare_builds(Some(side(&before)), side(&after), &["site"], &renames);
    let moved = page_prompt(&context(), find(&builds, "guides/setup.md"));
    assert!(
        moved.contains(
            "What changed: moved from site/content/guides/configure.md, no change to the page"
        ),
        "{moved}"
    );
    let linking = page_prompt(&context(), find(&builds, "guides/install.md"));
    assert!(
        linking.contains(
            "A link changed because a page it leads to moved: `site/content/guides/configure.md`"
        ),
        "{linking}"
    );
    assert!(!linking.contains("Changed through"), "{linking}");
}
