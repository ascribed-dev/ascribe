//! `ascribe fmt`: exit statuses, `--check`, and which files it visits.

#![allow(clippy::expect_used, clippy::panic)]

#[path = "../../../../tests/support/links.rs"]
mod links;

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const MESSY: &str = "@note{ type = tip }:Text.\n";
const CLEAN: &str = "@note {type=tip}: Text.\n";

/// A project in its own directory under the target directory.
struct Project {
    dir: PathBuf,
}

impl Project {
    fn new(name: &str) -> Project {
        let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("fmt-{name}"));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("docs")).expect("create the project");
        std::fs::write(
            dir.join("ascribe.toml"),
            "spec = \"0.1\"\n\n[project]\ncontent-root = \"docs\"\n",
        )
        .expect("write the model");
        Project { dir }
    }

    fn write(&self, path: &str, text: &str) {
        let path = self.dir.join(path);
        std::fs::create_dir_all(path.parent().expect("has a parent")).expect("create dirs");
        std::fs::write(path, text).expect("write a file");
    }

    fn read(&self, path: &str) -> String {
        std::fs::read_to_string(self.dir.join(path)).expect("read a file")
    }

    fn fmt(&self, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_ascribe"))
            .arg("fmt")
            .args(args)
            .current_dir(&self.dir)
            .output()
            .expect("run ascribe")
    }
}

fn code(output: &Output) -> i32 {
    output.status.code().expect("exited normally")
}

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

#[test]
fn check_exits_1_on_unformatted_input_and_changes_nothing() {
    let p = Project::new("check-messy");
    p.write("docs/a.md", MESSY);
    let out = p.fmt(&["--check"]);
    assert_eq!(code(&out), 1, "{}", stderr(&out));
    assert!(stdout(&out).contains("would reformat"), "{}", stdout(&out));
    assert!(stdout(&out).contains("a.md"));
    assert!(stderr(&out).contains("1 file would be reformatted"));
    assert_eq!(p.read("docs/a.md"), MESSY);
}

#[test]
fn check_exits_0_on_formatted_input() {
    let p = Project::new("check-clean");
    p.write("docs/a.md", CLEAN);
    p.write("docs/sub/b.md", "Just text.\n");
    let out = p.fmt(&["--check"]);
    assert_eq!(code(&out), 0, "{}{}", stdout(&out), stderr(&out));
    assert!(stdout(&out).is_empty());
}

#[test]
fn formatting_rewrites_files_in_place() {
    let p = Project::new("write");
    p.write("docs/a.md", MESSY);
    p.write("docs/sub/b.md", "@steps\n\n1. One\n");
    p.write("docs/c.md", CLEAN);
    let out = p.fmt(&[]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    assert_eq!(p.read("docs/a.md"), CLEAN);
    assert_eq!(p.read("docs/sub/b.md"), "@steps\n1. One\n");
    assert_eq!(p.read("docs/c.md"), CLEAN);
    let listed = stdout(&out);
    assert!(listed.contains("a.md") && listed.contains("b.md") && !listed.contains("c.md"));
    // Now it's formatted.
    let again = p.fmt(&["--check"]);
    assert_eq!(code(&again), 0);
    let twice = p.fmt(&[]);
    assert_eq!(code(&twice), 0);
    assert!(stdout(&twice).is_empty());
}

#[test]
fn paths_can_be_files_or_directories() {
    let p = Project::new("paths");
    p.write("docs/a.md", MESSY);
    p.write("docs/sub/b.md", MESSY);
    p.write("docs/other/c.md", MESSY);
    let out = p.fmt(&["docs/a.md", "docs/sub"]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    assert_eq!(p.read("docs/a.md"), CLEAN);
    assert_eq!(p.read("docs/sub/b.md"), CLEAN);
    assert_eq!(p.read("docs/other/c.md"), MESSY);
}

#[test]
fn with_no_path_only_the_content_root_is_formatted() {
    let p = Project::new("root");
    p.write("docs/a.md", MESSY);
    p.write("README.md", MESSY);
    p.fmt(&[]);
    assert_eq!(p.read("docs/a.md"), CLEAN);
    assert_eq!(p.read("README.md"), MESSY);
}

#[test]
fn hidden_directories_and_node_modules_are_skipped() {
    let p = Project::new("skip");
    p.write("docs/.ascribe/build/a.md", MESSY);
    p.write("docs/node_modules/pkg/b.md", MESSY);
    p.write("docs/notes.txt", MESSY);
    p.write("docs/real.md", MESSY);
    let out = p.fmt(&["--check"]);
    assert_eq!(code(&out), 1);
    assert_eq!(stdout(&out).lines().count(), 1, "{}", stdout(&out));
    assert!(stdout(&out).contains("real.md"));
}

#[test]
fn a_nested_project_is_skipped() {
    let p = Project::new("nested");
    p.write("docs/nested/ascribe.toml", "spec = \"0.1\"\n");
    p.write("docs/nested/content/a.md", MESSY);
    p.write("docs/real.md", MESSY);
    let out = p.fmt(&["--check"]);
    assert_eq!(code(&out), 1);
    assert_eq!(stdout(&out).lines().count(), 1, "{}", stdout(&out));
    assert!(stdout(&out).contains("real.md"));
}

#[test]
fn a_content_root_above_the_project_keeps_the_projects_own_folder() {
    // The project is `ws/proj`, and its content root is `ws`.
    let p = Project::new("own-folder");
    p.write(
        "ws/proj/ascribe.toml",
        "spec = \"0.1\"\n\n[project]\ncontent-root = \"..\"\noutput-dir = \"../../out\"\n",
    );
    p.write("ws/proj/own.md", MESSY);
    p.write("ws/other/ascribe.toml", "spec = \"0.1\"\n");
    p.write("ws/other/b.md", MESSY);
    p.write("ws/top.md", MESSY);
    let out = Command::new(env!("CARGO_BIN_EXE_ascribe"))
        .args(["fmt", "--check"])
        .current_dir(p.dir.join("ws").join("proj"))
        .output()
        .expect("run ascribe");
    assert_eq!(code(&out), 1, "{}", stderr(&out));
    assert_eq!(stdout(&out).lines().count(), 2, "{}", stdout(&out));
    assert!(stdout(&out).contains("own.md"), "{}", stdout(&out));
    assert!(stdout(&out).contains("top.md"), "{}", stdout(&out));
}

#[test]
fn a_project_is_found_from_a_subdirectory() {
    let p = Project::new("subdir");
    p.write("docs/a.md", MESSY);
    let out = Command::new(env!("CARGO_BIN_EXE_ascribe"))
        .args(["fmt", "--check"])
        .current_dir(p.dir.join("docs"))
        .output()
        .expect("run ascribe");
    assert_eq!(code(&out), 1, "{}", stderr(&out));
}

#[test]
fn a_widget_from_the_model_is_formatted_in_its_own_order() {
    let p = Project::new("widget");
    p.write(
        "ascribe.toml",
        "spec = \"0.1\"\n\n[project]\ncontent-root = \"docs\"\n\n[widgets.quill-labspace]\nforms = [\"line\"]\nprimary = \"none\"\nbinding = \"self\"\nplain-fallback = \"Lab.\"\n\n[widgets.quill-labspace.attributes]\nlab = \"string\"\nheight = \"number?\"\n",
    );
    p.write("docs/a.md", "@quill-labspace {height=3, lab=x}\n");
    let out = p.fmt(&[]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    assert_eq!(p.read("docs/a.md"), "@quill-labspace {lab=x, height=3}\n");
}

#[test]
fn problems_exit_2() {
    // No project.
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("fmt-no-project");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("create a directory");
    let out = Command::new(env!("CARGO_BIN_EXE_ascribe"))
        .args(["fmt", "--check"])
        .current_dir(&dir)
        .output()
        .expect("run ascribe");
    assert_eq!(code(&out), 2);
    assert!(stderr(&out).contains("no ascribe.toml"), "{}", stderr(&out));

    // An invalid model.
    let p = Project::new("bad-model");
    p.write("ascribe.toml", "spec = [");
    let out = p.fmt(&[]);
    assert_eq!(code(&out), 2);
    assert!(stderr(&out).contains("isn't a valid content model"));

    // A missing path, an unknown option, and a file that isn't UTF-8.
    let p = Project::new("bad-args");
    let out = p.fmt(&["nope.md"]);
    assert_eq!(code(&out), 2);
    assert!(stderr(&out).contains("nope.md"));
    let out = p.fmt(&["--wat"]);
    assert_eq!(code(&out), 2);
    assert!(stderr(&out).contains("unexpected argument"));
    std::fs::write(p.dir.join("docs/bytes.md"), [0xff, 0xfe, b'\n']).expect("write bytes");
    let out = p.fmt(&[]);
    assert_eq!(code(&out), 2);
    assert!(stderr(&out).contains("UTF-8"));
}

#[test]
fn a_file_with_errors_is_left_alone_and_is_not_a_failure() {
    let p = Project::new("errors");
    let broken = "@note {type = tip\nText.\n";
    p.write("docs/a.md", broken);
    let out = p.fmt(&["--check"]);
    assert_eq!(code(&out), 0, "{}", stdout(&out));
    assert_eq!(p.read("docs/a.md"), broken);
}

#[test]
fn a_link_out_of_the_content_root_is_reported_and_left_alone() {
    let p = Project::new("link-out");
    p.write("outside/far.md", MESSY);
    p.write("outside/folder/deep.md", MESSY);
    p.write("docs/real.md", MESSY);
    p.write("docs/.hidden/secret.md", MESSY);
    links::dir(Path::new("../outside/folder"), p.dir.join("docs/linked"));
    let files = links::file(Path::new("../outside/far.md"), p.dir.join("docs/far.md"))
        && links::file(Path::new(".hidden/secret.md"), p.dir.join("docs/secret.md"))
        && links::file(Path::new("real.md"), p.dir.join("docs/alias.md"));
    let refused: &[&str] = if files {
        &["far.md", "linked/deep.md", "secret.md"]
    } else {
        &["linked/deep.md"]
    };

    let out = p.fmt(&[]);
    assert_eq!(code(&out), 2, "{}{}", stdout(&out), stderr(&out));
    // Reported as `check` reports them, and neither read nor written.
    let err = stderr(&out);
    for path in refused {
        assert!(
            err.contains(&format!(
                "[ASC123] Error: this file can't be read: `{path}` is a symbolic link, or is in a linked folder, that leads to a file that isn't a source file of the content root"
            )),
            "{err}"
        );
    }
    assert_eq!(err.lines().count(), refused.len(), "{err}");
    assert_eq!(p.read("outside/far.md"), MESSY);
    assert_eq!(p.read("outside/folder/deep.md"), MESSY);
    assert_eq!(p.read("docs/.hidden/secret.md"), MESSY);
    // The rest is formatted, a link to a source file included.
    assert_eq!(p.read("docs/real.md"), CLEAN);
    assert!(!stdout(&out).contains("deep.md"), "{}", stdout(&out));

    // Named on its own, it's refused too.
    let out = p.fmt(&["--check", "docs/linked/deep.md"]);
    assert_eq!(code(&out), 2, "{}", stderr(&out));
    assert!(stdout(&out).is_empty(), "{}", stdout(&out));
    assert!(
        stderr(&out).contains("`linked/deep.md`"),
        "{}",
        stderr(&out)
    );

    // So is a `..` after the link, which Unix applies after following it:
    // `linked/..` is `outside`. (Windows works `..` out by name first.)
    p.write("outside/up.md", MESSY);
    if cfg!(unix) {
        let out = p.fmt(&["docs/linked/../up.md"]);
        assert_eq!(code(&out), 2, "{}{}", stdout(&out), stderr(&out));
        assert!(
            stderr(&out).contains("`linked/../up.md`"),
            "{}",
            stderr(&out)
        );
        assert_eq!(p.read("outside/up.md"), MESSY);
    }

    // And the content root spelled another way: through a link to it.
    links::dir(Path::new("docs"), p.dir.join("other-name"));
    let out = p.fmt(&["other-name/linked"]);
    assert_eq!(code(&out), 2, "{}{}", stdout(&out), stderr(&out));
    assert!(
        stderr(&out).contains("`linked/deep.md`"),
        "{}",
        stderr(&out)
    );
    assert_eq!(p.read("outside/folder/deep.md"), MESSY);

    // A file outside the content root named as itself is formatted, as before,
    // and so is one named through a `..` and no link.
    let out = p.fmt(&["outside/far.md", "docs/../outside/up.md"]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    assert_eq!(p.read("outside/far.md"), CLEAN);
    assert_eq!(p.read("outside/up.md"), CLEAN);
}
