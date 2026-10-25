//! `ascribe.nav`'s checks and their fixes, by running the binary on
//! `examples/astro-site` and on projects of their own.

#![allow(clippy::expect_used, clippy::panic)]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use tempfile::TempDir;

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

fn write(path: &Path, text: &str) {
    fs::create_dir_all(path.parent().expect("a parent")).expect("create directories");
    fs::write(path, text).expect("write a file");
}

fn read(path: &Path) -> String {
    fs::read_to_string(path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

fn copy_dir(from: &Path, to: &Path) {
    fs::create_dir_all(to).expect("a directory");
    for entry in fs::read_dir(from).expect("a listing") {
        let entry = entry.expect("an entry");
        let target = to.join(entry.file_name());
        if entry.file_type().expect("a type").is_dir() {
            copy_dir(&entry.path(), &target);
        } else {
            fs::copy(entry.path(), target).expect("a copy");
        }
    }
}

/// `examples/astro-site`'s model, content, and API spec, without the site.
fn astro_site() -> TempDir {
    let dir = tempfile::tempdir().expect("a temporary directory");
    let source = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples/astro-site");
    fs::copy(source.join("ascribe.toml"), dir.path().join("ascribe.toml")).expect("the model");
    copy_dir(&source.join("content"), &dir.path().join("content"));
    copy_dir(&source.join("api"), &dir.path().join("api"));
    dir
}

/// A diagnostic as these tests compare it: its slug, file, and line, and
/// its fixes' titles and applicability.
type Problem = (String, String, u64, Vec<(String, String)>);

/// The report's navigation diagnostics.
fn nav_problems(dir: &Path) -> Vec<Problem> {
    let out = ascribe(dir, &["check", "--format", "json"]);
    let report: serde_json::Value =
        serde_json::from_str(&stdout(&out)).unwrap_or_else(|e| panic!("{e}: {}", stdout(&out)));
    report["diagnostics"]
        .as_array()
        .expect("a list")
        .iter()
        .filter(|d| {
            let slug = d["slug"].as_str().expect("a slug");
            slug.starts_with("nav-") || slug == "section-page-both"
        })
        .map(|d| {
            let fixes = d["fixes"]
                .as_array()
                .map(|fixes| {
                    fixes
                        .iter()
                        .map(|f| {
                            (
                                f["title"].as_str().expect("a title").to_owned(),
                                f["applicability"].as_str().expect("a label").to_owned(),
                            )
                        })
                        .collect()
                })
                .unwrap_or_default();
            (
                d["slug"].as_str().expect("a slug").to_owned(),
                d["file"].as_str().expect("a file").to_owned(),
                d["range"]["start"]["line"].as_u64().expect("a line"),
                fixes,
            )
        })
        .collect()
}

#[test]
fn a_misspelled_page_is_an_error_where_its_written_with_a_fix() {
    let dir = astro_site();
    write(
        &dir.path().join("ascribe.nav"),
        "version = 1\n\n[[sections]]\nfolder = \"\"\n\n[[sections]]\nfolder = \"Reference/\"\npages = [\"Options.md\", \"Optoins.md\"]\n\n[[sections]]\nfolder = \"Guides/\"\n",
    );
    assert_eq!(
        nav_problems(dir.path()),
        [(
            "nav-page-unknown".to_owned(),
            "ascribe.nav".to_owned(),
            8,
            vec![("Change it to `Options.md`".to_owned(), "unsafe".to_owned())]
        )]
    );
}

#[test]
fn an_unplaced_page_is_advice_on_it_and_fix_places_it() {
    let dir = astro_site();
    let nav = dir.path().join("ascribe.nav");
    write(
        &nav,
        "version = 1\n\n[[sections]]\nfolder = \"Reference/\"\npages = [\"Options.md\"] # In order.\n\n[[sections]]\nfolder = \"Guides/\"\n",
    );
    write(
        &dir.path().join("content/Reference/Defaults.md"),
        "---\ntype: page\ntitle: Defaults\n---\n\nThe defaults.\n",
    );
    assert_eq!(
        nav_problems(dir.path()),
        [(
            "nav-page-unplaced".to_owned(),
            "content/Reference/Defaults.md".to_owned(),
            3,
            vec![(
                "Add `Defaults.md` to `Reference/`'s pages in ascribe.nav".to_owned(),
                "safe".to_owned()
            )]
        )]
    );
    let fixed = ascribe(dir.path(), &["check", "--fix"]);
    assert!(
        stdout(&fixed).contains("fixed 1 problem in ascribe.nav"),
        "{}",
        stdout(&fixed)
    );
    assert_eq!(
        read(&nav),
        "version = 1\n\n[[sections]]\nfolder = \"Reference/\"\npages = [\"Options.md\", \"Defaults.md\"] # In order.\n\n[[sections]]\nfolder = \"Guides/\"\n",
        "the page is appended, and the comment and layout are kept"
    );
    assert!(nav_problems(dir.path()).is_empty());
}

#[test]
fn sections_written_inline_get_the_advice_without_a_fix() {
    let dir = astro_site();
    let nav = dir.path().join("ascribe.nav");
    let text = "version = 1\nsections = [{ folder = \"Guides/\" }]\n";
    write(&nav, text);
    let problems = nav_problems(dir.path());
    assert_eq!(
        problems
            .iter()
            .map(|(slug, _, _, fixes)| (slug.as_str(), fixes.len()))
            .collect::<Vec<_>>(),
        [("nav-section-unplaced", 0)],
        "{problems:?}"
    );
    // Tables appended after the inline list would be a second `sections`.
    ascribe(dir.path(), &["check", "--fix"]);
    assert_eq!(read(&nav), text);
    assert_eq!(nav_problems(dir.path()), problems);
}

/// A project whose sections and pages the file leaves out in an order
/// that isn't their paths': titles against file names, and a type ahead of
/// the default type.
const MODEL: &str = "spec = \"0.1\"\n\n[project]\ncontent-root = \"docs\"\n\n[types.page.frontmatter]\ntitle = \"string\"\n\n[checks]\npage-orphan = \"off\"\n";

#[test]
fn fix_leaves_the_order_a_reader_sees_as_it_was() {
    let dir = tempfile::tempdir().expect("a temporary directory");
    let root = dir.path();
    write(&root.join("ascribe.toml"), MODEL);
    let page = |path: &str, title: &str| {
        write(
            &root.join("docs").join(path),
            &format!("---\ntype: page\ntitle: {title}\n---\n\nText.\n"),
        );
    };
    page("index.md", "Home");
    page("guides/zeta.md", "Zeta");
    page("guides/a.md", "Yankee");
    page("guides/b.md", "Alpha");
    page("guides/c.md", "Mike");
    page("zebra/index.md", "Zebra");
    page("apple/index.md", "Apple");
    page("middle/one.md", "One");
    let nav = root.join("ascribe.nav");
    write(
        &nav,
        "version = 1\n\n[[sections]]\nfolder = \"guides/\"\npages = [\n  \"zeta.md\",\n]\n",
    );
    let before_problems = nav_problems(root);
    assert_eq!(
        before_problems
            .iter()
            .map(|(slug, file, _, _)| (slug.as_str(), file.as_str()))
            .collect::<Vec<_>>(),
        [
            ("nav-section-unplaced", "docs/apple/index.md"),
            ("nav-page-unplaced", "docs/guides/a.md"),
            ("nav-page-unplaced", "docs/guides/b.md"),
            ("nav-page-unplaced", "docs/guides/c.md"),
            ("nav-section-unplaced", "docs/middle/one.md"),
            ("nav-section-unplaced", "docs/zebra/index.md"),
        ]
    );
    let built = || {
        let out = ascribe(root, &["build", "--emit", "site"]);
        assert!(
            out.status.success(),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
        let found: Vec<PathBuf> = walk(root)
            .into_iter()
            .filter(|p| p.ends_with("_ascribe/nav.json"))
            .collect();
        assert_eq!(found.len(), 1, "{found:?}");
        read(&found[0])
    };
    let before = built();

    let fixed = ascribe(root, &["check", "--fix"]);
    assert!(
        stdout(&fixed).contains("in ascribe.nav"),
        "{}",
        stdout(&fixed)
    );
    assert_eq!(
        read(&nav),
        "version = 1\n\n[[sections]]\nfolder = \"guides/\"\npages = [\n  \"zeta.md\",\n  \"b.md\",\n  \"c.md\",\n  \"a.md\",\n]\n\n[[sections]]\nfolder = \"apple/\"\n\n[[sections]]\nfolder = \"middle/\"\n\n[[sections]]\nfolder = \"zebra/\"\n",
        "every page and section is placed in one run, in the order it had"
    );
    assert!(nav_problems(root).is_empty());
    assert_eq!(built(), before, "the navigation is the same");
}

/// Every file under `dir`.
fn walk(dir: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let Ok(entries) = fs::read_dir(dir) else {
        return out;
    };
    for entry in entries {
        let path = entry.expect("an entry").path();
        if path.is_dir() {
            out.extend(walk(&path));
        } else {
            out.push(path);
        }
    }
    out
}

#[test]
fn a_folder_with_both_own_pages_is_an_error_on_its_index() {
    let dir = tempfile::tempdir().expect("a temporary directory");
    let root = dir.path();
    write(&root.join("ascribe.toml"), MODEL);
    for (path, title) in [
        ("index.md", "Home"),
        ("checks.md", "Checks"),
        ("checks/index.md", "About checks"),
    ] {
        write(
            &root.join("docs").join(path),
            &format!("---\ntype: page\ntitle: {title}\n---\n\nText.\n"),
        );
    }
    assert_eq!(
        nav_problems(root),
        [(
            "section-page-both".to_owned(),
            "docs/checks/index.md".to_owned(),
            3,
            vec![]
        )]
    );
}

#[test]
fn ascribe_model_reports_the_navigation_with_each_section_s_listed() {
    let dir = tempfile::tempdir().expect("a temporary directory");
    write(
        &dir.path().join("ascribe.toml"),
        "spec = \"0.1\"\n\n[project]\ncontent-root = \"docs\"\n\n[builds.site]\n",
    );
    write(
        &dir.path().join("ascribe.nav"),
        "version = 1\n\n[[sections]]\nfolder = \"reference/\"\nlabel = \"Look it up\"\n",
    );
    let docs = dir.path().join("docs");
    write(&docs.join("index.md"), "---\ntitle: Home\n---\n");
    write(&docs.join("guides/sync.md"), "---\ntitle: Sync\n---\n");
    write(
        &docs.join("guides/drafts.md"),
        "---\ntitle: Drafts\nlisting: unlisted\n---\n",
    );
    write(
        &docs.join("reference/options.md"),
        "---\ntitle: Options\n---\n",
    );
    let output = ascribe(dir.path(), &["model", "--format", "json"]);
    assert!(output.status.success(), "{output:?}");
    let report: serde_json::Value = serde_json::from_str(&stdout(&output)).expect("JSON");
    assert_eq!(
        report["navigation"],
        serde_json::json!({
            "folder": "",
            "label": "Start",
            "page": { "source": "index.md", "label": "Home" , "type": "page" },
            "pages": [],
            "sections": [
                {
                    "folder": "reference/",
                    "label": "Look it up",
                    "pages": [{ "source": "reference/options.md", "label": "Options" , "type": "page" }],
                    "sections": [],
                    "listed": true,
                },
                {
                    "folder": "guides/",
                    "label": "Guides",
                    "pages": [{ "source": "guides/sync.md", "label": "Sync" , "type": "page" }],
                    "sections": [],
                    "listed": false,
                },
            ],
            "listed": false,
        })
    );
    // A section asked for alone has no navigation.
    let output = ascribe(
        dir.path(),
        &["model", "--format", "json", "--section", "builds"],
    );
    let report: serde_json::Value = serde_json::from_str(&stdout(&output)).expect("JSON");
    assert!(report.get("navigation").is_none(), "{report}");
}
