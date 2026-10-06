//! The static report, on projects held in memory: what it renders, the
//! limits, images, and that it can't reach the network.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::collections::HashMap;
use std::sync::Arc;

use ascribe_core::{FileId, RelPath};
use ascribe_diff::html::{AssetFiles, MAX_IMAGE_BYTES, Version, write_html, write_html_with};
use ascribe_diff::{BaseInfo, Report, RepositoryInfo, SCHEMA_VERSION, Side, compare_builds};
use ascribe_resolve::{Layout, MemoryFs, Project};
use serde_json::Value;

const MODEL: &str = "spec = \"0.1\"\n[builds.site]\n[builds.other]\n";

struct Files(HashMap<String, Vec<u8>>);

impl AssetFiles for Files {
    fn read(&self, content_path: &RelPath) -> Option<Vec<u8>> {
        self.0.get(content_path.as_str()).cloned()
    }
}

fn project(model: &str, files: &[(&str, &str)]) -> Project {
    let loaded = ascribe_model::load_str(model, FileId::new(0)).expect("a valid model");
    let layout = Layout::from_model(&loaded);
    let mut fs = MemoryFs::new(&layout);
    for (path, text) in files {
        fs = fs.with_source(path, text);
    }
    Project::load(Arc::new(loaded), layout, &fs)
}

fn report(before: &Project, after: &Project) -> Report {
    let builds = compare_builds(
        Some(Side {
            project: before,
            model_text: MODEL,
        }),
        Side {
            project: after,
            model_text: MODEL,
        },
        &["site", "other"],
    );
    Report {
        schema_version: SCHEMA_VERSION,
        ascribe_version: "0.0.0",
        base: BaseInfo {
            requested: "main".into(),
            commit: "0123456789abcdef".into(),
            merge_base: None,
        },
        repository: RepositoryInfo {
            root: "/repo".into(),
            project_prefix: String::new(),
        },
        working_tree_errors: 0,
        builds,
    }
}

/// The data the report's script reads.
fn data(html: &str) -> Value {
    let open = "<script type=\"application/json\" id=\"ascribe-review-data\">";
    let start = html.find(open).expect("the data") + open.len();
    let end = start + html[start..].find("</script>").expect("its end");
    serde_json::from_str(&html[start..end]).expect("JSON")
}

fn no_files() -> Files {
    Files(HashMap::new())
}

#[test]
fn renders_both_sides_of_a_changed_page_with_anchors() {
    let before = project(MODEL, &[("a.md", "# A\n\nOld words here.\n\nGone.\n")]);
    let after = project(MODEL, &[("a.md", "# A\n\nNew words here.\n")]);
    let files = no_files();
    let html = write_html(
        &report(&before, &after),
        Some(Version {
            project: &before,
            files: &files,
        }),
        Version {
            project: &after,
            files: &files,
        },
    );
    assert!(html.starts_with("<!doctype html>\n"));
    assert!(html.contains("<title>Review: 1 changed page in 2 builds</title>"));
    let data = data(&html);
    let page = &data["builds"][0]["pages"][0];
    assert_eq!(page["path"], "a.md");
    assert_eq!(page["status"], "changed");
    assert_eq!(page["omitted"], false);
    let now = &data["pages"][page["now"].as_str().unwrap()]["html"];
    let was = &data["pages"][page["was"].as_str().unwrap()]["html"];
    assert!(
        now.as_str()
            .unwrap()
            .contains(r#"<p data-ascribe-source="a.md:3-3">New words here.</p>"#),
        "{now}"
    );
    assert!(
        was.as_str()
            .unwrap()
            .contains(r#"<p data-ascribe-source="a.md:5-5">Gone.</p>"#),
        "{was}"
    );
    // The changes are the JSON report's, keys and all.
    assert_eq!(page["changes"][0]["kind"], "changed");
    assert_eq!(page["changes"][0]["words"]["now_text"], "New words here.");
}

#[test]
fn a_page_identical_in_two_builds_is_stored_once() {
    let before = project(MODEL, &[("a.md", "# A\n\nOne.\n")]);
    let after = project(MODEL, &[("a.md", "# A\n\nTwo.\n")]);
    let files = no_files();
    let html = write_html(
        &report(&before, &after),
        Some(Version {
            project: &before,
            files: &files,
        }),
        Version {
            project: &after,
            files: &files,
        },
    );
    let data = data(&html);
    let site = &data["builds"][0]["pages"][0];
    let other = &data["builds"][1]["pages"][0];
    assert_eq!(site["now"], other["now"]);
    assert_eq!(site["was"], other["was"]);
    assert_eq!(data["pages"].as_object().unwrap().len(), 2);
}

#[test]
fn added_and_removed_pages_have_one_side() {
    let before = project(MODEL, &[("old.md", "# Old\n")]);
    let after = project(MODEL, &[("new.md", "# New\n")]);
    let files = no_files();
    let html = write_html(
        &report(&before, &after),
        Some(Version {
            project: &before,
            files: &files,
        }),
        Version {
            project: &after,
            files: &files,
        },
    );
    let data = data(&html);
    let pages = data["builds"][0]["pages"].as_array().unwrap();
    let added = pages.iter().find(|p| p["path"] == "new.md").unwrap();
    let removed = pages.iter().find(|p| p["path"] == "old.md").unwrap();
    assert!(added["now"].is_string() && added["was"].is_null());
    assert!(removed["was"].is_string() && removed["now"].is_null());
}

#[test]
fn pages_beyond_the_limit_are_listed_not_rendered() {
    let before = project(MODEL, &[]);
    let after = project(
        MODEL,
        &[("a.md", "# A\n"), ("b.md", "# B\n"), ("c.md", "# C\n")],
    );
    let files = no_files();
    let html = write_html_with(
        &report(&before, &after),
        Some(Version {
            project: &before,
            files: &files,
        }),
        Version {
            project: &after,
            files: &files,
        },
        2,
    );
    let data = data(&html);
    assert_eq!(data["limit"]["pages"], 2);
    // Three pages in each of two builds: two rendered, four listed.
    assert_eq!(data["limit"]["omitted"], 4);
    let site = data["builds"][0]["pages"].as_array().unwrap();
    assert_eq!(site[0]["omitted"], false);
    assert_eq!(site[1]["omitted"], false);
    assert_eq!(site[2]["omitted"], true);
    assert!(site[2]["now"].is_null());
    assert_eq!(site[2]["path"], "c.md");
}

#[test]
fn images_are_inlined_up_to_the_limit() {
    let before = project(MODEL, &[]);
    let after = project(
        MODEL,
        &[
            (
                "a.md",
                "# A\n\n![Small](img/small.png)\n\n![Big](img/big.png)\n\n![Lost](img/lost.png)\n",
            ),
            ("img/small.png", ""),
            ("img/big.png", ""),
        ],
    );
    let png: &[u8] = b"\x89PNG\r\n\x1a\n";
    let files = Files(HashMap::from([
        ("img/small.png".to_owned(), png.to_vec()),
        ("img/big.png".to_owned(), vec![0; MAX_IMAGE_BYTES + 1]),
    ]));
    let html = write_html(
        &report(&before, &after),
        None,
        Version {
            project: &after,
            files: &files,
        },
    );
    let data = data(&html);
    let key = data["builds"][0]["pages"][0]["now"].as_str().unwrap();
    let images = &data["pages"][key]["images"];
    let small = images
        .as_object()
        .unwrap()
        .values()
        .find(|i| i["path"] == "img/small.png")
        .unwrap();
    assert_eq!(
        data["images"][small["image"].as_str().unwrap()],
        "data:image/png;base64,iVBORw0KGgo="
    );
    let big = images
        .as_object()
        .unwrap()
        .values()
        .find(|i| i["path"] == "img/big.png")
        .unwrap();
    assert_eq!(big["bytes"], MAX_IMAGE_BYTES + 1);
    assert!(big.get("image").is_none());
    let lost = images
        .as_object()
        .unwrap()
        .values()
        .find(|i| i["path"] == "img/lost.png");
    assert!(lost.is_none_or(|l| l.get("image").is_none()));
    assert!(html.len() < MAX_IMAGE_BYTES);
}

#[test]
fn the_report_makes_no_requests() {
    let before = project(MODEL, &[]);
    let after = project(
        MODEL,
        &[(
            "a.md",
            "# A\n\n![Remote](https://example.com/a.png)\n\n<img src=\"http://example.com/b.png\">\n",
        )],
    );
    let files = no_files();
    let html = write_html(
        &report(&before, &after),
        None,
        Version {
            project: &after,
            files: &files,
        },
    );
    // The page's own HTML is data the script draws from: it replaces every
    // image the report doesn't hold with a placeholder, and the content
    // security policy blocks anything else. The rest of the file loads
    // nothing.
    let data = data(&html);
    let key = data["builds"][0]["pages"][0]["now"].as_str().unwrap();
    assert!(data["pages"][key]["images"].as_object().unwrap().is_empty());
    assert!(data["images"].as_object().unwrap().is_empty());
    assert_no_requests(&without_data(&html));
}

/// The file without the data its script reads.
fn without_data(html: &str) -> String {
    let open = "<script type=\"application/json\" id=\"ascribe-review-data\">";
    let start = html.find(open).expect("the data");
    let end = start + html[start..].find("</script>").expect("its end");
    format!("{}{}", &html[..start], &html[end..])
}

/// No `http` URL in a `src`, no stylesheet linked, no `url()` to the
/// network, and a content security policy that allows only `data:` images
/// and the report's own script.
pub fn assert_no_requests(html: &str) {
    let lower = html.to_ascii_lowercase();
    for (i, _) in lower.match_indices("src=") {
        let value = lower[i + 4..].trim_start_matches(['"', '\'']);
        assert!(
            !value.starts_with("http:") && !value.starts_with("https:") && !value.starts_with("//"),
            "a src that loads from the network: {}",
            &html[i..(i + 60).min(html.len())]
        );
    }
    assert!(!lower.contains("<link"), "a linked file");
    for (i, _) in lower.match_indices("url(") {
        let value = lower[i + 4..].trim_start_matches(['"', '\'', ' ']);
        assert!(
            value.starts_with("data:") || value.starts_with('#'),
            "a url() that isn't inline: {}",
            &html[i..(i + 60).min(html.len())]
        );
    }
    assert!(html.contains(
        "<meta http-equiv=\"Content-Security-Policy\" content=\"default-src 'none'; img-src data:; style-src 'unsafe-inline'; script-src 'sha256-"
    ));
    // Only the report's own script runs: no inline handler or
    // `javascript:` URL in a page's HTML.
    let policy = &html[html.find("Content-Security-Policy").unwrap()..];
    let policy = &policy[..policy.find('>').unwrap()];
    assert!(!policy.contains("script-src 'unsafe-inline'"), "{policy}");
}
