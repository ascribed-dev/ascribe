//! `ascribe rename`, by running the binary in temporary projects: what it
//! writes, what it prints, and its exit codes. Which places a rename
//! changes is held by `ascribe-lsp`'s tests (`tests/all/rename.rs`, and
//! `model_edits.rs`, where it's compared with the editor's rename).

#![allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use tempfile::TempDir;

const MODEL: &str =
    "spec = \"0.1\"\n\n[project]\ncontent-root = \"docs\"\n\n[phrases]\ncloud = \"Quill Cloud\"\n";

const KEYS: &str = "---\ntitle: Keys\n---\n\nSign in to {cloud}.\n\n## Rotate keys\n\nText.\n";

const INSTALL: &str =
    "---\ntitle: Install\n---\n\nRun it on {cloud}. See [](keys.md#rotate-keys).\n";

fn write(root: &Path, rel: &str, text: &str) {
    let path: PathBuf = rel.split('/').fold(root.to_path_buf(), |p, s| p.join(s));
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, text).unwrap();
}

fn read(root: &Path, rel: &str) -> String {
    let path: PathBuf = rel.split('/').fold(root.to_path_buf(), |p, s| p.join(s));
    fs::read_to_string(path).unwrap()
}

fn project() -> TempDir {
    let dir = tempfile::tempdir().unwrap();
    write(dir.path(), "ascribe.toml", MODEL);
    write(dir.path(), "docs/keys.md", KEYS);
    write(dir.path(), "docs/install.md", INSTALL);
    dir
}

fn ascribe(dir: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_ascribe"))
        .current_dir(dir)
        .args(args)
        .env("NO_COLOR", "1")
        .output()
        .expect("run ascribe")
}

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

fn code_of(output: &Output) -> i32 {
    output.status.code().expect("exited with a code")
}

#[test]
fn a_phrase_is_renamed_in_the_model_and_every_page_and_each_file_is_named() {
    let dir = project();
    let output = ascribe(dir.path(), &["rename", "phrase:{cloud}", "hosted"]);
    assert_eq!(code_of(&output), 0, "{}", stderr(&output));
    assert_eq!(
        stdout(&output),
        "changed ascribe.toml (1 place)\nchanged docs/install.md (1 place)\nchanged docs/keys.md (1 place)\n"
    );
    assert_eq!(
        read(dir.path(), "ascribe.toml"),
        MODEL.replace("cloud =", "hosted =")
    );
    assert_eq!(
        read(dir.path(), "docs/keys.md"),
        KEYS.replace("{cloud}", "{hosted}")
    );
    let check = ascribe(dir.path(), &["check"]);
    assert_eq!(code_of(&check), 0, "{}", stdout(&check));
}

#[test]
fn a_heading_and_a_page_are_named_from_the_current_folder() {
    let dir = project();
    let docs = dir.path().join("docs");
    let output = ascribe(&docs, &["rename", "keys.md#rotate-keys", "rotate"]);
    assert_eq!(code_of(&output), 0, "{}", stderr(&output));
    assert_eq!(
        stdout(&output),
        "changed install.md (1 place)\nchanged keys.md (1 place)\n"
    );
    assert!(read(dir.path(), "docs/keys.md").contains("## Rotate keys\n@id: rotate\n"));

    let output = ascribe(&docs, &["rename", "keys.md", "guides/keys.md"]);
    assert_eq!(code_of(&output), 0, "{}", stderr(&output));
    assert_eq!(
        stdout(&output),
        "changed install.md (1 place)\nmoved keys.md to guides/keys.md\n"
    );
    assert!(!docs.join("keys.md").exists());
    assert_eq!(
        read(dir.path(), "docs/install.md"),
        INSTALL.replace("keys.md#rotate-keys", "guides/keys.md#rotate")
    );
    let check = ascribe(dir.path(), &["check"]);
    assert_eq!(code_of(&check), 0, "{}", stdout(&check));
}

#[test]
fn what_isnt_there_exits_1_and_what_cant_be_renamed_exits_2_changing_nothing() {
    let dir = project();
    for (args, code) in [
        (&["rename", "phrase:product", "name"][..], 1),
        (&["rename", "docs/nothing.md", "docs/new.md"][..], 1),
        (&["rename", "docs/keys.md#nothing", "new"][..], 1),
        (&["rename", "phrase:cloud", "Hosted"][..], 2),
        (&["rename", "docs/keys.md", "docs/install.md"][..], 2),
        (&["rename", "docs/keys.md", "elsewhere/keys.md"][..], 2),
        (&["rename", "feature:sso", "login"][..], 2),
    ] {
        let output = ascribe(dir.path(), args);
        assert_eq!(code_of(&output), code, "{args:?}: {}", stderr(&output));
        assert!(stderr(&output).starts_with("error: "), "{args:?}");
        assert_eq!(stdout(&output), "", "{args:?}");
    }
    assert_eq!(read(dir.path(), "ascribe.toml"), MODEL);
    assert_eq!(read(dir.path(), "docs/keys.md"), KEYS);
    assert_eq!(read(dir.path(), "docs/install.md"), INSTALL);
}

#[test]
fn a_page_isnt_moved_through_a_folder_that_is_a_link() {
    let dir = project();
    let outside = tempfile::tempdir().unwrap();
    crate::links::dir(outside.path(), dir.path().join("docs").join("guides"));
    let output = ascribe(
        dir.path(),
        &["rename", "docs/keys.md", "docs/guides/keys.md"],
    );
    assert_eq!(code_of(&output), 2, "{}", stdout(&output));
    assert!(stderr(&output).contains("link"), "{}", stderr(&output));
    assert_eq!(stdout(&output), "");
    assert_eq!(read(dir.path(), "docs/keys.md"), KEYS);
    assert_eq!(read(dir.path(), "docs/install.md"), INSTALL);
    assert_eq!(fs::read_dir(outside.path()).unwrap().count(), 0);
}

#[test]
fn each_target_is_renamed_from_the_folder_above_with_config() {
    let above = tempfile::tempdir().unwrap();
    let model = MODEL.replace(
        "[phrases]",
        "[dimensions.platform]\nvalues = [\"linux\", \"macos\"]\n\n[phrases]",
    );
    write(above.path(), "site/ascribe.toml", &model);
    write(above.path(), "site/docs/keys.md", KEYS);
    write(
        above.path(),
        "site/docs/install.md",
        &format!("{INSTALL}\n@variant {{platform=linux}}:\nOn Linux.\n@end\n"),
    );
    let rename = |args: &[&str]| {
        let args = [&["rename"], args, &["--config", "site"]].concat();
        ascribe(above.path(), &args)
    };
    for (args, said) in [
        (
            &["phrase:cloud", "hosted"][..],
            "changed site/ascribe.toml (1 place)\nchanged site/docs/install.md (1 place)\nchanged site/docs/keys.md (1 place)\n",
        ),
        (
            &["value:linux", "gnu"][..],
            "changed site/ascribe.toml (1 place)\nchanged site/docs/install.md (1 place)\n",
        ),
        // From the content root, and from the current directory.
        (
            &["keys.md#rotate-keys", "rotate"][..],
            "changed site/docs/install.md (1 place)\nchanged site/docs/keys.md (1 place)\n",
        ),
        (
            &["keys.md", "guides/keys.md"][..],
            "changed site/docs/install.md (1 place)\nmoved site/docs/keys.md to site/docs/guides/keys.md\n",
        ),
        (
            &["site/docs/guides/keys.md", "site/docs/keys.md"][..],
            "changed site/docs/install.md (1 place)\nmoved site/docs/guides/keys.md to site/docs/keys.md\n",
        ),
    ] {
        let output = rename(args);
        assert_eq!(code_of(&output), 0, "{args:?}: {}", stderr(&output));
        assert_eq!(stdout(&output), said, "{args:?}");
    }
    let check = ascribe(above.path(), &["check", "--config", "site"]);
    assert_eq!(code_of(&check), 0, "{}", stdout(&check));

    let output = rename(&["nothing.md", "new.md"]);
    assert_eq!(code_of(&output), 1);
    assert_eq!(
        stderr(&output),
        "error: `nothing.md` doesn't exist: there's no page or fragment at `nothing.md` from the current directory, or at `site/docs/nothing.md` in the content root.\n"
    );
}

fn copy_dir(from: &Path, to: &Path) {
    fs::create_dir_all(to).unwrap();
    for entry in fs::read_dir(from).unwrap() {
        let entry = entry.unwrap();
        let target = to.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copy_dir(&entry.path(), &target);
        } else {
            fs::copy(entry.path(), &target).unwrap();
        }
    }
}

#[test]
fn a_memos_about_follows_its_heading_and_its_page_in_both_folders() {
    let dir = tempfile::tempdir().unwrap();
    let quill = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/quill");
    copy_dir(&quill, dir.path());
    // A personal memo, its `about` in quotes, which stay.
    let mine = "---\nabout: \"install-agent.md#install-agent\"\n---\n\nMine.\n";
    write(dir.path(), ".ascribe/memos/mine.md", mine);
    let team = read(dir.path(), "memos/agent-first.md");

    let output = ascribe(
        dir.path(),
        &["rename", "install-agent.md#install-agent", "install"],
    );
    assert_eq!(code_of(&output), 0, "{}", stderr(&output));
    let said = stdout(&output);
    for file in [".ascribe/memos/mine.md", "memos/agent-first.md"] {
        assert!(
            said.contains(&format!("changed {file} (1 place)\n")),
            "{said}"
        );
    }
    assert_eq!(
        read(dir.path(), "memos/agent-first.md"),
        team.replace("#install-agent\n", "#install\n")
    );
    assert_eq!(
        read(dir.path(), ".ascribe/memos/mine.md"),
        mine.replace("#install-agent\"", "#install\"")
    );
    let check = ascribe(dir.path(), &["check"]);
    assert_eq!(code_of(&check), 0, "{}", stdout(&check));

    let output = ascribe(
        dir.path(),
        &["rename", "install-agent.md", "guides/agent.md"],
    );
    assert_eq!(code_of(&output), 0, "{}", stderr(&output));
    assert!(
        read(dir.path(), "memos/agent-first.md").contains("\nabout: guides/agent.md#install\n")
    );
    assert!(
        read(dir.path(), ".ascribe/memos/mine.md")
            .contains("\nabout: \"guides/agent.md#install\"\n")
    );
    let check = ascribe(dir.path(), &["check"]);
    assert_eq!(code_of(&check), 0, "{}", stdout(&check));
    let outline = ascribe(dir.path(), &["outline", "guides/agent.md"]);
    assert!(
        stdout(&outline).contains("memos/agent-first.md"),
        "{}",
        stdout(&outline)
    );
}
