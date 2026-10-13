//! `ascribe observe`, by running the binary in temporary folders of code:
//! what it says before it runs anything, the observation it writes when the
//! test passes, that a failing test writes nothing and leaves the old
//! observation as it was, its JSON, and what it exits with when it can't
//! run. The commands are ones `sh` and `cmd` both run.

#![allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]

use std::fs;
use std::path::Path;
use std::process::{Command, Output};

use ascribe_model::examples::Observation;
use serde_json::Value;

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

/// A folder of code whose config runs `test`, with one example written with
/// CRLF line endings and a file that isn't an example.
fn folder(test: &str) -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    fs::write(
        dir.path().join("ascribe.examples.toml"),
        format!(
            "include = [\"examples/**\"]\nignore = [\"examples/skip.ts\"]\ntest = \"{test}\"\n\n[environment]\ntool = {{ label = \"Tool\", command = \"echo v1.2.3\" }}\n"
        ),
    )
    .unwrap();
    fs::create_dir_all(dir.path().join("examples/nested")).unwrap();
    fs::write(dir.path().join("examples/a.ts"), "a();\r\n").unwrap();
    fs::write(dir.path().join("examples/nested/b.txt"), "b\n").unwrap();
    fs::write(dir.path().join("examples/skip.ts"), "skip\n").unwrap();
    fs::write(dir.path().join("helper.ts"), "helper\n").unwrap();
    dir
}

const OBSERVATION: &str = "examples.observations/default.toml";

#[test]
fn a_passing_test_writes_what_it_passed_and_where() {
    let dir = folder("exit 0");
    let output = ascribe(dir.path(), &["observe"]);
    assert_eq!(code_of(&output), 0, "{}", stderr(&output));
    let os = match std::env::consts::OS {
        "linux" => "Linux",
        "macos" => "macOS",
        "windows" => "Windows",
        other => other,
    };
    assert_eq!(
        stdout(&output),
        format!(
            "Running the commands in ascribe.examples.toml:\n  test         exit 0\n  environment  echo v1.2.3\n\
             exit 0 passed.\nWrote {OBSERVATION}: 2 files, Tool 1.2.3 on {os}.\n"
        )
    );
    let text = fs::read_to_string(dir.path().join(OBSERVATION)).unwrap();
    let observation = Observation::parse(&text).unwrap();
    assert_eq!(observation.command, "exit 0");
    assert_eq!(observation.value("tool"), Some("v1.2.3"));
    assert_eq!(observation.value("os"), Some(std::env::consts::OS));
    let files: Vec<&str> = observation.files.keys().map(String::as_str).collect();
    assert_eq!(files, ["examples/a.ts", "examples/nested/b.txt"]);
    // CRLF is read as LF.
    assert_eq!(
        observation.files["examples/a.ts"],
        ascribe_model::example_hash(b"a();\n")
    );
    // A second run writes the same file.
    assert_eq!(code_of(&ascribe(dir.path(), &["observe"])), 0);
    assert_eq!(
        fs::read_to_string(dir.path().join(OBSERVATION)).unwrap(),
        text
    );
}

/// A link to a folder `include` matches, as pnpm's `node_modules` is full
/// of, is skipped, and a link to a file is read where it leads.
#[cfg(unix)]
#[test]
fn a_link_to_a_folder_is_skipped() {
    let dir = folder("exit 0");
    fs::create_dir_all(dir.path().join("other")).unwrap();
    fs::write(dir.path().join("other/c.ts"), "c\n").unwrap();
    std::os::unix::fs::symlink("../other", dir.path().join("examples/linked")).unwrap();
    std::os::unix::fs::symlink("../helper.ts", dir.path().join("examples/helper.ts")).unwrap();
    let output = ascribe(dir.path(), &["observe"]);
    assert_eq!(code_of(&output), 0, "{}", stderr(&output));
    let text = fs::read_to_string(dir.path().join(OBSERVATION)).unwrap();
    let observation = Observation::parse(&text).unwrap();
    let files: Vec<&str> = observation.files.keys().map(String::as_str).collect();
    assert_eq!(
        files,
        [
            "examples/a.ts",
            "examples/helper.ts",
            "examples/nested/b.txt"
        ]
    );
}

#[test]
fn a_failing_test_writes_nothing_and_exits_1() {
    let dir = folder("exit 0");
    assert_eq!(code_of(&ascribe(dir.path(), &["observe"])), 0);
    let before = fs::read(dir.path().join(OBSERVATION)).unwrap();
    fs::write(dir.path().join("examples/a.ts"), "changed();\n").unwrap();
    let config = fs::read_to_string(dir.path().join("ascribe.examples.toml")).unwrap();
    fs::write(
        dir.path().join("ascribe.examples.toml"),
        config.replace("exit 0", "exit 3"),
    )
    .unwrap();
    let output = ascribe(dir.path(), &["observe"]);
    assert_eq!(code_of(&output), 1);
    assert!(
        stderr(&output).contains("exit 3 failed (exit code 3); nothing was written"),
        "{}",
        stderr(&output)
    );
    assert_eq!(fs::read(dir.path().join(OBSERVATION)).unwrap(), before);
    let leftovers: Vec<_> = fs::read_dir(dir.path().join("examples.observations"))
        .unwrap()
        .collect();
    assert_eq!(leftovers.len(), 1);
}

#[test]
fn json_has_the_commands_the_environment_and_the_files() {
    let dir = folder("exit 0");
    let output = ascribe(dir.path(), &["observe", "--format", "json"]);
    assert_eq!(code_of(&output), 0, "{}", stderr(&output));
    let json: Value = serde_json::from_str(&stdout(&output)).unwrap();
    assert_eq!(json["schema_version"], 1);
    assert_eq!(json["config"], "ascribe.examples.toml");
    assert_eq!(json["environment"][0]["key"], "tool");
    assert_eq!(json["environment"][0]["kept"], "v1.2.3");
    assert_eq!(json["test"]["passed"], true);
    assert_eq!(json["test"]["exit_code"], 0);
    assert_eq!(json["observation"], OBSERVATION);
    assert_eq!(json["recorded"]["tool"], "v1.2.3");
    assert!(json["files"]["examples/a.ts"].is_string());
    assert!(
        json["tested"]
            .as_str()
            .unwrap()
            .starts_with("Tool 1.2.3 on ")
    );

    let failing = folder("exit 1");
    let output = ascribe(failing.path(), &["observe", "--format", "json"]);
    assert_eq!(code_of(&output), 1);
    let json: Value = serde_json::from_str(&stdout(&output)).unwrap();
    assert_eq!(json["test"]["passed"], false);
    assert_eq!(json["observation"], Value::Null);
}

#[test]
fn it_runs_only_the_config_in_its_own_folder() {
    let dir = folder("exit 0");
    let output = ascribe(&dir.path().join("examples"), &["observe"]);
    assert_eq!(code_of(&output), 2);
    assert!(
        stderr(&output).contains("there's no ascribe.examples.toml in this folder"),
        "{}",
        stderr(&output)
    );
    assert!(!dir.path().join(OBSERVATION).exists());
}

#[test]
fn a_config_that_cant_be_read_or_an_environment_that_cant_be_noted_is_2() {
    let dir = folder("exit 0");
    fs::write(
        dir.path().join("ascribe.examples.toml"),
        "include = []\ntest = \"exit 0\"\n",
    )
    .unwrap();
    let output = ascribe(dir.path(), &["observe"]);
    assert_eq!(code_of(&output), 2);
    assert_eq!(
        stderr(&output),
        "error: ascribe.examples.toml:1:11: `include` is empty, so no file is an example\n"
    );

    fs::write(
        dir.path().join("ascribe.examples.toml"),
        "include = [\"**\"]\ntest = \"exit 0\"\n[environment]\ntool = { command = \"exit 4\" }\n",
    )
    .unwrap();
    let output = ascribe(dir.path(), &["observe"]);
    assert_eq!(code_of(&output), 2);
    assert!(stderr(&output).contains("`exit 4` failed (exit code 4)"));
    assert!(!dir.path().join(OBSERVATION).exists());
}
