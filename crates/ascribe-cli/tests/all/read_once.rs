//! A command reads each source file once and lists the sources once: each
//! command's library entry, called as the command calls it, on
//! `examples/quill` loaded through a `FileSystem` that counts
//! (`tests/support/counting.rs`). The source index a command resolves with
//! is built from what loading read, not read again. What the commands
//! answer is tested in their own files; this holds only how often they
//! read. A project kept between calls, as the MCP server keeps one, reads
//! only the page an edit wrote before its next call.

#![allow(clippy::expect_used, clippy::unwrap_used)]

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Arc, Mutex};

use ascribe_check::Project;
use ascribe_core::RelPath;
use ascribe_diff::{DiffOptions, DriftOptions};
use ascribe_emit::{Output, WriteOptions};
use ascribe_model::SourceBound;
use ascribe_query::{Asked, Emit, Readings, Search};
use ascribe_resolve::{DiskFs, Layout};
use ascribe_workspace::{Kept, SharedFs, Workspace};
use tempfile::TempDir;

use crate::counting::Counting;

/// `examples/quill`, copied into a git repository and committed, so `diff`
/// and `drift` have a revision to compare with.
fn quill() -> TempDir {
    let dir = tempfile::tempdir().expect("a temporary directory");
    let from = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/quill");
    copy_dir(&from, dir.path());
    for args in [
        &["init", "-q", "-b", "main"][..],
        &["add", "-A"],
        &["commit", "-q", "-m", "Quill"],
    ] {
        let out = Command::new("git")
            .current_dir(dir.path())
            .args([
                "-c",
                "user.name=Test",
                "-c",
                "user.email=test@example.com",
                "-c",
                "commit.gpgsign=false",
            ])
            .args(args)
            .output()
            .expect("run git");
        assert!(out.status.success(), "git {args:?} failed");
    }
    dir
}

fn copy_dir(from: &Path, to: &Path) {
    fs::create_dir_all(to).expect("create a directory");
    for entry in fs::read_dir(from).expect("list a directory") {
        let entry = entry.expect("a directory entry");
        let target: PathBuf = to.join(entry.file_name());
        if entry.file_type().expect("a file type").is_dir() {
            copy_dir(&entry.path(), &target);
        } else {
            fs::copy(entry.path(), target).expect("copy a file");
        }
    }
}

/// The project in `dir`, loaded as a command loads it, through a counting
/// file system over the disk.
fn load(dir: &Path) -> (Project, Arc<Counting>) {
    let model = ascribe_check::LOADER
        .load_model(&dir.join("ascribe.toml"))
        .expect("Quill's model loads");
    let layout = Layout::from_model(&model.model);
    let counting = Counting::new(Arc::new(DiskFs::with_listing_cache(dir, &layout)));
    let workspace = Workspace::load_with(
        ascribe_check::LOADER.side_files,
        model,
        layout.content_root,
        counting.clone(),
    );
    (Project::new(workspace.snapshot()), counting)
}

/// Runs `command` on a freshly loaded Quill, then checks the sources were
/// listed once and each read once.
fn reads_once(dir: &Path, what: &str, command: impl FnOnce(&Project)) {
    let (project, counting) = load(dir);
    command(&project);
    assert_eq!(
        counting.listed(),
        1,
        "{what} lists the sources more than once"
    );
    let reads = counting.reads();
    let sources: Vec<&RelPath> = project.sources().iter().map(|s| &s.path).collect();
    assert_eq!(
        reads.keys().collect::<Vec<_>>(),
        sources,
        "{what} reads other files than the sources"
    );
    let again: Vec<_> = reads.iter().filter(|(_, n)| **n > 1).collect();
    assert!(again.is_empty(), "{what} reads sources again: {again:?}");
}

fn page(path: &str) -> RelPath {
    RelPath::parse(path).expect("a content path")
}

#[test]
fn each_command_reads_each_source_once_and_lists_them_once() {
    let dir = quill();
    let dir = dir.path();
    let quickstart = page("quickstart.md");

    reads_once(dir, "check", |project| {
        ascribe_check::diagnose(project, &[]).expect("every build");
    });
    reads_once(dir, "build", |project| {
        ascribe_check::diagnose(project, &[]).expect("every build");
        let options = WriteOptions {
            outputs: vec![Output::Site, Output::Plain, Output::Json],
            anchors: false,
        };
        ascribe_emit::outputs_in_memory(project.index(), project.root(), &options)
            .expect("Quill builds");
    });
    reads_once(dir, "diff", |project| {
        let options = DiffOptions {
            base: Some("HEAD"),
            base_exact: true,
            builds: &[],
        };
        ascribe_diff::diff_project(project, &options).expect("Quill compares with HEAD");
    });
    reads_once(dir, "drift", |project| {
        let options = DriftOptions {
            base: Some("HEAD"),
            base_exact: true,
            builds: &[],
        };
        ascribe_diff::drift_project(project, &options).expect("Quill compares with HEAD");
    });
    reads_once(dir, "outline", |project| {
        ascribe_query::outline(project.index(), &quickstart, None, project.all_memos())
            .expect("an outline");
    });
    reads_once(dir, "link", |project| {
        ascribe_query::link(
            project.index(),
            project.file_system(),
            &quickstart,
            "docs/quickstart.md",
            "upgrade.md",
        )
        .expect("a link");
    });
    reads_once(dir, "refs", |project| {
        let asked = Asked::Path {
            path: quickstart.clone(),
            id: None,
        };
        ascribe_query::refs(
            project.index(),
            &asked,
            "quickstart.md",
            usize::MAX,
            project.all_memos(),
        );
    });
    reads_once(dir, "resolve", |project| {
        let build = project.model().build("site").expect("the site build");
        ascribe_query::resolve(
            project.index(),
            project.root(),
            &quickstart,
            build,
            Emit::Plain,
            false,
        )
        .expect("a resolved page");
    });
    reads_once(dir, "search", |project| {
        let asked = Search {
            query: "Quill",
            builds: &[],
            scope: None,
            page_type: None,
            case: false,
            word: false,
            summary: false,
            limit: 10,
        };
        ascribe_query::search(project, project.index(), &Readings::default(), &asked)
            .expect("a search");
    });
    reads_once(dir, "permalinks", |project| {
        ascribe_query::permalinks::list(project.index(), project.permalinks())
            .expect("the permalinks");
    });
    reads_once(dir, "versions", |project| {
        ascribe_query::versions::list(project.index());
    });
}

/// A project kept between calls, as the MCP server keeps one
/// (`crate::mcp::cache`), reads only the page an edit wrote before its next
/// call, and checks the project as it is now.
#[test]
fn a_kept_project_reads_only_the_page_an_edit_wrote() {
    let dir = quill();
    let dir = dir.path();
    let made: Arc<Mutex<Option<Arc<Counting>>>> = Arc::default();
    let fs_for = {
        let made = made.clone();
        Box::new(move |root: &Path, layout: &Layout| -> SharedFs {
            let counting = Counting::new(Arc::new(DiskFs::new(root, layout)));
            *made.lock().unwrap() = Some(counting.clone());
            counting
        })
    };
    let mut kept = Kept::load_through(
        ascribe_check::LOADER,
        &dir.join("ascribe.toml"),
        SourceBound::Anywhere,
        fs_for,
    )
    .expect("Quill loads");
    let counting = made.lock().unwrap().clone().expect("a file system");
    let call = |kept: &mut Kept| {
        kept.refresh(SourceBound::Anywhere).expect("Quill loads");
        let project = Project::new(kept.snapshot());
        ascribe_check::Found::from(ascribe_check::diagnose(&project, &[]).expect("every build"))
    };
    call(&mut kept);
    let before = counting.reads();

    let edited = dir.join("docs/quickstart.md");
    let text = fs::read_to_string(&edited).expect("the page");
    fs::write(&edited, format!("{text}\n[Gone](gone.md)\n")).expect("an edit");
    let found = call(&mut kept);

    let mut read: BTreeMap<RelPath, usize> = counting.reads();
    for (path, n) in &before {
        if let Some(now) = read.get_mut(path) {
            *now -= n;
        }
    }
    read.retain(|_, n| *n > 0);
    assert_eq!(
        read,
        BTreeMap::from([(page("quickstart.md"), 1)]),
        "the next call read other sources"
    );
    assert_eq!(counting.listed(), 1, "the sources were listed again");
    // And it finds what a fresh load finds.
    let (fresh, _) = load(dir);
    let fresh =
        ascribe_check::Found::from(ascribe_check::diagnose(&fresh, &[]).expect("every build"));
    assert_eq!(
        format!("{:?}", found.diagnostics),
        format!("{:?}", fresh.diagnostics)
    );
    assert!(
        found
            .diagnostics
            .iter()
            .any(|d| d.message.contains("gone.md")),
        "the edit's broken link is found"
    );
}
