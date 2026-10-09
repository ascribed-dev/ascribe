//! The command output the docs show, in `tests/output/`: each file is what
//! `ascribe` prints for a project made here, and the pages take it from the
//! file with `@snippet`. This test runs each command and fails when a file is
//! out of date; run it with `ASCRIBE_BLESS=1` to rewrite them.
//!
//! The repositories are committed at a fixed time by a fixed author, so their
//! commits are the same on every run. What still differs from one run to the
//! next is replaced: the version, with the `{version}` phrase the pages
//! substitute, and the temporary directory, with a home directory's path.

#![allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use tempfile::TempDir;

const BLESS: &str = "ASCRIBE_BLESS=1 cargo test -p ascribe-cli --test all output::";

/// Where the repository in a JSON report is said to be.
const ROOT: &str = "/home/me/lantern";

fn output_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/output")
}

/// Compares `text` with `tests/output/<name>`, or writes it there when
/// blessing. Trailing spaces are dropped, and the file ends with a newline.
fn expect(name: &str, text: &str) {
    let mut text: String = text
        .lines()
        .map(|l| format!("{}\n", l.trim_end()))
        .collect();
    if text.is_empty() {
        text.push('\n');
    }
    let path = output_dir().join(name);
    if std::env::var_os("ASCRIBE_BLESS").is_some() {
        fs::create_dir_all(output_dir()).unwrap();
        fs::write(&path, &text).unwrap();
        return;
    }
    let committed = fs::read_to_string(&path)
        .unwrap_or_else(|_| panic!("tests/output/{name} is missing; run `{BLESS}`"))
        .replace("\r\n", "\n");
    assert!(
        committed == text,
        "tests/output/{name} isn't what ascribe prints; run `{BLESS}` and check the diff\n--- ascribe\n{text}"
    );
}

/// A JSON report with its version and repository root made the same on every
/// run.
fn json(output: &Output) -> String {
    let text = stdout(output);
    let value: serde_json::Value = serde_json::from_str(&text).expect("the report is JSON");
    let version = format!("\"ascribe_version\": \"{}\"", env!("CARGO_PKG_VERSION"));
    assert!(text.contains(&version), "{text}");
    let mut text = text.replace(&version, "\"ascribe_version\": \"{version}\"");
    if let Some(root) = value.pointer("/repository/root") {
        let written = format!("\"root\": {root}");
        assert!(text.contains(&written), "{text}");
        text = text.replace(&written, &format!("\"root\": \"{ROOT}\""));
    }
    text
}

fn git(dir: &Path, args: &[&str]) {
    let out = Command::new("git")
        .current_dir(dir)
        .args([
            "-c",
            "user.name=Mira Okafor",
            "-c",
            "user.email=mira@example.com",
            "-c",
            "commit.gpgsign=false",
            "-c",
            "core.autocrlf=false",
            "-c",
            "init.defaultBranch=main",
        ])
        .args(args)
        .env("GIT_AUTHOR_DATE", "2026-10-01T12:00:00+00:00")
        .env("GIT_COMMITTER_DATE", "2026-10-01T12:00:00+00:00")
        .output()
        .expect("run git");
    assert!(
        out.status.success(),
        "git {args:?} failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

fn commit(dir: &Path, message: &str) {
    git(dir, &["add", "-A"]);
    git(dir, &["commit", "-q", "-m", message]);
}

fn write(root: &Path, rel: &str, text: &str) {
    let path: PathBuf = rel.split('/').fold(root.to_path_buf(), |p, s| p.join(s));
    fs::create_dir_all(path.parent().expect("a parent")).expect("create directories");
    fs::write(path, text).expect("write a file");
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

/// Runs `ascribe` and expects it to exit with `code`.
fn run(dir: &Path, args: &[&str], code: i32) -> Output {
    let out = ascribe(dir, args);
    assert_eq!(
        out.status.code(),
        Some(code),
        "ascribe {args:?}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    out
}

/// A repository with `main` checked out.
fn repository() -> TempDir {
    let dir = tempfile::tempdir().expect("a temporary directory");
    git(dir.path(), &["init", "-q"]);
    dir
}

/// Copies the files under `from` into `to`.
fn copy_tree(from: &Path, to: &Path) {
    for entry in fs::read_dir(from).unwrap() {
        let entry = entry.unwrap();
        let target = to.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            fs::create_dir_all(&target).unwrap();
            copy_tree(&entry.path(), &target);
        } else {
            fs::copy(entry.path(), target).unwrap();
        }
    }
}

/// `ascribe check` on `examples/quill`, with a link to a page's route on line
/// 7 of `keys.md`: the command reference's text, JSON, concise, and prompt
/// output.
#[test]
fn check() {
    let dir = tempfile::tempdir().unwrap();
    copy_tree(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/quill"),
        dir.path(),
    );
    let keys = dir.path().join("docs").join("keys.md");
    let text = fs::read_to_string(&keys).unwrap().replace("\r\n", "\n");
    let mut lines: Vec<&str> = text.lines().collect();
    assert_eq!(
        lines[6], "@id: create-key",
        "examples/quill/docs/keys.md changed"
    );
    lines[6] = "See [Install](/install-agent/).";
    fs::write(&keys, lines.join("\n") + "\n").unwrap();

    expect("check.txt", &stdout(&run(dir.path(), &["check"], 0)));
    expect(
        "check.json",
        &json(&run(dir.path(), &["check", "--format", "json"], 0)),
    );
    expect(
        "check-concise.txt",
        &stdout(&run(
            dir.path(),
            &["check", "docs/keys.md", "--format", "concise"],
            0,
        )),
    );
    expect(
        "check-prompt.txt",
        &stdout(&run(
            dir.path(),
            &["check", "docs/keys.md", "--format", "prompt"],
            0,
        )),
    );
}

/// The project `ascribe diff` is shown on: a product's docs in `docs/`, with a
/// build for its cloud edition that leaves out the self-managed pages.
const LANTERN: &str = r#"spec = "0.1"

[dimensions.deployment]
values = ["cloud", "self-managed"]
versionless = ["cloud"]

[builds.site]

[builds.cloud]
variants = { deployment = "cloud" }
availability = { filter = "cloud" }
"#;

const PREREQS: &str = "You need:\n\n- Lantern agent 2.2 or later\n- A Lantern Cloud account\n";

const GETTING_STARTED: &str = "---
title: Get started
---

Lantern sends your services' logs to one place.

## Before you start

Check what your servers have.

@include: _fragments/prereqs.md
";

const ROLLOUTS: &str = "---
title: Roll out the agent
available: self-managed
---

Roll the agent out to a few servers first, then to the rest.

## Pick the first servers

Pick servers that see little traffic.

Tag each one with `canary`.

## Install

Install the agent on the tagged servers.

Watch their logs for an hour.

## Roll back

Remove the agent from the tagged servers.

Restart each server.

## Finish

Install the agent everywhere else.
";

/// `ROLLOUTS`, with five blocks changed, three added, one removed, and one
/// moved.
const ROLLOUTS_NOW: &str = "---
title: Roll out the agent
available: self-managed
---

Roll the agent out to a few servers first, then to all the others.

## Pick the first servers

Pick servers that see little traffic and run the newest release.

Tag each one with `canary`.

## Install the agent

Install the agent on the tagged servers, one at a time.

Restart each server.

Watch their logs for a day.

Check the error rate in the dashboard.

## Roll back

Remove the agent from the tagged servers, and its configuration.

Send us the logs.

## Finish

Install the agent everywhere else.

Untag the servers.
";

const LIMITS: &str = "---
title: Limits
available: self-managed
---

Each agent sends at most 10,000 lines a second.
";

const SCHEDULES: &str = "---
title: Schedules
available: self-managed
---

Send logs on a schedule instead of as they're written.
";

/// The lantern repository: the docs on `main`, a `feature` branch checked
/// out, and a commit on `main` since it branched.
fn lantern() -> TempDir {
    let dir = repository();
    let root = dir.path();
    write(root, "README.md", "# Lantern\n");
    write(root, "docs/ascribe.toml", LANTERN);
    write(root, "docs/docs/_fragments/prereqs.md", PREREQS);
    write(root, "docs/docs/getting-started.md", GETTING_STARTED);
    write(root, "docs/docs/guides/rollouts.md", ROLLOUTS);
    write(root, "docs/docs/reference/limits.md", LIMITS);
    commit(root, "Write the docs");
    git(root, &["checkout", "-q", "-b", "feature"]);
    git(root, &["checkout", "-q", "main"]);
    write(root, "README.md", "# Lantern\n\nLogs, in one place.\n");
    commit(root, "Describe Lantern");
    git(root, &["checkout", "-q", "feature"]);
    dir
}

/// The agent's new version, in the fragment every page's prerequisites come
/// from.
fn new_agent(root: &Path) {
    write(
        root,
        "docs/docs/_fragments/prereqs.md",
        &PREREQS.replace("2.2", "2.4"),
    );
}

/// `ascribe diff`'s text output, for a change to every kind of thing.
#[test]
fn diff_text() {
    let dir = lantern();
    let root = dir.path();
    write(
        root,
        "docs/docs/_fragments/prereqs.md",
        &(PREREQS.replace("2.2", "2.4") + "- Port 4318 open\n"),
    );
    write(root, "docs/docs/guides/rollouts.md", ROLLOUTS_NOW);
    write(root, "docs/docs/guides/schedules.md", SCHEDULES);
    write(
        root,
        "docs/docs/reference/limits.md",
        &LIMITS.replace("title: Limits", "title: Rate limits"),
    );
    commit(root, "Document the new agent");
    let docs = root.join("docs");
    expect(
        "diff.txt",
        &stdout(&run(&docs, &["diff", "--base", "main"], 0)),
    );
}

/// `ascribe diff --format json`, for a change that reaches a page through a
/// fragment.
#[test]
fn diff_json() {
    let dir = lantern();
    let root = dir.path();
    new_agent(root);
    commit(root, "Require the new agent");
    let docs = root.join("docs");
    expect(
        "diff.json",
        &json(&run(
            &docs,
            &[
                "diff", "--base", "main", "--build", "site", "--format", "json",
            ],
            0,
        )),
    );
}

/// `ascribe diff --format prompt` about a page that changed through a
/// fragment: what the review guide and the agents guide show.
#[test]
fn diff_prompt() {
    let dir = lantern();
    let root = dir.path();
    new_agent(root);
    commit(root, "Require the new agent");
    let docs = root.join("docs");
    expect(
        "diff-prompt.txt",
        &stdout(&run(
            &docs,
            &[
                "diff",
                "docs/getting-started.md",
                "--base",
                "main",
                "--build",
                "site",
                "--format",
                "prompt",
            ],
            0,
        )),
    );
}

/// Code with tagged regions, as a page takes it.
const CLIENT: &str = "from lantern import Client, load_token


def connect(host, port):
    # :snippet-start: connect
    client = Client(host, port)
    client.authenticate(load_token())  # :remove:
    client.open()
    # :snippet-end:
    return client


def send(client, lines):
    # :snippet-start: retry
    for line in lines:
        client.send(line)
    # :snippet-end:
";

/// `CLIENT`, with `connect` opening the client with a timeout, and `send`
/// retrying.
const CLIENT_NOW: &str = "from lantern import Client, load_token


def connect(host, port):
    # :snippet-start: connect
    client = Client(host, port)
    client.authenticate(load_token())  # :remove:
    client.timeout = 30
    client.open(retries=3)
    # :snippet-end:
    return client


def send(client, lines):
    # :snippet-start: retry
    for line in lines:
        for attempt in range(3):
            try:
                client.send(line)
                break
            except TimeoutError:
                continue
    # :snippet-end:
";

const SERVICE_MODEL: &str = r#"spec = "0.1"

[sources.code]
path = ".."
include = ["service/**"]
"#;

const INSTALL: &str = "---
title: Install
---

Connect to the server:

@snippet: code:service/client.py#connect
";

const RETRIES: &str = "---
title: Retries
---

Sending a line can time out:

@snippet: code:service/client.py#retry
";

/// The drift guide's report: one example changed alone, and one with its page.
#[test]
fn drift_guide() {
    let dir = repository();
    let root = dir.path();
    write(root, "service/client.py", CLIENT);
    write(root, "docs/ascribe.toml", SERVICE_MODEL);
    write(root, "docs/docs/install.md", INSTALL);
    write(root, "docs/docs/guides/retries.md", RETRIES);
    commit(root, "Document the client");
    git(root, &["checkout", "-q", "-b", "feature"]);
    git(root, &["checkout", "-q", "main"]);
    write(root, "README.md", "# Lantern\n");
    commit(root, "Add a README");
    // As a CI checkout has it: the base branch only as `origin/main`.
    let main = String::from_utf8(
        Command::new("git")
            .current_dir(root)
            .args(["rev-parse", "main"])
            .output()
            .unwrap()
            .stdout,
    )
    .unwrap();
    git(
        root,
        &["update-ref", "refs/remotes/origin/main", main.trim()],
    );
    git(root, &["checkout", "-q", "feature"]);
    write(root, "service/client.py", CLIENT_NOW);
    write(
        root,
        "docs/docs/guides/retries.md",
        &RETRIES.replace(
            "can time out:",
            "can time out, so try each line three times:",
        ),
    );
    commit(root, "Retry");
    let docs = root.join("docs");
    expect(
        "drift-guide.txt",
        &stdout(&run(&docs, &["drift", "--base", "origin/main"], 0)),
    );
}

/// A content model whose regions the command reference's pages show.
const QUILL_MODEL: &str = r#"spec = "0.1"

# :snippet-start: dimensions
[dimensions.pm]
values = ["npm", "pnpm"]
# :snippet-end:

# :snippet-start: builds
[builds.site]
variants = "switch"
# :snippet-end:
"#;

/// `QUILL_MODEL`, with a dimension value added and a build's settings changed.
const QUILL_MODEL_NOW: &str = r#"spec = "0.1"

# :snippet-start: dimensions
[dimensions.pm]
label = "Package manager"
values = ["npm", "pnpm", "yarn"]
labels = { yarn = "Yarn" }

# :snippet-end:

# :snippet-start: builds
[builds.site]
variants = "switch"
availability = "badge"
# :snippet-end:
"#;

const CONTENT_MODEL_PAGE: &str = "---
title: ascribe.toml
---

Declare the dimensions your pages vary by:

@snippet: code:examples/quill/ascribe.toml#dimensions
";

const REVIEW_PAGE: &str = "---
title: Review
---

The review report renders the site build:

@snippet: code:examples/quill/ascribe.toml#builds
";

/// The command reference's report: every group.
#[test]
fn drift_reference() {
    let dir = lantern_with_code();
    let root = dir.path();
    write(root, "examples/quill/ascribe.toml", QUILL_MODEL_NOW);
    write(
        root,
        "service/client.py",
        &CLIENT.replace("snippet-start: connect", "snippet-start: open"),
    );
    write(
        root,
        "docs/docs/guides/review.md",
        &REVIEW_PAGE.replace(
            "renders the site build",
            "renders the site build, with badges",
        ),
    );
    commit(root, "Change the examples");
    let docs = root.join("docs");
    expect(
        "drift.txt",
        &stdout(&run(&docs, &["drift", "--base", "main"], 0)),
    );
}

/// A repository with code in `service/` and `examples/`, and docs that show
/// it, with a `feature` branch checked out and a commit on `main` since it
/// branched.
fn lantern_with_code() -> TempDir {
    let dir = repository();
    let root = dir.path();
    write(root, "service/client.py", CLIENT);
    write(root, "examples/quill/ascribe.toml", QUILL_MODEL);
    write(
        root,
        "docs/ascribe.toml",
        &SERVICE_MODEL.replace(r#"["service/**"]"#, r#"["examples/**", "service/**"]"#),
    );
    write(root, "docs/docs/guides/install.md", INSTALL);
    write(root, "docs/docs/guides/review.md", REVIEW_PAGE);
    write(
        root,
        "docs/docs/reference/content-model.md",
        CONTENT_MODEL_PAGE,
    );
    commit(root, "Document the examples");
    git(root, &["checkout", "-q", "-b", "feature"]);
    git(root, &["checkout", "-q", "main"]);
    write(root, "README.md", "# Lantern\n");
    commit(root, "Add a README");
    git(root, &["checkout", "-q", "feature"]);
    dir
}

/// `ascribe check` on `examples/getting-started`, the getting started guide's
/// project, which links to a page it doesn't have yet.
#[test]
fn getting_started() {
    let example = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/getting-started");
    expect(
        "getting-started-check.txt",
        &stdout(&run(&example, &["check"], 1)),
    );
}

fn file_url(dir: &Path) -> String {
    let text = dir.display().to_string().replace('\\', "/");
    if text.starts_with('/') {
        format!("file://{text}")
    } else {
        format!("file:///{text}")
    }
}

/// `ascribe sources update`, after three commits to the code repository: the
/// command reference's text output.
#[test]
fn sources_update() {
    let api = repository();
    write(
        api.path(),
        "src/auth.rs",
        "// :snippet-start: login\nfn login() {}\n// :snippet-end:\n",
    );
    write(api.path(), "examples/login.sh", "lantern login\n");
    commit(api.path(), "Add login");

    let docs = repository();
    let cache = tempfile::tempdir().unwrap();
    let model = format!(
        "spec = \"0.1\"\n\n[sources.api]\ngit = \"{}\"\nbranch = \"main\"\ninclude = [\"src/**\", \"examples/**\"]\n",
        file_url(api.path())
    );
    write(docs.path(), "ascribe.toml", &model);
    write(
        docs.path(),
        "docs/guides/auth.md",
        "---\ntitle: Sign in\n---\n\nLog in with:\n\n@snippet: api:src/auth.rs#login\n",
    );
    let sources = |args: &[&str]| {
        let out = Command::new(env!("CARGO_BIN_EXE_ascribe"))
            .current_dir(docs.path())
            .args(args)
            .env("NO_COLOR", "1")
            .env("ASCRIBE_CACHE_DIR", cache.path())
            .output()
            .expect("run ascribe");
        assert_eq!(
            out.status.code(),
            Some(0),
            "ascribe {args:?}: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        out
    };
    sources(&["sources", "fetch"]);
    commit(docs.path(), "Document signing in");

    write(api.path(), "examples/sh/login.sh", "lantern login\n");
    fs::remove_file(api.path().join("examples").join("login.sh")).unwrap();
    commit(api.path(), "Rename the examples folder");
    write(api.path(), "README.md", "# API\n");
    commit(api.path(), "Document the client");
    write(
        api.path(),
        "src/auth.rs",
        "// :snippet-start: login\nfn login(user: &str) {}\n// :snippet-end:\n",
    );
    commit(api.path(), "Take a user when logging in");

    expect(
        "sources-update.txt",
        &stdout(&sources(&["sources", "update"])),
    );
}

/// The commands that answer questions, on `examples/quill`, run from the
/// repository's root with paths into it: the command reference's examples.
#[test]
fn answers() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let quill = "examples/quill";
    let keys = "examples/quill/docs/keys.md";
    let install = "examples/quill/docs/install-agent.md";

    expect(
        "explain.txt",
        &stdout(&run(&root, &["explain", "link-target-missing"], 0)),
    );
    expect("model.txt", &stdout(&run(&root, &["model", quill], 0)));
    expect("outline.txt", &stdout(&run(&root, &["outline", keys], 0)));
    expect(
        "outline.json",
        &json(&run(&root, &["outline", keys, "--format", "json"], 0)),
    );
    expect(
        "link.txt",
        &stdout(&run(
            &root,
            &["link", "keys.md#rotate-keys", "--from", install],
            0,
        )),
    );
    expect(
        "link-missing.txt",
        &stdout(&run(
            &root,
            &["link", "keys.md#rotate", "--from", install],
            1,
        )),
    );
    expect(
        "link.json",
        &json(&run(
            &root,
            &[
                "link",
                "keys.md#rotate-keys",
                "--from",
                install,
                "--format",
                "json",
            ],
            0,
        )),
    );
    expect(
        "refs.txt",
        &stdout(&run(
            &root,
            &["refs", "phrase:cloud", "--project", quill, "--limit", "3"],
            0,
        )),
    );
    expect(
        "refs.json",
        &json(&run(
            &root,
            &[
                "refs",
                "phrase:cloud",
                "--project",
                quill,
                "--limit",
                "3",
                "--format",
                "json",
            ],
            0,
        )),
    );
    expect(
        "render.txt",
        &stdout(&run(&root, &["render", keys, "--build", "cloud"], 0)),
    );
}
