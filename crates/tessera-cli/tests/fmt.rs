//! `tessera fmt`: exit statuses, `--check`, and which files it visits.

#![allow(clippy::expect_used, clippy::panic)]

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
            dir.join("tessera.toml"),
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
        Command::new(env!("CARGO_BIN_EXE_tessera"))
            .arg("fmt")
            .args(args)
            .current_dir(&self.dir)
            .output()
            .expect("run tessera")
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
    p.write("docs/.tessera/build/a.md", MESSY);
    p.write("docs/node_modules/pkg/b.md", MESSY);
    p.write("docs/notes.txt", MESSY);
    p.write("docs/real.md", MESSY);
    let out = p.fmt(&["--check"]);
    assert_eq!(code(&out), 1);
    assert_eq!(stdout(&out).lines().count(), 1, "{}", stdout(&out));
    assert!(stdout(&out).contains("real.md"));
}

#[test]
fn a_project_is_found_from_a_subdirectory() {
    let p = Project::new("subdir");
    p.write("docs/a.md", MESSY);
    let out = Command::new(env!("CARGO_BIN_EXE_tessera"))
        .args(["fmt", "--check"])
        .current_dir(p.dir.join("docs"))
        .output()
        .expect("run tessera");
    assert_eq!(code(&out), 1, "{}", stderr(&out));
}

#[test]
fn a_widget_from_the_model_is_formatted_in_its_own_order() {
    let p = Project::new("widget");
    p.write(
        "tessera.toml",
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
    let out = Command::new(env!("CARGO_BIN_EXE_tessera"))
        .args(["fmt", "--check"])
        .current_dir(&dir)
        .output()
        .expect("run tessera");
    assert_eq!(code(&out), 2);
    assert!(stderr(&out).contains("no tessera.toml"), "{}", stderr(&out));

    // An invalid model.
    let p = Project::new("bad-model");
    p.write("tessera.toml", "spec = [");
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
