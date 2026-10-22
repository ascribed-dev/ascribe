#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! `[from]` (SPEC §7.9): what one project takes from another's
//! `ascribe.toml`, read through a [`TakenFiles`] that holds the other files
//! in memory, in a folder that's a git repository.

use std::collections::HashMap;
use std::io;
use std::path::{Path, PathBuf};

use ascribe_core::{FileId, Issue, LineIndex, RelPath};
use ascribe_model::{ContentModel, SourceBound, TakenFiles, load_str_taking};

/// The files the loader may read, by their path from the project's folder.
struct Files(HashMap<&'static str, &'static str>);

impl TakenFiles for Files {
    fn read_taken(&self, project_path: &RelPath) -> io::Result<String> {
        self.0
            .get(project_path.as_str())
            .map(|text| (*text).to_owned())
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "no such file"))
    }
}

/// The docs' content model, which the site takes from.
const DOCS: &str = r#"spec = "0.1"

[dimensions.pm]
label = "Package manager"
values = ["npm", "pnpm"]

[releases.ascribe]
live = ["0.4.1"]
planned = ["0.4.2"]

[phrases]
version = { line = "ascribe" }
node = "24"

[sources.code]
path = ".."
include = ["examples/**"]

[consumer]
site = "https://example.com"
trailing-slash = "never"
"#;

/// A repository with the docs in `docs/` and the project in `site/home/`,
/// whose content root is there.
fn repository(name: &str) -> PathBuf {
    let dir =
        std::env::temp_dir().join(format!("ascribe-model-from-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join(".git")).unwrap();
    std::fs::create_dir_all(dir.join("site/home/content")).unwrap();
    std::fs::create_dir_all(dir.join("docs")).unwrap();
    std::fs::create_dir_all(dir.join("examples")).unwrap();
    dir
}

fn files() -> Files {
    Files(HashMap::from([("../../docs/ascribe.toml", DOCS)]))
}

fn load(dir: &Path, text: &str) -> Result<ContentModel, Vec<Issue>> {
    load_str_taking(
        text,
        FileId::new(0),
        &dir.join("site/home"),
        SourceBound::Anywhere,
        &files(),
    )
}

const SITE: &str = r#"spec = "0.1"

[project]
content-root = "content"

[from."../../docs/ascribe.toml"]
take = ["releases", "dimensions.pm", "phrases.version", "sources.code", "consumer.site"]

[phrases]
site = "https://example.com"

[consumer]
base-path = "/"
"#;

#[test]
fn what_is_taken_is_read_as_if_written_here() {
    let dir = repository("taken");
    let model = load(&dir, SITE).unwrap_or_else(|issues| panic!("{issues:#?}"));
    // A whole table, and a table with one key.
    assert_eq!(model.releases[0].target, "ascribe");
    assert_eq!(
        model.dimension("pm").map(|d| d.values.len()),
        Some(2),
        "{:?}",
        model.dimensions
    );
    // One key of a table, beside this file's own keys there.
    let phrases: Vec<&str> = model.phrases.iter().map(|p| p.key.as_str()).collect();
    assert_eq!(phrases, ["site", "version"]);
    assert_eq!(model.consumer.site.as_deref(), Some("https://example.com"));
    assert_eq!(model.consumer.base_path, "/");
    // The key it didn't take is the default, not the other file's.
    assert_eq!(
        model.consumer.trailing_slash,
        ascribe_model::TrailingSlash::Always
    );
    // A source's path is from the other file's folder, where it's written.
    assert_eq!(model.source("code").map(|s| s.path.as_str()), Some("../.."));

    // Where each came from, at the name in `take`.
    let taken: Vec<(&str, usize)> = model
        .taken
        .iter()
        .map(|t| (t.key.as_str(), t.line))
        .collect();
    assert_eq!(
        taken,
        [
            ("releases", 7),
            ("dimensions.pm", 3),
            ("phrases.version", 12),
            ("sources.code", 15),
            ("consumer.site", 20),
        ]
    );
    let take = &model.taken[0];
    assert_eq!(take.file, "../../docs/ascribe.toml");
    assert_eq!(&SITE[take.span.range()], "\"releases\"");
    let _ = std::fs::remove_dir_all(&dir);
}

/// The slug, the variant, and the line of `text` of each issue loading
/// `text` reports.
fn issues(dir: &Path, text: &str) -> Vec<(String, Option<&'static str>, u32)> {
    let index = LineIndex::new(text);
    let issues = match load(dir, text) {
        Ok(model) => model.warnings,
        Err(issues) => issues,
    };
    issues
        .iter()
        .map(|i| {
            let line = index.line_col(i.location.span.start()).unwrap().line + 1;
            (i.slug.as_str().to_owned(), i.variant, line)
        })
        .collect()
}

#[test]
fn a_key_taken_and_written_here_is_an_error() {
    let dir = repository("written");
    let text = "spec = \"0.1\"\n[project]\ncontent-root = \"content\"\n[from.\"../../docs/ascribe.toml\"]\ntake = [\"consumer.site\", \"phrases\"]\n[consumer]\nsite = \"https://example.org\"\n[phrases]\nversion = \"0.4\"\n";
    assert_eq!(
        issues(&dir, text),
        [
            ("model-from-written".to_owned(), None, 5),
            ("model-from-written".to_owned(), None, 5),
        ]
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn what_the_other_file_doesnt_have_cant_be_taken() {
    let dir = repository("missing-key");
    let text = "spec = \"0.1\"\n[project]\ncontent-root = \"content\"\n[from.\"../../docs/ascribe.toml\"]\ntake = [\"phrases.product\"]\n";
    assert_eq!(
        issues(&dir, text),
        [("model-from-take".to_owned(), None, 5)]
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn the_file_must_be_there_be_toml_and_be_in_the_repository() {
    let dir = repository("file");
    let model = |path: &str| {
        format!(
            "spec = \"0.1\"\n[project]\ncontent-root = \"content\"\n[from.\"{path}\"]\ntake = [\"phrases\"]\n"
        )
    };
    assert_eq!(
        issues(&dir, &model("../../api/ascribe.toml")),
        [("model-from-file".to_owned(), None, 4)]
    );
    assert_eq!(
        issues(&dir, &model("../../../elsewhere/ascribe.toml")),
        [("model-from-file".to_owned(), Some("outside-repository"), 4)]
    );
    let not_toml = Files(HashMap::from([("../../docs/ascribe.toml", "spec = = 1\n")]));
    let issues = load_str_taking(
        &model("../../docs/ascribe.toml"),
        FileId::new(0),
        &dir.join("site/home"),
        SourceBound::Anywhere,
        &not_toml,
    )
    .unwrap_err();
    assert_eq!(issues[0].slug.as_str(), "model-from-file");
    assert_eq!(issues[0].variant, Some("unreadable"));
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn the_other_files_own_from_isnt_followed() {
    let dir = repository("one-step");
    let docs = "spec = \"0.1\"\n[from.\"../api/ascribe.toml\"]\ntake = [\"phrases.product\"]\n[phrases]\nnode = \"24\"\n";
    let files = Files(HashMap::from([("../../docs/ascribe.toml", docs)]));
    let text = "spec = \"0.1\"\n[project]\ncontent-root = \"content\"\n[from.\"../../docs/ascribe.toml\"]\ntake = [\"phrases\"]\n";
    let model = load_str_taking(
        text,
        FileId::new(0),
        &dir.join("site/home"),
        SourceBound::Anywhere,
        &files,
    )
    .unwrap_or_else(|issues| panic!("{issues:#?}"));
    let phrases: Vec<&str> = model.phrases.iter().map(|p| p.key.as_str()).collect();
    assert_eq!(phrases, ["node"]);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn without_a_reader_nothing_is_taken() {
    let text = "spec = \"0.1\"\n[from.\"../docs/ascribe.toml\"]\ntake = [\"phrases.version\"]\n";
    let model = ascribe_model::load_str(text, FileId::new(0)).unwrap();
    assert!(model.taken.is_empty());
    assert!(model.phrases.is_empty());
}
