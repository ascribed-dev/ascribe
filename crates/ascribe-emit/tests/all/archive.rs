//! The archive of Quill's `self-managed 3.4`, in memory: what's in the
//! folder, and that it needs nothing else. The end-to-end suite opens it in
//! a browser (`examples/astro-site/test/e2e/archive.test.ts`).

#![allow(clippy::expect_used, clippy::panic)]

use crate::support;

use std::collections::BTreeMap;

use ascribe_core::RelPath;
use ascribe_emit::{Archive, ArchiveOptions, Contents, FileKind, archive, preview};
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

/// `ascribe preview`'s rendering of a build is the archive's: the same
/// files, the same bytes but for the statement at the top of each page,
/// what each page's title says it's of, and the search box's label.
#[test]
fn a_preview_is_the_archive_with_its_own_statement() {
    let root = quill();
    let project = load(&root);
    let line = ascribe_model::named_line("self-managed 3.4")
        .expect("a line")
        .1;
    let build = project
        .model()
        .line_build("site", &line)
        .expect("the line's build");
    let previewed = preview(&project, &root, &build).expect("previewed");
    let archived = archived(None);
    let paths = |a: &Archive| {
        a.files
            .iter()
            .map(|f| f.path.to_string())
            .collect::<Vec<_>>()
    };
    assert_eq!(paths(&previewed), paths(&archived));
    assert_eq!(previewed.pages, archived.pages);
    let statement =
        "<p>Documentation for self-managed 3.4. This version is no longer maintained.</p>";
    let previewing = format!(
        "<p>A preview of build <code>{}</code>, written by <code>ascribe preview</code>. The published site has its own look.</p>",
        build.name
    );
    let archived = texts(&archived);
    for (path, text) in texts(&previewed) {
        let expected = archived[&path]
            .replace(statement, &previewing)
            .replace(
                " · self-managed 3.4</title>",
                &format!(" · {} preview</title>", build.name),
            )
            .replace(
                "\" defer></script>",
                &format!(
                    "\" data-label=\"Search build {}\" defer></script>",
                    build.name
                ),
            );
        assert_eq!(text, expected, "{path}");
    }
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
            "search-index.js",
            "search.js",
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
fn every_page_says_what_it_is_keeps_crawlers_out_and_runs_only_the_search() {
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
        assert!(
            html.contains(
                "<meta http-equiv=\"Content-Security-Policy\" content=\"script-src 'self' file:\">"
            ),
            "{path}"
        );
        let scripts = html.matches("<script").count();
        if path == "all.html" {
            assert_eq!(scripts, 0, "{path}");
        } else {
            assert_eq!(scripts, 1, "{path}");
            assert!(
                html.contains(
                    "<script src=\"./search.js\" data-index=\"./search-index.js\" data-root=\"./\" defer></script>"
                ),
                "{path}"
            );
        }
    }
}

#[test]
fn the_search_index_has_each_page_by_heading_with_its_code() {
    let archive = archived(None);
    assert_eq!(
        archive.search_index,
        texts(&archive)["search-index.js"].len()
    );
    let script = &texts(&archive)["search-index.js"];
    let json = script
        .split_once(" = ")
        .and_then(|(_, rest)| rest.strip_suffix(";\n"))
        .expect("the index's JSON");
    let index: serde_json::Value = serde_json::from_str(json).expect("JSON");
    let pages = index["pages"].as_array().expect("pages");
    let hrefs: Vec<&str> = pages.iter().filter_map(|p| p["href"].as_str()).collect();
    assert_eq!(
        hrefs,
        [
            "keys.html",
            "configure-legacy.html",
            "install-agent.html",
            "quickstart.html",
            "upgrade.html",
        ],
        "in reading order"
    );
    let install = &pages[2];
    assert_eq!(install["title"], "Install the Quill agent");
    let section = install["sections"]
        .as_array()
        .expect("sections")
        .iter()
        .find(|s| s["id"] == "install-agent")
        .expect("the heading");
    assert_eq!(section["heading"], "Install the agent");
    let text = section["text"].as_str().expect("text");
    assert!(text.contains("npm install -g @quill/agent"), "{text}");
    assert!(text.contains("quill --version"), "{text}");
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
fn sections_are_named_by_their_own_page_and_the_contents_page_moves_aside_for_the_root() {
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
        sidebar.contains("<p class=\"archive-folder\">Ref</p>"),
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

/// The source paths of `_ascribe/nav.json`'s pages as `sidebar()` in
/// `@ascribed/astro` draws them: the root's own page and its pages that
/// aren't beside a top folder, then each top section's group.
fn sidebar_order(nav: &serde_json::Value) -> Vec<String> {
    let root = &nav["root"];
    let source = |page: &serde_json::Value| page["source"].as_str().map(str::to_owned);
    let sections = root["sections"].as_array().cloned().unwrap_or_default();
    let beside: Vec<String> = sections.iter().filter_map(|s| source(&s["page"])).collect();
    let mut out: Vec<String> = source(&root["page"]).into_iter().collect();
    out.extend(
        root["pages"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(source)
            .filter(|page| !beside.contains(page)),
    );
    for section in &sections {
        reading_order(section, true, &mut out);
    }
    out
}

/// The source paths of a section of `_ascribe/nav.json` in its group's
/// order: its own page, then its pages, each nested section whose own page
/// is one of them right after it, then the other nested sections.
fn reading_order(section: &serde_json::Value, own: bool, out: &mut Vec<String>) {
    let source = |page: &serde_json::Value| page["source"].as_str().map(str::to_owned);
    if own && let Some(page) = source(&section["page"]) {
        out.push(page);
    }
    let pages = section["pages"].as_array().cloned().unwrap_or_default();
    let nested = section["sections"].as_array().cloned().unwrap_or_default();
    let mut read = vec![false; nested.len()];
    for page in &pages {
        out.extend(source(page));
        for (at, n) in nested.iter().enumerate() {
            if !read[at] && source(&n["page"]).is_some() && source(&n["page"]) == source(page) {
                read[at] = true;
                reading_order(n, false, out);
            }
        }
    }
    for (n, read) in nested.iter().zip(read) {
        if !read {
            reading_order(n, true, out);
        }
    }
}

#[test]
fn an_archive_lists_the_pages_its_line_s_site_sidebar_lists_in_its_order() {
    let nav = "version = 1\n\n\
        [[sections]]\nfolder = \"ref/\"\nlabel = \"Look it up\"\n\n\
        [[sections]]\nfolder = \"guide/\"\npages = [\"b.md\", \"deep.md\"]\n";
    // Both lines live, so 1.0 is an older line, without what 2.0 adds.
    let model = FOLDERS.replace(
        "live = [\"1.0\"]\nplanned = [\"2.0\"]",
        "live = [\"2.0\", \"1.0\"]",
    );
    assert_ne!(model, FOLDERS, "the lines moved");
    let project = support::memory_project_with_files(
        &model,
        &[
            ("index.md", "---\ntitle: Home\n---\n\nText.\n"),
            ("guide/index.md", "---\ntitle: The guide\n---\n\nText.\n"),
            (
                "guide/a.md",
                "---\ntitle: A\navailable: app 2.0\n---\n\nNew.\n",
            ),
            ("guide/b.md", "---\ntitle: B\n---\n\nText.\n"),
            ("guide/deep.md", "---\ntitle: Deep\n---\n\nText.\n"),
            ("guide/deep/x.md", "---\ntitle: X\n---\n\nText.\n"),
            ("ref.md", "---\ntitle: Reference\n---\n\nText.\n"),
            ("ref/z.md", "---\ntitle: Z\n---\n\nText.\n"),
            ("zfaq.md", "---\ntitle: Zebra FAQ\n---\n\nText.\n"),
        ],
        &[("ascribe.nav", nav)],
    );
    for (line, expected) in [
        (
            "app 1.0",
            vec![
                "index.md",
                "zfaq.md",
                "ref.md",
                "ref/z.md",
                "guide/index.md",
                "guide/b.md",
                "guide/deep.md",
                "guide/deep/x.md",
            ],
        ),
        (
            "app 2.0",
            vec![
                "index.md",
                "zfaq.md",
                "ref.md",
                "ref/z.md",
                "guide/index.md",
                "guide/b.md",
                "guide/deep.md",
                "guide/deep/x.md",
                "guide/a.md",
            ],
        ),
    ] {
        let line = ascribe_model::named_line(line).expect("a line").1;
        let build = project
            .model()
            .line_build("site", &line)
            .expect("the line's build");
        // The line's site output, and its sidebar's reading order.
        let index = project.for_build(&build);
        let router = ascribe_resolve::AstroRouter::from_consumer(&index.model().consumer);
        let resolved = index.resolve_build(&build, &router);
        let cx = ascribe_emit::EmitContext::new(&index, std::path::Path::new("/nowhere"), &build);
        let site = ascribe_emit::emit(
            &ascribe_emit::SiteEmitter::new(index.model()),
            &cx,
            &resolved,
        )
        .expect("the site emits");
        let nav_json = site
            .files
            .iter()
            .find(|f| f.path.as_str() == "_ascribe/nav.json")
            .and_then(|f| match &f.contents {
                Contents::Text(text) => serde_json::from_str::<serde_json::Value>(text).ok(),
                Contents::Copy(_) => None,
            })
            .expect("the site's navigation");
        assert_eq!(
            sidebar_order(&nav_json),
            expected,
            "the site's sidebar of {line:?}"
        );
        // The archive's sidebar, on its contents page.
        let archive = archive(
            &project,
            std::path::Path::new("/nowhere"),
            &build,
            &ArchiveOptions::default(),
        )
        .expect("archived");
        let texts = texts(&archive);
        let contents = &texts["contents.html"];
        let side = &contents[contents.find("archive-sidebar").expect("a sidebar")..];
        let side = &side[..side.find("</nav>").expect("its end")];
        let hrefs: Vec<String> = values(side, "href")
            .into_iter()
            .skip(1)
            .map(|h| h.trim_start_matches("./").replace(".html", ".md"))
            .collect();
        assert_eq!(hrefs, expected, "the archive's sidebar of {line:?}");
        assert!(
            side.contains("<p class=\"archive-folder\">Look it up</p>"),
            "{side}"
        );
        assert!(
            side.contains("<li><a href=\"./guide/deep.html\">Deep</a>\n<ul>\n<li><a href=\"./guide/deep/x.html\">X</a></li>\n</ul>\n</li>"),
            "the nested section in its page's place: {side}"
        );
        // Previous and next go down the sidebar: from the root's last page
        // into the first group, `ref.md` beside `ref/` heading it.
        let faq = &texts["zfaq.html"];
        assert!(
            faq.contains("<a rel=\"next\" href=\"./ref.html\">Next: Reference</a>"),
            "{faq}"
        );
        let z = &texts["ref/z.html"];
        assert!(
            z.contains("<a rel=\"next\" href=\"../guide/index.html\">Next: The guide</a>"),
            "{z}"
        );
        let x = &texts["guide/deep/x.html"];
        assert!(
            x.contains("<a rel=\"prev\" href=\"../deep.html\">Previous: Deep</a>"),
            "{x}"
        );
    }
}
