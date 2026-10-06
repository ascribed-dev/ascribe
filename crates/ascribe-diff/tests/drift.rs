//! Which pages' examples changed, in a temporary repository: the project is
//! in `site/`, its code beside it, and the change is a branch.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::path::PathBuf;
use std::process::Command;
use std::sync::Arc;

use ascribe_diff::{DiffError, DriftPage, DriftReport, Repository, Side, drift};
use ascribe_resolve::{DiskFs, Layout, Project};

const MODEL: &str = "spec = \"0.1\"\n[project]\ncontent-root = \"docs\"\n\
                     [dimensions.deployment]\nvalues = [\"cloud\", \"self-managed\"]\n\
                     [sources.code]\npath = \"..\"\ninclude = [\"service/**\"]\n\
                     [builds.site]\nvariants = \"switch\"\n\
                     [builds.cloud]\nvariants = { deployment = \"cloud\" }\n";

/// A Python file with one region, `main`, and lines before and after it.
fn code(before: &str, main: &str, after: &str) -> String {
    format!("{before}def run():\n    # :snippet-start: main\n{main}    # :snippet-end:\n{after}")
}

const MAIN: &str = "    client = connect()\n    client.open()\n";

/// A temporary repository with a `main` branch.
struct Repo {
    dir: tempfile::TempDir,
}

impl Repo {
    /// A repository with the project in `site/` and `service/app.py`, on
    /// `main`, and a `feature` branch checked out.
    fn new(pages: &[(&str, &str)]) -> Repo {
        let repo = Repo {
            dir: tempfile::tempdir().unwrap(),
        };
        repo.git(&["init", "-q", "-b", "main"]);
        repo.write("site/ascribe.toml", MODEL);
        repo.write("service/app.py", &code("import os\n", MAIN, "run()\n"));
        for (path, text) in pages {
            repo.write(&format!("site/docs/{path}"), text);
        }
        repo.commit("first");
        repo.git(&["checkout", "-q", "-b", "feature"]);
        repo
    }

    fn path(&self, rel: &str) -> PathBuf {
        rel.split('/')
            .fold(self.dir.path().to_path_buf(), |p, s| p.join(s))
    }

    fn git(&self, args: &[&str]) -> String {
        let out = Command::new("git")
            .current_dir(self.dir.path())
            .args([
                "-c",
                "user.name=Test",
                "-c",
                "user.email=test@example.com",
                "-c",
                "commit.gpgsign=false",
                "-c",
                "core.autocrlf=false",
            ])
            .args(args)
            .output()
            .unwrap();
        assert!(
            out.status.success(),
            "git {args:?}: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        String::from_utf8(out.stdout).unwrap().trim().to_owned()
    }

    fn write(&self, rel: &str, text: &str) {
        let path = self.path(rel);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, text).unwrap();
    }

    fn commit(&self, message: &str) {
        self.git(&["add", "-A"]);
        self.git(&["commit", "-q", "--allow-empty", "-m", message]);
    }

    /// The report for the project in `dir`, compared with `main`, for
    /// `builds`.
    fn drift_in(&self, dir: &str, builds: &[&str]) -> Result<DriftReport, DiffError> {
        let root = self.path(dir);
        let model_text = std::fs::read_to_string(root.join("ascribe.toml")).unwrap();
        let model = ascribe_model::load_str(&model_text, ascribe_core::FileId::new(0)).unwrap();
        let layout = Layout::from_model(&model);
        let fs = DiskFs::new(&root, &layout);
        let project = Project::load(Arc::new(model), layout, &fs);
        let repo = Repository::discover(&root)?;
        let base = repo.base(Some("main"), false)?;
        let now = Side {
            project: &project,
            model_text: &model_text,
        };
        drift(&repo, &base, now, &fs, builds)
    }

    fn drift(&self) -> DriftReport {
        self.drift_in("site", &["site", "cloud"]).unwrap()
    }
}

/// Each page, `changed` when the page changed too, with its examples and
/// how much each changed.
fn summary(report: &DriftReport) -> Vec<String> {
    report.pages.iter().map(line).collect()
}

fn line(page: &DriftPage) -> String {
    let mut examples: Vec<String> = page
        .examples
        .iter()
        .map(|e| format!("{} +{} -{}", e.address, e.added, e.removed))
        .collect();
    examples.extend(
        page.broken
            .iter()
            .map(|b| format!("{} broken: {}", b.address, b.reason)),
    );
    let changed = if page.page_changed { " changed" } else { "" };
    format!("{}{changed}: {}", page.path, examples.join(", "))
}

const RUN: &str = "# Run\n\nConnect first:\n\n@snippet: code:service/app.py#main\n\nThen run.\n";

#[test]
fn a_region_that_changed_while_the_page_didnt() {
    let repo = Repo::new(&[("run.md", RUN), ("other.md", "# Other\n")]);
    repo.write(
        "service/app.py",
        &code(
            "import os\n",
            "    client = connect(retries=3)\n    client.open()\n    client.ping()\n",
            "run()\n",
        ),
    );
    repo.commit("change the code");
    let report = repo.drift();
    assert_eq!(summary(&report), ["run.md: code:service/app.py#main +2 -1"]);
    let page = &report.pages[0];
    assert_eq!(page.builds, ["site", "cloud"]);
    assert_eq!(page.route, "/run/");
    let example = &page.examples[0];
    assert_eq!(example.source, "code");
    assert_eq!(example.file, "service/app.py");
    assert_eq!(example.was_file, None);
    assert_eq!(report.unchanged_pages().count(), 1);
}

#[test]
fn a_region_that_changed_with_the_page() {
    let repo = Repo::new(&[("run.md", RUN)]);
    repo.write("service/app.py", &code("", "    connect()\n", ""));
    repo.write(
        "site/docs/run.md",
        &RUN.replace("Then run.", "Then run it."),
    );
    repo.commit("change both");
    assert_eq!(
        summary(&repo.drift()),
        ["run.md changed: code:service/app.py#main +1 -2"]
    );
}

#[test]
fn an_edit_outside_the_region_a_move_and_whitespace_arent_changes() {
    let repo = Repo::new(&[("run.md", RUN)]);
    // Lines added above and below move the region; its own lines get
    // trailing spaces and a deeper indent, which dedenting removes.
    repo.write(
        "service/app.py",
        &code(
            "import os\nimport sys\n\n",
            "        client = connect()  \n        client.open()\n",
            "run()\nprint('done')\n",
        ),
    );
    repo.commit("touch the file");
    assert!(repo.drift().pages.is_empty());
}

#[test]
fn a_page_changed_only_through_a_fragment() {
    let page = "# Run\n\n@include: _fragments/intro.md\n\n@snippet: code:service/app.py#main\n";
    let repo = Repo::new(&[
        ("run.md", page),
        ("_fragments/intro.md", "Connect first.\n"),
    ]);
    repo.write("site/docs/_fragments/intro.md", "Connect, then open.\n");
    repo.write("service/app.py", &code("", "    connect()\n", ""));
    repo.commit("change the fragment and the code");
    assert_eq!(
        summary(&repo.drift()),
        ["run.md changed: code:service/app.py#main +1 -2"]
    );
}

#[test]
fn a_fragments_snippet_counts_for_every_page_that_includes_it() {
    let fragment = "Connect first:\n\n@snippet: code:service/app.py#main\n";
    let repo = Repo::new(&[
        ("_fragments/connect.md", fragment),
        ("run.md", "# Run\n\n@include: _fragments/connect.md\n"),
        ("deploy.md", "# Deploy\n\n@include: _fragments/connect.md\n"),
        ("other.md", "# Other\n"),
    ]);
    repo.write("service/app.py", &code("", "    connect()\n", ""));
    repo.commit("change the code");
    assert_eq!(
        summary(&repo.drift()),
        [
            "deploy.md: code:service/app.py#main +1 -2",
            "run.md: code:service/app.py#main +1 -2",
        ]
    );
}

#[test]
fn a_whole_file_snippet() {
    let page = "# Config\n\n@snippet: code:service/config.toml\n";
    let repo = Repo::new(&[("config.md", page)]);
    repo.write("service/config.toml", "port = 80\n");
    repo.commit("add the config");
    repo.git(&["checkout", "-q", "main"]);
    repo.git(&["merge", "-q", "feature"]);
    repo.git(&["checkout", "-q", "feature"]);
    repo.write("service/config.toml", "port = 8080\nhost = \"::\"\n");
    repo.commit("change the config");
    assert_eq!(
        summary(&repo.drift()),
        ["config.md: code:service/config.toml +2 -1"]
    );
}

#[test]
fn a_renamed_file_is_followed() {
    let repo = Repo::new(&[("run.md", RUN)]);
    repo.git(&["mv", "service/app.py", "service/client.py"]);
    repo.write(
        "service/client.py",
        &code("import os\n", &MAIN.replace("open()", "start()"), "run()\n"),
    );
    repo.write(
        "site/docs/run.md",
        &RUN.replace("service/app.py", "service/client.py"),
    );
    repo.commit("rename the file");
    let report = repo.drift();
    assert_eq!(
        summary(&report),
        ["run.md changed: code:service/client.py#main +1 -1"]
    );
    let example = &report.pages[0].examples[0];
    assert_eq!(example.file, "service/client.py");
    assert_eq!(example.was_file.as_deref(), Some("service/app.py"));
}

#[test]
fn a_moved_source_folder_is_followed() {
    let repo = Repo::new(&[("run.md", RUN.replace("service/", "").as_str())]);
    repo.write(
        "site/ascribe.toml",
        &MODEL.replace(
            "path = \"..\"\ninclude = [\"service/**\"]",
            "path = \"../service\"",
        ),
    );
    repo.commit("a source of its own");
    repo.git(&["checkout", "-q", "main"]);
    repo.git(&["merge", "-q", "feature"]);
    repo.git(&["checkout", "-q", "feature"]);
    // The page's address stays; its file moves with the source's folder.
    repo.git(&["mv", "service", "server"]);
    repo.write(
        "site/ascribe.toml",
        &MODEL.replace(
            "path = \"..\"\ninclude = [\"service/**\"]",
            "path = \"../server\"",
        ),
    );
    repo.write(
        "server/app.py",
        &code("import os\n", &MAIN.replace("open()", "start()"), "run()\n"),
    );
    repo.commit("move the source");
    let report = repo.drift();
    assert_eq!(summary(&report), ["run.md: code:app.py#main +1 -1"]);
    assert_eq!(
        report.pages[0].examples[0].was_file.as_deref(),
        Some("service/app.py")
    );
}

#[test]
fn a_new_region_is_a_new_example() {
    let repo = Repo::new(&[("run.md", "# Run\n")]);
    repo.write(
        "service/app.py",
        &format!(
            "{}# :snippet-start: more\nmore()\n# :snippet-end:\n",
            code("import os\n", MAIN, "run()\n")
        ),
    );
    repo.write(
        "site/docs/run.md",
        "# Run\n\n@snippet: code:service/app.py#more\n",
    );
    repo.commit("a new example");
    assert!(repo.drift().pages.is_empty());
}

#[test]
fn only_the_builds_that_show_the_example() {
    let page = "# Run\n\n@variant {deployment=self-managed}:\n@snippet: code:service/app.py#main\n@end\n\n@variant {deployment=cloud}:\nNothing to run.\n@end\n";
    let repo = Repo::new(&[("run.md", page)]);
    repo.write("service/app.py", &code("", "    connect()\n", ""));
    repo.commit("change the code");
    let report = repo.drift();
    assert_eq!(summary(&report), ["run.md: code:service/app.py#main +1 -2"]);
    assert_eq!(report.pages[0].builds, ["site"]);
    assert!(
        repo.drift_in("site", &["cloud"]).unwrap().pages.is_empty(),
        "the cloud build doesn't show the example"
    );
}

#[test]
fn a_project_at_the_root_of_the_repository() {
    let repo = Repo::new(&[]);
    repo.write(
        "ascribe.toml",
        &MODEL.replace("path = \"..\"", "path = \".\""),
    );
    repo.write("docs/run.md", RUN);
    repo.commit("a project at the root");
    repo.git(&["checkout", "-q", "main"]);
    repo.git(&["merge", "-q", "feature"]);
    repo.git(&["checkout", "-q", "feature"]);
    repo.write("service/app.py", &code("", "    connect()\n", ""));
    // Uncommitted: the working tree is what's compared.
    let report = repo.drift_in(".", &["site"]).unwrap();
    assert_eq!(summary(&report), ["run.md: code:service/app.py#main +1 -2"]);
    assert_eq!(report.repository.project_prefix, "");
}

#[test]
fn a_project_without_sources_has_nothing_to_report() {
    let repo = Repo::new(&[("run.md", "# Run\n")]);
    repo.write(
        "site/ascribe.toml",
        "spec = \"0.1\"\n[project]\ncontent-root = \"docs\"\n[builds.site]\n",
    );
    repo.write("site/docs/run.md", "# Run it\n");
    repo.commit("no sources");
    let report = repo.drift_in("site", &["site"]).unwrap();
    assert!(report.pages.is_empty());
    assert_eq!(report.repository.project_prefix, "site/");
}

#[test]
fn a_renamed_region_is_a_broken_example() {
    let repo = Repo::new(&[("run.md", RUN), ("other.md", "# Other\n")]);
    repo.write(
        "service/app.py",
        &code("import os\n", MAIN, "run()\n").replace(": main", ": entry"),
    );
    repo.commit("rename the region");
    let report = repo.drift();
    assert_eq!(
        summary(&report),
        ["run.md: code:service/app.py#main broken: the file has no region `main`"]
    );
    let broken = &report.pages[0].broken[0];
    assert_eq!(broken.problem, "snippet-region-missing");
    assert_eq!(broken.source, "code");
    assert_eq!(report.pages[0].builds, ["site", "cloud"]);
    assert!(report.needs_reading());
    assert_eq!(report.unchanged_pages().count(), 0);
    assert_eq!(report.broken_pages().count(), 1);
}

#[test]
fn a_renamed_file_the_page_still_names_is_a_broken_example() {
    let fragment = "Connect first:\n\n@snippet: code:service/app.py#main\n";
    let repo = Repo::new(&[
        ("_fragments/connect.md", fragment),
        ("run.md", "# Run\n\n@include: _fragments/connect.md\n"),
    ]);
    repo.git(&["mv", "service/app.py", "service/client.py"]);
    repo.commit("rename the file");
    assert_eq!(
        summary(&repo.drift()),
        ["run.md: code:service/app.py#main broken: the file isn't there"]
    );
}

#[test]
fn a_snippet_that_never_resolved_isnt_reported() {
    let repo = Repo::new(&[("run.md", "# Run\n")]);
    repo.write(
        "site/docs/run.md",
        "# Run\n\n@snippet: code:service/app.py#nope\n",
    );
    repo.commit("a broken snippet, new");
    assert!(repo.drift().pages.is_empty());
}
