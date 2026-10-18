//! Memo files (SPEC §7.7): read from `memos/` through the file system, and
//! placed against the source index on a page, a section, an entry, or the
//! block their quote is in.

#![allow(clippy::expect_used, clippy::panic)]

use std::sync::Arc;

use ascribe_core::{FileId, RelPath};
use ascribe_resolve::{
    Committed, Invalid, Layout, MemoFile, MemoState, MemoStore, MemoryFs, Personal, Project,
    Usable, folder_in_content, is_memo_path, place,
};

const MODEL: &str = r#"
spec = "0.1"

[project]
content-root = "docs"

[phrases]
product = "Quill"
"#;

const GUIDE: &str = "---\ntitle: Guide\n---\n\nThe guide covers setup.\n\n## Install\n@id: install\n\nRun the installer\nas an administrator.\n\nRun the installer again to repair it.\n\n- Restart the agent.\n\n@memo: Run the installer, said only here.\n\n@include: _shared.md\n";

const SHARED: &str = "## Shared steps\n@id: shared\n\nCheck the logs.\n";

fn layout() -> Layout {
    let model = ascribe_model::load_str(MODEL, FileId::new(0)).expect("the model loads");
    Layout::from_model(&model)
}

fn fs(memos: &[(&str, &str)]) -> MemoryFs {
    let mut fs = MemoryFs::new(&layout())
        .with_source("guide.md", GUIDE)
        .with_source("_shared.md", SHARED);
    for (path, text) in memos {
        fs = fs.with_file(path, text);
    }
    fs
}

fn project(fs: &MemoryFs) -> Project {
    let model = ascribe_model::load_str(MODEL, FileId::new(0)).expect("the model loads");
    Project::load(Arc::new(model), layout(), fs)
}

fn memo(about: &str, quote: Option<&str>) -> MemoFile {
    let quote = quote.map_or(String::new(), |q| format!("quote: \"{q}\"\n"));
    MemoFile::parse(
        RelPath::parse("memos/m.md").expect("a path"),
        format!("---\nabout: {about}\n{quote}---\n\nWhy.\n"),
    )
}

fn path(text: &str) -> RelPath {
    RelPath::parse(text).expect("a path")
}

/// The state of `memo`, and the text of the block it's on, if any.
fn placed(project: &Project, memo: &MemoFile) -> (MemoState, Option<Usable>, Option<String>) {
    let placement = place(memo, project).expect("a valid memo");
    let block = placement.block.map(|span| {
        let Some(Usable::File(file) | Usable::Heading { file, .. }) = &placement.target else {
            panic!("a block of something that isn't in a file");
        };
        project.file(file).expect("the file").source[span.range()].to_owned()
    });
    (placement.state, placement.target, block)
}

#[test]
fn the_committed_store_reads_every_md_file_in_memos_and_its_subfolders() {
    let fs = fs(&[
        ("memos/b.md", "---\nabout: guide.md\n---\nB.\n"),
        ("memos/team/a.md", "---\nabout: guide.md\n---\nA.\n"),
        ("memos/notes.txt", "Not a memo."),
        ("memos/.drafts/c.md", "---\nabout: guide.md\n---\nC.\n"),
        ("docs/memos/d.md", "---\ntitle: A page\n---\n"),
    ]);
    let layout = layout();
    let memos = Committed::new(&fs, &layout).memos();
    let paths: Vec<&str> = memos.iter().map(|m| m.path.as_str()).collect();
    assert_eq!(paths, ["memos/b.md", "memos/team/a.md"]);
    assert_eq!(memos[1].first_line(), "A.");
}

#[test]
fn the_personal_store_reads_ascribe_memos_in_the_same_format() {
    let fs = fs(&[
        ("memos/team.md", "---\nabout: guide.md\n---\nTeam.\n"),
        (
            ".ascribe/memos/mine.md",
            "---\nabout: guide.md#install\n---\nMine.\n",
        ),
        (
            ".ascribe/memos/sub/more.md",
            "---\nabout: guide.md\n---\nMore.\n",
        ),
        (
            ".ascribe/build/index.md",
            "---\nabout: guide.md\n---\nBuilt.\n",
        ),
    ]);
    let layout = layout();
    let personal = Personal::new(&fs).memos();
    let paths: Vec<&str> = personal.iter().map(|m| m.path.as_str()).collect();
    assert_eq!(
        paths,
        [".ascribe/memos/mine.md", ".ascribe/memos/sub/more.md"]
    );
    assert!(personal.iter().all(MemoFile::personal));
    assert!(personal[0].problems.is_empty());
    let committed = Committed::new(&fs, &layout).memos();
    assert_eq!(committed.len(), 1);
    assert!(!committed[0].personal());
    for (file, memo) in [
        ("memos/team.md", true),
        (".ascribe/memos/sub/more.md", true),
        ("memos/.drafts/a.md", false),
        ("memos/notes.txt", false),
        (".ascribe/build/index.md", false),
        ("docs/guide.md", false),
    ] {
        assert_eq!(is_memo_path(&path(file), &layout), memo, "{file}");
    }
}

#[test]
fn a_memo_folder_in_the_content_root_holds_pages_not_memos() {
    let layout = Layout {
        content_root: RelPath::root(),
        output_dir: path(".ascribe/build"),
    };
    let fs = MemoryFs::new(&layout).with_file("memos/a.md", "---\nabout: guide.md\n---\n");
    assert!(Committed::new(&fs, &layout).memos().is_empty());
    assert_eq!(folder_in_content(&layout), Some(path("memos")));
    assert_eq!(folder_in_content(&self::layout()), None);
}

#[test]
fn a_memo_without_a_quote_is_on_what_it_names() {
    let project = project(&fs(&[]));
    for (about, target) in [
        ("guide.md", Usable::File(path("guide.md"))),
        (
            "guide.md#install",
            Usable::Heading {
                file: path("guide.md"),
                id: "install".to_owned(),
            },
        ),
        ("phrase:product", Usable::Phrase("product".to_owned())),
        ("phrase:{product}", Usable::Phrase("product".to_owned())),
    ] {
        assert_eq!(
            placed(&project, &memo(about, None)),
            (MemoState::Placed, Some(target), None),
            "{about}"
        );
    }
}

#[test]
fn a_quote_is_on_the_first_block_that_holds_it() {
    let project = project(&fs(&[]));
    let (state, _, block) = placed(
        &project,
        &memo("guide.md#install", Some("Run the installer")),
    );
    assert_eq!(state, MemoState::Placed);
    assert_eq!(
        block.as_deref(),
        Some("Run the installer\nas an administrator.")
    );
    // A list item's paragraph is a block.
    let (_, _, block) = placed(&project, &memo("guide.md", Some("Restart the agent")));
    assert_eq!(block.as_deref(), Some("Restart the agent."));
}

#[test]
fn whitespace_runs_read_as_one_space_and_case_counts() {
    let project = project(&fs(&[]));
    let across_lines = memo("guide.md", Some("installer as  an\\tadministrator"));
    assert_eq!(placed(&project, &across_lines).0, MemoState::Placed);
    let cased = memo("guide.md", Some("run the installer"));
    assert_eq!(placed(&project, &cased).0, MemoState::Outdated);
}

#[test]
fn a_quote_across_two_blocks_or_outside_the_section_is_outdated() {
    let project = project(&fs(&[]));
    let across = memo("guide.md", Some("administrator. Run the installer again"));
    assert_eq!(
        placed(&project, &across),
        (
            MemoState::Outdated,
            Some(Usable::File(path("guide.md"))),
            None
        )
    );
    // The page's first paragraph isn't in the section.
    let outside = memo("guide.md#install", Some("covers setup"));
    assert_eq!(placed(&project, &outside).0, MemoState::Outdated);
    // A memo in the page isn't one of its blocks.
    let in_memo = memo("guide.md", Some("said only here"));
    assert_eq!(placed(&project, &in_memo).0, MemoState::Outdated);
}

#[test]
fn a_quote_is_found_in_the_fragment_it_names() {
    let project = project(&fs(&[]));
    let (state, target, block) =
        placed(&project, &memo("_shared.md#shared", Some("Check the logs")));
    assert_eq!(state, MemoState::Placed);
    assert_eq!(
        target,
        Some(Usable::Heading {
            file: path("_shared.md"),
            id: "shared".to_owned()
        })
    );
    assert_eq!(block.as_deref(), Some("Check the logs."));
    // The page that includes it doesn't hold the fragment's words.
    let through = memo("guide.md", Some("Check the logs"));
    assert_eq!(placed(&project, &through).0, MemoState::Outdated);
}

#[test]
fn a_memo_about_nothing_is_detached() {
    let project = project(&fs(&[]));
    for about in ["gone.md", "guide.md#gone", "phrase:gone"] {
        assert_eq!(
            placed(&project, &memo(about, None)).0,
            MemoState::Detached,
            "{about}"
        );
    }
}

#[test]
fn a_memo_that_isnt_one_is_not_placed() {
    let project = project(&fs(&[]));
    let quoted_entry = memo("phrase:product", Some("Quill"));
    assert_eq!(
        quoted_entry
            .problems
            .iter()
            .map(|p| &p.kind)
            .collect::<Vec<_>>(),
        [&Invalid::QuoteWithoutText]
    );
    assert!(place(&quoted_entry, &project).is_none());
    let no_about = MemoFile::parse(path("memos/m.md"), "---\nquote: x\n---\n".to_owned());
    assert!(place(&no_about, &project).is_none());
}

#[test]
fn a_memo_about_a_term_is_on_its_heading() {
    const GLOSSARY_MODEL: &str = "spec = \"0.1\"\n\n[project]\ncontent-root = \"docs\"\n\n[glossary]\npages = [\"glossary.md\"]\n";
    let model = ascribe_model::load_str(GLOSSARY_MODEL, FileId::new(0)).expect("the model loads");
    let layout = Layout::from_model(&model);
    let fs = MemoryFs::new(&layout).with_source(
        "glossary.md",
        "---\ntitle: Glossary\n---\n\nThe words the docs use.\n\n## Region\n@id: area\n@term\n\nWhere a workspace runs.\n",
    );
    let project = Project::load(Arc::new(model), layout, &fs);
    let heading = Usable::Heading {
        file: path("glossary.md"),
        id: "area".to_owned(),
    };
    assert_eq!(
        placed(&project, &memo("term:area", None)),
        (MemoState::Placed, Some(heading.clone()), None)
    );
    let quoted = memo("term:area", Some("Where a workspace runs"));
    assert!(quoted.problems.is_empty(), "{:?}", quoted.problems);
    assert_eq!(
        placed(&project, &quoted),
        (
            MemoState::Placed,
            Some(heading),
            Some("Where a workspace runs.".to_owned())
        )
    );
    let outside = memo("term:area", Some("The words the docs use"));
    assert_eq!(placed(&project, &outside).0, MemoState::Outdated);
    assert_eq!(
        placed(&project, &memo("term:zone", Some("Where"))).0,
        MemoState::Detached
    );
}
