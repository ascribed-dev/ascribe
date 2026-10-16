//! The archive of Quill's `self-managed 3.4`, in memory: what's in the
//! folder, and that it needs nothing else. The end-to-end suite opens it in
//! a browser (`examples/astro-site/test/e2e/archive.test.ts`).

#![allow(clippy::expect_used, clippy::panic)]

use crate::support;

use std::collections::BTreeMap;

use ascribe_core::RelPath;
use ascribe_emit::{Archive, ArchiveOptions, Contents, FileKind, archive};
use support::{load, quill};

fn archived(current: Option<&str>) -> Archive {
    let root = quill();
    let project = load(&root);
    let line = ascribe_model::named_line("self-managed 3.4")
        .expect("a line")
        .1;
    let build = project
        .model()
        .line_build("site", &line)
        .expect("the line's build");
    let options = ArchiveOptions {
        current: current.map(str::to_owned),
    };
    archive(&project, &root, &build, &options).expect("archived")
}

/// Each text file by its path.
fn texts(archive: &Archive) -> BTreeMap<String, String> {
    archive
        .files
        .iter()
        .filter_map(|f| match &f.contents {
            Contents::Text(text) => Some((f.path.to_string(), text.clone())),
            Contents::Copy(_) => None,
        })
        .collect()
}

/// The value of every `name="…"` in `html`.
fn values<'a>(html: &'a str, name: &str) -> Vec<&'a str> {
    let open = format!(" {name}=\"");
    html.match_indices(&open)
        .filter_map(|(at, _)| {
            let start = at + open.len();
            html[start..].find('"').map(|len| &html[start..start + len])
        })
        .collect()
}

#[test]
fn the_folder_has_the_pages_their_assets_and_its_own_files() {
    let archive = archived(None);
    let paths: Vec<String> = archive.files.iter().map(|f| f.path.to_string()).collect();
    assert_eq!(
        paths,
        [
            "_fragments/prerequisites.png",
            "all.html",
            "configure-legacy.html",
            "index.html",
            "install-agent.html",
            "keys.html",
            "playground.png",
            "quickstart.html",
            "robots.txt",
            "style.css",
            "upgrade.html",
        ]
    );
    assert_eq!(archive.pages, 5);
    assert_eq!(archive.contents.as_str(), "index.html");
    assert_eq!(
        texts(&archive)["robots.txt"],
        "User-agent: *\nDisallow: /\n"
    );
}

#[test]
fn every_link_and_image_in_the_folder_leads_to_a_file_in_it() {
    let archive = archived(None);
    let files: Vec<String> = archive.files.iter().map(|f| f.path.to_string()).collect();
    for (path, html) in texts(&archive).iter().filter(|(p, _)| p.ends_with(".html")) {
        let dir = RelPath::parse(path)
            .expect("a path")
            .parent()
            .expect("a folder");
        for value in values(html, "href").into_iter().chain(values(html, "src")) {
            if value.starts_with('#') || value.starts_with("https://") {
                continue;
            }
            let file = value.split('#').next().unwrap_or_default();
            let target = dir.join(file).expect("a path");
            assert!(
                files.contains(&target.to_string()),
                "{path} links to {value}, which isn't in the folder"
            );
        }
    }
}

#[test]
fn every_page_says_what_it_is_keeps_crawlers_out_and_has_no_script() {
    let archive = archived(Some("https://docs.quill.dev/"));
    for (path, html) in texts(&archive).iter().filter(|(p, _)| p.ends_with(".html")) {
        assert!(
            html.contains(
                "Documentation for self-managed 3.4. This version is no longer maintained."
            ),
            "{path}"
        );
        assert!(
            html.contains(
                "<a href=\"https://docs.quill.dev/\">Read the current documentation.</a>"
            ),
            "{path}"
        );
        assert!(
            html.contains("<meta name=\"robots\" content=\"noindex\">"),
            "{path}"
        );
        assert!(!html.contains("<script"), "{path}");
    }
}

#[test]
fn a_link_out_of_the_line_is_its_text_and_listed() {
    let archive = archived(None);
    let upgrade = &texts(&archive)["upgrade.html"];
    assert!(upgrade.contains("3.5 adds webhooks, and"), "{upgrade}");
    assert!(!upgrade.contains("data-ascribe-release-line"));
    let unlinked: Vec<String> = archive
        .unlinked
        .iter()
        .map(|u| format!("{} {} {:?} {}", u.page, u.href, u.line, u.text))
        .collect();
    assert_eq!(unlinked, ["upgrade.md /webhooks/ Some(\"3.5\") webhooks"]);
}

#[test]
fn every_page_is_in_the_sidebar_of_every_other_and_in_all_html_once() {
    let archive = archived(None);
    let texts = texts(&archive);
    let pages: Vec<&str> = archive
        .files
        .iter()
        .filter(|f| f.kind == FileKind::Page)
        .map(|f| f.path.as_str())
        .collect();
    for (path, html) in texts
        .iter()
        .filter(|(p, _)| p.ends_with(".html") && *p != "all.html")
    {
        let sidebar = &html[html.find("archive-sidebar").expect("a sidebar")..];
        let sidebar = &sidebar[..sidebar.find("</nav>").expect("its end")];
        for page in &pages {
            assert!(
                sidebar.contains(&format!("href=\"./{page}\"")),
                "{path}: {page}"
            );
        }
    }
    let all = &texts["all.html"];
    assert_eq!(all.matches("<article>").count(), pages.len());
    assert!(!all.contains("archive-sidebar"));
}

#[test]
fn an_archive_is_the_same_bytes_made_twice() {
    let (a, b) = (archived(None), archived(None));
    assert_eq!(texts(&a), texts(&b));
}

/// A project with folders and a root page, its line 2.0 planned.
const FOLDERS: &str = r#"spec = "0.1"

[dimensions.product]
values = ["app"]

[releases.app]
live = ["1.0"]
planned = ["2.0"]

[builds.site]
availability = "badge"
versions = "app"
"#;

#[test]
fn folders_are_named_by_their_index_page_and_the_contents_page_moves_aside_for_the_root() {
    let project = support::memory_project(
        FOLDERS,
        &[
            (
                "index.md",
                "---\ntitle: Home\n---\n\nSee [b](guide/b.md).\n",
            ),
            ("guide/index.md", "---\ntitle: The guide\n---\n\nText.\n"),
            (
                "guide/b.md",
                "---\ntitle: B\n---\n\nBack [home](../index.md).\n",
            ),
            (
                "guide/a.md",
                "---\ntitle: A\navailable: app 2.0\n---\n\nNew.\n",
            ),
            ("ref/z.md", "---\ntitle: Z\n---\n\nText.\n"),
        ],
    );
    let line = ascribe_model::named_line("app 2.0").expect("a line").1;
    let build = project
        .model()
        .line_build("site", &line)
        .expect("a planned line's build");
    assert_eq!(build.name, "site@2.0");
    let archive = archive(
        &project,
        std::path::Path::new("/nowhere"),
        &build,
        &ArchiveOptions::default(),
    )
    .expect("archived");
    assert_eq!(archive.contents.as_str(), "contents.html");
    let texts = texts(&archive);
    let b = &texts["guide/b.html"];
    assert!(b.contains("<a href=\"../index.html\">home</a>"), "{b}");
    assert!(
        b.contains("<link rel=\"stylesheet\" href=\"../style.css\">"),
        "{b}"
    );
    let sidebar = &b[b.find("archive-sidebar").expect("a sidebar")..];
    let sidebar = &sidebar[..sidebar.find("</nav>").expect("its end")];
    assert_eq!(
        values(sidebar, "href"),
        [
            "../contents.html",
            "../index.html",
            "./index.html",
            "./a.html",
            "./b.html",
            "../ref/z.html"
        ]
    );
    assert!(
        sidebar.contains("<p class=\"archive-folder\">The guide</p>"),
        "{sidebar}"
    );
    assert!(
        sidebar.contains("<p class=\"archive-folder\">ref</p>"),
        "{sidebar}"
    );
    assert!(
        b.contains("<a rel=\"prev\" href=\"./a.html\">Previous: A</a>"),
        "{b}"
    );
    assert!(
        b.contains("<a rel=\"next\" href=\"../ref/z.html\">Next: Z</a>"),
        "{b}"
    );
    let all = &texts["all.html"];
    assert!(all.contains("<a href=\"./index.html\">home</a>"), "{all}");
}
